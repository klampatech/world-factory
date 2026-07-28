"""Deterministic generation pipeline for the world factory."""

import hashlib
import json
import math
from collections.abc import Callable

from world_factory.atmosphere import atmospheric_pressure_grid
from world_factory.constants import (
    CONTINENTAL_INTERIOR_BASE_ELEVATION_METERS,
    CONVERGENT_BOUNDARY_UPLIFT_METERS,
    DETERMINISTIC_ALGORITHM_VERSION,
    DIVERGENT_BOUNDARY_RIFT_METERS,
    ELEVATION_NOISE_RANGE_METERS,
    MAXIMUM_ELEVATION_METERS,
    MINIMUM_ELEVATION_METERS,
    MODEL_VERSION,
    OCEANIC_INTERIOR_BASE_ELEVATION_METERS,
    SCHEMA_VERSION,
)
from world_factory.determinism import sample_unit_interval
from world_factory.geology import generate_geology
from world_factory.hydrology import generate_hydrology, hydrology_provenance
from world_factory.models import (
    BiomeClass,
    BiomeLayer,
    BoundaryType,
    ClimateClass,
    ClimateLayer,
    GeographyLayer,
    GeologyLayer,
    ProvenanceRecord,
    WorldConfig,
    WorldMetadata,
    WorldModel,
    WorldScale,
)

_GRID_DIMENSIONS = {
    WorldScale.SMALL: (24, 12),
    WorldScale.MEDIUM: (48, 24),
    WorldScale.LARGE: (256, 128),
}
_CLIMATE_BASE_TEMPERATURE_CELSIUS = {
    ClimateClass.COLD: 2.0,
    ClimateClass.TEMPERATE: 15.0,
    ClimateClass.HOT: 28.0,
}
_SEA_LEVEL_METERS = 0.0
_ELEVATION_LAPSE_RATE_CELSIUS_PER_METER = 0.0065

FloatGrid = tuple[tuple[float, ...], ...]


def generate_world(config: WorldConfig) -> WorldModel:
    """Generate a deterministic, physically coherent world from parameters."""
    width, height = _GRID_DIMENSIONS[config.scale]
    geology = generate_geology(config.seed, config.plate_count, config.scale)
    elevation = _generate_elevation(config.seed, geology)
    temperature = _generate_temperature(config, elevation)
    precipitation = _generate_precipitation(config.seed, elevation)
    geography = GeographyLayer(
        width=width,
        height=height,
        sea_level_meters=_SEA_LEVEL_METERS,
        elevation_meters=elevation,
    )
    climate = ClimateLayer(
        atmospheric_pressure_kpa=atmospheric_pressure_grid(elevation),
        temperature_celsius=temperature,
        annual_precipitation_mm=precipitation,
    )
    hydrology = generate_hydrology(
        elevation=elevation,
        precipitation=precipitation,
        sea_level=_SEA_LEVEL_METERS,
        seed=config.seed,
    )
    return WorldModel(
        metadata=_create_metadata(config),
        geology=geology,
        geography=geography,
        hydrology=hydrology,
        climate=climate,
        biomes=BiomeLayer(
            classifications=_classify_biomes(elevation, temperature, precipitation)
        ),
        provenance=_create_provenance(),
    )


def _generate_grid(width: int, height: int, cell: Callable[[int, int], float]) -> FloatGrid:
    """Build a rounded immutable grid in row-major order."""
    return tuple(tuple(round(cell(x, y), 6) for x in range(width)) for y in range(height))


def _generate_elevation(seed: int, geology: GeologyLayer) -> FloatGrid:
    """Derive elevation from plate composition, boundaries, and deterministic noise."""
    plate_types = {plate.id: plate.plate_type for plate in geology.plates}
    plate_by_id = {plate.id: plate for plate in geology.plates}
    boundary_types = geology.boundary_type_grid
    width, height = geology.width, geology.height

    def elevation_at(x: int, y: int) -> float:
        plate_id = geology.plate_id_grid[y][x]
        plate_type = plate_types[plate_id]
        base = (
            CONTINENTAL_INTERIOR_BASE_ELEVATION_METERS
            if plate_type.value == "continental"
            else OCEANIC_INTERIOR_BASE_ELEVATION_METERS
        )
        boundary = boundary_types[y][x]
        uplift = 0.0
        if boundary is BoundaryType.CONVERGENT:
            neighbor_id = _neighbor_id_on_boundary(geology.plate_id_grid, x, y, plate_id)
            neighbor = plate_by_id.get(neighbor_id) if neighbor_id is not None else None
            if neighbor is not None and neighbor.plate_type.value == "continental":
                uplift = CONVERGENT_BOUNDARY_UPLIFT_METERS
            else:
                uplift = CONVERGENT_BOUNDARY_UPLIFT_METERS * 0.4
        elif boundary is BoundaryType.DIVERGENT:
            uplift = DIVERGENT_BOUNDARY_RIFT_METERS
        noise = (
            sample_unit_interval(seed, "geography.elevation", x, y) - 0.5
        ) * ELEVATION_NOISE_RANGE_METERS
        latitude = ((y + 0.5) / height) * math.pi - math.pi / 2.0
        longitudinal_variation = (
            math.sin((x / width) * math.tau * 2.0) * math.cos(latitude) * 700.0
        )
        return min(
            MAXIMUM_ELEVATION_METERS,
            max(
                MINIMUM_ELEVATION_METERS,
                base + uplift + noise + longitudinal_variation,
            ),
        )

    return _generate_grid(width, height, elevation_at)


def _neighbor_id_on_boundary(
    plate_id_grid: tuple[tuple[int, ...], ...],
    x: int,
    y: int,
    plate_id: int,
) -> int | None:
    """Return a neighboring plate id on a boundary cell, or None."""
    height = len(plate_id_grid)
    width = len(plate_id_grid[0])
    for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        nx, ny = x + dx, y + dy
        if 0 <= nx < width and 0 <= ny < height:
            neighbor_id = plate_id_grid[ny][nx]
            if neighbor_id != plate_id:
                return neighbor_id
    return None


def _generate_temperature(config: WorldConfig, elevation: FloatGrid) -> FloatGrid:
    """Approximate temperature from latitude, climate class, and lapse rate."""
    height = len(elevation)
    base_temperature = _CLIMATE_BASE_TEMPERATURE_CELSIUS[config.climate_class]

    def temperature_at(x: int, y: int) -> float:
        latitude_factor = abs(((y + 0.5) / height) * 2.0 - 1.0)
        return (
            base_temperature
            - latitude_factor * 38.0
            - max(elevation[y][x], 0.0) * _ELEVATION_LAPSE_RATE_CELSIUS_PER_METER
        )

    return _generate_grid(len(elevation[0]), height, temperature_at)


def _generate_precipitation(seed: int, elevation: FloatGrid) -> FloatGrid:
    """Generate a bounded deterministic precipitation field."""
    height, width = len(elevation), len(elevation[0])

    def precipitation_at(x: int, y: int) -> float:
        moisture = sample_unit_interval(seed, "climate.precipitation", x, y)
        return max(0.0, 250.0 + moisture * 2_200.0 - max(elevation[y][x], 0.0) * 0.12)

    return _generate_grid(width, height, precipitation_at)


def _classify_biomes(
    elevation: FloatGrid,
    temperature: FloatGrid,
    precipitation: FloatGrid,
) -> tuple[tuple[BiomeClass, ...], ...]:
    """Classify each cell using elevation, temperature, and precipitation."""
    return tuple(
        tuple(
            _classify_biome(
                elevation[y][x], temperature[y][x], precipitation[y][x]
            )
            for x in range(len(elevation[y]))
        )
        for y in range(len(elevation))
    )


def _classify_biome(elevation: float, temperature: float, precipitation: float) -> BiomeClass:
    """Return the first matching physical biome class."""
    if elevation <= _SEA_LEVEL_METERS:
        return BiomeClass.OCEAN
    if temperature < -10.0:
        return BiomeClass.ICE
    if elevation > 2_500.0:
        return BiomeClass.ALPINE
    if precipitation < 350.0:
        return BiomeClass.DESERT
    if temperature > 20.0 and precipitation > 1_400.0:
        return BiomeClass.TROPICAL_FOREST
    if precipitation > 900.0:
        return BiomeClass.TEMPERATE_FOREST
    return BiomeClass.GRASSLAND


def _create_metadata(config: WorldConfig) -> WorldMetadata:
    """Create stable identity metadata from canonical configuration JSON."""
    canonical_config = json.dumps(
        config.model_dump(mode="json"), sort_keys=True, separators=(",", ":")
    )
    world_id = hashlib.blake2b(canonical_config.encode(), digest_size=16).hexdigest()
    return WorldMetadata(
        world_id=world_id,
        schema_version=SCHEMA_VERSION,
        model_version=MODEL_VERSION,
        config=config,
    )


def _create_provenance() -> tuple[ProvenanceRecord, ...]:
    """Describe the algorithms and inputs for generated physical layers."""
    algorithm = DETERMINISTIC_ALGORITHM_VERSION
    return (
        ProvenanceRecord(
            output_path="geology",
            process="tectonic-voronoi",
            input_paths=("metadata.config.seed", "metadata.config.plate_count"),
            algorithm_version=algorithm,
        ),
        ProvenanceRecord(
            output_path="geography.elevation_meters",
            process="plate-uplift-heightfield",
            input_paths=("geology", "metadata.config.seed"),
            algorithm_version=algorithm,
        ),
        hydrology_provenance(),
        ProvenanceRecord(
            output_path="climate",
            process="barometric-latitude-climate",
            input_paths=("geography.elevation_meters", "metadata.config.climate_class"),
            algorithm_version=algorithm,
        ),
        ProvenanceRecord(
            output_path="biomes.classifications",
            process="physical-biome-classifier",
            input_paths=("geography", "climate"),
            algorithm_version=algorithm,
        ),
    )
