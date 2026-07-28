"""River network, per-cell discharge, and watershed delineation.

Phase 1b hydrology derives:

1. **Flow direction** — D8 (8-neighbor) steepest-descent per cell; sinks
   route to their lowest neighbor so every cell has a downstream target.
2. **Flow accumulation** — topological pass by descending elevation;
   each cell's accumulation equals 1 plus the sum of its upstream cells.
3. **Discharge** — accumulation × precipitation × runoff × cell-area
   factor, in m^3/year.
4. **River segments** — traced from each headwater source to ocean;
   source cells clear the (contributing-cells, precipitation, elevation)
   thresholds.
5. **Watersheds** — every non-ocean cell is labeled with the basin id of
   its terminal ocean outflow.

All outputs are deterministic given (seed, geology, geography, climate)
and use only the stdlib.
"""

import math

from world_factory.constants import (
    GRID_CELL_AREA_KILOMETERS_SQUARED,
    HYDROLOGY_ALGORITHM_VERSION,
    MINIMUM_HEADWATER_BASIN_CELLS,
    MINIMUM_HEADWATER_ELEVATION_METERS,
    MINIMUM_RUNOFF_PRECIPITATION_MM,
    RUNOFF_COEFFICIENT,
)
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.models import (
    HydrologyLayer,
    ProvenanceRecord,
    RiverSegment,
    WorldModel,
)

FloatGrid = tuple[tuple[float, ...], ...]
OptionalIntGrid = tuple[tuple[int | None, ...], ...]

_NEIGHBOR_OFFSETS: tuple[tuple[int, int, float], ...] = (
    (-1, -1, math.sqrt(2.0)),  # NW
    (0, -1, 1.0),  # N
    (1, -1, math.sqrt(2.0)),  # NE
    (-1, 0, 1.0),  # W
    (1, 0, 1.0),  # E
    (-1, 1, math.sqrt(2.0)),  # SW
    (0, 1, 1.0),  # S
    (1, 1, math.sqrt(2.0)),  # SE
)

_HEADWATER_ELEVATION_PHASE_0 = 750.0
_HEADWATER_PRECIPITATION_PHASE_0 = 1_200.0


def _ocean_mask(elevation: FloatGrid, sea_level: float) -> list[list[bool]]:
    height = len(elevation)
    width = len(elevation[0])
    return [
        [elevation[y][x] <= sea_level for x in range(width)] for y in range(height)
    ]


def _flow_direction_grid(
    elevation: FloatGrid,
    ocean: list[list[bool]],
    seed: int,
) -> list[list[tuple[int, int] | None]]:
    """Per-cell downstream target. None for ocean cells.

    Non-ocean cells flow to the 8-neighbor with the steepest descent
    (tiebreak via `hydrology.flow_direction` RNG). Cells with no
    descending neighbor (sinks) flow to the lowest neighbor so the
    drainage network has no dead ends and accumulation propagates.
    """
    height = len(elevation)
    width = len(elevation[0])
    grid: list[list[tuple[int, int] | None]] = [
        [None] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            if ocean[y][x]:
                continue
            elevation_self = elevation[y][x]
            best_slope = 0.0
            best_target: tuple[int, int] | None = None
            best_tiebreak = -1.0
            lowest_neighbor_elevation = math.inf
            lowest_neighbor: tuple[int, int] | None = None
            for dx, dy, distance in _NEIGHBOR_OFFSETS:
                nx, ny = x + dx, y + dy
                if not (0 <= nx < width and 0 <= ny < height):
                    continue
                neighbor_elevation = elevation[ny][nx]
                if neighbor_elevation < lowest_neighbor_elevation:
                    lowest_neighbor_elevation = neighbor_elevation
                    lowest_neighbor = (nx, ny)
                slope = (elevation_self - neighbor_elevation) / distance
                tiebreak = sample_unit_interval(
                    seed, "hydrology.flow_direction", x, y, dx + 1, dy + 1
                )
                if slope > best_slope or (
                    slope == best_slope and tiebreak > best_tiebreak
                ):
                    best_slope = slope
                    best_target = (nx, ny)
                    best_tiebreak = tiebreak
            grid[y][x] = best_target if best_target is not None else lowest_neighbor
    return grid


def _flow_accumulation(
    elevation: FloatGrid,
    flow_dir: list[list[tuple[int, int] | None]],
) -> list[list[int]]:
    """Per-cell contributing cell count. Ocean cells hold 1.

    Cells are processed in descending elevation order so every upstream
    contribution has already been added to the downstream cell before
    that downstream cell is itself propagated. Sink-routed cells
    (elevation_self ≤ elevation_neighbor) are processed before their
    downstream neighbor because the sort key is strictly on elevation.
    """
    height = len(elevation)
    width = len(elevation[0])
    accum: list[list[int]] = [[1] * width for _ in range(height)]
    cells = sorted(
        ((y, x) for y in range(height) for x in range(width)),
        key=lambda c: (-elevation[c[0]][c[1]], c[1], c[0]),
    )
    for y, x in cells:
        target = flow_dir[y][x]
        if target is None:
            continue
        nx, ny = target
        if 0 <= nx < width and 0 <= ny < height:
            accum[ny][nx] += accum[y][x]
    return accum


def _discharge_grid(
    accumulation: list[list[int]],
    precipitation: FloatGrid,
) -> FloatGrid:
    """Per-cell discharge in m^3/year.

    discharge = accumulation × precipitation_mm × runoff × cell_area_km² × 1000
    (the trailing 1000 converts mm·km² to m³).
    """
    height = len(precipitation)
    width = len(precipitation[0])
    scale = GRID_CELL_AREA_KILOMETERS_SQUARED * RUNOFF_COEFFICIENT * 1000.0
    grid: list[list[float]] = []
    for y in range(height):
        row: list[float] = []
        for x in range(width):
            value = accumulation[y][x] * precipitation[y][x] * scale
            row.append(round(value, 6))
        grid.append(row)
    return tuple(tuple(row) for row in grid)


def _headwater_candidates(
    elevation: FloatGrid,
    accumulation: list[list[int]],
    precipitation: FloatGrid,
    ocean: list[list[bool]],
    flow_dir: list[list[tuple[int, int] | None]],
    ocean_distance: list[list[tuple[int, int]]],
) -> list[tuple[int, int]]:
    """Headwater cell coordinates meeting elevation, precipitation, and
    basin-size thresholds.

    A headwater is a local flow-direction maximum (no upstream neighbor
    flows into it) whose drainage basin terminates at a cell whose
    accumulation is at least MINIMUM_HEADWATER_BASIN_CELLS. This selects
    rivers of meaningful length and avoids trivial basins.
    """
    height = len(elevation)
    width = len(elevation[0])
    is_local_max: list[list[bool]] = [[True] * width for _ in range(height)]
    for y in range(height):
        for x in range(width):
            if ocean[y][x]:
                continue
            for dx, dy, _ in _NEIGHBOR_OFFSETS:
                nx, ny = x + dx, y + dy
                if not (0 <= nx < width and 0 <= ny < height):
                    continue
                if flow_dir[ny][nx] == (x, y):
                    is_local_max[y][x] = False
                    break
    candidates: list[tuple[int, int]] = []
    for y in range(height):
        for x in range(width):
            if ocean[y][x]:
                continue
            if not is_local_max[y][x]:
                continue
            if elevation[y][x] < MINIMUM_HEADWATER_ELEVATION_METERS:
                continue
            if precipitation[y][x] < MINIMUM_RUNOFF_PRECIPITATION_MM:
                continue
            terminal = _walk_to_ocean((x, y), flow_dir, ocean, ocean_distance)
            basin_size = accumulation[terminal[1]][terminal[0]]
            if basin_size < MINIMUM_HEADWATER_BASIN_CELLS:
                continue
            candidates.append((x, y))
    candidates.sort(
        key=lambda coord: (
            -elevation[coord[1]][coord[0]],
            coord[0],
            coord[1],
        )
    )
    return candidates


def _trace_path(
    source: tuple[int, int],
    flow_dir: list[list[tuple[int, int] | None]],
    ocean: list[list[bool]],
    width: int,
    height: int,
) -> list[tuple[int, int]]:
    """Walk the flow-direction path from source to ocean or terminal sink."""
    path: list[tuple[int, int]] = [source]
    seen = {source}
    current = source
    while True:
        target = flow_dir[current[1]][current[0]]
        if target is None:
            return path
        if ocean[target[1]][target[0]]:
            path.append(target)
            return path
        if target in seen:
            return path
        seen.add(target)
        path.append(target)
        current = target
        if len(path) > width * height + 1:
            return path


def _terminal_watershed_ids(
    flow_dir: list[list[tuple[int, int] | None]],
    ocean: list[list[bool]],
    ocean_distance: list[list[tuple[int, int]]],
) -> dict[tuple[int, int], int]:
    """Walk every non-ocean cell to its ocean terminal; assign stable ids
    by order of first encounter."""
    ids: dict[tuple[int, int], int] = {}
    order: list[tuple[int, int]] = []
    for y in range(len(ocean)):
        for x in range(len(ocean[0])):
            if ocean[y][x]:
                continue
            cell = (x, y)
            terminal = _walk_to_ocean(cell, flow_dir, ocean, ocean_distance)
            if terminal not in ids:
                ids[terminal] = len(order)
                order.append(terminal)
    return ids


def _walk_to_ocean(
    cell: tuple[int, int],
    flow_dir: list[list[tuple[int, int] | None]],
    ocean: list[list[bool]],
    ocean_distance: list[list[tuple[int, int]]],
) -> tuple[int, int]:
    """Walk flow direction to ocean. Falls back to the precomputed
    nearest-ocean cell when the flow chain cycles or terminates inland."""
    seen = {cell}
    current = cell
    while True:
        target = flow_dir[current[1]][current[0]]
        if target is None:
            return ocean_distance[current[1]][current[0]]
        if ocean[target[1]][target[0]]:
            return target
        if target in seen:
            return ocean_distance[current[1]][current[0]]
        seen.add(target)
        current = target


def _ocean_distance_grid(
    width: int, height: int, ocean: list[list[bool]]
) -> list[list[tuple[int, int]]]:
    """Multi-source BFS from every ocean cell. Returns, for each cell,
    the ocean cell closest in grid steps. Land cells in a disconnected
    landmass (no adjacent ocean) are mapped to their nearest reachable
    ocean via interior BFS."""
    from collections import deque

    grid: list[list[tuple[int, int] | None]] = [
        [None] * width for _ in range(height)
    ]
    queue: deque[tuple[int, int]] = deque()
    for y in range(height):
        for x in range(width):
            if ocean[y][x]:
                grid[y][x] = (x, y)
                queue.append((x, y))
    while queue:
        x, y = queue.popleft()
        for dx, dy, _ in _NEIGHBOR_OFFSETS:
            nx, ny = x + dx, y + dy
            if not (0 <= nx < width and 0 <= ny < height):
                continue
            if grid[ny][nx] is not None:
                continue
            grid[ny][nx] = grid[y][x]
            queue.append((nx, ny))
    fallback: list[list[tuple[int, int]]] = [
        [cell if cell is not None else (0, 0) for cell in row] for row in grid
    ]
    return fallback


def _build_river_segments(
    sources: list[tuple[int, int]],
    flow_dir: list[list[tuple[int, int] | None]],
    elevation: FloatGrid,
    discharge: FloatGrid,
    ocean: list[list[bool]],
    width: int,
    height: int,
    watershed_ids: dict[tuple[int, int], int],
) -> list[RiverSegment]:
    """Trace one segment per headwater source; drop paths that don't reach
    ocean."""
    segments: list[RiverSegment] = []
    next_segment_id = 0
    for source in sources:
        path = _trace_path(source, flow_dir, ocean, width, height)
        if len(path) < 2:
            continue
        mouth = path[-1]
        if not ocean[mouth[1]][mouth[0]]:
            continue
        watershed_id = watershed_ids.get(mouth, 0)
        length = len(path) - 1
        elevations = [elevation[p[1]][p[0]] for p in path]
        mean_slope = (elevations[0] - elevations[-1]) / length if length else 0.0
        discharges = [discharge[p[1]][p[0]] for p in path]
        mean_discharge = sum(discharges) / len(discharges)
        segments.append(
            RiverSegment(
                id=next_segment_id,
                source=source,
                mouth=mouth,
                length_cells=length,
                mean_discharge=round(mean_discharge, 6),
                mean_slope=round(mean_slope, 6),
                watershed_id=watershed_id,
            )
        )
        next_segment_id += 1
    return segments


def _build_watershed_grid(
    width: int,
    height: int,
    ocean: list[list[bool]],
    flow_dir: list[list[tuple[int, int] | None]],
    watershed_ids: dict[tuple[int, int], int],
    ocean_distance: list[list[tuple[int, int]]],
) -> OptionalIntGrid:
    grid: list[list[int | None]] = [[None] * width for _ in range(height)]
    for y in range(height):
        for x in range(width):
            if ocean[y][x]:
                continue
            terminal = _walk_to_ocean((x, y), flow_dir, ocean, ocean_distance)
            grid[y][x] = watershed_ids.get(terminal)
    return tuple(tuple(row) for row in grid)


def _headwater_candidate_count_legacy(
    elevation: FloatGrid,
    precipitation: FloatGrid,
    ocean: list[list[bool]],
) -> int:
    """Phase 0 headwater count: high-elevation, high-precipitation cells."""
    height = len(elevation)
    width = len(elevation[0])
    return sum(
        1
        for y in range(height)
        for x in range(width)
        if not ocean[y][x]
        and elevation[y][x] >= _HEADWATER_ELEVATION_PHASE_0
        and precipitation[y][x] >= _HEADWATER_PRECIPITATION_PHASE_0
    )


def generate_hydrology(
    elevation: FloatGrid,
    precipitation: FloatGrid,
    sea_level: float,
    seed: int,
) -> HydrologyLayer:
    """Build the Phase 1b hydrology layer from physical fields."""
    height = len(elevation)
    width = len(elevation[0])
    ocean = _ocean_mask(elevation, sea_level)
    ocean_distance = _ocean_distance_grid(width, height, ocean)
    flow_dir = _flow_direction_grid(elevation, ocean, seed)
    accumulation = _flow_accumulation(elevation, flow_dir)
    discharge = _discharge_grid(accumulation, precipitation)
    watershed_ids = _terminal_watershed_ids(flow_dir, ocean, ocean_distance)
    sources = _headwater_candidates(
        elevation=elevation,
        accumulation=accumulation,
        precipitation=precipitation,
        ocean=ocean,
        flow_dir=flow_dir,
        ocean_distance=ocean_distance,
    )
    segments = _build_river_segments(
        sources=sources,
        flow_dir=flow_dir,
        elevation=elevation,
        discharge=discharge,
        ocean=ocean,
        width=width,
        height=height,
        watershed_ids=watershed_ids,
    )
    watershed_grid = _build_watershed_grid(
        width, height, ocean, flow_dir, watershed_ids, ocean_distance
    )
    ocean_cells = sum(1 for row in ocean for cell in row if cell)
    total_cells = width * height
    return HydrologyLayer(
        surface_water_fraction=round(ocean_cells / total_cells, 6),
        headwater_candidate_count=_headwater_candidate_count_legacy(
            elevation, precipitation, ocean
        ),
        river_segments=tuple(segments),
        discharge_grid=discharge,
        watershed_id_grid=watershed_grid,
    )


def hydrology_provenance() -> ProvenanceRecord:
    """Provenance record describing the hydrology algorithm."""
    return ProvenanceRecord(
        output_path="hydrology",
        process="d8-flow-routing-with-sink-flat-drainage",
        input_paths=(
            "geography.elevation_meters",
            "climate.annual_precipitation_mm",
            "metadata.config.seed",
        ),
        algorithm_version=HYDROLOGY_ALGORITHM_VERSION,
    )


def validate_hydrology_layer(world: WorldModel) -> list[InvariantViolation]:
    """P1 hydrographic consistency for the hydrology layer.

    - Every river mouth at sea level.
    - River lengths and discharges are positive.
    - Ocean cells carry no watershed id.
    - Land cells carry a watershed id.
    - Per-cell discharge is non-negative.
    """
    from world_factory.invariants import violation as _violation

    violations: list[InvariantViolation] = []
    sea_level = world.geography.sea_level_meters
    elevation = world.geography.elevation_meters
    segment: RiverSegment
    for segment in world.hydrology.river_segments:
        mouth_x, mouth_y = segment.mouth
        if not (0 <= mouth_y < len(elevation) and 0 <= mouth_x < len(elevation[0])):
            violations.append(
                _violation(
                    "river-mouth-out-of-bounds",
                    f"hydrology.river_segments.{segment.id}.mouth",
                    f"mouth ({mouth_x}, {mouth_y}) is outside the grid",
                )
            )
            continue
        if elevation[mouth_y][mouth_x] > sea_level:
            violations.append(
                _violation(
                    "river-mouth-above-sea-level",
                    f"hydrology.river_segments.{segment.id}.mouth",
                    (
                        f"mouth ({mouth_x}, {mouth_y}) elevation "
                        f"{elevation[mouth_y][mouth_x]:.2f}m > sea level {sea_level:.2f}m"
                    ),
                )
            )
        if segment.length_cells < 1:
            violations.append(
                _violation(
                    "river-length-non-positive",
                    f"hydrology.river_segments.{segment.id}.length_cells",
                    f"river length {segment.length_cells} must be >= 1",
                )
            )
        if segment.mean_discharge < 0.0:
            violations.append(
                _violation(
                    "river-discharge-negative",
                    f"hydrology.river_segments.{segment.id}.mean_discharge",
                    f"river mean discharge {segment.mean_discharge} is negative",
                )
            )
    for y, row in enumerate(world.hydrology.discharge_grid):
        for x, value in enumerate(row):
            if value < 0.0:
                violations.append(
                    _violation(
                        "discharge-negative",
                        f"hydrology.discharge_grid[{y}][{x}]",
                        f"discharge {value} is negative",
                    )
                )
    width = len(world.hydrology.watershed_id_grid[0])
    for y, ws_row in enumerate(world.hydrology.watershed_id_grid):
        for x, ws_label in enumerate(ws_row):
            if len(ws_row) != width:
                continue
            is_ocean = elevation[y][x] <= sea_level
            if is_ocean and ws_label is not None:
                violations.append(
                    _violation(
                        "watershed-label-on-ocean",
                        f"hydrology.watershed_id_grid[{y}][{x}]",
                        f"ocean cell carries watershed id {ws_label}",
                    )
                )
            if not is_ocean and ws_label is None:
                violations.append(
                    _violation(
                        "missing-watershed-label",
                        f"hydrology.watershed_id_grid[{y}][{x}]",
                        "land cell carries no watershed id",
                    )
                )
    return violations