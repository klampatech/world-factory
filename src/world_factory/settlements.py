"""Phase 3a settlement placement via deterministic candidate scoring.

Algorithm:

1. Generate candidate cells on a coarse grid (spacing
   min(width, height) // SETTLEMENT_CANDIDATE_GRID_DIVISOR).
2. Score each candidate on five weighted contributions:
   water_access, arable_land, defensibility, climate_suitability,
   mineral_proximity.
3. Sort by score descending; pick top-K candidates with rejection
   sampling on a minimum spacing.
4. Assign population from arable_land + water_access +
   mineral_proximity.
"""

from world_factory.constants import (
    SETTLEMENT_CANDIDATE_GRID_DIVISOR,
    SETTLEMENT_CLIMATE_HIGH_CELSIUS,
    SETTLEMENT_CLIMATE_LOW_CELSIUS,
    SETTLEMENT_CLIMATE_LOWER_BOUND_CELSIUS,
    SETTLEMENT_CLIMATE_RAMP_COLD_CELSIUS,
    SETTLEMENT_CLIMATE_RAMP_HOT_CELSIUS,
    SETTLEMENT_DEFENSIBILITY_HIGH_METERS,
    SETTLEMENT_DEFENSIBILITY_LOW_METERS,
    SETTLEMENT_DEFENSIBILITY_RAMP_METERS,
    SETTLEMENT_MIN_COUNT,
    SETTLEMENT_MIN_SPACING_CELLS,
    SETTLEMENT_PER_PLATE_COUNT,
    SETTLEMENT_POPULATION_ARABLE_BASE,
    SETTLEMENT_POPULATION_MINERAL_BONUS,
    SETTLEMENT_POPULATION_WATER_BONUS,
    SETTLEMENT_WATER_DECAY_DIVISOR,
    SETTLEMENTS_ALGORITHM_VERSION,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiomeClass,
    ProvenanceRecord,
    RiverSegment,
    Settlement,
    SettlementsLayer,
    WorldModel,
)

FloatGrid = tuple[tuple[float, ...], ...]
BiomeGrid = tuple[tuple[BiomeClass, ...], ...]


def _candidate_spacing(width: int, height: int) -> int:
    return max(1, min(width, height) // SETTLEMENT_CANDIDATE_GRID_DIVISOR)


def _water_access_score(
    candidate_x: int,
    candidate_y: int,
    river_segments: tuple[RiverSegment, ...],
) -> float:
    """Score inversely with Manhattan distance to the nearest river
    segment mouth (cheap proxy: distance to a high-discharge cell).
    Score = 1 / (1 + d / SETTLEMENT_WATER_DECAY_DIVISOR)."""
    best = 0.0
    for segment in river_segments:
        if segment.length_cells <= 0:
            continue
        mouth_x, mouth_y = segment.mouth
        distance = abs(candidate_x - mouth_x) + abs(candidate_y - mouth_y)
        score = 1.0 / (1.0 + distance / SETTLEMENT_WATER_DECAY_DIVISOR)
        if score > best:
            best = score
    return min(best, 1.0)


def _arable_land_score(biome: BiomeClass) -> float:
    if biome in {
        BiomeClass.TEMPERATE_FOREST,
        BiomeClass.GRASSLAND,
        BiomeClass.TROPICAL_FOREST,
    }:
        return 1.0
    if biome in {BiomeClass.OCEAN, BiomeClass.ICE, BiomeClass.ALPINE}:
        return 0.0
    return 0.5


def _defensibility_score(elevation_meters: float) -> float:
    if elevation_meters < SETTLEMENT_DEFENSIBILITY_LOW_METERS:
        return elevation_meters / SETTLEMENT_DEFENSIBILITY_LOW_METERS
    if elevation_meters > SETTLEMENT_DEFENSIBILITY_HIGH_METERS:
        return max(
            0.0,
            1.0 - (elevation_meters - SETTLEMENT_DEFENSIBILITY_HIGH_METERS)
            / SETTLEMENT_DEFENSIBILITY_RAMP_METERS,
        )
    return 1.0


def _climate_suitability_score(temperature_celsius: float) -> float:
    if temperature_celsius < SETTLEMENT_CLIMATE_LOW_CELSIUS:
        return max(
            0.0,
            (temperature_celsius - SETTLEMENT_CLIMATE_LOWER_BOUND_CELSIUS)
            / SETTLEMENT_CLIMATE_RAMP_COLD_CELSIUS,
        )
    if temperature_celsius > SETTLEMENT_CLIMATE_HIGH_CELSIUS:
        return max(
            0.0,
            1.0
            - (temperature_celsius - SETTLEMENT_CLIMATE_HIGH_CELSIUS)
            / SETTLEMENT_CLIMATE_RAMP_HOT_CELSIUS,
        )
    return 1.0


def _mineral_proximity_score(
    candidate_x: int, candidate_y: int, ore_grid: tuple[tuple[bool, ...], ...]
) -> float:
    height = len(ore_grid)
    width = len(ore_grid[0])
    radius = 3
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            ny, nx = candidate_y + dy, candidate_x + dx
            if 0 <= ny < height and 0 <= nx < width and ore_grid[ny][nx]:
                return 1.0
    return 0.0


def _score_candidate(
    candidate_x: int,
    candidate_y: int,
    elevation: FloatGrid,
    temperature: FloatGrid,
    biome_grid: BiomeGrid,
    ore_grid: tuple[tuple[bool, ...], ...],
    river_segments: tuple[RiverSegment, ...],
) -> float:
    biome = biome_grid[candidate_y][candidate_x]
    water = _water_access_score(candidate_x, candidate_y, river_segments)
    arable = _arable_land_score(biome)
    defensibility = _defensibility_score(elevation[candidate_y][candidate_x])
    climate = _climate_suitability_score(temperature[candidate_y][candidate_x])
    mineral = _mineral_proximity_score(candidate_x, candidate_y, ore_grid)
    return 0.30 * water + 0.30 * arable + 0.10 * defensibility + 0.20 * climate + 0.10 * mineral


def _population_for(
    candidate_x: int,
    candidate_y: int,
    elevation: FloatGrid,
    temperature: FloatGrid,
    biome_grid: BiomeGrid,
    ore_grid: tuple[tuple[bool, ...], ...],
    river_segments: tuple[RiverSegment, ...],
) -> int:
    biome = biome_grid[candidate_y][candidate_x]
    arable = _arable_land_score(biome)
    water = _water_access_score(candidate_x, candidate_y, river_segments)
    mineral = _mineral_proximity_score(candidate_x, candidate_y, ore_grid)
    return int(
        SETTLEMENT_POPULATION_ARABLE_BASE * arable
        + SETTLEMENT_POPULATION_WATER_BONUS * water
        + SETTLEMENT_POPULATION_MINERAL_BONUS * mineral
    )


def _within_grid(x: int, y: int, width: int, height: int) -> bool:
    return 0 <= x < width and 0 <= y < height


def _min_distance_to_picked(
    x: int, y: int, picked: list[tuple[int, int]]
) -> int:
    return min(
        (
            abs(x - px) + abs(y - py)
            for px, py in picked
        ),
        default=10**9,
    )


def build_settlements(
    elevation: FloatGrid,
    temperature: FloatGrid,
    biome_grid: BiomeGrid,
    ore_grid: tuple[tuple[bool, ...], ...],
    river_segments: tuple[RiverSegment, ...],
    plate_count: int,
) -> SettlementsLayer:
    """Build the Phase 3a settlements layer from physical + climate
    fields.

    Algorithm:
    1. Walk a coarse candidate grid (spacing = min(width, height)
       // SETTLEMENT_CANDIDATE_GRID_DIVISOR).
    2. Score each candidate on water_access, arable_land,
       defensibility, climate_suitability, mineral_proximity.
    3. Sort by score descending; pick top-K (K = max(20,
       plate_count * 3)) with rejection sampling on
       SETTLEMENT_MIN_SPACING_CELLS spacing.
    4. Tiebreak by (x, y) ascending for determinism.
    5. Assign population from arable_land + water_access +
       mineral_proximity.
    """
    height = len(elevation)
    width = len(elevation[0])
    spacing = _candidate_spacing(width, height)
    candidates: list[tuple[float, int, int]] = []
    for y in range(0, height, spacing):
        for x in range(0, width, spacing):
            if not _within_grid(x, y, width, height):
                continue
            if biome_grid[y][x] is BiomeClass.OCEAN:
                continue
            score = _score_candidate(
                x, y, elevation, temperature, biome_grid, ore_grid, river_segments
            )
            candidates.append((score, x, y))
    candidates.sort(key=lambda triple: (-triple[0], triple[1], triple[2]))
    target_count = max(SETTLEMENT_MIN_COUNT, SETTLEMENT_PER_PLATE_COUNT * plate_count)
    picked: list[tuple[int, int]] = []
    for _score, x, y in candidates:
        if len(picked) >= target_count:
            break
        if _min_distance_to_picked(x, y, picked) < SETTLEMENT_MIN_SPACING_CELLS:
            continue
        picked.append((x, y))
    settlements: list[Settlement] = []
    for settlement_id, (x, y) in enumerate(picked):
        score = _score_candidate(
            x, y, elevation, temperature, biome_grid, ore_grid, river_segments
        )
        population = _population_for(
            x, y, elevation, temperature, biome_grid, ore_grid, river_segments
        )
        settlements.append(
            Settlement(
                id=settlement_id,
                x=x,
                y=y,
                population=population,
                founding_score=round(score, 6),
            )
        )
    return SettlementsLayer(settlements=tuple(settlements))


def settlements_provenance() -> ProvenanceRecord:
    """Provenance record describing the settlement placement algorithm."""
    return ProvenanceRecord(
        output_path="settlements",
        process="candidate-scoring-with-rejection-sampling",
        input_paths=(
            "geography.elevation_meters",
            "climate.temperature_celsius",
            "biomes.classifications",
            "geology.ore_presence_grid",
            "hydrology.river_segments",
            "metadata.config.plate_count",
        ),
        algorithm_version=SETTLEMENTS_ALGORITHM_VERSION,
    )


def validate_settlements_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3a settlement invariants."""
    violations: list[InvariantViolation] = []
    height = world.geography.height
    width = world.geography.width
    for settlement in world.settlements.settlements:
        if not (0 <= settlement.x < width and 0 <= settlement.y < height):
            violations.append(
                _violation(
                    "settlement-out-of-bounds",
                    f"settlements.settlements.{settlement.id}",
                    f"position ({settlement.x}, {settlement.y}) is outside the grid",
                )
            )
        if settlement.population < 0:
            violations.append(
                _violation(
                    "settlement-population-negative",
                    f"settlements.settlements.{settlement.id}.population",
                    f"population {settlement.population} is negative",
                )
            )
        if not 0.0 <= settlement.founding_score <= 1.0:
            violations.append(
                _violation(
                    "settlement-founding-score-bounds",
                    f"settlements.settlements.{settlement.id}.founding_score",
                    f"score {settlement.founding_score} outside [0, 1]",
                )
            )
    return violations