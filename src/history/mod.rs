//! History Module
//! 
//! Phase 2 implementation of species-agnostic historical systems.
//! 
//! This module provides the species-agnostic core guarantee:
//! - All types that could be species-specific implement `OnlyInHistory`
//! - Core engine types do NOT implement this trait
//! - Static analysis can verify species-agnosticism
//! 
//! # Modules
//! 
//! - `species` — Extended species templates with behaviors, stats, and society types
//! - `society` — Society entities and registry for civilization tracking
//! - `population` — Population growth simulation with food/disease factors
//! - `population_adapter` — Converts population results to history events

pub mod species;
pub mod society;
pub mod population;
pub mod population_adapter;
pub mod generator;

// Re-export commonly used types
pub use species::{
    SpeciesTemplate, SpeciesHistory, TemplateLoader,
    SpeciesBehaviors, SpeciesBehavior, SpeciesStats,
    SpeciesSocietyType, SocietyEvolution,
    OnlyInHistory, SpeciesHistoryError,
};

// Re-export generator types
pub use generator::{HistoryGenerator, GeneratorConfig, GenerationResult, GenerationStats, SimulationRunResult};

// Re-export society types
pub use society::{
    Society, SocietyRegistry, SocietyType, SocietyError,
    PopulationSample,
};

// Re-export population types
pub use population::{
    PopulationGrowthService, GrowthConfig,
    PopulationTickResult, SocietyTransition,
    SimulationResult, SimulationStats,
    FoodAvailability, SettlementFoodCalculator,
};

// Re-export population adapter
pub use population_adapter::{
    PopulationEventAdapter, PopulationEventConfig,
};
