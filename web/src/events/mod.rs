//! Events module - Historical events and probability engine

use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::history::generator::PreHistoryConfig;
use crate::settlements::Settlement;
use crate::species::Species;

/// Event generated during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub year: u32,
    pub event_type: EventType,
    pub description: String,
    pub magnitude: f32,
    pub significance: f32,
    pub affected_settlements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    NaturalDisaster,
    Wildfire,
    PopulationBoom,
    Famine,
    Plenty,
    War,
    Peace,
    Discovery,
    Migration,
}

impl Event {
    pub fn can_create_artifact(&self) -> bool {
        matches!(
            self.event_type,
            EventType::Discovery | EventType::War | EventType::PopulationBoom | EventType::Wildfire
        ) && self.significance >= 0.5
    }
}

/// Event generator
pub struct EventGenerator;

impl EventGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_events(
        &mut self,
        year: u32,
        settlements: &[Settlement],
        species: &[Species],
        config: &PreHistoryConfig,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let base_probability = config.event_probability_base;

        // D.4: Run probability engine for each event type
        for settlement in settlements {
            // Check for natural disaster
            if rand::random::<f32>() < base_probability * 0.1 {
                events.push(self.generate_natural_disaster(year, settlement));
            }

            // Check for population events
            if rand::random::<f32>() < base_probability * 0.2 {
                let total_pop = species.iter().map(|s| s.population).sum::<u32>();
                if total_pop > 100 {
                    events.push(self.generate_population_event(year, settlement, total_pop));
                }
            }

            // Check for conflict
            if rand::random::<f32>() < base_probability * 0.15 {
                events.push(self.generate_conflict_event(year, settlement));
            }

            // Check for discovery
            if rand::random::<f32>() < base_probability * 0.05 {
                events.push(self.generate_discovery_event(year, settlement));
            }

            // Check for migration
            if rand::random::<f32>() < base_probability * 0.08 {
                events.push(self.generate_migration_event(year, settlement));
            }
        }

        events
    }

    fn generate_natural_disaster(&self, year: u32, settlement: &Settlement) -> Event {
        let disaster_types = vec!["famine", "flood", "drought", "disease", "wildfire"];
        let idx = (rand::random::<f32>() * (disaster_types.len() as f32 - 0.001)) as usize;
        let disaster = disaster_types[idx];
        
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            event_type: EventType::NaturalDisaster,
            description: format!("A {} struck near {}", disaster, settlement.name),
            magnitude: rand::random::<f32>() * 0.5 + 0.5,
            significance: 0.6,
            affected_settlements: vec![settlement.id.clone()],
        }
    }

    fn generate_population_event(&self, year: u32, settlement: &Settlement, _total_pop: u32) -> Event {
        let is_boom = rand::random::<bool>();
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            event_type: if is_boom { EventType::PopulationBoom } else { EventType::Famine },
            description: if is_boom {
                format!("Population boom in {}", settlement.name)
            } else {
                format!("Famine struck {}", settlement.name)
            },
            magnitude: rand::random::<f32>() * 0.5 + 0.5,
            significance: 0.5,
            affected_settlements: vec![settlement.id.clone()],
        }
    }

    fn generate_conflict_event(&self, year: u32, settlement: &Settlement) -> Event {
        let is_war = rand::random::<bool>();
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            event_type: if is_war { EventType::War } else { EventType::Peace },
            description: if is_war {
                format!("War broke out affecting {}", settlement.name)
            } else {
                format!("Peace treaty signed near {}", settlement.name)
            },
            magnitude: rand::random::<f32>() * 0.7 + 0.3,
            significance: 0.7,
            affected_settlements: vec![settlement.id.clone()],
        }
    }

    fn generate_discovery_event(&self, year: u32, settlement: &Settlement) -> Event {
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            event_type: EventType::Discovery,
            description: format!("A new discovery was made by {}", settlement.name),
            magnitude: rand::random::<f32>() * 0.8 + 0.2,
            significance: 0.8,
            affected_settlements: vec![settlement.id.clone()],
        }
    }

    fn generate_migration_event(&self, year: u32, settlement: &Settlement) -> Event {
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            event_type: EventType::Migration,
            description: format!("A migration wave passed through {}", settlement.name),
            magnitude: rand::random::<f32>() * 0.4 + 0.3,
            significance: 0.5,
            affected_settlements: vec![settlement.id.clone()],
        }
    }
}

impl Default for EventGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Probability engine for event generation
pub struct ProbabilityEngine;

impl ProbabilityEngine {
    pub fn should_occur(&self, base_probability: f32, modifiers: &[f32]) -> bool {
        let final_prob = modifiers.iter().fold(base_probability, |acc, m| acc * m);
        rand::random::<f32>() < final_prob
    }
}
