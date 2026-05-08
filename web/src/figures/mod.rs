//! Figures module - Historical figures generated from significant events

use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::events::{Event, EventType};

/// A specific deed or accomplishment by a historical figure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deed {
    pub id: String,
    pub year: u32,
    pub description: String,
    pub event_type: String,
    pub significance: f32,
    pub affected_settlements: Vec<String>,
}

impl Deed {
    /// Create a deed from an event
    pub fn from_event(event: &Event, year: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            year,
            description: event.description.clone(),
            event_type: format!("{:?}", event.event_type),
            significance: event.significance,
            affected_settlements: event.affected_settlements.clone(),
        }
    }
}

/// Historical figure - a NotableFigure with a typed collection of deeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableFigure {
    pub id: String,
    pub name: String,
    pub figure_type: String,
    pub species_id: String,
    pub birth_year: u32,
    pub death_year: Option<u32>,
    pub significance: f32,
    pub deeds: Vec<Deed>,
    pub related_event_id: Option<String>,
}

/// Figure generator - creates figures from significant events
pub struct FigureGenerator;

impl FigureGenerator {
    pub fn new() -> Self {
        Self
    }

    /// D.5: Create figure from significant event
    pub fn from_event(&mut self, event: &Event, year: u32) -> Option<NotableFigure> {
        // Only create figures for high-significance events
        if event.significance < 0.7 {
            return None;
        }

        let figure_type = match event.event_type {
            EventType::War => "Warrior",
            EventType::Discovery => "Scholar",
            EventType::Peace => "Diplomat",
            EventType::PopulationBoom => "Leader",
            EventType::Migration => "Explorer",
            _ => "Figure",
        };

        let names = ["Aldric", "Brynn", "Cedric", "Diana", "Eldric", "Freya", "Gareth", "Helena"];
        let name = names[(rand::random::<f32>() * (names.len() as f32 - 0.001)) as usize].to_string();

        // Backfill deed from the triggering event
        let deed = Deed::from_event(event, year);

        Some(NotableFigure {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            figure_type: figure_type.to_string(),
            species_id: "default".to_string(),
            birth_year: year,
            death_year: None,
            significance: event.significance,
            deeds: vec![deed],
            related_event_id: Some(event.id.clone()),
        })
    }

    pub fn generate_historical_figure(&mut self, name: String, figure_type: String, year: u32) -> NotableFigure {
        NotableFigure {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            figure_type,
            species_id: "default".to_string(),
            birth_year: year,
            death_year: None,
            significance: rand::random::<f32>() * 0.5 + 0.3,
            deeds: Vec::new(),
            related_event_id: None,
        }
    }
}

impl Default for FigureGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deed_from_event() {
        let event = Event {
            id: "evt-001".to_string(),
            year: 42,
            event_type: EventType::War,
            description: "Battle of the valley".to_string(),
            magnitude: 0.8,
            significance: 0.9,
            affected_settlements: vec!["settle-1".to_string()],
        };

        let deed = Deed::from_event(&event, 42);
        
        assert_eq!(deed.description, "Battle of the valley");
        assert_eq!(deed.event_type, "War");
        assert_eq!(deed.significance, 0.9);
        assert_eq!(deed.affected_settlements, vec!["settle-1"]);
    }

    #[test]
    fn test_notable_figure_from_event() {
        let mut generator = FigureGenerator::new();
        let event = Event {
            id: "evt-002".to_string(),
            year: 100,
            event_type: EventType::Discovery,
            description: "Discovery of fire".to_string(),
            magnitude: 1.0,
            significance: 0.8,
            affected_settlements: vec![],
        };

        let figure = generator.from_event(&event, 100);
        
        assert!(figure.is_some());
        let figure = figure.unwrap();
        assert_eq!(figure.figure_type, "Scholar");
        assert_eq!(figure.deeds.len(), 1);
        assert_eq!(figure.deeds[0].description, "Discovery of fire");
    }

    #[test]
    fn test_notable_figure_below_significance_threshold() {
        let mut generator = FigureGenerator::new();
        let event = Event {
            id: "evt-003".to_string(),
            year: 100,
            event_type: EventType::War,
            description: "Minor skirmish".to_string(),
            magnitude: 0.2,
            significance: 0.3, // Below 0.7 threshold
            affected_settlements: vec![],
        };

        let figure = generator.from_event(&event, 100);
        assert!(figure.is_none());
    }
}