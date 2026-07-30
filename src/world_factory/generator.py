"""Deterministic generation pipeline for the world factory."""

import hashlib
import json
import math
from collections.abc import Callable

from world_factory.agriculture import agriculture_provenance, build_agriculture
from world_factory.astronomy import astronomy_provenance, build_astronomy
from world_factory.atmosphere import atmosphere_provenance, refine_climate
from world_factory.biology import biology_provenance, build_biology
from world_factory.causal_graph import (
    build_causal_graph,
    causal_graph_provenance,
)
from world_factory.constants import (
    CONTINENTAL_INTERIOR_BASE_ELEVATION_METERS,
    CONVERGENT_BOUNDARY_UPLIFT_METERS,
    DEMOGRAPHY_DEFAULT_TIME_STEPS,
    DETERMINISTIC_ALGORITHM_VERSION,
    DIVERGENT_BOUNDARY_RIFT_METERS,
    ELEVATION_NOISE_RANGE_METERS,
    MAXIMUM_ELEVATION_METERS,
    MINIMUM_ELEVATION_METERS,
    MODEL_VERSION,
    OCEANIC_INTERIOR_BASE_ELEVATION_METERS,
    SCHEMA_VERSION,
    SEASONAL_TEMPERATURE_AMPLITUDE,
)
from world_factory.cultures import build_cultures, cultures_provenance
from world_factory.demography import build_demography, demography_provenance
from world_factory.determinism import sample_unit_interval
from world_factory.event_log import build_event_log, event_log_provenance
from world_factory.geology import (
    generate_geology,
    generate_geology_sublayers,
    geology_sublayer_provenance,
)
from world_factory.historiography import (
    build_historiography,
    historiography_provenance,
)
from world_factory.hydrology import generate_hydrology, hydrology_provenance
from world_factory.infrastructure import (
    build_infrastructure,
    infrastructure_provenance,
)
from world_factory.kinship import build_kinship, kinship_provenance
from world_factory.language import build_languages, language_provenance
from world_factory.models import (
    AgricultureLayer,
    AstronomyLayer,
    BiomeClass,
    BiomeLayer,
    BoundaryType,
    CausalGraphLayer,
    ClimateClass,
    ClimateLayer,
    CultureLayer,
    DemographyLayer,
    EventLog,
    GeographyLayer,
    GeologyLayer,
    HistoriographyLayer,
    InfrastructureLayer,
    KinshipLayer,
    LanguageLayer,
    PolityLayer,
    ProvenanceRecord,
    ReligionLayer,
    WorldConfig,
    WorldMetadata,
    WorldModel,
    WorldScale,
)
from world_factory.polities import build_polities, polities_provenance
from world_factory.religion import build_religion, religion_provenance
from world_factory.settlements import build_settlements, settlements_provenance

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
    temperature_base = _generate_temperature(config, elevation)
    precipitation = _generate_precipitation(config.seed, elevation)
    geography = GeographyLayer(
        width=width,
        height=height,
        sea_level_meters=_SEA_LEVEL_METERS,
        elevation_meters=elevation,
    )
    geology = generate_geology_sublayers(
        geology=geology,
        elevation=elevation,
        temperature=temperature_base,
        precipitation=precipitation,
        sea_level=_SEA_LEVEL_METERS,
        seed=config.seed,
    )
    biomes_layer = BiomeLayer(
        classifications=_classify_biomes(
            elevation, temperature_base, precipitation
        ),
    )
    biology = build_biology(
        classifications=biomes_layer.classifications,
        elevation=elevation,
        sea_level=_SEA_LEVEL_METERS,
    )
    astronomy = build_astronomy(
        width=width,
        height=height,
        axial_tilt_degrees=config.axial_tilt_degrees,
        orbital_eccentricity=config.orbital_eccentricity,
        season_day=config.season_day,
        orbital_period_days=config.orbital_period_days,
    )
    temperature = _apply_seasonal_correction(temperature_base, astronomy)
    (
        atmospheric_pressure_kpa,
        wind_direction_grid,
        specific_humidity_grid,
        refined_precipitation,
    ) = refine_climate(
        elevation=elevation,
        temperature=temperature,
        base_precipitation=precipitation,
        sea_level=_SEA_LEVEL_METERS,
    )
    climate = ClimateLayer(
        atmospheric_pressure_kpa=atmospheric_pressure_kpa,
        temperature_celsius=temperature,
        annual_precipitation_mm=refined_precipitation,
        wind_direction_grid=wind_direction_grid,
        specific_humidity_grid=specific_humidity_grid,
    )
    hydrology = generate_hydrology(
        elevation=elevation,
        precipitation=refined_precipitation,
        sea_level=_SEA_LEVEL_METERS,
        seed=config.seed,
    )
    settlements = build_settlements(
        elevation=elevation,
        temperature=temperature_base,
        biome_grid=biomes_layer.classifications,
        ore_grid=geology.ore_presence_grid,
        river_segments=hydrology.river_segments,
        plate_count=config.plate_count,
    )
    provisional_world = WorldModel(
        metadata=_create_metadata(config),
        geology=geology,
        geography=geography,
        hydrology=hydrology,
        climate=climate,
        biomes=biomes_layer,
        astronomy=astronomy,
        biology=biology,
        settlements=settlements,
        agriculture=AgricultureLayer(agriculture=()),
        infrastructure=InfrastructureLayer(roads=(), ports=(), canals=()),
        demography=DemographyLayer(pools=(), migrations=(), events=()),
        events=EventLog(events=(), algorithm_version=""),
        cultures=CultureLayer(cultures=(), algorithm_version=""),
        religions=ReligionLayer(religions=(), rituals=(), algorithm_version=""),
        kinship=KinshipLayer(lineages=(), name_pools=(), algorithm_version=""),
languages=LanguageLayer(languages=(), families=(), algorithm_version=''),
        polities=PolityLayer(
            polities=(), memberships=(), borders=(),
            events=(), algorithm_version=""
        ),
        causal_graph=CausalGraphLayer(edges=(), algorithm_version=""),
        historiography=HistoriographyLayer(
            source_gaps=(), disputed_events=(), algorithm_version=""
        ),
        provenance=(),
    )
    agriculture = build_agriculture(provisional_world)
    populated_world = WorldModel(
        metadata=_create_metadata(config),
        geology=geology,
        geography=geography,
        hydrology=hydrology,
        climate=climate,
        biomes=biomes_layer,
        astronomy=astronomy,
        biology=biology,
        settlements=settlements,
        agriculture=agriculture,
        infrastructure=InfrastructureLayer(roads=(), ports=(), canals=()),
        demography=DemographyLayer(pools=(), migrations=(), events=()),
        events=EventLog(events=(), algorithm_version=""),
        cultures=CultureLayer(cultures=(), algorithm_version=""),
        religions=ReligionLayer(religions=(), rituals=(), algorithm_version=""),
        kinship=KinshipLayer(lineages=(), name_pools=(), algorithm_version=""),
languages=LanguageLayer(languages=(), families=(), algorithm_version=''),
        polities=PolityLayer(
            polities=(), memberships=(), borders=(),
            events=(), algorithm_version=""
        ),
        causal_graph=CausalGraphLayer(edges=(), algorithm_version=""),
        historiography=HistoriographyLayer(
            source_gaps=(), disputed_events=(), algorithm_version=""
        ),
        provenance=(),
    )
    infrastructure = build_infrastructure(populated_world)
    demography_ready_world = WorldModel(
        metadata=_create_metadata(config),
        geology=geology,
        geography=geography,
        hydrology=hydrology,
        climate=climate,
        biomes=biomes_layer,
        astronomy=astronomy,
        biology=biology,
        settlements=settlements,
        agriculture=agriculture,
        infrastructure=infrastructure,
        demography=DemographyLayer(pools=(), migrations=(), events=()),
        events=EventLog(events=(), algorithm_version=""),
        cultures=CultureLayer(cultures=(), algorithm_version=""),
        religions=ReligionLayer(religions=(), rituals=(), algorithm_version=""),
        kinship=KinshipLayer(lineages=(), name_pools=(), algorithm_version=""),
languages=LanguageLayer(languages=(), families=(), algorithm_version=''),
        polities=PolityLayer(
            polities=(), memberships=(), borders=(),
            events=(), algorithm_version=""
        ),
        causal_graph=CausalGraphLayer(edges=(), algorithm_version=""),
        historiography=HistoriographyLayer(
            source_gaps=(), disputed_events=(), algorithm_version=""
        ),
        provenance=(),
    )
    demography = build_demography(
        demography_ready_world, time_steps=DEMOGRAPHY_DEFAULT_TIME_STEPS
    )
    world_with_demography = demography_ready_world.model_copy(
        update={"demography": demography}
    )
    cultures, culture_events = build_cultures(world_with_demography)
    world_with_cultures = world_with_demography.model_copy(
        update={"cultures": cultures}
    )
    religions, religion_events = build_religion(world_with_cultures)
    world_with_religions = world_with_cultures.model_copy(
        update={"religions": religions}
    )
    kinship, kinship_events = build_kinship(world_with_religions)
    world_with_kinship = world_with_religions.model_copy(
        update={"kinship": kinship}
    )
    languages, _language_counts = build_languages(world_with_kinship)
    world_with_languages = world_with_kinship.model_copy(
        update={"languages": languages}
    )
    polities = build_polities(world_with_languages)
    world_with_polities = world_with_languages.model_copy(
        update={"polities": polities}
    )
    event_log = build_event_log(
        world_with_demography,
        culture_events=culture_events,
        religion_events=religion_events,
        kinship_events=kinship_events,
        polity_events=polities.events,
    )
    world_with_events = world_with_polities.model_copy(
        update={"events": event_log}
    )
    causal_graph_layer = build_causal_graph(world_with_events)
    historiography_layer = build_historiography(world_with_events)
    return WorldModel(
        metadata=_create_metadata(config),
        geology=geology,
        geography=geography,
        hydrology=hydrology,
        climate=climate,
        biomes=biomes_layer,
        astronomy=astronomy,
        biology=biology,
        settlements=settlements,
        agriculture=agriculture,
        infrastructure=infrastructure,
        demography=demography,
        events=event_log,
        cultures=cultures,
        religions=religions,
        kinship=kinship,
        languages=languages,
        polities=polities,
        causal_graph=causal_graph_layer,
        historiography=historiography_layer,
        provenance=_create_provenance(),
    )


def _apply_seasonal_correction(
    base_temperature: FloatGrid, astronomy: "AstronomyLayer"
) -> FloatGrid:
    """Apply a per-cell seasonal correction driven by insolation.

    `T_corrected = T_base × (1 + SEASONAL_TEMPERATURE_AMPLITUDE ×
    (insolation_factor − 0.5))`. Equatorial sub-solar cells read
    slightly hotter than the latitude-only baseline; antisolar
    poles read slightly cooler.
    """
    height = len(base_temperature)
    width = len(base_temperature[0])
    corrected: list[list[float]] = []
    for y in range(height):
        row: list[float] = []
        for x in range(width):
            factor = 1.0 + SEASONAL_TEMPERATURE_AMPLITUDE * (
                astronomy.insolation_factor[y][x] - 0.5
            )
            row.append(round(base_temperature[y][x] * factor, 6))
        corrected.append(row)
    return tuple(tuple(row) for row in corrected)


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
        geology_sublayer_provenance(),
        ProvenanceRecord(
            output_path="geography.elevation_meters",
            process="plate-uplift-heightfield",
            input_paths=("geology", "metadata.config.seed"),
            algorithm_version=algorithm,
        ),
        astronomy_provenance(),
        hydrology_provenance(),
        atmosphere_provenance(),
        ProvenanceRecord(
            output_path="biomes.classifications",
            process="physical-biome-classifier",
            input_paths=("geography", "climate"),
            algorithm_version=algorithm,
        ),
        biology_provenance(),
        settlements_provenance(),
        agriculture_provenance(),
        infrastructure_provenance(),
        demography_provenance(),
        event_log_provenance(),
        cultures_provenance(),
        religion_provenance(),
        kinship_provenance(),
        language_provenance(),
        polities_provenance(),
        causal_graph_provenance(),
        historiography_provenance(),
    )
