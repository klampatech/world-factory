//! Settlements module - settlement entities and management

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::history::result::SocietyType;

/// Settlement entity in simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub species_id: String,
    pub territory_id: String,
    pub population: u32,
    pub society_type: SocietyType,
}

impl Settlement {
    pub fn new_band(name: String, species_id: String, territory_id: String, population: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            species_id,
            territory_id,
            population,
            society_type: SocietyType::Band,
        }
    }

    pub fn apply_event_effect(&mut self, _event: &crate::events::Event) {
        // Apply event effects - stub implementation
    }
}
