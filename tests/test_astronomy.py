"""Phase 1d astronomy invariants: axial tilt, day length, insolation."""

import pytest

from world_factory.astronomy import (
    _day_length_hours,
    _insolation_factor,
    _latitude_degrees,
    _solar_declination_degrees,
    build_astronomy,
)
from world_factory.constants import EARTH_AXIAL_TILT_DEGREES
from world_factory.generator import generate_world
from world_factory.models import WorldConfig, WorldScale


def test_solar_declination_zero_at_spring_equinox() -> None:
    assert _solar_declination_degrees(23.5, 0.0, 365.25) == 0.0


def test_solar_declination_max_at_summer_solstice() -> None:
    declination = _solar_declination_degrees(EARTH_AXIAL_TILT_DEGREES, 365.25 / 4.0, 365.25)
    assert abs(declination - EARTH_AXIAL_TILT_DEGREES) < 1e-6


def test_solar_declination_min_at_winter_solstice() -> None:
    declination = _solar_declination_degrees(EARTH_AXIAL_TILT_DEGREES, 3 * 365.25 / 4.0, 365.25)
    assert abs(declination - (-EARTH_AXIAL_TILT_DEGREES)) < 1e-6


def test_latitude_mapping() -> None:
    assert _latitude_degrees(0, 12) < -80.0
    assert _latitude_degrees(11, 12) > 80.0


def test_day_length_at_equator_is_12_hours_at_equinox() -> None:
    assert abs(_day_length_hours(0.0, 0.0) - 12.0) < 1e-6


def test_day_length_at_equator_in_summer() -> None:
    assert _day_length_hours(0.0, EARTH_AXIAL_TILT_DEGREES) == pytest.approx(12.0, abs=1e-3)


def test_day_length_at_polar_region_in_summer_is_24_hours() -> None:
    """At latitudes above 90 - axial_tilt, summer day length saturates at
    24 hours (midnight sun)."""
    assert _day_length_hours(80.0, EARTH_AXIAL_TILT_DEGREES) == 24.0


def test_day_length_at_polar_region_in_winter_is_zero_hours() -> None:
    assert _day_length_hours(80.0, -EARTH_AXIAL_TILT_DEGREES) == 0.0


def test_day_length_at_southern_polar_region_in_winter_is_zero_hours() -> None:
    """Southern hemisphere, NH summer solstice (declination +23.5°)
    is SH winter, so -80° latitude has 0 hours of daylight."""
    assert _day_length_hours(-80.0, EARTH_AXIAL_TILT_DEGREES) == 0.0


def test_insolation_at_subsolar_point_is_one() -> None:
    assert _insolation_factor(0.0, 0.0) == pytest.approx(1.0, abs=1e-6)


def test_insolation_at_antisolar_pole_is_zero() -> None:
    assert _insolation_factor(90.0, -90.0) == 0.0


def test_build_astronomy_returns_correct_shape() -> None:
    layer = build_astronomy(
        width=10,
        height=5,
        axial_tilt_degrees=23.5,
        orbital_eccentricity=0.0167,
        season_day=0.0,
        orbital_period_days=365.25,
    )
    assert len(layer.day_length_hours) == 5
    assert len(layer.day_length_hours[0]) == 10
    assert len(layer.insolation_factor) == 5
    assert len(layer.insolation_factor[0]) == 10


def test_build_astronomy_day_length_within_bounds() -> None:
    layer = build_astronomy(
        width=24,
        height=12,
        axial_tilt_degrees=23.5,
        orbital_eccentricity=0.0167,
        season_day=0.0,
        orbital_period_days=365.25,
    )
    for row in layer.day_length_hours:
        for value in row:
            assert 0.0 <= value <= 24.0


def test_build_astronomy_insolation_within_bounds() -> None:
    layer = build_astronomy(
        width=24,
        height=12,
        axial_tilt_degrees=23.5,
        orbital_eccentricity=0.0167,
        season_day=0.0,
        orbital_period_days=365.25,
    )
    for row in layer.insolation_factor:
        for value in row:
            assert 0.0 <= value <= 1.0


def test_world_model_includes_astronomy_layer() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.astronomy is not None
    assert world.astronomy.axial_tilt_degrees == 23.5
    assert world.astronomy.orbital_eccentricity == 0.0167
    assert world.astronomy.season_day == 0.0


def test_astronomy_grid_shapes_match_geography() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    height = world.geography.height
    width = world.geography.width
    assert len(world.astronomy.day_length_hours) == height
    assert len(world.astronomy.day_length_hours[0]) == width
    assert len(world.astronomy.insolation_factor) == height
    assert len(world.astronomy.insolation_factor[0]) == width


def test_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.astronomy.day_length_hours == b.astronomy.day_length_hours
    assert a.astronomy.insolation_factor == b.astronomy.insolation_factor


def test_season_day_changes_solar_declination() -> None:
    """Different season_day values produce different solar declinations,
    which propagate to different per-cell temperatures."""
    spring = generate_world(WorldConfig(seed=42, season_day=0.0))
    summer = generate_world(WorldConfig(seed=42, season_day=91.31))
    assert spring.astronomy.solar_declination_degrees == pytest.approx(0.0, abs=1e-6)
    assert summer.astronomy.solar_declination_degrees == pytest.approx(
        EARTH_AXIAL_TILT_DEGREES, abs=1e-6
    )


def test_season_day_propagates_to_temperature_grid() -> None:
    spring = generate_world(WorldConfig(seed=42, season_day=0.0))
    summer = generate_world(WorldConfig(seed=42, season_day=91.31))
    spring_temps = [
        value for row in spring.climate.temperature_celsius for value in row
    ]
    summer_temps = [
        value for row in summer.climate.temperature_celsius for value in row
    ]
    assert spring_temps != summer_temps