//! World Factory - Procedural World Generation Engine
//!
//! A deterministic procedural generation system for fictional worlds.
//! Generates complete universes with geography, civilizations, and histories.

pub mod terrain;
pub mod entity;
pub mod storage;
pub mod packaging;
pub mod generation;
pub mod hydro;
pub mod settlements;
pub mod species;
pub mod simulation;
pub mod util;
pub mod types;
#[cfg(feature = "api")]
pub mod api;
pub mod config;
pub mod events;
pub mod world;
pub mod figures;
pub mod history;

pub mod artifacts;
pub mod cataclysms;

// Re-export commonly used types
pub use terrain::{TerrainGrid, TerrainGenerator, BiomeType, BiomeAssignmentMatrix};
pub use terrain::climate_calculator::{ClimateCalculator, ClimateCalculatorConfig, PolygonClimate, WindDirection};
pub use terrain::elevation::{Polygon, PolygonGraph, ElevationStats};
pub use terrain::elevation_assignment::{
    ElevationAssigner, ElevationConfig, ElevationAssignmentResult
};
pub use terrain::elevation_grid::ElevationGrid;
pub use terrain::ocean::{OceanDetector, OceanDetectionConfig, OceanZone, CoastalMetrics};
pub use terrain::resource_types::{
    ResourceType, ResourceCategory, ResourceRichness,
    ResourceDeposit, ResourceSet, ResourceGenerator, ResourceGenConfig,
    ALL_RESOURCE_TYPES, ALL_RESOURCE_CATEGORIES
};
pub use terrain::resource_spawner::{ResourceSpawner, ResourceSpawnConfig, RegionResourceSpawn, ResourceSpawnStats, TectonicBoundaryData, BoundaryEffectType};
pub use terrain::tectonic::{TectonicSimulator, TectonicSimConfig, TectonicResult, BoundaryEffect, ElevationModifier};
pub use settlements::{SettlementGenerator, SettlementConfig, SettlementResult};
pub use species::{Species, SpeciesId, SpeciesData, SpeciesTrait, NameTemplate};
pub use species::loader::{SpeciesLoader, SpeciesTemplateFile, TemplateMetadata, TemplateError, merge_with_defaults};
pub use types::{World, Region, Settlement, SettlementType, GeoLocation};
pub use hydro::{River, RiverId, RiverConfig, RiverGenerator, DrainTarget};
pub use hydro::polygon_rivers::{PolygonRiver, PolygonRiverGenerator, Confluence};
pub use hydro::{PolygonDrainageBasin, DrainageBasinCalculator, DrainageConfig, OutletType};
pub use events::{Event, EventType, EventBuilder, EventStore, EventTimeline, EventEffect, EventCategory};
pub use types::HistoricalTime;

// Notable Figures module
pub use figures::{NotableFigure, FigureType, FigureStore, FigureGenerator, FigureGeneratorConfig, FigureName};

// Artifact module
pub use artifacts::{
    Artifact, ArtifactCategory, ArtifactCondition, ArtifactStore, 
    ArtifactProperty, ArtifactPropertyType, ArtifactRarity,
    ArtifactEffect, ArtifactEffectType, EffectScope,
    ArtifactCreationCondition, ArtifactCreationConditionType, ArtifactCreationContext,
    CataclysmTriggerSystem,
};

// Cataclysm module
pub use cataclysms::{Cataclysm, CataclysmType, CataclysmSeverity, CataclysmStore, CataclysmEffect, CataclysmEffectType, RegionImpact, RecoveryState};

// Simulation module
pub use simulation::{PopulationModel, PopulationConfig, PopulationChange};

// History module - Species data model with behaviors, stats, and plugin loader
pub use history::{
    SpeciesTemplate, SpeciesHistory, TemplateLoader,
    SpeciesBehaviors, SpeciesBehavior, SpeciesStats,
    SpeciesSocietyType, SocietyEvolution,
    OnlyInHistory, SpeciesHistoryError,
};

// Society and population module exports
pub use history::{
    Society, SocietyRegistry, SocietyType, SocietyError,
    PopulationSample,
};
pub use history::population::{
    PopulationGrowthService, GrowthConfig,
    PopulationTickResult, SocietyTransition,
    SimulationResult, SimulationStats,
    FoodAvailability, SettlementFoodCalculator,
};
pub use history::population_adapter::{
    PopulationEventAdapter, PopulationEventConfig,
};

// Voronoi generation with Lloyd relaxation
pub use generation::voronoi::{VoronoiConfig, VoronoiGenerator, generate_voronoi_graph, quick_voronoi};

// Mesh and geometry types for rendering/export
pub use terrain::mesh::{MeshId, Mesh, MeshVertex, MeshFace, MeshMetadata, MeshConfig, BoundingBox3D};

// Level-of-detail mesh types
pub use terrain::lod::{LodMeshId, LodConfig, LodLevel, LodMesh, LodTransition, LodLevelSpec};


// Polygon topology and adjacency types
pub use terrain::topology::{TopologyId, PolygonEdge, PolygonTopology, PolygonTopologyMap, BorderType};

// Configuration system
pub use config::{WorldConfig, ConfigError, Dimensions, TerrainSettings, RiverSettings, BiomeSettings};
pub use config::validation::validate_world_config;

// Planet & Geography Types (WOR-8)
pub use world::{
    Planet, PlanetDimensions, PlanetValidationError,
    Geography, ClimateClassification, SoilType,
    Temperature, TemperatureZone, Precipitation, PrecipitationZone,
    DrainageType, DrainageBasin, DrainageError,
    TectonicPlate, TectonicPlateType, TectonicBoundary, TectonicBoundaryType,
    TectonicError, SubductionType,
    Point2D, BoundingBox, Polygon as WorldPolygon, Triangle, PolygonMesh,
};

// TODO: Add entity system
// TODO: Add world state management

// Package save/load for .wfw files
pub use packaging::{
    WorldPackage, PackageManifest, PackageError,
    save_world, save_world_package, load_world, inspect_package, load_world_metadata
};

// Storage directory management
pub use storage::{
    StorageManager, StorageConfig, StorageError, StorageStats, StorageResult,
    WorldStorageInfo, default_base_dir, get_storage_dir, bytes_to_human,
    WORLD_FACTORY_DIR_ENV, is_writable_dir
};
