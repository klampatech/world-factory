//! Notable Figures System for World Factory
//! 
//! This module provides historical figure management for procedural world generation.
//! Figures are generated from significant events and linked to settlements, species,
//! and other world entities.
//!
//! ## Core Types
//!
//! - [`NotableFigure`] - A person with historical significance, extending the base `Person` type
//! - [`FigureType`] - Enum categorizing figure roles (Monarch, MilitaryLeader, Scholar, etc.)
//! - [`FigureStore`] - In-memory storage for figure collections
//!
//! ## Generation
//!
//! The [`FigureGenerator`] creates statistically valid historical figures based on:
//! - World era and timeline density
//! - Significant historical events
//! - Settlement populations and cultures
//! - Power-law significance distribution

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::{EntityId, EntityType, Timestamp, HistoricalTime};
use crate::events::{Event, EventType};
use crate::util::Rng;

// ============================================================================
// Figure Types
// ============================================================================

/// Types of notable figures in world history
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FigureType {
    /// Ruling monarch (king, queen, emperor, etc.)
    Monarch,
    /// Military commander or general
    MilitaryLeader,
    /// Scholar, philosopher, scientist
    Scholar,
    /// Artist, musician, poet
    Artist,
    /// Religious leader or prophet
    ReligiousLeader,
    /// Explorer or adventurer
    Explorer,
    /// Inventor or engineer
    Inventor,
    /// Hero who performed notable deeds
    Hero,
    /// Villain or antagonist figure
    Villain,
    /// Folk hero from oral tradition
    FolkHero,
    /// Legendary or mythical figure
    Legendary,
}

impl FigureType {
    /// Get human-readable label for the figure type
    pub fn label(&self) -> &'static str {
        match self {
            FigureType::Monarch => "Monarch",
            FigureType::MilitaryLeader => "Military Leader",
            FigureType::Scholar => "Scholar",
            FigureType::Artist => "Artist",
            FigureType::ReligiousLeader => "Religious Leader",
            FigureType::Explorer => "Explorer",
            FigureType::Inventor => "Inventor",
            FigureType::Hero => "Hero",
            FigureType::Villain => "Villain",
            FigureType::FolkHero => "Folk Hero",
            FigureType::Legendary => "Legendary Figure",
        }
    }

    /// Get typical lifespan modifier for this figure type
    /// Monarchs and scholars often live longer, military leaders in wartime may die younger
    pub fn lifespan_modifier(&self) -> f32 {
        match self {
            FigureType::Monarch => 1.1,        // Better nutrition, care
            FigureType::MilitaryLeader => 0.85, // Combat risk
            FigureType::Scholar => 1.15,       // Sedentary life, access to medicine
            FigureType::Artist => 1.0,
            FigureType::ReligiousLeader => 1.1,
            FigureType::Explorer => 0.8,       // Hardship and danger
            FigureType::Inventor => 1.05,
            FigureType::Hero => 0.75,          // Heroic deaths
            FigureType::Villain => 0.9,
            FigureType::FolkHero => 1.0,
            FigureType::Legendary => 1.5,      // Mythical longevity
        }
    }

    /// Determine figure type from an event
    pub fn from_event(event_type: &EventType) -> Option<Self> {
        match event_type {
            EventType::SettlementFounded => Some(FigureType::Monarch),
            EventType::WarDeclared | EventType::Battle => Some(FigureType::MilitaryLeader),
            EventType::TreatySigned | EventType::AllianceFormed => Some(FigureType::Monarch),
            EventType::Plague | EventType::Famine => Some(FigureType::ReligiousLeader),
            EventType::Earthquake | EventType::Flood | EventType::Volcano => Some(FigureType::Hero),
            EventType::ArtCreated | EventType::Festival => Some(FigureType::Artist),
            EventType::Exploration | EventType::Discovery => Some(FigureType::Explorer),
            EventType::Invention => Some(FigureType::Inventor),
            EventType::ReligiousReformation | EventType::ReligiousReveal => Some(FigureType::ReligiousLeader),
            EventType::ScholarlyWork => Some(FigureType::Scholar),
            EventType::Conquest => Some(FigureType::MilitaryLeader),
            EventType::Assassination => Some(FigureType::Villain),
            EventType::HeroicAct => Some(FigureType::Hero),
            _ => None,
        }
    }
}

/// Person name with optional components for figures
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FigureName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epithet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl FigureName {
    pub fn new(given: Option<String>, family: Option<String>) -> Self {
        Self {
            given,
            family,
            epithet: None,
            title: None,
        }
    }

    pub fn with_epithet(mut self, epithet: String) -> Self {
        self.epithet = Some(epithet);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
}

/// Notable figure - a person with historical significance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableFigure {
    /// Unique identifier
    pub id: EntityId,
    
    /// World this figure belongs to
    pub world_id: Uuid,
    
    /// Person name components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<FigureName>,
    
    /// Type/category of figure
    pub figure_type: FigureType,
    
    /// Era or period name (e.g., "The Age of Kings", "The Dark Century")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era: Option<String>,
    
    /// Birth year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    
    /// Death year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_year: Option<i32>,
    
    /// Birthplace settlement ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace_id: Option<Uuid>,
    
    /// Primary culture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,
    
    /// Titles held (King, General, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titles: Option<Vec<String>>,
    
    /// One-line description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Full biography text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    
    /// Major accomplishments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accomplishments: Option<Vec<String>>,
    
    /// Historical significance (0.0 to 1.0)
    pub significance: f32,
    
    /// Related event IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_events: Option<Vec<Uuid>>,
    
    /// Related figure IDs (allies, rivals, family)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_figures: Option<Vec<Uuid>>,
    
    /// Species ID if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_id: Option<Uuid>,
    
    /// Region of influence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influence_region_id: Option<Uuid>,
    
    /// Timestamp
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl NotableFigure {
    /// Create a new notable figure
    pub fn new(world_id: Uuid, figure_type: FigureType, significance: f32) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Event), // Uses Event type for figures
            world_id,
            name: None,
            figure_type,
            era: None,
            birth_year: None,
            death_year: None,
            birthplace_id: None,
            culture: None,
            titles: None,
            description: None,
            biography: None,
            accomplishments: None,
            significance: significance.clamp(0.0, 1.0),
            related_events: None,
            related_figures: None,
            species_id: None,
            influence_region_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a related event
    pub fn add_event(&mut self, event_id: Uuid) {
        match &mut self.related_events {
            Some(events) => {
                if !events.contains(&event_id) {
                    events.push(event_id);
                }
            }
            None => self.related_events = Some(vec![event_id]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Add a related figure
    pub fn add_related_figure(&mut self, figure_id: Uuid) {
        match &mut self.related_figures {
            Some(figures) => {
                if !figures.contains(&figure_id) {
                    figures.push(figure_id);
                }
            }
            None => self.related_figures = Some(vec![figure_id]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Add an accomplishment
    pub fn add_accomplishment(&mut self, accomplishment: String) {
        match &mut self.accomplishments {
            Some(accomplishments) => accomplishments.push(accomplishment),
            None => self.accomplishments = Some(vec![accomplishment]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Get lifespan in years, if birth/death known
    pub fn lifespan(&self) -> Option<i32> {
        match (self.birth_year, self.death_year) {
            (Some(b), Some(d)) if d > b => Some(d - b),
            _ => None,
        }
    }
}

// ============================================================================
// Figure Store
// ============================================================================

/// In-memory storage for notable figures
#[derive(Debug, Clone, Default)]
pub struct FigureStore {
    figures: std::collections::HashMap<Uuid, NotableFigure>,
}

impl FigureStore {
    pub fn new() -> Self {
        Self {
            figures: std::collections::HashMap::new(),
        }
    }

    /// Add a figure to the store
    pub fn add(&mut self, figure: NotableFigure) {
        self.figures.insert(figure.id.to_uuid(), figure);
    }

    /// Get a figure by ID
    pub fn get(&self, id: &Uuid) -> Option<&NotableFigure> {
        self.figures.get(id)
    }

    /// Get a mutable figure by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut NotableFigure> {
        self.figures.get_mut(id)
    }

    /// Get all figures for a world
    pub fn get_by_world(&self, world_id: &Uuid) -> Vec<&NotableFigure> {
        self.figures.values().filter(|f| f.world_id == *world_id).collect()
    }

    /// Get figures filtered by type
    pub fn get_by_type(&self, world_id: &Uuid, figure_type: FigureType) -> Vec<&NotableFigure> {
        self.figures.values()
            .filter(|f| f.world_id == *world_id && f.figure_type == figure_type)
            .collect()
    }

    /// Get figures above a significance threshold
    pub fn get_by_significance(&self, world_id: &Uuid, min_significance: f32) -> Vec<&NotableFigure> {
        self.figures.values()
            .filter(|f| f.world_id == *world_id && f.significance >= min_significance)
            .collect()
    }

    /// Get figures by region
    pub fn get_by_region(&self, world_id: &Uuid, region_id: &Uuid) -> Vec<&NotableFigure> {
        self.figures.values()
            .filter(|f| f.world_id == *world_id && f.influence_region_id == Some(*region_id))
            .collect()
    }

    /// Get total count for a world
    pub fn count(&self, world_id: &Uuid) -> usize {
        self.figures.values().filter(|f| f.world_id == *world_id).count()
    }

    /// List all figures (for iteration)
    pub fn figures(&self) -> impl Iterator<Item = &NotableFigure> {
        self.figures.values()
    }
}

// ============================================================================
// Figure Generator
// ============================================================================

/// Configuration for figure generation
#[derive(Debug, Clone)]
pub struct FigureGeneratorConfig {
    /// Maximum figures per world
    pub max_figures: u32,
    /// Ratio of figures that are "notable" (significance > 0.5)
    pub notable_ratio: f32,
    /// Average lifespan in years
    pub average_lifespan: f32,
    /// Base significance for generated figures
    pub base_significance: f32,
}

impl Default for FigureGeneratorConfig {
    fn default() -> Self {
        Self {
            max_figures: 500,
            notable_ratio: 0.1,      // 10% are highly notable
            average_lifespan: 70.0,
            base_significance: 0.3,
        }
    }
}

/// Figure generator that creates notable figures from events
#[derive(Debug, Clone)]
pub struct FigureGenerator {
    config: FigureGeneratorConfig,
}

impl FigureGenerator {
    pub fn new(config: FigureGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate figures from a set of significant events
    pub fn generate_from_events(
        &self,
        world_id: Uuid,
        events: &[Event],
        settlement_ids: &[Uuid],
        cultures: &[String],
        rng: &mut Rng,
    ) -> Vec<NotableFigure> {
        let mut figures = Vec::new();
        
        // Filter significant events (significance >= 0.5)
        let significant_events: Vec<_> = events.iter()
            .filter(|e| e.significance.unwrap_or(0.0) >= 0.5)
            .collect();

        for event in significant_events {
            // Skip if too many figures
            if figures.len() >= self.config.max_figures as usize {
                break;
            }

            // Determine figure type from event
            let Some(figure_type) = FigureType::from_event(&event.event_type) else {
                continue;
            };

            // Generate significance based on event significance
            let base_sig = event.significance.unwrap_or(self.config.base_significance);
            let significance = self.generate_significance(base_sig, rng);

            // Only generate notable figures above threshold
            if significance < 0.5 {
                continue;
            }

            let mut figure = NotableFigure::new(world_id, figure_type, significance);
            
            // Set birth/death years based on event time
            let event_year = event.time.get_year();
            let lifespan = self.generate_lifespan(figure_type, rng);
            
            // Figures born before event (they participate in it)
            figure.birth_year = Some(event_year - (lifespan * 0.6) as i32);
            figure.death_year = Some(event_year + ((lifespan * 0.4) as i32).max(1));
            
            // Link to event
            figure.add_event(event.id.to_uuid());
            
            // Add description from event
            figure.description = Some(event.description.clone());
            
            // Assign culture if available
            if !cultures.is_empty() {
                figure.culture = Some(cultures[rng.random_usize() % cultures.len()].clone());
            }
            
            // Assign birthplace if settlements available
            if !settlement_ids.is_empty() {
                figure.birthplace_id = Some(settlement_ids[rng.random_usize() % settlement_ids.len()]);
            }

            // Generate accomplishments based on figure type
            self.generate_accomplishments(&mut figure, &event, rng);

            figures.push(figure);
        }

        figures
    }

    /// Generate a random notable figure
    pub fn generate_random(
        &self,
        world_id: Uuid,
        year: i32,
        rng: &mut Rng,
    ) -> NotableFigure {
        let figure_types = [
            FigureType::Monarch,
            FigureType::MilitaryLeader,
            FigureType::Scholar,
            FigureType::Artist,
            FigureType::ReligiousLeader,
        ];
        
        let figure_type = figure_types[rng.random_usize() % figure_types.len()];
        let lifespan = self.generate_lifespan(figure_type, rng);
        let significance = self.generate_significance(self.config.base_significance, rng);

        let mut figure = NotableFigure::new(world_id, figure_type, significance);
        figure.birth_year = Some(year - (lifespan * 0.6) as i32);
        figure.death_year = Some(year + ((lifespan * 0.4) as i32).max(1));
        
        figure
    }

    /// Generate lifespan based on figure type
    fn generate_lifespan(&self, figure_type: FigureType, rng: &mut Rng) -> f32 {
        let base = self.config.average_lifespan;
        let modifier = figure_type.lifespan_modifier();
        
        // Add some randomness (+/- 20%)
        let variance = rng.random_f32() * 0.4 - 0.2;
        (base * modifier * (1.0 + variance)).max(20.0)
    }

    /// Generate significance using power-law distribution
    /// Most figures are less notable, few are legendary
    fn generate_significance(&self, base: f32, rng: &mut Rng) -> f32 {
        // Power-law: sqrt(random) gives distribution favoring lower values
        let power = rng.random_f32().sqrt();
        let significance = base + (1.0 - base) * (1.0 - power);
        significance.clamp(0.0, 1.0)
    }

    /// Generate accomplishments for a figure based on their type
    fn generate_accomplishments(
        &self,
        figure: &mut NotableFigure,
        event: &Event,
        rng: &mut Rng,
    ) {
        let accomplishments = match figure.figure_type {
            FigureType::Monarch => vec![
                format!("Ruled during the {}", event.name),
                "Expanded the kingdom's borders".to_string(),
                "Established a new dynasty".to_string(),
            ],
            FigureType::MilitaryLeader => vec![
                format!("Led the forces during {}", event.name),
                "Never lost a major battle".to_string(),
                "Conquered neighboring territories".to_string(),
            ],
            FigureType::Scholar => vec![
                format!("Authored the famous treatise on {}", event.name),
                "Founded a school of thought".to_string(),
                "Mentored the next generation".to_string(),
            ],
            FigureType::Artist => vec![
                format!("Created masterworks inspired by {}", event.name),
                "Revolutionized the art form".to_string(),
            ],
            FigureType::ReligiousLeader => vec![
                format!("Led the faithful through {}", event.name),
                "Established new religious practices".to_string(),
            ],
            FigureType::Explorer => vec![
                format!("Discovered new lands during {}", event.name),
                "Mapped previously unknown territories".to_string(),
            ],
            FigureType::Inventor => vec![
                format!("Invented new technology during {}", event.name),
                "Revolutionized daily life".to_string(),
            ],
            FigureType::Hero => vec![
                format!("Saved the realm during {}", event.name),
                "Became a symbol of courage".to_string(),
            ],
            FigureType::Villain => vec![
                format!("Perpetrated {}", event.name),
                "Brought terror to the land".to_string(),
            ],
            FigureType::FolkHero => vec![
                format!("Legend of {}", event.name),
                "Stories told for generations".to_string(),
            ],
            FigureType::Legendary => vec![
                format!("Figured in the myths of {}", event.name),
                "Said to possess supernatural powers".to_string(),
            ],
        };

        // Select 1-3 random accomplishments
        let count = 1 + rng.random_usize() % 3;
        for i in 0..count.min(accomplishments.len()) {
            figure.add_accomplishment(accomplishments[i].clone());
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventType;

    #[test]
    fn test_figure_type_from_event() {
        assert_eq!(
            FigureType::from_event(&EventType::Battle),
            Some(FigureType::MilitaryLeader)
        );
        assert_eq!(
            FigureType::from_event(&EventType::Invention),
            Some(FigureType::Inventor)
        );
        assert_eq!(
            FigureType::from_event(&EventType::ReligiousReformation),
            Some(FigureType::ReligiousLeader)
        );
    }

    #[test]
    fn test_figure_lifespan() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::Monarch, 0.8);
        figure.birth_year = Some(1000);
        figure.death_year = Some(1060);
        
        assert_eq!(figure.lifespan(), Some(60));
    }

    #[test]
    fn test_figure_store() {
        let mut store = FigureStore::new();
        let world_id = Uuid::new_v4();
        
        let figure = NotableFigure::new(world_id, FigureType::Hero, 0.9);
        let id = figure.id.to_uuid();
        store.add(figure);
        
        assert_eq!(store.count(&world_id), 1);
        assert!(store.get(&id).is_some());
        
        let by_type = store.get_by_type(&world_id, FigureType::Hero);
        assert_eq!(by_type.len(), 1);
    }

    #[test]
    fn test_significance_generation() {
        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);
        let mut rng = Rng::new(crate::util::Seed::new(42));
        
        let mut total = 0.0;
        let iterations = 1000;
        
        for _ in 0..iterations {
            let sig = generator.generate_significance(0.3, &mut rng);
            total += sig;
        }
        
        let average = total / iterations as f32;
        // Average should be roughly in the 0.3-0.6 range
        assert!(average > 0.3 && average < 0.7);
    }
}
