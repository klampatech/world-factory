"""Cross-layer plausibility invariants must hold for every generated world."""

import pytest

from world_factory.constants import (
    MAXIMUM_ELEVATION_METERS,
    MAXIMUM_OCEAN_FRACTION,
    MAXIMUM_SURFACE_TEMPERATURE_CELSIUS,
    MINIMUM_ELEVATION_METERS,
    MINIMUM_OCEAN_FRACTION,
    MINIMUM_SURFACE_TEMPERATURE_CELSIUS,
)
from world_factory.generator import generate_world
from world_factory.models import ClimateClass, WorldConfig, WorldScale
from world_factory.validation import validate_world


def _configs() -> list[WorldConfig]:
    return [
        WorldConfig(seed=seed, scale=scale, climate_class=climate)
        for seed in (0, 1, 42, 1 << 32)
        for scale in (WorldScale.SMALL, WorldScale.MEDIUM)
        for climate in (ClimateClass.COLD, ClimateClass.TEMPERATE, ClimateClass.HOT)
    ]


@pytest.mark.parametrize("config", _configs())
def test_generated_world_passes_invariants(config: WorldConfig) -> None:
    report = validate_world(generate_world(config))
    assert report.is_valid, report.model_dump(mode="json")


def test_elevation_within_bounds() -> None:
    world = generate_world(WorldConfig(seed=42))
    values = [v for row in world.geography.elevation_meters for v in row]
    assert values
    assert min(values) >= MINIMUM_ELEVATION_METERS
    assert max(values) <= MAXIMUM_ELEVATION_METERS


def test_temperature_within_bounds() -> None:
    world = generate_world(WorldConfig(seed=42))
    values = [v for row in world.climate.temperature_celsius for v in row]
    assert values
    assert min(values) >= MINIMUM_SURFACE_TEMPERATURE_CELSIUS
    assert max(values) <= MAXIMUM_SURFACE_TEMPERATURE_CELSIUS


def test_precipitation_non_negative() -> None:
    world = generate_world(WorldConfig(seed=42))
    values = [v for row in world.climate.annual_precipitation_mm for v in row]
    assert values
    assert min(values) >= 0.0


def test_surface_water_fraction_within_bounds() -> None:
    world = generate_world(WorldConfig(seed=42))
    fraction = world.hydrology.surface_water_fraction
    assert MINIMUM_OCEAN_FRACTION <= fraction <= MAXIMUM_OCEAN_FRACTION


def test_grids_share_dimensions() -> None:
    world = generate_world(WorldConfig(seed=42))
    expected = (world.geography.height, world.geography.width)
    for path, grid in (
        ("climate.temperature_celsius", world.climate.temperature_celsius),
        ("climate.annual_precipitation_mm", world.climate.annual_precipitation_mm),
        ("biomes.classifications", world.biomes.classifications),
    ):
        actual = (len(grid), len(grid[0]) if grid else 0)
        assert actual == expected, f"{path} has shape {actual}, expected {expected}"


def test_provenance_records_required_outputs() -> None:
    world = generate_world(WorldConfig(seed=42))
    paths = {record.output_path for record in world.provenance}
    assert "geography.elevation_meters" in paths
    assert "climate" in paths
    assert "biomes.classifications" in paths


def test_biomes_only_use_known_classifications() -> None:
    world = generate_world(WorldConfig(seed=42))
    allowed = {
        "ocean",
        "ice",
        "alpine",
        "desert",
        "tropical-forest",
        "temperate-forest",
        "grassland",
    }
    classifications = {cell for row in world.biomes.classifications for cell in row}
    assert classifications <= allowed, classifications - allowed
