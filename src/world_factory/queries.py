"""Phase 6 query surface — programmatic world exploration API.

Provides a typed, stable surface for reading cells and discovering
settlements without poking at nested tuple grids directly. All
queries are pure deterministic functions of a `WorldModel`.
"""

from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiomeClass,
    FaunaType,
    FloraType,
    RiverSegment,
    RockType,
    Settlement,
    SoilType,
    StrictModel,
    WindDirection,
    WorldModel,
)


class CellSummary(StrictModel):
    """Composite view of a single world cell.

    v1 surface: elevation, climate, geology sublayer, biology, and
    settlements / river segments that touch this cell. Designed
    so downstream consumers can read the world without indexing
    into the nested grid tuples.
    """

    x: int
    y: int
    elevation_meters: float
    sea_level_meters: float
    is_ocean: bool
    biome: BiomeClass
    temperature_celsius: float
    annual_precipitation_mm: float
    atmospheric_pressure_kpa: float
    wind_direction: WindDirection
    rock_type: RockType
    soil_type: SoilType
    has_ore: bool
    flora: FloraType | None
    fauna: FaunaType | None
    settlements: tuple[Settlement, ...]
    river_segments: tuple[RiverSegment, ...]


def _river_segments_through_cell(
    world: WorldModel, x: int, y: int
) -> tuple[RiverSegment, ...]:
    """Return the river segments whose mouth is at (x, y).

    v1 approximation: a segment is considered to touch (x, y) if
    its mouth is at (x, y). Future phases can replace this with a
    full path grid if needed.
    """
    return tuple(
        segment for segment in world.hydrology.river_segments if segment.mouth == (x, y)
    )


def summary_at(world: WorldModel, x: int, y: int) -> CellSummary:
    """Return a `CellSummary` for the given cell coordinates."""
    elevation = world.geography.elevation_meters[y][x]
    sea_level = world.geography.sea_level_meters
    is_ocean = elevation <= sea_level
    return CellSummary(
        x=x,
        y=y,
        elevation_meters=elevation,
        sea_level_meters=sea_level,
        is_ocean=is_ocean,
        biome=world.biomes.classifications[y][x],
        temperature_celsius=world.climate.temperature_celsius[y][x],
        annual_precipitation_mm=world.climate.annual_precipitation_mm[y][x],
        atmospheric_pressure_kpa=world.climate.atmospheric_pressure_kpa[y][x],
        wind_direction=world.climate.wind_direction_grid[y][x],
        rock_type=world.geology.rock_type_grid[y][x],
        soil_type=world.geology.soil_type_grid[y][x],
        has_ore=world.geology.ore_presence_grid[y][x],
        flora=world.biology.flora_grid[y][x],
        fauna=world.biology.fauna_grid[y][x],
        settlements=tuple(
            settlement
            for settlement in world.settlements.settlements
            if settlement.x == x and settlement.y == y
        ),
        river_segments=_river_segments_through_cell(world, x, y),
    )


def settlements_within(
    world: WorldModel, radius: int, x: int, y: int
) -> tuple[Settlement, ...]:
    """Return settlements within Chebyshev distance `radius` of (x, y).

    Chebyshev (max of |dx|, |dy|) matches the 8-neighbor topology
    used by Phase 1b D8 flow routing, so this query returns the
    settlements reachable within `radius` flow steps.
    """
    return tuple(
        settlement
        for settlement in world.settlements.settlements
        if max(abs(settlement.x - x), abs(settlement.y - y)) <= radius
    )


def summary_in_bounding_box(
    world: WorldModel,
    x_min: int,
    y_min: int,
    x_max: int,
    y_max: int,
) -> list[CellSummary]:
    """Return `CellSummary` for every cell inside the inclusive
    bounding box. y_max / x_max are clamped to the grid."""
    width = world.geography.width
    height = world.geography.height
    x_max = min(x_max, width - 1)
    y_max = min(y_max, height - 1)
    out: list[CellSummary] = []
    for y in range(max(0, y_min), y_max + 1):
        for x in range(max(0, x_min), x_max + 1):
            out.append(summary_at(world, x, y))
    return out


def validate_query_surface(world: WorldModel) -> list[InvariantViolation]:
    """Phase 6 query-surface sanity checks.

    v1: every settlement's (x, y) must be inside the grid; the
    round-trip through `summary_at` for each settlement must
    include that settlement in its `settlements` field.
    """
    violations: list[InvariantViolation] = []
    width = world.geography.width
    height = world.geography.height
    for settlement in world.settlements.settlements:
        if not (0 <= settlement.x < width and 0 <= settlement.y < height):
            violations.append(
                _violation(
                    "settlement-out-of-bounds-query",
                    f"settlements.settlements.{settlement.id}",
                    f"position ({settlement.x}, {settlement.y}) is outside the grid",
                )
            )
            continue
        cell_summary = summary_at(world, settlement.x, settlement.y)
        if settlement not in cell_summary.settlements:
            violations.append(
                _violation(
                    "settlement-round-trip-mismatch",
                    f"settlements.settlements.{settlement.id}",
                    f"summary_at({settlement.x}, {settlement.y}) did not include this settlement",
                )
            )
    return violations