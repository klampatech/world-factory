"""Deterministic generation pipeline for the Phase 0 world seam."""

import hashlib
import json
import math
from collections.abc import Callable

from world_factory.constants import (
    MAXIMUM_ELEVATION_METERS,
    MINIMUM_ELEVATION_METERS,
    MODEL_VERSION,
    SCHEMA_VERSION,
    STANDARD_ATMOSPHERIC_PRESSURE_KPA,
)
from world_factory.determinism import deterministic_algorithm_version, sample_unit_interval
from world_factory.models import (
    BiomeLayer,
    ClimateClass,
    ClimateLayer,
    GeographyLayer,
    HydrologyLayer,
    ProvenanceRecord,
    WorldConfig,
    WorldMetadata,
    WorldModel,
    WorldScale,
)

_GRID_WIDTH_BY_SCALE = {WorldScale.SMALL: 24, WorldScale.MEDIUM: 48}
_CLIMATE_BASE_TEMPERATURE_CELSIUS = {
    ClimateClass.COLD: 2.0,
    ClimateClass.TEMPERATE: 15.0,
    ClimateClass.HOT: 28.0,
}
_SEA_LEVEL_METERS = 0.0
_ELEVATION_RANGE_METERS = 8_500.0
_CONTINENTAL_WAVE_METERS = 2_200.0
_ELEVATION_LAPSE_RATE_CELSIUS_PER_METER = 0.0065
_HEADWATER_PRECIPITATION_THRESHOLD_MM = 1_200.0
_HEADWATER_ELEVATION_THRESHOLD_METERS = 750.0

FloatGrid = tuple[tuple[float, ...], ...]
StringGrid = tuple[tuple[str, ...], ...]


def generate_world(config: WorldConfig) -> WorldModel:
    """Generate a deterministic, validated world from explicit parameters."""
    width = _GRID_WIDTH_BY_SCALE[config.scale]
    height = width // 2
    elevation = _generate_elevation(config.seed, width, height)
    temperature = _generate_temperature(config, elevation)
    precipitation = _generate_precipitation(config.seed, elevation)
    geography = GeographyLayer(
        width=width,
        height=height,
        sea_level_meters=_SEA_LEVEL_METERS,
        elevation_meters=elevation,
    )
    return WorldModel(
        metadata=_create_metadata(config),
        geography=geography,
        hydrology=_create_hydrology(elevation, precipitation),
        climate=ClimateLayer(
            atmospheric_pressure_kpa=STANDARD_ATMOSPHERIC_PRESSURE_KPA,
            temperature_celsius=temperature,
            annual_precipitation_mm=precipitation,
        ),
        biomes=BiomeLayer(classifications=_classify_biomes(elevation, temperature, precipitation)),
        provenance=_create_provenance(),
    )


def _generate_grid(width: int, height: int, cell: Callable[[int, int], float]) -> FloatGrid:
    return tuple(tuple(round(cell(x, y), 6) for x in range(width)) for y in range(height))


def _generate_elevation(seed: int, width: int, height: int) -> FloatGrid:
    def elevation_at(x: int, y: int) -> float:
        longitude = (x / width) * math.tau
        latitude = ((y + 0.5) / height) * math.pi - (math.pi / 2.0)
        wave = math.sin(longitude * 2.0) * math.cos(latitude) * _CONTINENTAL_WAVE_METERS
        noise = (sample_unit_interval(seed, "elevation", x, y) - 0.5) * _ELEVATION_RANGE_METERS
        return min(MAXIMUM_ELEVATION_METERS, max(MINIMUM_ELEVATION_METERS, wave + noise))

    return _generate_grid(width, height, elevation_at)


def _generate_temperature(config: WorldConfig, elevation: FloatGrid) -> FloatGrid:
    height = len(elevation)
    width = len(elevation[0])
    base_temperature = _CLIMATE_BASE_TEMPERATURE_CELSIUS[config.climate_class]

    def temperature_at(x: int, y: int) -> float:
        latitude_factor = abs(((y + 0.5) / height) * 2.0 - 1.0)
        latitude_cooling = latitude_factor * 38.0
        altitude_cooling = max(elevation[y][x], 0.0) * _ELEVATION_LAPSE_RATE_CELSIUS_PER_METER
        return base_temperature - latitude_cooling - altitude_cooling

    return _generate_grid(width, height, temperature_at)


def _generate_precipitation(seed: int, elevation: FloatGrid) -> FloatGrid:
    height = len(elevation)
    width = len(elevation[0])

    def precipitation_at(x: int, y: int) -> float:
        moisture = sample_unit_interval(seed, "precipitation", x, y)
        elevation_penalty = max(elevation[y][x], 0.0) * 0.12
        return max(0.0, 250.0 + moisture * 2_200.0 - elevation_penalty)

    return _generate_grid(width, height, precipitation_at)


def _create_hydrology(elevation: FloatGrid, precipitation: FloatGrid) -> HydrologyLayer:
    cells = sum(len(row) for row in elevation)
    ocean_cells = sum(value <= _SEA_LEVEL_METERS for row in elevation for value in row)
    headwaters = sum(
        elevation[y][x] >= _HEADWATER_ELEVATION_THRESHOLD_METERS
        and precipitation[y][x] >= _HEADWATER_PRECIPITATION_THRESHOLD_MM
        for y in range(len(elevation))
        for x in range(len(elevation[y]))
    )
    return HydrologyLayer(
        surface_water_fraction=round(ocean_cells / cells, 6),
        headwater_candidate_count=headwaters,
    )


def _classify_biomes(
    elevation: FloatGrid, temperature: FloatGrid, precipitation: FloatGrid
) -> StringGrid:
    return tuple(
        tuple(
            _classify_biome(elevation[y][x], temperature[y][x], precipitation[y][x])
            for x in range(len(elevation[y]))
        )
        for y in range(len(elevation))
    )


def _classify_biome(elevation: float, temperature: float, precipitation: float) -> str:
    if elevation <= _SEA_LEVEL_METERS:
        return "ocean"
    if temperature < -10.0:
        return "ice"
    if elevation > 2_500.0:
        return "alpine"
    if precipitation < 350.0:
        return "desert"
    if temperature > 20.0 and precipitation > 1_400.0:
        return "tropical-forest"
    if precipitation > 900.0:
        return "temperate-forest"
    return "grassland"


def _create_metadata(config: WorldConfig) -> WorldMetadata:
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
    algorithm_version = deterministic_algorithm_version()
    return (
        ProvenanceRecord(
            output_path="geography.elevation_meters",
            process="deterministic-heightfield",
            input_paths=("metadata.config.seed", "metadata.config.scale"),
            algorithm_version=algorithm_version,
        ),
        ProvenanceRecord(
            output_path="climate",
            process="latitude-altitude-climate",
            input_paths=("geography.elevation_meters", "metadata.config.climate_class"),
            algorithm_version=algorithm_version,
        ),
        ProvenanceRecord(
            output_path="biomes.classifications",
            process="physical-biome-classifier",
            input_paths=("geography", "climate"),
            algorithm_version=algorithm_version,
        ),
    )
