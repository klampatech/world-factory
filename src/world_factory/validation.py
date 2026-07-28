"""Cross-layer plausibility checks for generated worlds."""

from pydantic import Field

from world_factory.constants import (
    MAXIMUM_ELEVATION_METERS,
    MAXIMUM_OCEAN_FRACTION,
    MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG,
    MAXIMUM_SURFACE_TEMPERATURE_CELSIUS,
    MINIMUM_ATMOSPHERIC_PRESSURE_KPA,
    MINIMUM_ELEVATION_METERS,
    MINIMUM_OCEAN_FRACTION,
    MINIMUM_SURFACE_TEMPERATURE_CELSIUS,
)
from world_factory.models import RiverSegment, StrictModel, WindDirection, WorldModel


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
        *_validate_atmosphere(world),
        *_validate_surface_water(world),
        *_validate_hydrology(world),
        *_validate_astronomy(world),
        *_validate_provenance(world),
    ]
    return ValidationReport(is_valid=not violations, violations=tuple(violations))


def _validate_grid_dimensions(world: WorldModel) -> list[InvariantViolation]:
    expected = (world.geography.height, world.geography.width)
    grids = {
        "geology.plate_id_grid": world.geology.plate_id_grid,
        "geology.boundary_type_grid": world.geology.boundary_type_grid,
        "geography.elevation_meters": world.geography.elevation_meters,
        "climate.atmospheric_pressure_kpa": world.climate.atmospheric_pressure_kpa,
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
    pressure = [value for row in world.climate.atmospheric_pressure_kpa for value in row]
    if not pressure or min(pressure) < MINIMUM_ATMOSPHERIC_PRESSURE_KPA:
        violations.append(
            _violation(
                "pressure-bounds",
                "climate.atmospheric_pressure_kpa",
                "atmospheric pressure is below the model minimum",
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


def _validate_atmosphere(world: WorldModel) -> list[InvariantViolation]:
    """Phase 1c wind-direction and humidity bounds."""
    violations: list[InvariantViolation] = []
    valid_directions = set(WindDirection)
    for y, row in enumerate(world.climate.wind_direction_grid):
        for x, direction in enumerate(row):
            if direction not in valid_directions:
                violations.append(
                    _violation(
                        "wind-direction-invalid",
                        f"climate.wind_direction_grid[{y}][{x}]",
                        f"wind direction {direction!r} is not a valid WindDirection",
                    )
                )
    for y, humidity_row in enumerate(world.climate.specific_humidity_grid):
        for x, value in enumerate(humidity_row):
            if value < 0.0 or value > MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG:
                violations.append(
                    _violation(
                        "specific-humidity-bounds",
                        f"climate.specific_humidity_grid[{y}][{x}]",
                        f"humidity {value} outside [0, {MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG}]",
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


def _validate_hydrology(world: WorldModel) -> list[InvariantViolation]:
    """P1: hydrographic consistency — every river mouth at sea level,
    river lengths and discharges are positive, watershed labels are
    consistent."""
    violations: list[InvariantViolation] = []
    sea_level = world.geography.sea_level_meters
    elevation = world.geography.elevation_meters
    segment: RiverSegment
    for segment in world.hydrology.river_segments:
        mouth_x, mouth_y = segment.mouth
        if not (0 <= mouth_y < len(elevation) and 0 <= mouth_x < len(elevation[0])):
            violations.append(
                _violation(
                    "river-mouth-out-of-bounds",
                    f"hydrology.river_segments.{segment.id}.mouth",
                    f"mouth ({mouth_x}, {mouth_y}) is outside the grid",
                )
            )
            continue
        if elevation[mouth_y][mouth_x] > sea_level:
            violations.append(
                _violation(
                    "river-mouth-above-sea-level",
                    f"hydrology.river_segments.{segment.id}.mouth",
                    (
                        f"mouth ({mouth_x}, {mouth_y}) elevation "
                        f"{elevation[mouth_y][mouth_x]:.2f}m > sea level {sea_level:.2f}m"
                    ),
                )
            )
        if segment.length_cells < 1:
            violations.append(
                _violation(
                    "river-length-non-positive",
                    f"hydrology.river_segments.{segment.id}.length_cells",
                    f"river length {segment.length_cells} must be ≥ 1",
                )
            )
        if segment.mean_discharge < 0.0:
            violations.append(
                _violation(
                    "river-discharge-negative",
                    f"hydrology.river_segments.{segment.id}.mean_discharge",
                    f"river mean discharge {segment.mean_discharge} is negative",
                )
            )
    for y, row in enumerate(world.hydrology.discharge_grid):
        for x, value in enumerate(row):
            if value < 0.0:
                violations.append(
                    _violation(
                        "discharge-negative",
                        f"hydrology.discharge_grid[{y}][{x}]",
                        f"discharge {value} is negative",
                    )
                )
    width = len(world.hydrology.watershed_id_grid[0])
    for y, ws_row in enumerate(world.hydrology.watershed_id_grid):
        for x, ws_label in enumerate(ws_row):
            if len(ws_row) != width:
                continue
            is_ocean = elevation[y][x] <= sea_level
            if is_ocean and ws_label is not None:
                violations.append(
                    _violation(
                        "watershed-label-on-ocean",
                        f"hydrology.watershed_id_grid[{y}][{x}]",
                        f"ocean cell carries watershed id {ws_label}",
                    )
                )
            if not is_ocean and ws_label is None:
                violations.append(
                    _violation(
                        "missing-watershed-label",
                        f"hydrology.watershed_id_grid[{y}][{x}]",
                        "land cell carries no watershed id",
                    )
                )
    return violations


def _validate_astronomy(world: WorldModel) -> list[InvariantViolation]:
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


def _validate_provenance(world: WorldModel) -> list[InvariantViolation]:
    required_paths = {
        "geography.elevation_meters",
        "astronomy",
        "hydrology",
        "climate",
        "biomes.classifications",
    }
    recorded_paths = {record.output_path for record in world.provenance}
    missing = sorted(required_paths - recorded_paths)
    if not missing:
        return []
    return [
        _violation("missing-provenance", "provenance", f"missing records for: {', '.join(missing)}")
    ]


def _violation(code: str, path: str, message: str) -> InvariantViolation:
    return InvariantViolation(code=code, path=path, message=message)
