//! World Factory - Procedural World Generation Engine
//!
//! A deterministic procedural generation system for fictional worlds.
//! Generates complete universes with geography, civilizations, and histories.

pub mod faction;
pub mod faction_integration;

#[cfg(feature = "api")]
pub mod api;

pub mod config;
pub mod entity;
pub mod events;
pub mod figures;
pub mod generation;
pub mod history;
pub mod hydro;
pub mod packaging;
pub mod settlements;
pub mod simulation;
pub mod species;
pub mod storage;
pub mod terrain;
pub mod types;
pub mod util;
pub mod world;

pub mod artifacts;
pub mod cataclysms;

pub use faction::{
    AssetCategory, Faction, FactionAsset, FactionGoal, FactionRelation, FactionRegistry,
    FactionTurnState, FactionType, TurnPhase,
};

// Re-export commonly used types
pub use events::{
    Event, EventBuilder, EventCategory, EventEffect, EventStore, EventTimeline, EventType,
};
pub use hydro::polygon_rivers::{Confluence, PolygonRiver, PolygonRiverGenerator};
pub use hydro::{DrainTarget, River, RiverConfig, RiverGenerator, RiverId};
pub use hydro::{DrainageBasinCalculator, DrainageConfig, OutletType, PolygonDrainageBasin};
pub use settlements::{SettlementConfig, SettlementGenerator, SettlementResult};
pub use species::loader::{
    merge_with_defaults, SpeciesLoader, SpeciesTemplateFile, TemplateError, TemplateMetadata,
};
pub use species::{NameTemplate, Species, SpeciesData, SpeciesId, SpeciesTrait};
pub use terrain::biome::{ClimateZone, ElevationZone, MoistureLevel, VegetationType};
pub use terrain::climate_calculator::{
    ClimateCalculator, ClimateCalculatorConfig, PolygonClimate, WindDirection,
};
pub use terrain::elevation::{ElevationStats, Polygon, PolygonGraph};
pub use terrain::elevation_assignment::{
    ElevationAssigner, ElevationAssignmentResult, ElevationConfig,
};
pub use terrain::elevation_grid::ElevationGrid;
pub use terrain::ocean::{CoastalMetrics, OceanDetectionConfig, OceanDetector, OceanZone};
pub use terrain::resource_spawner::{
    BoundaryEffectType, RegionResourceSpawn, ResourceSpawnConfig, ResourceSpawnStats,
    ResourceSpawner, TectonicBoundaryData,
};
pub use terrain::resource_types::{
    ResourceCategory, ResourceDeposit, ResourceGenConfig, ResourceGenerator, ResourceRichness,
    ResourceSet, ResourceType, ALL_RESOURCE_CATEGORIES, ALL_RESOURCE_TYPES,
};
pub use terrain::tectonic::{
    BoundaryEffect, ElevationModifier, TectonicResult, TectonicSimConfig, TectonicSimulator,
};
pub use terrain::terrain_generator::{TerrainConfig, TerrainLayer};
pub use terrain::terrain_grid::TerrainCell;
pub use terrain::{BiomeAssignmentMatrix, BiomeType, TerrainGenerator, TerrainGrid};
pub use types::HistoricalTime;
pub use types::{GeoLocation, Region, Settlement, SettlementType, World};
pub use uuid::Uuid;

// Notable Figures module
pub use figures::{
    FigureGenerator, FigureGeneratorConfig, FigureName, FigureNameGenerator, FigureStore,
    FigureType, NotableFigure,
};

// Extended figure types
pub use figures::{
    Dynasty, DynastyStore, FigureLifecycleState, FigureRelationship, FigureRelationshipGraph,
    FigureRelationshipType, RegionInfluence,
};

// Artifact module
pub use artifacts::{
    Artifact, ArtifactCategory, ArtifactCondition, ArtifactCreationCondition,
    ArtifactCreationConditionType, ArtifactCreationContext, ArtifactEffect, ArtifactEffectType,
    ArtifactProperty, ArtifactPropertyType, ArtifactRarity, ArtifactStore, CataclysmTriggerSystem,
    EffectScope,
};

// Cataclysm module
pub use cataclysms::{
    Cataclysm, CataclysmEffect, CataclysmEffectType, CataclysmSeverity, CataclysmStore,
    CataclysmType, RecoveryState, RegionImpact,
};

// Simulation module
pub use simulation::{PopulationChange, PopulationConfig, PopulationModel};

// History module - Species data model with behaviors, stats, and plugin loader
// Also includes HistoryGenerator for orchestrating full history generation
pub use history::{
    OnlyInHistory, SocietyEvolution, SpeciesBehavior, SpeciesBehaviors, SpeciesHistory,
    SpeciesHistoryError, SpeciesSocietyType, SpeciesStats, SpeciesTemplate, TemplateLoader,
};

// Re-export HistoryGenerator for Phase 2 integration
pub use history::generator::{
    GenerationResult, GenerationStats, GeneratorConfig, HistoryGenerator,
};

// Society and population module exports
pub use history::population::{
    FoodAvailability, GrowthConfig, PopulationGrowthService, PopulationTickResult,
    SettlementFoodCalculator, SimulationResult, SimulationStats, SocietyTransition,
};
pub use history::population_adapter::{PopulationEventAdapter, PopulationEventConfig};
pub use history::{PopulationSample, Society, SocietyError, SocietyRegistry, SocietyType};

// Voronoi generation with Lloyd relaxation
pub use generation::voronoi::{
    generate_voronoi_graph, quick_voronoi, VoronoiConfig, VoronoiGenerator,
};

// Mesh and geometry types for rendering/export
pub use terrain::mesh::{
    BoundingBox3D, Mesh, MeshConfig, MeshFace, MeshId, MeshMetadata, MeshVertex,
};

// Level-of-detail mesh types
pub use terrain::lod::{LodConfig, LodLevel, LodLevelSpec, LodMesh, LodMeshId, LodTransition};

// Polygon topology and adjacency types
pub use terrain::topology::{
    BorderType, PolygonEdge, PolygonTopology, PolygonTopologyMap, TopologyId,
};

// Configuration system
pub use config::validation::validate_world_config;
pub use config::{
    BiomeSettings, ConfigError, Dimensions, RiverSettings, TerrainSettings, WorldConfig,
};

// Planet & Geography Types (WOR-8)
pub use world::{
    BoundingBox, ClimateClassification, DrainageBasin, DrainageError, DrainageType, Geography,
    Planet, PlanetDimensions, PlanetValidationError, Point2D, Polygon as WorldPolygon, PolygonMesh,
    Precipitation, PrecipitationZone, SoilType, SubductionType, TectonicBoundary,
    TectonicBoundaryType, TectonicError, TectonicPlate, TectonicPlateType, Temperature,
    TemperatureZone, Triangle,
};

// TODO: Add entity system
// TODO: Add world state management

// Package save/load for .wfw files
pub use packaging::{
    inspect_package, load_world, load_world_metadata, save_world, save_world_package, PackageError,
    PackageManifest, WorldPackage,
};

// Storage directory management
pub use storage::{
    bytes_to_human, default_base_dir, get_storage_dir, is_writable_dir, StorageConfig,
    StorageError, StorageManager, StorageResult, StorageStats, WorldStorageInfo,
    WORLD_FACTORY_DIR_ENV,
};
