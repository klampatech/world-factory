//! History result types

use serde::{Deserialize, Serialize};

/// Result of the entire prehistory generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub world_id: String,
    pub total_years: u32,
    pub years: Vec<YearResult>,
    pub final_state: WorldState,
    pub timeline_events: Vec<TimelineEvent>,
}

/// Result of a single year
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearResult {
    pub year: u32,
    pub population_delta: i32,
    pub events_generated: u32,
    pub figures_created: u32,
    pub artifacts_created: u32,
    pub society_transitions: Vec<SocietyTransition>,
}

/// Final world state at end of prehistory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub populations: Vec<PopulationState>,
    pub settlements: Vec<SettlementState>,
    pub territories: Vec<TerritoryState>,
    pub figures: Vec<FigureState>,
    pub artifacts: Vec<ArtifactState>,
}

/// Population state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationState {
    pub species_id: String,
    pub population: u32,
    pub growth_rate: f32,
    pub carrying_capacity: u32,
}

/// Settlement state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementState {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub society_type: SocietyType,
    pub territory_id: String,
}

/// Territory state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryState {
    pub id: String,
    pub name: String,
    pub owner_species: String,
    pub size: u32,
    pub clusters: Vec<String>,
}

/// Deed - a specific accomplishment by a figure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedState {
    pub id: String,
    pub year: u32,
    pub description: String,
    pub event_type: String,
    pub significance: f32,
    pub affected_settlements: Vec<String>,
}

/// Figure state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureState {
    pub id: String,
    pub name: String,
    pub figure_type: String,
    pub species_id: String,
    pub birth_year: u32,
    pub significance: f32,
    pub deeds: Vec<DeedState>,
}

/// Artifact state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactState {
    pub id: String,
    pub name: String,
    pub artifact_type: String,
    pub creator_id: Option<String>,
    pub creation_year: u32,
    pub significance: f32,
}

/// Society type progression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocietyType {
    Band,
    Tribe,
    Chiefdom,
    Nation,
}

impl SocietyType {
    pub fn next(&self) -> Option<Self> {
        match self {
            SocietyType::Band => Some(SocietyType::Tribe),
            SocietyType::Tribe => Some(SocietyType::Chiefdom),
            SocietyType::Chiefdom => Some(SocietyType::Nation),
            SocietyType::Nation => None,
        }
    }
}

/// Society transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocietyTransition {
    pub settlement_id: String,
    pub from: SocietyType,
    pub to: SocietyType,
    pub year: u32,
}

/// Timeline event for history archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub year: u32,
    pub event_type: String,
    pub description: String,
    pub affected_entities: Vec<String>,
    pub significance: f32,
}
