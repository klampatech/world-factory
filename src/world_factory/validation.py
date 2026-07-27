"""Cross-layer plausibility checks for generated worlds."""

from pydantic import Field

from world_factory.constants import (
    MAXIMUM_ELEVATION_METERS,
    MAXIMUM_OCEAN_FRACTION,
    MAXIMUM_SURFACE_TEMPERATURE_CELSIUS,
    MINIMUM_ELEVATION_METERS,
    MINIMUM_OCEAN_FRACTION,
    MINIMUM_SURFACE_TEMPERATURE_CELSIUS,
)
from world_factory.models import StrictModel, WorldModel


class InvariantViolation(StrictModel):
    """A machine-readable cross-layer plausibility failure."""

    code: str
    path: str
    message: str


class ValidationReport(StrictModel):
    """Complete result of evaluating the Phase 0 invariant set."""

    is_valid: bool
    violations: tuple[InvariantViolation, ...] = Field(default_factory=tuple)


def validate_world(world: WorldModel) -> ValidationReport:
    """Evaluate dimensions, physical bounds, and provenance coverage."""
    violations = [
        *_validate_grid_dimensions(world),
        *_validate_elevation(world),
        *_validate_climate(world),
        *_validate_surface_water(world),
        *_validate_provenance(world),
    ]
    return ValidationReport(is_valid=not violations, violations=tuple(violations))


def _validate_grid_dimensions(world: WorldModel) -> list[InvariantViolation]:
    expected = (world.geography.height, world.geography.width)
    grids = {
        "geography.elevation_meters": world.geography.elevation_meters,
        "climate.temperature_celsius": world.climate.temperature_celsius,
        "climate.annual_precipitation_mm": world.climate.annual_precipitation_mm,
        "biomes.classifications": world.biomes.classifications,
    }
    violations: list[InvariantViolation] = []
    for path, grid in grids.items():
        actual = (len(grid), len(grid[0]) if grid else 0)
        if actual != expected or any(len(row) != expected[1] for row in grid):
            violations.append(
                _violation("grid-shape", path, f"expected {expected}, found {actual}")
            )
    return violations


def _validate_elevation(world: WorldModel) -> list[InvariantViolation]:
    values = [value for row in world.geography.elevation_meters for value in row]
    if (
        values
        and min(values) >= MINIMUM_ELEVATION_METERS
        and max(values) <= MAXIMUM_ELEVATION_METERS
    ):
        return []
    return [
        _violation(
            "elevation-bounds", "geography.elevation_meters", "elevation is outside model bounds"
        )
    ]


def _validate_climate(world: WorldModel) -> list[InvariantViolation]:
    temperatures = [value for row in world.climate.temperature_celsius for value in row]
    precipitation = [value for row in world.climate.annual_precipitation_mm for value in row]
    violations: list[InvariantViolation] = []
    if (
        not temperatures
        or min(temperatures) < MINIMUM_SURFACE_TEMPERATURE_CELSIUS
        or max(temperatures) > MAXIMUM_SURFACE_TEMPERATURE_CELSIUS
    ):
        violations.append(
            _violation(
                "temperature-bounds",
                "climate.temperature_celsius",
                "temperature is outside model bounds",
            )
        )
    if not precipitation or min(precipitation) < 0.0:
        violations.append(
            _violation(
                "negative-precipitation",
                "climate.annual_precipitation_mm",
                "precipitation cannot be negative",
            )
        )
    return violations


def _validate_surface_water(world: WorldModel) -> list[InvariantViolation]:
    fraction = world.hydrology.surface_water_fraction
    if MINIMUM_OCEAN_FRACTION <= fraction <= MAXIMUM_OCEAN_FRACTION:
        return []
    return [
        _violation(
            "surface-water-fraction",
            "hydrology.surface_water_fraction",
            "surface water fraction is implausible for the Phase 0 Earth analog",
        )
    ]


def _validate_provenance(world: WorldModel) -> list[InvariantViolation]:
    required_paths = {"geography.elevation_meters", "climate", "biomes.classifications"}
    recorded_paths = {record.output_path for record in world.provenance}
    missing = sorted(required_paths - recorded_paths)
    if not missing:
        return []
    return [
        _violation("missing-provenance", "provenance", f"missing records for: {', '.join(missing)}")
    ]


def _violation(code: str, path: str, message: str) -> InvariantViolation:
    return InvariantViolation(code=code, path=path, message=message)
