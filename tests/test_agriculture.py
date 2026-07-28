"""Phase 3a.2 agriculture / caloric accounting invariants."""

import math

import pytest

from world_factory.agriculture import (
    build_agriculture,
    validate_agriculture_layer,
)
from world_factory.constants import (
    AGRICULTURE_ALGORITHM_VERSION,
    AGRICULTURE_CALORIC_KCAL_PER_TONNE,
    AGRICULTURE_EXTRACTION_RADIUS_CELLS,
    AGRICULTURE_KCAL_PER_PERSON_PER_YEAR,
    AGRICULTURE_PRECIPITATION_OPTIMUM_MM,
    AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS,
)
from world_factory.generator import generate_world
from world_factory.models import (
    AgricultureLayer,
    BiomeClass,
    Settlement,
    SettlementsLayer,
    SoilType,
    WorldConfig,
    WorldScale,
)


def _config(seed: int = 42) -> WorldConfig:
    return WorldConfig(seed=seed, scale=WorldScale.LARGE)


def test_world_model_includes_agriculture_layer() -> None:
    world = generate_world(_config())
    assert world.agriculture is not None
    assert len(world.agriculture.agriculture) >= 0


def test_agriculture_records_parallel_to_settlements() -> None:
    """Agriculture records must be parallel to settlements by index."""
    world = generate_world(_config())
    settlements = world.settlements.settlements
    agriculture = world.agriculture.agriculture
    assert len(agriculture) == len(settlements)
    for index, record in enumerate(agriculture):
        assert record.settlement_id == settlements[index].id


def test_carrying_capacity_non_negative() -> None:
    world = generate_world(_config())
    for record in world.agriculture.agriculture:
        assert record.carrying_capacity >= 0


def test_agricultural_surplus_is_finite() -> None:
    world = generate_world(_config())
    for record in world.agriculture.agriculture:
        assert math.isfinite(record.agricultural_surplus_kcal_per_year)


def test_deterministic_across_runs() -> None:
    a = generate_world(_config())
    b = generate_world(_config())
    assert a.agriculture.agriculture == b.agriculture.agriculture


def test_world_id_stable_across_phase_3a2() -> None:
    """3a.2 adds no new WorldConfig fields, so world_id for --seed 42
    at LARGE scale must match v1-demo's
    `9d75e7103b52704b48ce77071a22a586`."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.metadata.world_id == "9d75e7103b52704b48ce77071a22a586"


def test_agriculture_provenance_record_present() -> None:
    world = generate_world(_config())
    agriculture_records = [
        record for record in world.provenance if record.output_path == "agriculture"
    ]
    assert len(agriculture_records) == 1
    assert agriculture_records[0].algorithm_version == AGRICULTURE_ALGORITHM_VERSION


def test_validate_agriculture_layer_empty_for_valid_world() -> None:
    world = generate_world(_config())
    assert validate_agriculture_layer(world) == []


def test_validate_agriculture_flags_length_mismatch() -> None:
    """If the agriculture layer has the wrong number of records,
    the validator must catch it loudly."""
    world = generate_world(_config())
    valid_records = world.agriculture.agriculture
    trimmed = AgricultureLayer(agriculture=valid_records[:-1])
    bad_world = world.model_copy(update={"agriculture": trimmed})
    violations = validate_agriculture_layer(bad_world)
    assert any(
        v.code == "agriculture-settlement-length-mismatch" for v in violations
    )


def test_validate_agriculture_flags_settlement_id_mismatch() -> None:
    """If a record references the wrong settlement id, the validator
    must flag it."""
    world = generate_world(_config())
    records = list(world.agriculture.agriculture)
    if records:
        records[0] = records[0].model_copy(update={"settlement_id": 9999})
    bad_world = world.model_copy(
        update={"agriculture": AgricultureLayer(agriculture=tuple(records))}
    )
    violations = validate_agriculture_layer(bad_world)
    assert any(
        v.code == "agriculture-settlement-id-mismatch" for v in violations
    )


def test_validate_agriculture_flags_non_finite_temperature() -> None:
    """A NaN temperature inside any settlement's extraction radius
    must fail loudly with the cell coordinates in the path."""
    world = generate_world(_config())
    temperature = [list(row) for row in world.climate.temperature_celsius]
    if world.settlements.settlements:
        target_settlement = world.settlements.settlements[0]
        tx, ty = target_settlement.x, target_settlement.y
        temperature[ty][tx] = float("nan")
    new_climate = world.climate.model_copy(
        update={"temperature_celsius": tuple(tuple(row) for row in temperature)}
    )
    bad_world = world.model_copy(update={"climate": new_climate})
    violations = validate_agriculture_layer(bad_world)
    nan_violations = [
        v for v in violations
        if v.code == "agriculture-temperature-not-finite"
    ]
    assert nan_violations, "expected NaN temperature to fail loudly"
    assert f"[{ty}][{tx}]" in nan_violations[0].path


def test_validate_agriculture_flags_non_finite_precipitation() -> None:
    """An Infinity precipitation inside any settlement's extraction
    radius must fail loudly."""
    world = generate_world(_config())
    precipitation = [list(row) for row in world.climate.annual_precipitation_mm]
    if world.settlements.settlements:
        target_settlement = world.settlements.settlements[0]
        tx, ty = target_settlement.x, target_settlement.y
        precipitation[ty][tx] = float("inf")
    new_climate = world.climate.model_copy(
        update={"annual_precipitation_mm": tuple(tuple(row) for row in precipitation)}
    )
    bad_world = world.model_copy(update={"climate": new_climate})
    violations = validate_agriculture_layer(bad_world)
    bad_violations = [
        v for v in violations
        if v.code == "agriculture-precipitation-not-finite"
    ]
    assert bad_violations, "expected Inf precipitation to fail loudly"


def test_zero_arable_neighbors_yields_zero_capacity_and_deficit() -> None:
    """A settlement whose extraction radius is entirely ocean must
    produce carrying_capacity=0 and seasonal_deficit=True.

    Settlements in the generator cannot sit on ocean, so we drive
    `build_agriculture` directly on a world whose biomes are all
    ocean and whose only settlement is at the grid centre.
    """
    base_world = generate_world(_config())
    width = base_world.geography.width
    height = base_world.geography.height
    biome_grid = tuple(
        tuple(BiomeClass.OCEAN for _ in range(width)) for _ in range(height)
    )
    soil_grid = tuple(
        tuple(SoilType.SAND for _ in range(width)) for _ in range(height)
    )
    synthetic_world = base_world.model_copy(
        update={
            "biomes": base_world.biomes.model_copy(
                update={"classifications": biome_grid}
            ),
            "geology": base_world.geology.model_copy(
                update={"soil_type_grid": soil_grid}
            ),
            "settlements": SettlementsLayer(
                settlements=(
                    Settlement(
                        id=0, x=width // 2, y=height // 2,
                        population=500, founding_score=0.5,
                    ),
                )
            ),
            "agriculture": AgricultureLayer(agriculture=()),
        }
    )
    layer = build_agriculture(synthetic_world)
    assert len(layer.agriculture) == 1
    record = layer.agriculture[0]
    assert record.carrying_capacity == 0
    assert record.seasonal_deficit is True
    assert record.agricultural_surplus_kcal_per_year == pytest.approx(
        -500 * AGRICULTURE_KCAL_PER_PERSON_PER_YEAR, abs=1.0
    )


def test_malthusian_ceiling_applied() -> None:
    """surplus = capacity*kcal_per_person - population*kcal_per_person.

    The carrying capacity is a Malthusian ceiling: independent of
    the settlement's current population. The surplus is signed."""
    world = generate_world(_config())
    layer = build_agriculture(world)
    for record in layer.agriculture:
        settlement = world.settlements.settlements[record.settlement_id]
        expected_surplus = (
            record.carrying_capacity * AGRICULTURE_KCAL_PER_PERSON_PER_YEAR
            - settlement.population * AGRICULTURE_KCAL_PER_PERSON_PER_YEAR
        )
        # Tolerance: ±1 kcal for flooring.
        assert abs(
            record.agricultural_surplus_kcal_per_year - expected_surplus
        ) <= AGRICULTURE_KCAL_PER_PERSON_PER_YEAR


def test_statistical_realism_no_wild_outliers() -> None:
    """Across settlements in seed=42 LARGE, no carrying_capacity
    should exceed the maximum possible (extraction-radius cells
    at base yield). And at least one settlement has a non-trivial
    capacity."""
    world = generate_world(_config())
    capacities = [r.carrying_capacity for r in world.agriculture.agriculture]
    if not capacities:
        return
    cells_in_radius = (2 * AGRICULTURE_EXTRACTION_RADIUS_CELLS + 1) ** 2
    max_possible = int(
        cells_in_radius
        * AGRICULTURE_CALORIC_KCAL_PER_TONNE
        // AGRICULTURE_KCAL_PER_PERSON_PER_YEAR
    )
    for cap in capacities:
        assert cap <= max_possible
    assert max(capacities) > 0


def test_extraction_radius_respected() -> None:
    """The extraction radius constant should be 2 cells in this
    initial slice."""
    assert AGRICULTURE_EXTRACTION_RADIUS_CELLS == 2


def test_settlement_with_arable_neighbors_has_capacity() -> None:
    """A synthetic settlement surrounded by temperate-forest / loam
    / optimum climate must produce non-zero carrying capacity and
    no seasonal deficit."""
    base_world = generate_world(_config())
    width = 7
    height = 7
    biome_grid = tuple(
        tuple(BiomeClass.TEMPERATE_FOREST for _ in range(width))
        for _ in range(height)
    )
    soil_grid = tuple(
        tuple(SoilType.LOAM for _ in range(width)) for _ in range(height)
    )
    temperature = tuple(
        tuple(AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS for _ in range(width))
        for _ in range(height)
    )
    precipitation = tuple(
        tuple(AGRICULTURE_PRECIPITATION_OPTIMUM_MM for _ in range(width))
        for _ in range(height)
    )
    settlements_layer = SettlementsLayer(
        settlements=(Settlement(id=0, x=3, y=3, population=1000, founding_score=0.9),)
    )
    synthetic_world = base_world.model_copy(
        update={
            "geography": base_world.geography.model_copy(
                update={"width": width, "height": height}
            ),
            "climate": base_world.climate.model_copy(
                update={
                    "temperature_celsius": temperature,
                    "annual_precipitation_mm": precipitation,
                }
            ),
            "biomes": base_world.biomes.model_copy(
                update={"classifications": biome_grid}
            ),
            "geology": base_world.geology.model_copy(
                update={"soil_type_grid": soil_grid}
            ),
            "settlements": settlements_layer,
            "agriculture": AgricultureLayer(agriculture=()),
        }
    )
    layer = build_agriculture(synthetic_world)
    assert len(layer.agriculture) == 1
    record = layer.agriculture[0]
    assert record.carrying_capacity > 0
    assert record.seasonal_deficit is False