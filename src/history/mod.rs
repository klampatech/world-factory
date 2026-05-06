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

pub mod generator;
pub mod population;
pub mod population_adapter;
pub mod society;
pub mod species;

// Re-export commonly used types
pub use species::{
    OnlyInHistory, SocietyEvolution, SpeciesBehavior, SpeciesBehaviors, SpeciesHistory,
    SpeciesHistoryError, SpeciesSocietyType, SpeciesStats, SpeciesTemplate, TemplateLoader,
};

// Re-export generator types
pub use generator::{
    GenerationResult, GenerationStats, GeneratorConfig, HistoryGenerator, SimulationRunResult,
};

// Re-export society types
pub use society::{PopulationSample, Society, SocietyError, SocietyRegistry, SocietyType};

// Re-export population types
pub use population::{
    FoodAvailability, GrowthConfig, PopulationGrowthService, PopulationTickResult,
    SettlementFoodCalculator, SimulationResult, SimulationStats, SocietyTransition,
};

// Re-export population adapter
pub use population_adapter::{PopulationEventAdapter, PopulationEventConfig};
