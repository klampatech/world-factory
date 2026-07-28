"""Typed, versioned contracts for generated worlds."""

from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field

from world_factory.constants import (
    EARTH_AXIAL_TILT_DEGREES,
    EARTH_ORBITAL_ECCENTRICITY,
    EARTH_ORBITAL_PERIOD_DAYS,
    EARTH_ROTATION_PERIOD_HOURS,
    MAXIMUM_PLATE_COUNT,
    MAXIMUM_SEED,
    MINIMUM_PLATE_COUNT,
    MINIMUM_SEED,
)


class StrictModel(BaseModel):
    """Reject unknown or coerced data and prevent model reassignment."""

    model_config = ConfigDict(extra="forbid", frozen=True, strict=True)


class WorldScale(StrEnum):
    """Supported grid sizes. Phase 1a adds LARGE for v1 demo worlds."""

    SMALL = "small"
    MEDIUM = "medium"
    LARGE = "large"


class ClimateClass(StrEnum):
    """Broad planetary climate controls implemented in Phase 0."""

    COLD = "cold"
    TEMPERATE = "temperate"
    HOT = "hot"


class WindDirection(StrEnum):
    """Prevailing surface wind direction per cell. Phase 1c."""

    EAST = "east"
    WEST = "west"
    NORTH = "north"
    SOUTH = "south"
    NORTH_EAST = "north-east"
    NORTH_WEST = "north-west"
    SOUTH_EAST = "south-east"
    SOUTH_WEST = "south-west"
    CALM = "calm"


class BiomeClass(StrEnum):
    """Per-cell biome classification derived from physical conditions."""

    OCEAN = "ocean"
    ICE = "ice"
    ALPINE = "alpine"
    DESERT = "desert"
    TROPICAL_FOREST = "tropical-forest"
    TEMPERATE_FOREST = "temperate-forest"
    GRASSLAND = "grassland"


class PlateType(StrEnum):
    """Lithospheric plate composition. Continental plates ride higher
    than oceanic plates; convergent boundaries between oceanic and
    continental plates are subduction zones."""

    CONTINENTAL = "continental"
    OCEANIC = "oceanic"


class BoundaryType(StrEnum):
    """Plate boundary classification derived from relative plate motion."""

    CONVERGENT = "convergent"
    DIVERGENT = "divergent"
    TRANSFORM = "transform"


class RockType(StrEnum):
    """Per-cell rock classification. Phase 1e sublayer."""

    BASALT = "basalt"
    GRANITE = "granite"
    SEDIMENTARY = "sedimentary"
    METAMORPHIC = "metamorphic"
    VOLCANIC = "volcanic"


class SoilType(StrEnum):
    """Per-cell soil classification. Phase 1e sublayer."""

    PERMAFROST = "permafrost"
    SAND = "sand"
    LOAM = "loam"
    CLAY = "clay"
    PEAT = "peat"


class FloraType(StrEnum):
    """Per-cell primary flora species or community. Phase 2."""

    CONIFER = "conifer"
    BROADLEAF = "broadleaf"
    SHRUB = "shrub"
    GRASS = "grass"
    MOSS = "moss"
    LICHEN = "lichen"
    ALGAE = "algae"
    SEAGRASS = "seagrass"
    CORAL = "coral"


class FaunaType(StrEnum):
    """Per-cell primary fauna species or community. Phase 2."""

    HERBIVORE_LARGE = "herbivore-large"
    HERBIVORE_SMALL = "herbivore-small"
    CARNIVORE_LARGE = "carnivore-large"
    CARNIVORE_SMALL = "carnivore-small"
    FISH = "fish"
    BIRD = "bird"
    INSECT = "insect"
    REPTILE = "reptile"


class WorldConfig(StrictModel):
    """Validated parameters that define a world's identity."""

    seed: int = Field(ge=MINIMUM_SEED, le=MAXIMUM_SEED)
    scale: WorldScale = WorldScale.SMALL
    climate_class: ClimateClass = ClimateClass.TEMPERATE
    sentience_enabled: bool = True
    magic_enabled: bool = False
    plate_count: int = Field(
        default=12, ge=MINIMUM_PLATE_COUNT, le=MAXIMUM_PLATE_COUNT
    )
    axial_tilt_degrees: float = Field(
        default=EARTH_AXIAL_TILT_DEGREES, ge=0.0, le=90.0
    )
    orbital_eccentricity: float = Field(
        default=EARTH_ORBITAL_ECCENTRICITY, ge=0.0, le=0.5
    )
    rotation_period_hours: float = Field(
        default=EARTH_ROTATION_PERIOD_HOURS, gt=0.0
    )
    orbital_period_days: float = Field(
        default=EARTH_ORBITAL_PERIOD_DAYS, gt=0.0
    )
    season_day: float = Field(default=0.0, ge=0.0, lt=1_000_000.0)


class WorldMetadata(StrictModel):
    """Stable identity and version information for a generated world."""

    world_id: str = Field(min_length=16, max_length=64)
    schema_version: str
    model_version: str
    config: WorldConfig


class PlateRecord(StrictModel):
    """A single tectonic plate. Plates own a contiguous Voronoi cell set
    on the world grid."""

    id: int = Field(ge=0)
    plate_type: PlateType
    centroid_x: float = Field(ge=0.0)
    centroid_y: float = Field(ge=0.0)
    motion_heading_radians: float = Field(ge=0.0)
    motion_speed: float = Field(ge=0.0)
    cell_count: int = Field(gt=0)


class BoundaryRecord(StrictModel):
    """A single plate-boundary cell classification."""

    x: int = Field(ge=0)
    y: int = Field(ge=0)
    boundary_type: BoundaryType
    plate_a: int = Field(ge=0)
    plate_b: int = Field(ge=0)


class GeologyLayer(StrictModel):
    """Tectonic state of the world. Phase 1a first PR ships the
    geometry; Phase 1e adds rock, ore, and soil sublayers."""

    width: int = Field(gt=0)
    height: int = Field(gt=0)
    plates: tuple[PlateRecord, ...]
    boundaries: tuple[BoundaryRecord, ...]
    plate_id_grid: tuple[tuple[int, ...], ...]
    boundary_type_grid: tuple[tuple[BoundaryType | None, ...], ...]
    rock_type_grid: tuple[tuple[RockType, ...], ...]
    ore_presence_grid: tuple[tuple[bool, ...], ...]
    soil_type_grid: tuple[tuple[SoilType, ...], ...]


class GeographyLayer(StrictModel):
    """Regular-grid topography produced by the active geography module."""

    width: int = Field(gt=0)
    height: int = Field(gt=0)
    sea_level_meters: float
    elevation_meters: tuple[tuple[float, ...], ...]


class AstronomyLayer(StrictModel):
    """Astronomical forcing and its per-cell consequences. Phase 1d
    adds axial tilt and seasonal cycles; the layer records the
    forcing parameters and the per-cell day length + insolation."""

    axial_tilt_degrees: float = Field(ge=0.0, le=90.0)
    orbital_eccentricity: float = Field(ge=0.0, le=0.5)
    season_day: float = Field(ge=0.0)
    solar_declination_degrees: float = Field(ge=-90.0, le=90.0)
    day_length_hours: tuple[tuple[float, ...], ...]
    insolation_factor: tuple[tuple[float, ...], ...]


class RiverSegment(StrictModel):
    """A traced river from headwater source to ocean mouth."""

    id: int = Field(ge=0)
    source: tuple[int, int]
    mouth: tuple[int, int]
    length_cells: int = Field(ge=1)
    mean_discharge: float = Field(ge=0.0)
    mean_slope: float = Field(ge=0.0)
    watershed_id: int = Field(ge=0)


class HydrologyLayer(StrictModel):
    """River network, per-cell discharge, and watershed delineation.
    Phase 0 emitted an aggregate stub; Phase 1b adds the actual network."""

    surface_water_fraction: float = Field(ge=0.0, le=1.0)
    headwater_candidate_count: int = Field(ge=0)
    river_segments: tuple[RiverSegment, ...]
    discharge_grid: tuple[tuple[float, ...], ...]
    watershed_id_grid: tuple[tuple[int | None, ...], ...]


class ClimateLayer(StrictModel):
    """Regular-grid climate state derived from topography, parameters,
    and the Phase 1c atmospheric circulation model."""

    atmospheric_pressure_kpa: tuple[tuple[float, ...], ...]
    temperature_celsius: tuple[tuple[float, ...], ...]
    annual_precipitation_mm: tuple[tuple[float, ...], ...]
    wind_direction_grid: tuple[tuple[WindDirection, ...], ...]
    specific_humidity_grid: tuple[tuple[float, ...], ...]


class BiomeLayer(StrictModel):
    """Biome classification grid derived from physical conditions."""

    classifications: tuple[tuple[BiomeClass, ...], ...]


class BiologyLayer(StrictModel):
    """Per-cell flora and fauna assignments. Phase 2.

    Each cell carries a primary flora species (or `None` for ocean
    cells that capture their biota via the ALGAE / FISH defaults)
    and a primary fauna species. The mapping is biome-driven;
    ocean cells override biome defaults with marine biota."""

    flora_grid: tuple[tuple[FloraType | None, ...], ...]
    fauna_grid: tuple[tuple[FaunaType | None, ...], ...]


class ProvenanceRecord(StrictModel):
    """Inspectable evidence linking an output path to its generating process."""

    output_path: str
    process: str
    input_paths: tuple[str, ...]
    algorithm_version: str


class WorldModel(StrictModel):
    """Composable root contract shared by generation and simulation layers."""

    metadata: WorldMetadata
    geology: GeologyLayer
    geography: GeographyLayer
    hydrology: HydrologyLayer
    climate: ClimateLayer
    biomes: BiomeLayer
    astronomy: AstronomyLayer
    biology: BiologyLayer
    provenance: tuple[ProvenanceRecord, ...]