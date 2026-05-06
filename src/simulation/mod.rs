//! Simulation Module
//!
//! Provides simulation capabilities for World Factory world dynamics.
//!
//! # Submodules
//!
//! - `population` — Population growth model for settlements
//! - `economic` — Economic simulation (planned)
//! - `cultural` — Cultural dynamics simulation (planned)

pub mod population;

pub use population::{
    ActiveDisease, Disaster, DisasterType, DiseaseOutbreak, DiseaseType, FoodAvailability,
    FoodSecurity, PopulationChange, PopulationConfig, PopulationModel, PopulationSample,
    SettlementPopulation, SocietyType,
};

// Note: SocietyType is re-exported via population module (which re-exports from history::society)
