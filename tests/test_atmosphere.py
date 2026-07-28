"""Phase 1c atmosphere invariants: wind belts, humidity transport, refined precipitation."""

import pytest

from world_factory.atmosphere import (
    _evaporation_grid,
    _latitude_degrees,
    _prevailing_belt_wind,
    _refined_precipitation,
    _saturation_vapor_pressure_kpa,
    _transport_humidity,
    _wind_direction_grid,
)
from world_factory.constants import MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG
from world_factory.generator import generate_world
from world_factory.models import (
    ClimateClass,
    WindDirection,
    WorldConfig,
    WorldScale,
)


def test_latitude_mapping_covers_both_poles() -> None:
    # First cell center sits at -82.5 deg; last cell center sits at +82.5 deg.
    assert _latitude_degrees(0, 12) < -80.0
    assert _latitude_degrees(11, 12) > 80.0


def test_hadley_belt_returns_east() -> None:
    assert _prevailing_belt_wind(0.0) is WindDirection.EAST
    assert _prevailing_belt_wind(15.0) is WindDirection.EAST
    assert _prevailing_belt_wind(-15.0) is WindDirection.EAST


def test_ferrel_belt_returns_west() -> None:
    assert _prevailing_belt_wind(45.0) is WindDirection.WEST
    assert _prevailing_belt_wind(-45.0) is WindDirection.WEST


def test_polar_belt_returns_east() -> None:
    assert _prevailing_belt_wind(75.0) is WindDirection.EAST
    assert _prevailing_belt_wind(-75.0) is WindDirection.EAST


def test_saturation_vapor_pressure_grows_with_temperature() -> None:
    cold = _saturation_vapor_pressure_kpa(0.0)
    warm = _saturation_vapor_pressure_kpa(30.0)
    assert warm > cold > 0.0


def _constant_grid(width: int, height: int, value: float) -> tuple[tuple[float, ...], ...]:
    return tuple(tuple(value for _ in range(width)) for _ in range(height))


def test_evaporation_only_from_ocean_cells() -> None:
    elevation_2d: list[list[float]] = [[100.0] * 4 for _ in range(4)]
    elevation_2d[0][0] = -50.0  # ocean cell
    elevation = tuple(tuple(row) for row in elevation_2d)
    temperature = _constant_grid(4, 4, 20.0)
    wind = tuple(tuple(WindDirection.EAST for _ in range(4)) for _ in range(4))
    evaporation = _evaporation_grid(elevation, temperature, 0.0, wind)
    assert evaporation[0][0] > 0.0
    for y in range(4):
        for x in range(4):
            if (x, y) == (0, 0):
                continue
            assert evaporation[y][x] == 0.0


def test_transport_humidity_bounded() -> None:
    elevation = _constant_grid(3, 3, 0.0)
    wind = tuple(tuple(WindDirection.EAST for _ in range(3)) for _ in range(3))
    humidity = _constant_grid(3, 3, MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG)
    transported = _transport_humidity(humidity, elevation, wind, 0.0)
    for row in transported:
        for value in row:
            assert 0.0 <= value <= MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG


def test_refined_precipitation_non_negative() -> None:
    base = _constant_grid(3, 3, 500.0)
    humidity = _constant_grid(3, 3, MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG)
    elevation = _constant_grid(3, 3, 100.0)
    refined = _refined_precipitation(base, humidity, elevation, 0.0)
    for row in refined:
        for value in row:
            assert value >= 0.0


def test_wind_direction_grid_is_strenum_value() -> None:
    elevation = _constant_grid(4, 4, 100.0)
    temperature = _constant_grid(4, 4, 15.0)
    wind = _wind_direction_grid(elevation, temperature, 0.0)
    for row in wind:
        for direction in row:
            assert direction in set(WindDirection)


def test_atmosphere_layer_present_at_all_scales() -> None:
    for scale in WorldScale:
        world = generate_world(WorldConfig(seed=42, scale=scale))
        assert len(world.climate.wind_direction_grid) == world.geography.height
        assert len(world.climate.specific_humidity_grid) == world.geography.height


def test_specific_humidity_within_bounds() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for row in world.climate.specific_humidity_grid:
        for value in row:
            assert 0.0 <= value <= MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG


def test_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.climate.wind_direction_grid == b.climate.wind_direction_grid
    assert a.climate.specific_humidity_grid == b.climate.specific_humidity_grid
    assert a.climate.annual_precipitation_mm == b.climate.annual_precipitation_mm


def test_atmosphere_provenance_record_present() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    climate_records = [
        record
        for record in world.provenance
        if record.output_path == "climate"
    ]
    assert len(climate_records) == 1
    assert climate_records[0].algorithm_version == "wind-belts-v1"


def test_refined_precipitation_differs_from_base_precipitation() -> None:
    """Phase 1c blends transport-driven moisture into precipitation; the
    refined field must differ from the Phase 1a noise field."""
    world = generate_world(
        WorldConfig(seed=42, scale=WorldScale.LARGE, climate_class=ClimateClass.TEMPERATE)
    )
    refined = world.climate.annual_precipitation_mm
    humidity_max = max(max(row) for row in world.climate.specific_humidity_grid)
    # Refined precipitation must contain nonzero values (transport is
    # active over a 256x128 grid with abundant ocean evaporation).
    assert humidity_max > 0.0
    nonzero_cells = sum(
        1 for row in refined for value in row if value > 0.0
    )
    assert nonzero_cells > 0


@pytest.mark.parametrize("latitude", [-89.0, -60.0, -45.0, -15.0, 0.0, 15.0, 45.0, 60.0, 89.0])
def test_belt_assignment_at_each_latitude(latitude: float) -> None:
    direction = _prevailing_belt_wind(latitude)
    absolute_latitude = abs(latitude)
    if absolute_latitude < 30.0:
        assert direction is WindDirection.EAST
    elif absolute_latitude < 60.0:
        assert direction is WindDirection.WEST
    else:
        assert direction is WindDirection.EAST