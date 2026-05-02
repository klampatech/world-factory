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
    PopulationModel, PopulationConfig, PopulationChange, SettlementPopulation,
    SocietyType, PopulationSample, DiseaseOutbreak, DiseaseType, Disaster, DisasterType,
    FoodAvailability, FoodSecurity, ActiveDisease,
};