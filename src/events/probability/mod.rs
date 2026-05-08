//! Event Probability & Effect Engine
//!
//! Handles event triggering probability calculation and effect application.
//!
//! ## Probability Calculation
//!
//! Event probability is computed from multiple factors:
//! - **Base probability**: Type-specific base rate
//! - **Environmental modifiers**: Biome, terrain, climate
//! - **Population factors**: Settlement size, density
//! - **Historical context**: Prior events, dependencies
//! - **Random seed**: Deterministic pseudo-randomness from world seed
//!
//! ## Effect Application
//!
//! Effects are applied through a cascade system:
//! 1. Effect validation (target entities exist)
//! 2. Magnitude calculation based on event significance
//! 3. Application order (population > territory > military > economic)
//! 4. Secondary event triggering
//!
//! ## Secondary Event Triggering
//!
//! Primary events can trigger secondary events based on rules:
//! - War → Plague, Famine, Migration
//! - Plague → Migration, Collapse
//! - Battle → Victory, Defeat
//! - Conquest → Government Reform, Cultural Adoption
//!
//! ## Usage
//!
//! ```rust
//! use world_factory::events::probability::{ProbabilityEngine, ProbabilityConfig};
//!
//! let config = ProbabilityConfig::default();
//! let engine = ProbabilityEngine::new(seed, config);
//!
//! // Calculate probability for an event at a location
//! let probability = engine.calculate_event_probability(
//!     EventType::WarDeclared,
//!     &context,
//!     current_year,
//! );
//!
//! // Apply event effects to world state
//! engine.apply_effects(&event, &mut world_state);
//! ```

use crate::events::EventType;
use crate::terrain::biome::BiomeType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub mod effect_application;
pub mod engine;
pub mod event_predictor;
pub mod secondary_events;

pub use effect_application::EffectApplicator;
pub use engine::ProbabilityEngine;
pub use event_predictor::EventPredictor;
pub use secondary_events::{
    default_trigger_rules, SecondaryEventCandidate, SecondaryEventProcessor, SecondaryEventQueue,
    TriggerRule,
};

// ============================================================================
// Probability Configuration
// ============================================================================

/// Configuration for probability calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityConfig {
    /// Base probability multiplier for all events.
    pub base_multiplier: f32,

    /// Enable historical dependency calculations.
    pub enable_historical_context: bool,

    /// Enable population-based probability scaling.
    pub enable_population_scaling: bool,

    /// Enable environmental modifier calculations.
    pub enable_environmental_modifiers: bool,

    /// Maximum probability cap (to prevent near-certain events).
    pub max_probability: f32,

    /// Minimum probability floor (to prevent impossible events).
    pub min_probability: f32,

    /// Variance for random probability modifier.
    pub random_variance: f32,
}

impl Default for ProbabilityConfig {
    fn default() -> Self {
        Self {
            base_multiplier: 1.0,
            enable_historical_context: true,
            enable_population_scaling: true,
            enable_environmental_modifiers: true,
            max_probability: 0.95,
            min_probability: 0.001,
            random_variance: 0.05,
        }
    }
}

// ============================================================================
// Event Context (Input for Probability Calculation)
// ============================================================================

/// Context information needed for probability calculation.
///
/// This struct contains all the environmental and state information
/// that affects event probability calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    /// Primary location ID (region, settlement, etc.).
    pub location_id: Option<Uuid>,

    /// Biome at the location.
    pub biome: Option<BiomeType>,

    /// Population at the location (if applicable).
    pub population: Option<u64>,

    /// Total world population (for density calculations).
    pub world_population: Option<u64>,

    /// Latitude (affects climate-based events).
    pub latitude: Option<f64>,

    /// Season of the year (for seasonal events).
    pub season: Option<Season>,

    /// Active ongoing events that might prevent new events.
    pub active_events: Vec<Uuid>,

    /// Recent event IDs (last N years) - for frequency calculations.
    pub recent_events: Vec<RecentEventInfo>,

    /// neighboring entity IDs for conflict probability.
    pub neighboring_entities: Vec<Uuid>,

    /// Current war/conflict state.
    pub is_at_war: bool,

    /// Trade route connections.
    pub trade_connections: Vec<Uuid>,

    /// Religious/cultural factors.
    pub cultural_tensions: f32,

    /// Economic prosperity level (0.0-1.0).
    pub economic_health: f32,
    /// Figures present at this location.
    #[serde(default)]
    pub figures: Vec<Uuid>,


    /// Figure types present (for quick lookup).
    #[serde(default)]
    pub figure_types: HashSet<super::super::figures::FigureType>,
}

impl EventContext {
    /// Add a figure to this context.
    pub fn add_figure(&mut self, figure_type: super::super::figures::FigureType, figure_id: Uuid) {
        self.figures.push(figure_id);
        self.figure_types.insert(figure_type);
    }

    /// Check if a figure type is present in this context.
    pub fn has_figure_type(&self, figure_type: super::super::figures::FigureType) -> bool {
        self.figure_types.contains(&figure_type)
    }
    /// Count figures of a specific type.
    pub fn figure_count(&self, _figure_type: super::super::figures::FigureType) -> usize {
        // Simplified: return total figure count
        // Full implementation would track counts per type
        self.figures.len()
    }
}

/// Information about recent events for probability calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEventInfo {
    pub event_type: EventType,
    pub year: i32,
    pub significance: f32,
}

/// Season for seasonal event calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Get season from month (1-12).
    pub fn from_month(month: u8) -> Self {
        match month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

// ============================================================================
// Probability Result
// ============================================================================

/// Result of probability calculation with breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityResult {
    /// Final probability value (0.0-1.0).
    pub probability: f32,

    /// Base probability for this event type.
    pub base_probability: f32,

    /// Environmental modifier (biome, climate).
    pub environmental_modifier: f32,

    /// Population modifier.
    pub population_modifier: f32,

    /// Historical context modifier (recent events, dependencies).
    pub historical_modifier: f32,

    /// Random modifier (deterministic seed-based).
    pub random_modifier: f32,

    /// Factors that affect probability.
    pub factors: Vec<ProbabilityFactor>,
}

/// A single factor affecting probability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityFactor {
    pub name: String,
    pub value: f32,
    pub multiplier: f32,
    pub reason: String,
}

impl ProbabilityFactor {
    pub fn new(name: &str, value: f32, multiplier: f32, reason: &str) -> Self {
        Self {
            name: name.to_string(),
            value,
            multiplier,
            reason: reason.to_string(),
        }
    }
}

// ============================================================================
// Effect Application Result
// ============================================================================

/// Result of applying event effects to world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectApplicationResult {
    /// Whether effects were applied successfully.
    pub success: bool,

    /// Number of effects applied.
    pub effects_applied: usize,

    /// Number of secondary events triggered.
    pub secondary_events_triggered: usize,

    /// Errors encountered during application.
    pub errors: Vec<String>,

    /// State changes that occurred.
    pub state_changes: Vec<StateChange>,
}

/// A change to world state from effect application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub entity_id: Uuid,
    pub change_type: StateChangeType,
    pub old_value: String,
    pub new_value: String,
}

/// Types of state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateChangeType {
    Population,
    Territory,
    Military,
    Economic,
    Cultural,
    Political,
    Reputation,
}

impl Default for EffectApplicationResult {
    fn default() -> Self {
        Self {
            success: true,
            effects_applied: 0,
            secondary_events_triggered: 0,
            errors: Vec::new(),
            state_changes: Vec::new(),
        }
    }
}

impl EffectApplicationResult {
    /// Create a successful result.
    pub fn success() -> Self {
        Self::default()
    }

    /// Add a state change.
    pub fn add_state_change(&mut self, change: StateChange) {
        self.state_changes.push(change);
    }

    /// Record an error.
    pub fn add_error(&mut self, error: &str) {
        self.success = false;
        self.errors.push(error.to_string());
    }
}
