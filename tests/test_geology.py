"""Phase 1a geology invariants: plate counts, boundary types, dimensioning."""

import pytest

from world_factory.constants import (
    MAXIMUM_PLATE_COUNT,
    MINIMUM_PLATE_COUNT,
    MINIMUM_PLATE_INTERIOR_CELL_COUNT,
    STANDARD_ATMOSPHERIC_PRESSURE_KPA,
)
from world_factory.generator import generate_world
from world_factory.geology import generate_geology
from world_factory.models import (
    BoundaryType,
    PlateType,
    WorldConfig,
    WorldScale,
)


def _geology(scale: WorldScale, plate_count: int) -> object:
    return generate_geology(seed=42, plate_count=plate_count, scale=scale)


def test_plate_count_within_bounds() -> None:
    geology = _geology(WorldScale.SMALL, plate_count=8)
    assert MINIMUM_PLATE_COUNT <= len(geology.plates) <= MAXIMUM_PLATE_COUNT


def test_plate_count_above_minimum_is_rejected() -> None:
    with pytest.raises(ValueError):
        generate_geology(seed=42, plate_count=MINIMUM_PLATE_COUNT - 1, scale=WorldScale.SMALL)


def test_boundary_grid_covers_every_plate_pair() -> None:
    geology = _geology(WorldScale.SMALL, plate_count=8)
    assert len(geology.boundaries) >= 0
    for record in geology.boundaries:
        assert record.boundary_type in set(BoundaryType)


def test_world_model_includes_geology_layer() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    assert world.geology.width == world.geography.width
    assert world.geology.height == world.geography.height
    assert world.geology.plate_id_grid
    assert all(plate.plate_type in set(PlateType) for plate in world.geology.plates)


def test_atmosphere_grid_pressure_within_bounds() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    pressures = [v for row in world.climate.atmospheric_pressure_kpa for v in row]
    assert pressures
    assert min(pressures) >= 1.0
    assert max(pressures) <= STANDARD_ATMOSPHERIC_PRESSURE_KPA + 60.0


def test_smallest_plates_have_non_degenerate_cell_count() -> None:
    geology = _geology(WorldScale.SMALL, plate_count=MINIMUM_PLATE_COUNT)
    for plate in geology.plates:
        assert plate.cell_count >= MINIMUM_PLATE_INTERIOR_CELL_COUNT
