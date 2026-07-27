"""Typed, versioned contracts for generated worlds."""

from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field

from world_factory.constants import MAXIMUM_SEED, MINIMUM_SEED


class StrictModel(BaseModel):
    """Reject unknown or coerced data and prevent model reassignment."""

    model_config = ConfigDict(extra="forbid", frozen=True, strict=True)


class WorldScale(StrEnum):
    """Supported Phase 0 grid sizes."""

    SMALL = "small"
    MEDIUM = "medium"


class ClimateClass(StrEnum):
    """Broad planetary climate controls implemented in Phase 0."""

    COLD = "cold"
    TEMPERATE = "temperate"
    HOT = "hot"


class BiomeClass(StrEnum):
    """Per-cell biome classification derived from physical conditions."""

    OCEAN = "ocean"
    ICE = "ice"
    ALPINE = "alpine"
    DESERT = "desert"
    TROPICAL_FOREST = "tropical-forest"
    TEMPERATE_FOREST = "temperate-forest"
    GRASSLAND = "grassland"


class WorldConfig(StrictModel):
    """Validated parameters that define a world's identity."""

    seed: int = Field(ge=MINIMUM_SEED, le=MAXIMUM_SEED)
    scale: WorldScale = WorldScale.SMALL
    climate_class: ClimateClass = ClimateClass.TEMPERATE
    sentience_enabled: bool = True
    magic_enabled: bool = False


class WorldMetadata(StrictModel):
    """Stable identity and version information for a generated world."""

    world_id: str = Field(min_length=16, max_length=64)
    schema_version: str
    model_version: str
    config: WorldConfig


class GeographyLayer(StrictModel):
    """Regular-grid topography produced by the active geography module."""

    width: int = Field(gt=0)
    height: int = Field(gt=0)
    sea_level_meters: float
    elevation_meters: tuple[tuple[float, ...], ...]


class HydrologyLayer(StrictModel):
    """Aggregate hydrology outputs available at the Phase 0 seam."""

    surface_water_fraction: float = Field(ge=0.0, le=1.0)
    headwater_candidate_count: int = Field(ge=0)


class ClimateLayer(StrictModel):
    """Regular-grid climate state derived from topography and parameters."""

    atmospheric_pressure_kpa: float = Field(gt=0.0)
    temperature_celsius: tuple[tuple[float, ...], ...]
    annual_precipitation_mm: tuple[tuple[float, ...], ...]


class BiomeLayer(StrictModel):
    """Biome classification grid derived from physical conditions."""

    classifications: tuple[tuple[BiomeClass, ...], ...]


class ProvenanceRecord(StrictModel):
    """Inspectable evidence linking an output path to its generating process."""

    output_path: str
    process: str
    input_paths: tuple[str, ...]
    algorithm_version: str


class WorldModel(StrictModel):
    """Composable root contract shared by generation and simulation layers."""

    metadata: WorldMetadata
    geography: GeographyLayer
    hydrology: HydrologyLayer
    climate: ClimateLayer
    biomes: BiomeLayer
    provenance: tuple[ProvenanceRecord, ...]
