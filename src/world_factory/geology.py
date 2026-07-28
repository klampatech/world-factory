"""Tectonic plate generation: Voronoi tessellation, plate metadata,
boundary classification, and Phase 1e rock/ore/soil sublayers."""

import math

from world_factory.constants import (
    GEOLOGY_SUBLEYER_ALGORITHM_VERSION,
    LOAM_PRECIPITATION_THRESHOLD_MM,
    MAXIMUM_PLATE_COUNT,
    MINIMUM_ORE_PROBABILITY,
    MINIMUM_PLATE_COUNT,
    ORE_PROBABILITY_SCALE,
    PEAT_PRECIPITATION_THRESHOLD_MM,
    PERMAFROST_TEMPERATURE_CELSIUS,
    SEDIMENTARY_ELEVATION_CAP_METERS,
)
from world_factory.determinism import sample_unit_interval
from world_factory.models import (
    BoundaryRecord,
    BoundaryType,
    GeologyLayer,
    PlateRecord,
    PlateType,
    ProvenanceRecord,
    RockType,
    SoilType,
    WorldScale,
)

FloatGrid = tuple[tuple[float, ...], ...]
IntGrid = tuple[tuple[int, ...], ...]
BoundaryGrid = tuple[tuple[BoundaryType | None, ...], ...]


def _grid_width(scale: WorldScale) -> int:
    return {
        WorldScale.SMALL: 24,
        WorldScale.MEDIUM: 48,
        WorldScale.LARGE: 256,
    }[scale]


def _grid_height(scale: WorldScale) -> int:
    return {
        WorldScale.SMALL: 12,
        WorldScale.MEDIUM: 24,
        WorldScale.LARGE: 128,
    }[scale]


def _normalized_coords(x: int, y: int, width: int, height: int) -> tuple[float, float]:
    return (x / max(width - 1, 1), y / max(height - 1, 1))


def _plate_seed_points(seed: int, plate_count: int) -> list[tuple[float, float]]:
    return [
        (
            sample_unit_interval(seed, "geography.plate.seed.x", index),
            sample_unit_interval(seed, "geography.plate.seed.y", index),
        )
        for index in range(plate_count)
    ]


def _assign_plate_id_grid(
    seed_points: list[tuple[float, float]], width: int, height: int
) -> IntGrid:
    return tuple(
        tuple(
            _closest_plate_id(
                _normalized_coords(x, y, width, height), seed_points
            )
            for x in range(width)
        )
        for y in range(height)
    )


def _closest_plate_id(
    coord: tuple[float, float], seed_points: list[tuple[float, float]]
) -> int:
    px, py = coord
    best_index = 0
    best_distance = math.inf
    for index, (sx, sy) in enumerate(seed_points):
        distance = (sx - px) ** 2 + (sy - py) ** 2
        if distance < best_distance:
            best_distance = distance
            best_index = index
    return best_index


def _plate_metadata(
    seed: int,
    plate_id_grid: IntGrid,
) -> tuple[PlateRecord, ...]:
    height = len(plate_id_grid)
    width = len(plate_id_grid[0])
    plate_ids = sorted({cell for row in plate_id_grid for cell in row})
    records: list[PlateRecord] = []
    for plate_id in plate_ids:
        cells = [
            (x, y)
            for y in range(height)
            for x in range(width)
            if plate_id_grid[y][x] == plate_id
        ]
        cell_count = len(cells)
        if cell_count == 0:
            continue
        sum_x = sum(x for x, _ in cells)
        sum_y = sum(y for _, y in cells)
        records.append(
            PlateRecord(
                id=plate_id,
                plate_type=_draw_plate_type(seed, plate_id),
                centroid_x=sum_x / cell_count,
                centroid_y=sum_y / cell_count,
                motion_heading_radians=sample_unit_interval(
                    seed, "geography.plate.motion.heading", plate_id
                )
                * math.tau,
                motion_speed=sample_unit_interval(
                    seed, "geography.plate.motion.speed", plate_id
                ),
                cell_count=cell_count,
            )
        )
    return tuple(records)


def _draw_plate_type(seed: int, plate_id: int) -> PlateType:
    draw = sample_unit_interval(seed, "geography.plate.type", plate_id)
    return PlateType.CONTINENTAL if draw < 0.45 else PlateType.OCEANIC


def _motion_vector(plate: PlateRecord) -> tuple[float, float]:
    return (
        math.cos(plate.motion_heading_radians) * plate.motion_speed,
        math.sin(plate.motion_heading_radians) * plate.motion_speed,
    )


def _classify_boundary(
    plate_a: PlateRecord, plate_b: PlateRecord
) -> BoundaryType:
    ax, ay = _motion_vector(plate_a)
    bx, by = _motion_vector(plate_b)
    dot = ax * bx + ay * by
    speed_a = plate_a.motion_speed
    speed_b = plate_b.motion_speed
    relative_speed = math.sqrt((ax - bx) ** 2 + (ay - by) ** 2)
    threshold = 0.1 * relative_speed
    if dot < -threshold and speed_a > 0.05 and speed_b > 0.05:
        return BoundaryType.CONVERGENT
    if dot > threshold and speed_a > 0.05 and speed_b > 0.05:
        return BoundaryType.DIVERGENT
    return BoundaryType.TRANSFORM


def _boundary_grid(
    plate_id_grid: IntGrid, plates: tuple[PlateRecord, ...]
) -> tuple[BoundaryGrid, tuple[BoundaryRecord, ...]]:
    height = len(plate_id_grid)
    width = len(plate_id_grid[0])
    plate_by_id = {plate.id: plate for plate in plates}
    type_grid: list[list[BoundaryType | None]] = [
        [None] * width for _ in range(height)
    ]
    boundary_records: list[BoundaryRecord] = []
    for y in range(height):
        for x in range(width):
            plate_id = plate_id_grid[y][x]
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < width and 0 <= ny < height:
                    neighbor_id = plate_id_grid[ny][nx]
                    if neighbor_id != plate_id:
                        pair = tuple(sorted((plate_id, neighbor_id)))
                        if type_grid[y][x] is None:
                            plate_a = plate_by_id[pair[0]]
                            plate_b = plate_by_id[pair[1]]
                            boundary_type = _classify_boundary(plate_a, plate_b)
                            type_grid[y][x] = boundary_type
                            boundary_records.append(
                                BoundaryRecord(
                                    x=x,
                                    y=y,
                                    boundary_type=boundary_type,
                                    plate_a=pair[0],
                                    plate_b=pair[1],
                                )
                            )
                        break
    return (
        tuple(tuple(row) for row in type_grid),
        tuple(boundary_records),
    )


def generate_geology(seed: int, plate_count: int, scale: WorldScale) -> GeologyLayer:
    """Generate a deterministic tectonic plate layout for the world."""
    if not MINIMUM_PLATE_COUNT <= plate_count <= MAXIMUM_PLATE_COUNT:
        raise ValueError(
            f"plate_count {plate_count} outside [{MINIMUM_PLATE_COUNT}, {MAXIMUM_PLATE_COUNT}]"
        )
    width = _grid_width(scale)
    height = _grid_height(scale)
    seed_points = _plate_seed_points(seed, plate_count)
    plate_id_grid = _assign_plate_id_grid(seed_points, width, height)
    plates = _plate_metadata(seed, plate_id_grid)
    type_grid, boundaries = _boundary_grid(plate_id_grid, plates)
    return GeologyLayer(
        width=width,
        height=height,
        plates=plates,
        boundaries=boundaries,
        plate_id_grid=plate_id_grid,
        boundary_type_grid=type_grid,
        rock_type_grid=tuple(tuple(RockType.BASALT for _ in range(width)) for _ in range(height)),
        ore_presence_grid=tuple(tuple(False for _ in range(width)) for _ in range(height)),
        soil_type_grid=tuple(tuple(SoilType.LOAM for _ in range(width)) for _ in range(height)),
    )


def generate_geology_sublayers(
    geology: GeologyLayer,
    elevation: FloatGrid,
    temperature: FloatGrid,
    precipitation: FloatGrid,
    sea_level: float,
    seed: int,
) -> GeologyLayer:
    """Populate the Phase 1e rock / ore / soil sublayers on a Phase 1a
    GeologyLayer. Returns a new layer with the three sublayer grids set
    and the existing fields preserved."""
    rock_grid = _rock_type_grid(geology, elevation, sea_level)
    ore_grid = _ore_presence_grid(rock_grid, geology, seed)
    soil_grid = _soil_type_grid(rock_grid, temperature, precipitation, sea_level)
    return geology.model_copy(
        update={
            "rock_type_grid": rock_grid,
            "ore_presence_grid": ore_grid,
            "soil_type_grid": soil_grid,
        }
    )


def _rock_type_grid(
    geology: GeologyLayer,
    elevation: FloatGrid,
    sea_level: float,
) -> tuple[tuple[RockType, ...], ...]:
    """Per-cell rock type from plate composition, boundary class, and
    elevation."""
    height = geology.height
    width = geology.width
    plate_types = {plate.id: plate.plate_type for plate in geology.plates}
    grid: list[list[RockType]] = [
        [RockType.BASALT] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            plate_id = geology.plate_id_grid[y][x]
            plate_type = plate_types[plate_id]
            boundary = geology.boundary_type_grid[y][x]
            elevation_value = elevation[y][x]
            if plate_type is PlateType.OCEANIC:
                grid[y][x] = RockType.BASALT
            elif boundary is BoundaryType.CONVERGENT:
                grid[y][x] = RockType.VOLCANIC
            elif boundary is BoundaryType.DIVERGENT:
                grid[y][x] = RockType.BASALT
            elif boundary is BoundaryType.TRANSFORM:
                grid[y][x] = RockType.METAMORPHIC
            elif elevation_value < sea_level + SEDIMENTARY_ELEVATION_CAP_METERS:
                grid[y][x] = RockType.SEDIMENTARY
            else:
                grid[y][x] = RockType.GRANITE
    return tuple(tuple(row) for row in grid)


_ROCK_ORE_MULTIPLIER: dict[RockType, float] = {
    RockType.BASALT: 0.10,
    RockType.GRANITE: 0.20,
    RockType.SEDIMENTARY: 0.15,
    RockType.METAMORPHIC: 0.25,
    RockType.VOLCANIC: 0.40,
}


def _ore_presence_grid(
    rock_grid: tuple[tuple[RockType, ...], ...],
    geology: GeologyLayer,
    seed: int,
) -> tuple[tuple[bool, ...], ...]:
    """Per-cell ore presence. Probability scales with rock type and
    proximity to plate boundaries; a deterministic RNG draw decides
    which cells cross the threshold."""
    height = geology.height
    width = geology.width
    boundary_distance = _boundary_distance_grid(geology)
    grid: list[list[bool]] = [
        [False] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            rock = rock_grid[y][x]
            multiplier = _ROCK_ORE_MULTIPLIER[rock]
            proximity = 1.0 / (1.0 + boundary_distance[y][x])
            probability = multiplier * proximity * ORE_PROBABILITY_SCALE
            if probability < MINIMUM_ORE_PROBABILITY:
                continue
            draw = sample_unit_interval(seed, "geology.ore_presence", x, y)
            if draw < probability:
                grid[y][x] = True
    return tuple(tuple(row) for row in grid)


def _boundary_distance_grid(geology: GeologyLayer) -> list[list[int]]:
    """Distance (in cell steps) from each cell to the nearest plate
    boundary."""
    from collections import deque

    height = geology.height
    width = geology.width
    distance: list[list[int]] = [
        [10**6] * width for _ in range(height)
    ]
    queue: deque[tuple[int, int]] = deque()
    for y in range(height):
        for x in range(width):
            if geology.boundary_type_grid[y][x] is not None:
                distance[y][x] = 0
                queue.append((x, y))
    while queue:
        x, y = queue.popleft()
        for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            nx, ny = x + dx, y + dy
            if not (0 <= nx < width and 0 <= ny < height):
                continue
            if distance[ny][nx] > distance[y][x] + 1:
                distance[ny][nx] = distance[y][x] + 1
                queue.append((nx, ny))
    return distance


def _soil_type_grid(
    rock_grid: tuple[tuple[RockType, ...], ...],
    temperature: FloatGrid,
    precipitation: FloatGrid,
    sea_level: float,
) -> tuple[tuple[SoilType, ...], ...]:
    """Per-cell soil from climate and rock. PERMAFROST in cold cells,
    SAND in arid cells, PEAT in wet cells, LOAM or CLAY in temperate
    cells based on rock."""
    height = len(rock_grid)
    width = len(rock_grid[0])
    grid: list[list[SoilType]] = [
        [SoilType.LOAM] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            temperature_value = temperature[y][x]
            precipitation_value = precipitation[y][x]
            rock = rock_grid[y][x]
            if temperature_value < PERMAFROST_TEMPERATURE_CELSIUS:
                grid[y][x] = SoilType.PERMAFROST
            elif precipitation_value < LOAM_PRECIPITATION_THRESHOLD_MM:
                grid[y][x] = SoilType.SAND
            elif precipitation_value >= PEAT_PRECIPITATION_THRESHOLD_MM:
                grid[y][x] = SoilType.PEAT
            elif rock is RockType.BASALT:
                grid[y][x] = SoilType.CLAY
            else:
                grid[y][x] = SoilType.LOAM
    return tuple(tuple(row) for row in grid)


def geology_sublayer_provenance() -> ProvenanceRecord:
    """Provenance record describing the geology sublayer algorithm."""
    return ProvenanceRecord(
        output_path="geology.sublayers",
        process="rock-ore-soil-tagging",
        input_paths=(
            "geology",
            "geography.elevation_meters",
            "climate.temperature_celsius",
            "climate.annual_precipitation_mm",
            "metadata.config.seed",
        ),
        algorithm_version=GEOLOGY_SUBLEYER_ALGORITHM_VERSION,
    )
