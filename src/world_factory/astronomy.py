"""Astronomical forcing: axial tilt, day length, insolation, season.

Phase 1d derives:

1. **Solar declination** — angle between the sun's rays and Earth's
   equatorial plane. Driven by `axial_tilt_degrees` and `season_day`
   via `δ = T × sin(2π × season_day / orbital_period_days)`.
2. **Day length** — per-cell hours of daylight. Standard formula
   `ω = arccos(-tan(latitude) × tan(declination))` →
   `day_length = 24 × ω / π`. Clamped to `[0, 24]` to handle polar
   night and midnight sun correctly.
3. **Insolation factor** — per-cell normalized 0–1 value. The
   sub-solar point reads 1.0; the antisolar pole reads 0.0.

The astronomy module is purely a function of WorldConfig parameters;
no RNG draws are required (no per-cell randomness).
"""

import math

from world_factory.constants import ASTRONOMY_ALGORITHM_VERSION, EARTH_ROTATION_PERIOD_HOURS
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import AstronomyLayer, ProvenanceRecord, WorldModel

FloatGrid = tuple[tuple[float, ...], ...]

_DAY_LENGTH_MIN_HOURS = 0.0
_DAY_LENGTH_MAX_HOURS = 24.0


def _solar_declination_degrees(
    axial_tilt_degrees: float,
    season_day: float,
    orbital_period_days: float,
) -> float:
    """Solar declination from axial tilt and season day."""
    if orbital_period_days <= 0.0:
        return 0.0
    angle = (math.tau * season_day) / orbital_period_days
    return axial_tilt_degrees * math.sin(angle)


def _latitude_degrees(y: int, height: int) -> float:
    """Latitude from row index. y=0 is south pole (-90°), y=height-1
    is north pole (+90°)."""
    return -90.0 + (y + 0.5) / height * 180.0


def _day_length_hours(latitude_degrees: float, solar_declination_degrees: float) -> float:
    """Hours of daylight for the given latitude and solar declination.

    Standard formula: cos(ω) = −tan(φ) · tan(δ), where ω is the
    half-day-length in radians. Day length = 24 · ω / π.

    When |−tan(φ) · tan(δ)| > 1, the sun never rises (polar night,
    argument > 1) or never sets (midnight sun, argument < −1).
    Those cases are clamped to 0 and 24 hours respectively so the
    formula never returns NaN.
    """
    latitude_radians = math.radians(latitude_degrees)
    declination_radians = math.radians(solar_declination_degrees)
    tan_latitude = math.tan(latitude_radians)
    tan_declination = math.tan(declination_radians)
    argument = -tan_latitude * tan_declination
    if argument > 1.0:
        return _DAY_LENGTH_MIN_HOURS
    if argument < -1.0:
        return _DAY_LENGTH_MAX_HOURS
    omega = math.acos(argument)
    return EARTH_ROTATION_PERIOD_HOURS * omega / math.pi


def _insolation_factor(latitude_degrees: float, solar_declination_degrees: float) -> float:
    """Normalized 0-1 insolation. Sub-solar point = 1, antisolar
    pole = 0. Sub-solar point is at latitude = declination, so the
    angular distance is `|latitude − declination|`. We use cos to
    model Lambert's law for direct-beam intensity."""
    return max(
        0.0,
        math.cos(math.radians(latitude_degrees - solar_declination_degrees)),
    )


def _astronomy_grids(
    height: int,
    width: int,
    solar_declination_degrees: float,
) -> tuple[FloatGrid, FloatGrid]:
    day_length: list[list[float]] = []
    insolation: list[list[float]] = []
    for y in range(height):
        day_length_row: list[float] = []
        insolation_row: list[float] = []
        for _x in range(width):
            latitude = _latitude_degrees(y, height)
            length = _day_length_hours(latitude, solar_declination_degrees)
            length = max(_DAY_LENGTH_MIN_HOURS, min(_DAY_LENGTH_MAX_HOURS, length))
            insolation_value = _insolation_factor(latitude, solar_declination_degrees)
            day_length_row.append(round(length, 6))
            insolation_row.append(round(insolation_value, 6))
        day_length.append(day_length_row)
        insolation.append(insolation_row)
    return (
        tuple(tuple(row) for row in day_length),
        tuple(tuple(row) for row in insolation),
    )


def build_astronomy(
    width: int,
    height: int,
    axial_tilt_degrees: float,
    orbital_eccentricity: float,
    season_day: float,
    orbital_period_days: float,
) -> AstronomyLayer:
    """Build the Phase 1d astronomy layer from WorldConfig parameters."""
    solar_declination = _solar_declination_degrees(
        axial_tilt_degrees=axial_tilt_degrees,
        season_day=season_day,
        orbital_period_days=orbital_period_days,
    )
    day_length, insolation = _astronomy_grids(
        width=width,
        height=height,
        solar_declination_degrees=solar_declination,
    )
    return AstronomyLayer(
        axial_tilt_degrees=axial_tilt_degrees,
        orbital_eccentricity=orbital_eccentricity,
        season_day=season_day,
        solar_declination_degrees=solar_declination,
        day_length_hours=day_length,
        insolation_factor=insolation,
    )


def astronomy_provenance() -> ProvenanceRecord:
    """Provenance record describing the astronomy algorithm."""
    return ProvenanceRecord(
        output_path="astronomy",
        process="axial-tilt-with-seasonal-forcing",
        input_paths=(
            "metadata.config.axial_tilt_degrees",
            "metadata.config.orbital_eccentricity",
            "metadata.config.season_day",
            "metadata.config.orbital_period_days",
        ),
        algorithm_version=ASTRONOMY_ALGORITHM_VERSION,
    )


def validate_astronomy_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 1d astronomy bounds."""
    violations: list[InvariantViolation] = []
    height = world.geography.height
    if len(world.astronomy.day_length_hours) != height:
        violations.append(
            _violation(
                "day-length-grid-shape",
                "astronomy.day_length_hours",
                f"expected {height} rows, found {len(world.astronomy.day_length_hours)}",
            )
        )
    if len(world.astronomy.insolation_factor) != height:
        violations.append(
            _violation(
                "insolation-grid-shape",
                "astronomy.insolation_factor",
                f"expected {height} rows, found {len(world.astronomy.insolation_factor)}",
            )
        )
    for y, row in enumerate(world.astronomy.day_length_hours):
        for x, value in enumerate(row):
            if value < 0.0 or value > 24.0:
                violations.append(
                    _violation(
                        "day-length-bounds",
                        f"astronomy.day_length_hours[{y}][{x}]",
                        f"value {value} outside [0, 24]",
                    )
                )
    for y, row in enumerate(world.astronomy.insolation_factor):
        for x, value in enumerate(row):
            if value < 0.0 or value > 1.0:
                violations.append(
                    _violation(
                        "insolation-bounds",
                        f"astronomy.insolation_factor[{y}][{x}]",
                        f"value {value} outside [0, 1]",
                    )
                )
    axial_tilt = world.astronomy.axial_tilt_degrees
    declination = world.astronomy.solar_declination_degrees
    if declination < -axial_tilt - 1e-6 or declination > axial_tilt + 1e-6:
        violations.append(
            _violation(
                "solar-declination-bounds",
                "astronomy.solar_declination_degrees",
                f"value {declination} outside [-{axial_tilt}, +{axial_tilt}]",
            )
        )
    return violations