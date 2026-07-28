"""Phase 1f validator consolidation invariants.

Verifies that:
- Each per-layer validator is exported from its owning module.
- `validate_world` delegates to all per-layer validators.
- Cross-cutting types (`InvariantViolation`, `ValidationReport`)
  remain exported from `world_factory.validation`.
"""

import pytest

from world_factory.astronomy import validate_astronomy_layer
from world_factory.atmosphere import validate_atmosphere_layer
from world_factory.generator import generate_world
from world_factory.geology import validate_geology_sublayer_shapes
from world_factory.hydrology import validate_hydrology_layer
from world_factory.models import WorldConfig, WorldScale
from world_factory.validation import (
    InvariantViolation,
    ValidationReport,
    validate_world,
)


def test_invariant_violation_and_report_exported() -> None:
    assert InvariantViolation is not None
    assert ValidationReport is not None


def test_per_layer_validators_are_callable() -> None:
    assert callable(validate_atmosphere_layer)
    assert callable(validate_astronomy_layer)
    assert callable(validate_geology_sublayer_shapes)
    assert callable(validate_hydrology_layer)
    assert callable(validate_world)


def test_validate_world_returns_validation_report() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    report = validate_world(world)
    assert isinstance(report, ValidationReport)
    assert report.is_valid


def test_per_layer_validators_return_empty_for_valid_world() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert validate_atmosphere_layer(world) == []
    assert validate_astronomy_layer(world) == []
    assert validate_geology_sublayer_shapes(world) == []
    assert validate_hydrology_layer(world) == []


def test_watershed_validator_flags_ocean_label() -> None:
    """Inject a watershed label on an ocean cell and confirm the
    validator catches it. Smoke test that per-layer validators
    actually run on the input they receive."""
    from world_factory.models import HydrologyLayer

    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    target_y = 0
    target_x = 0
    if world.geography.elevation_meters[target_y][target_x] > sea_level:
        for y in range(world.geography.height):
            for x in range(world.geography.width):
                if world.geography.elevation_meters[y][x] <= sea_level:
                    target_y, target_x = y, x
                    break
    polluted_watershed = tuple(
        tuple(
            0 if (y == target_y and x == target_x) else cell
            for x, cell in enumerate(row)
        )
        for y, row in enumerate(world.hydrology.watershed_id_grid)
    )
    polluted_hydrology = HydrologyLayer(
        surface_water_fraction=world.hydrology.surface_water_fraction,
        headwater_candidate_count=world.hydrology.headwater_candidate_count,
        river_segments=world.hydrology.river_segments,
        discharge_grid=world.hydrology.discharge_grid,
        watershed_id_grid=polluted_watershed,
    )
    polluted_world = world.model_copy(update={"hydrology": polluted_hydrology})
    violations = validate_hydrology_layer(polluted_world)
    codes = {v.code for v in violations}
    assert "watershed-label-on-ocean" in codes


@pytest.mark.parametrize("scale", [WorldScale.SMALL, WorldScale.MEDIUM, WorldScale.LARGE])
def test_validate_world_is_valid_at_all_scales(scale: WorldScale) -> None:
    world = generate_world(WorldConfig(seed=42, scale=scale))
    report = validate_world(world)
    assert report.is_valid, report.model_dump(mode="json")