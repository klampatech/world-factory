//! Artifacts module - Objects created from significant historical events

use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::events::Event;

/// Historical artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub description: String,
    pub creator_id: Option<String>,
    pub creation_year: u32,
    pub significance: f32,
    pub material: String,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Weapon,
    Tool,
    Religious,
    Art,
    Record,
    Treasure,
}

/// Artifact generator - creates artifacts from events
pub struct ArtifactGenerator;

impl ArtifactGenerator {
    pub fn new() -> Self {
        Self
    }

    /// D.6: Create artifact from event when conditions are met
    pub fn create_from_event(&mut self, event: &Event, year: u32) -> Option<Artifact> {
        let artifact_type = match event.event_type {
            crate::events::EventType::War => ArtifactType::Weapon,
            crate::events::EventType::Discovery => ArtifactType::Tool,
            crate::events::EventType::PopulationBoom => ArtifactType::Treasure,
            crate::events::EventType::Peace => ArtifactType::Religious,
            _ => return None,
        };

        let artifact = self.generate_artifact(artifact_type, event, year);
        Some(artifact)
    }

    fn generate_artifact(&mut self, artifact_type: ArtifactType, event: &Event, year: u32) -> Artifact {
        let names = match artifact_type {
            ArtifactType::Weapon => vec!["Blade of Ages", "Spear of Dawn", "War Axe"],
            ArtifactType::Tool => vec!["Ancient Compass", "First Plow", "Stone Hammer"],
            ArtifactType::Religious => vec!["Sacred Idol", "Altar Stone", "Holy Relic"],
            ArtifactType::Art => vec!["Ancient Mural", "Clay Tablet", "Carved Totem"],
            ArtifactType::Record => vec!["Chronicle Stone", "Story Tablets", "Memory Vase"],
            ArtifactType::Treasure => vec!["Golden Chalice", "Jeweled Crown", "Royal Scepter"],
        };

        let materials = vec!["stone", "bronze", "wood", "bone", "clay", "gold"];

        Artifact {
            id: uuid::Uuid::new_v4().to_string(),
            name: names[(rand::random::<f32>() * (names.len() as f32 - 0.001)) as usize].to_string(),
            artifact_type,
            description: format!("Created during: {}", event.description),
            creator_id: event.affected_settlements.first().cloned(),
            creation_year: year,
            significance: event.significance * 0.8,
            material: materials[(rand::random::<f32>() * (materials.len() as f32 - 0.001)) as usize].to_string(),
            properties: vec![format!("Historical significance: {}", event.significance)],
        }
    }
}

impl Default for ArtifactGenerator {
    fn default() -> Self {
        Self::new()
    }
}
