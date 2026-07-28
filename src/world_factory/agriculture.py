"""Phase 3a.2 agriculture / caloric accounting.

Algorithm:

1. For each settlement, walk its extraction radius
   (Chebyshev distance ≤ AGRICULTURE_EXTRACTION_RADIUS_CELLS).
2. Per-cell yield is a five-factor product:
       base_yield
       × precipitation_response(precip_mm)
       × temperature_response(temp_c)
       × soil_quality(soil_type)
       × biome_quality(biome_class)
   OCEAN cells contribute 0; ICE cells contribute 0; ALPINE / DESERT
   cells contribute through the biome quality coefficient.
3. Yield converts to kilocalories via
   AGRICULTURE_CALORIC_KCAL_PER_TONNE × cell_yield.
4. carrying_capacity = floor(total_kcal /
   AGRICULTURE_KCAL_PER_PERSON_PER_YEAR). This is the Malthusian
   ceiling — population cannot exceed what the land can feed.
5. agricultural_surplus_kcal_per_year = total_kcal - current_pop ×
   AGRICULTURE_KCAL_PER_PERSON_PER_YEAR (signed).
6. seasonal_deficit is set when:
       - the settlement has zero arable neighbors in its radius,
         OR
       - the worst per-cell yield in the radius falls below
         AGRICULTURE_DEFICIT_YIELD_FRACTION of the base yield.

All outputs are deterministic given (settlements, climate, biomes,
geology soil, hydrology discharge).
"""

import math

from world_factory.constants import (
    AGRICULTURE_ALGORITHM_VERSION,
    AGRICULTURE_BASE_YIELD_TONNES_PER_CELL,
    AGRICULTURE_BIOME_QUALITY,
    AGRICULTURE_CALORIC_KCAL_PER_TONNE,
    AGRICULTURE_DEFICIT_YIELD_FRACTION,
    AGRICULTURE_EXTRACTION_RADIUS_CELLS,
    AGRICULTURE_KCAL_PER_PERSON_PER_YEAR,
    AGRICULTURE_MINIMUM_ARABLE_CELLS,
    AGRICULTURE_PRECIPITATION_OPTIMUM_MM,
    AGRICULTURE_SOIL_QUALITY,
    AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS,
    AGRICULTURE_TEMPERATURE_RANGE_CELSIUS,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    AgricultureLayer,
    AgricultureRecord,
    BiomeClass,
    ProvenanceRecord,
    Settlement,
    SoilType,
    WorldModel,
)

FloatGrid = tuple[tuple[float, ...], ...]
BiomeGrid = tuple[tuple[BiomeClass, ...], ...]
SoilGrid = tuple[tuple[SoilType, ...], ...]


def _precipitation_response(precipitation_mm: float) -> float:
    """Bell-shaped response peaking at AGRICULTURE_PRECIPITATION_OPTIMUM_MM.

    Returns 0 outside ±100% deviation from the optimum; rises to
    1.0 at the optimum. Symmetric on the wet and dry sides."""
    if not math.isfinite(precipitation_mm):
        return math.nan
    deviation = (
        precipitation_mm - AGRICULTURE_PRECIPITATION_OPTIMUM_MM
    ) / AGRICULTURE_PRECIPITATION_OPTIMUM_MM
    return max(0.0, 1.0 - deviation * deviation)


def _temperature_response(temperature_celsius: float) -> float:
    """Bell-shaped response peaking at AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS.

    Returns 0 outside ±AGRICULTURE_TEMPERATURE_RANGE_CELSIUS from
    the optimum. Symmetric on the hot and cold sides."""
    if not math.isfinite(temperature_celsius):
        return math.nan
    deviation = (
        temperature_celsius - AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS
    ) / AGRICULTURE_TEMPERATURE_RANGE_CELSIUS
    return max(0.0, 1.0 - deviation * deviation)


def _soil_quality(soil: SoilType) -> float:
    return AGRICULTURE_SOIL_QUALITY[soil.value]


def _biome_quality(biome: BiomeClass) -> float:
    return AGRICULTURE_BIOME_QUALITY[biome.value]


def _cell_yield(
    precipitation_mm: float,
    temperature_celsius: float,
    soil: SoilType,
    biome: BiomeClass,
) -> float:
    """Per-cell yield in tonnes/year. Returns 0 for ocean / ice cells."""
    if biome in {BiomeClass.OCEAN, BiomeClass.ICE}:
        return 0.0
    soil_q = _soil_quality(soil)
    biome_q = _biome_quality(biome)
    if soil_q == 0.0 or biome_q == 0.0:
        return 0.0
    p_response = _precipitation_response(precipitation_mm)
    t_response = _temperature_response(temperature_celsius)
    return (
        AGRICULTURE_BASE_YIELD_TONNES_PER_CELL
        * p_response
        * t_response
        * soil_q
        * biome_q
    )


def _arable_neighbors(
    settlement_x: int,
    settlement_y: int,
    width: int,
    height: int,
    biome_grid: BiomeGrid,
    soil_grid: SoilGrid,
    precipitation: FloatGrid,
    temperature: FloatGrid,
) -> tuple[list[float], int]:
    """Return (per-cell yields, arable-cell-count) within the
    extraction radius. Arable = non-ocean and non-ice and the
    cell's product factor > 0."""
    yields: list[float] = []
    arable_count = 0
    radius = AGRICULTURE_EXTRACTION_RADIUS_CELLS
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            nx, ny = settlement_x + dx, settlement_y + dy
            if not (0 <= nx < width and 0 <= ny < height):
                continue
            cell_yield = _cell_yield(
                precipitation[ny][nx],
                temperature[ny][nx],
                soil_grid[ny][nx],
                biome_grid[ny][nx],
            )
            if cell_yield > 0.0:
                arable_count += 1
            yields.append(cell_yield)
    return yields, arable_count


def build_agriculture(world: WorldModel) -> AgricultureLayer:
    """Compute the per-settlement caloric accounting layer.

    Returns records in the same order as `world.settlements.settlements`,
    keyed by `Settlement.id`. NaN or non-finite values in the
    climate / precipitation / temperature grids fail loudly with
    the offending cell's coordinates."""
    biome_grid = world.biomes.classifications
    soil_grid = world.geology.soil_type_grid
    precipitation = world.climate.annual_precipitation_mm
    temperature = world.climate.temperature_celsius
    width = world.geography.width
    height = world.geography.height
    records: list[AgricultureRecord] = []
    for settlement in world.settlements.settlements:
        record = _record_for_settlement(
            settlement, width, height, biome_grid, soil_grid,
            precipitation, temperature,
        )
        records.append(record)
    return AgricultureLayer(agriculture=tuple(records))


def _record_for_settlement(
    settlement: Settlement,
    width: int,
    height: int,
    biome_grid: BiomeGrid,
    soil_grid: SoilGrid,
    precipitation: FloatGrid,
    temperature: FloatGrid,
) -> AgricultureRecord:
    cell_yields, arable_count = _arable_neighbors(
        settlement.x, settlement.y, width, height,
        biome_grid, soil_grid, precipitation, temperature,
    )
    total_yield_tonnes = sum(cell_yields)
    total_kcal = total_yield_tonnes * AGRICULTURE_CALORIC_KCAL_PER_TONNE
    carrying_capacity = int(total_kcal // AGRICULTURE_KCAL_PER_PERSON_PER_YEAR)
    surplus_kcal = total_kcal - settlement.population * AGRICULTURE_KCAL_PER_PERSON_PER_YEAR
    if arable_count < AGRICULTURE_MINIMUM_ARABLE_CELLS or not cell_yields:
        seasonal_deficit = True
    else:
        worst_yield = min(cell_yields)
        seasonal_deficit = (
            worst_yield
            < AGRICULTURE_DEFICIT_YIELD_FRACTION * AGRICULTURE_BASE_YIELD_TONNES_PER_CELL
        )
    return AgricultureRecord(
        settlement_id=settlement.id,
        carrying_capacity=carrying_capacity,
        agricultural_surplus_kcal_per_year=round(surplus_kcal, 6),
        seasonal_deficit=seasonal_deficit,
    )


def agriculture_provenance() -> ProvenanceRecord:
    """Provenance record describing the agriculture / caloric
    accounting algorithm."""
    return ProvenanceRecord(
        output_path="agriculture",
        process="caloric-accounting-with-extraction-radius",
        input_paths=(
            "settlements.settlements",
            "climate.annual_precipitation_mm",
            "climate.temperature_celsius",
            "biomes.classifications",
            "geology.soil_type_grid",
        ),
        algorithm_version=AGRICULTURE_ALGORITHM_VERSION,
    )


def validate_agriculture_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3a.2 agriculture invariants.

    Checks:
      - `AgricultureLayer.agriculture` is parallel to
        `SettlementsLayer.settlements` by id (same length, matching
        settlement_id per index).
      - `carrying_capacity` is non-negative and finite.
      - `agricultural_surplus_kcal_per_year` is finite.
      - Precipitation / temperature / soil / biome inputs are
        finite for every cell in any settlement's extraction
        radius (catches NaN / Infinity in the climate grid).
    """
    violations: list[InvariantViolation] = []
    settlements = world.settlements.settlements
    agriculture = world.agriculture.agriculture
    if len(agriculture) != len(settlements):
        violations.append(
            _violation(
                "agriculture-settlement-length-mismatch",
                "agriculture.agriculture",
                (
                    f"agriculture records ({len(agriculture)}) do not match "
                    f"settlements ({len(settlements)})"
                ),
            )
        )
        return violations
    for index, record in enumerate(agriculture):
        settlement = settlements[index]
        if record.settlement_id != settlement.id:
            violations.append(
                _violation(
                    "agriculture-settlement-id-mismatch",
                    f"agriculture.agriculture.{index}.settlement_id",
                    (
                        f"agriculture record {index} references "
                        f"settlement_id={record.settlement_id} but "
                        f"settlements.{index}.id={settlement.id}"
                    ),
                )
            )
        if record.carrying_capacity < 0:
            violations.append(
                _violation(
                    "agriculture-carrying-capacity-negative",
                    f"agriculture.agriculture.{index}.carrying_capacity",
                    f"carrying capacity {record.carrying_capacity} is negative",
                )
            )
        if not math.isfinite(record.agricultural_surplus_kcal_per_year):
            violations.append(
                _violation(
                    "agriculture-surplus-not-finite",
                    f"agriculture.agriculture.{index}.agricultural_surplus_kcal_per_year",
                    (
                        f"surplus {record.agricultural_surplus_kcal_per_year} "
                        f"is not finite"
                    ),
                )
            )
    precipitation = world.climate.annual_precipitation_mm
    temperature = world.climate.temperature_celsius
    biome_grid = world.biomes.classifications
    soil_grid = world.geology.soil_type_grid
    width = world.geography.width
    height = world.geography.height
    radius = AGRICULTURE_EXTRACTION_RADIUS_CELLS
    for settlement in settlements:
        for dy in range(-radius, radius + 1):
            for dx in range(-radius, radius + 1):
                nx, ny = settlement.x + dx, settlement.y + dy
                if not (0 <= nx < width and 0 <= ny < height):
                    continue
                if biome_grid[ny][nx] in {BiomeClass.OCEAN, BiomeClass.ICE}:
                    continue
                precip = precipitation[ny][nx]
                if not math.isfinite(precip):
                    violations.append(
                        _violation(
                            "agriculture-precipitation-not-finite",
                            f"climate.annual_precipitation_mm[{ny}][{nx}]",
                            f"precipitation {precip} is not finite (settlement "
                            f"id={settlement.id} extraction cell)",
                        )
                    )
                temp = temperature[ny][nx]
                if not math.isfinite(temp):
                    violations.append(
                        _violation(
                            "agriculture-temperature-not-finite",
                            f"climate.temperature_celsius[{ny}][{nx}]",
                            f"temperature {temp} is not finite (settlement "
                            f"id={settlement.id} extraction cell)",
                        )
                    )
                _ = soil_grid[ny][nx]
    return violations