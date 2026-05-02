//! History Module
//! 
//! Phase 2 implementation of species-agnostic historical systems.
//! 
//! This module provides the species-agnostic core guarantee:
//! - All types that could be species-specific implement `OnlyInHistory`
//! - Core engine types do NOT implement this trait
//! - Static analysis can verify species-agnosticism

pub mod species;

// Re-export commonly used types
pub use species::{
    SpeciesTemplate, SpeciesHistory, TemplateLoader,
    SpeciesBehaviors, SpeciesBehavior, SpeciesStats,
    SpeciesSocietyType, SocietyEvolution,
    OnlyInHistory, SpeciesHistoryError,
};
