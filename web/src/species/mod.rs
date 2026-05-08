//! Species module - species data structures and management

use serde::{Deserialize, Serialize};

/// Species data from world configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesData {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub carrying_capacity: u32,
    pub traits: Vec<String>,
}

/// Species entity in simulation
#[derive(Debug, Clone)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub carrying_capacity: u32,
    pub traits: Vec<String>,
}

impl Species {
    pub fn from_data(data: SpeciesData) -> Self {
        Self {
            id: data.id,
            name: data.name,
            population: data.population,
            carrying_capacity: data.carrying_capacity,
            traits: data.traits,
        }
    }
}
