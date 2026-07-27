"""Tectonic plate generation: Voronoi tessellation, plate metadata,
boundary classification. Phase 1a first PR."""

import math

from world_factory.constants import (
    MAXIMUM_PLATE_COUNT,
    MINIMUM_PLATE_COUNT,
    MINIMUM_PLATE_INTERIOR_CELL_COUNT,
)
from world_factory.determinism import sample_unit_interval
from world_factory.models import (
    BoundaryRecord,
    BoundaryType,
    GeologyLayer,
    PlateRecord,
    PlateType,
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
    return PlateType.CONTINENTAL if draw < 0.4 else PlateType.OCEANIC


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
    )


def minimum_plate_count_for_grid(width: int, height: int) -> int:
    """Smallest plate_count that keeps every plate non-degenerate."""
    return max(MINIMUM_PLATE_COUNT, 1)


def is_degenerate_plate(plate: PlateRecord) -> bool:
    """A plate is degenerate if it owns fewer cells than the minimum
    interior threshold."""
    return plate.cell_count < MINIMUM_PLATE_INTERIOR_CELL_COUNT