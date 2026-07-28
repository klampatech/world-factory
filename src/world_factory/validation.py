"""Cross-layer plausibility checks for generated worlds.

The orchestrating `validate_world` calls per-layer validator
functions exported from each layer module
(`world_factory.atmosphere.validate_atmosphere_layer`,
`world_factory.astronomy.validate_astronomy_layer`,
`world_factory.geology.validate_geology_sublayer_shapes`,
`world_factory.hydrology.validate_hydrology_layer`). This module
holds only the cross-cutting invariants (grid-shape parity,
provenance coverage). The shared `InvariantViolation` /
`ValidationReport` types live in `world_factory.invariants` so the
per-layer modules can import them without circularity.
"""

from world_factory.agriculture import validate_agriculture_layer
from world_factory.astronomy import validate_astronomy_layer
from world_factory.atmosphere import validate_atmosphere_layer
from world_factory.biology import validate_biology_layer
from world_factory.constants import (
    MAXIMUM_ELEVATION_METERS,
    MAXIMUM_OCEAN_FRACTION,
    MAXIMUM_SURFACE_TEMPERATURE_CELSIUS,
    MINIMUM_ATMOSPHERIC_PRESSURE_KPA,
    MINIMUM_ELEVATION_METERS,
    MINIMUM_OCEAN_FRACTION,
    MINIMUM_SURFACE_TEMPERATURE_CELSIUS,
)
from world_factory.demography import validate_demography_layer
from world_factory.event_log import validate_event_log
from world_factory.geology import validate_geology_sublayer_shapes
from world_factory.hydrology import validate_hydrology_layer
from world_factory.infrastructure import validate_infrastructure_layer
from world_factory.invariants import InvariantViolation, ValidationReport
from world_factory.models import WorldModel
from world_factory.queries import validate_query_surface
from world_factory.settlements import validate_settlements_layer

__all__ = [
    "InvariantViolation",
    "ValidationReport",
    "validate_world",
]


def validate_world(world: WorldModel) -> ValidationReport:
    """Evaluate dimensions, physical bounds, and provenance coverage.

    The validator delegates to per-layer modules for layer-specific
    invariants (geology sublayers, atmosphere, astronomy,
    hydrology, biology) and handles the cross-cutting grid-shape
    and provenance invariants here.
    """
    violations = [
        *_validate_grid_dimensions(world),
        *_validate_elevation(world),
        *_validate_climate(world),
        *validate_atmosphere_layer(world),
        *validate_hydrology_layer(world),
        *validate_astronomy_layer(world),
        *validate_geology_sublayer_shapes(world),
        *validate_biology_layer(world),
        *validate_settlements_layer(world),
        *validate_agriculture_layer(world),
        *validate_infrastructure_layer(world),
        *validate_demography_layer(world),
        *validate_event_log(world),
        *validate_query_surface(world),
        *_validate_provenance(world),
    ]
    return ValidationReport(is_valid=not violations, violations=tuple(violations))


def _violation(code: str, path: str, message: str) -> InvariantViolation:
    return InvariantViolation(code=code, path=path, message=message)


def _validate_grid_dimensions(world: WorldModel) -> list[InvariantViolation]:
    expected = (world.geography.height, world.geography.width)
    grids = {
        "geology.plate_id_grid": world.geology.plate_id_grid,
        "geology.boundary_type_grid": world.geology.boundary_type_grid,
        "geology.rock_type_grid": world.geology.rock_type_grid,
        "geology.ore_presence_grid": world.geology.ore_presence_grid,
        "geology.soil_type_grid": world.geology.soil_type_grid,
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
    surface_water = world.hydrology.surface_water_fraction
    if not (
        MINIMUM_OCEAN_FRACTION <= surface_water <= MAXIMUM_OCEAN_FRACTION
    ):
        violations.append(
            _violation(
                "surface-water-fraction",
                "hydrology.surface_water_fraction",
                "surface water fraction is implausible for the Phase 0 Earth analog",
            )
        )
    return violations


def _validate_provenance(world: WorldModel) -> list[InvariantViolation]:
    required_paths = {
        "geography.elevation_meters",
        "geology.sublayers",
        "astronomy",
        "hydrology",
        "climate",
        "biomes.classifications",
        "biology",
        "settlements",
        "agriculture",
        "infrastructure",
        "demography",
        "events",
    }
    recorded_paths = {record.output_path for record in world.provenance}
    missing = sorted(required_paths - recorded_paths)
    if not missing:
        return []
    return [
        _violation("missing-provenance", "provenance", f"missing records for: {', '.join(missing)}")
    ]
