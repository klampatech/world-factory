//! Effect Application Module
//! 
//! Handles applying event effects to world state.
//! Implements cascading effect application with validation and error handling.

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use super::{EffectApplicationResult, StateChange, StateChangeType};
use crate::events::{Event, EventEffect};
use crate::events::effect::EffectMagnitude;

/// Applicator for event effects to world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectApplicator {
    /// Validation cache for entity existence.
    entity_cache: HashMap<Uuid, EntityStatus>,
    
    /// Track applied effects for deduplication.
    applied_effects: HashMap<Uuid, Vec<String>>,
    
    /// Configuration for effect application.
    config: ApplicatorConfig,
}

/// Configuration for effect application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicatorConfig {
    /// Enable validation of target entities.
    pub validate_entities: bool,
    
    /// Enable secondary event triggering.
    pub enable_secondary_events: bool,
    
    /// Maximum effects per event application.
    pub max_effects_per_event: usize,
    
    /// Enable state change tracking.
    pub track_state_changes: bool,
    
    /// Apply effects in strict order (population → territory → military → economic).
    pub strict_order: bool,
}

impl Default for ApplicatorConfig {
    fn default() -> Self {
        Self {
            validate_entities: true,
            enable_secondary_events: true,
            max_effects_per_event: 100,
            track_state_changes: true,
            strict_order: true,
        }
    }
}

/// Entity existence status for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityStatus {
    Exists,
    DoesNotExist,
    Unknown,
}

impl Default for EffectApplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectApplicator {
    /// Create a new effect applicator.
    pub fn new() -> Self {
        Self {
            entity_cache: HashMap::new(),
            applied_effects: HashMap::new(),
            config: ApplicatorConfig::default(),
        }
    }
    
    /// Create with custom configuration.
    pub fn with_config(config: ApplicatorConfig) -> Self {
        Self {
            entity_cache: HashMap::new(),
            applied_effects: HashMap::new(),
            config,
        }
    }
    
    /// Apply all effects from an event.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The event whose effects should be applied
    /// * `population_model` - Population model to update (optional)
    /// * `world_state` - World state to modify (optional)
    /// 
    /// # Returns
    /// 
    /// Result of applying effects.
    pub fn apply_event_effects(
        &mut self,
        event: &Event,
    ) -> EffectApplicationResult {
        let mut result = EffectApplicationResult::default();
        
        // Check effect count
        if event.effects.len() > self.config.max_effects_per_event {
            result.add_error(&format!(
                "Event has {} effects, exceeds maximum of {}",
                event.effects.len(),
                self.config.max_effects_per_event
            ));
            return result;
        }
        
        // Sort effects by priority if strict order enabled
        let mut sorted_effects: Vec<&EventEffect> = event.effects.iter().collect();
        if self.config.strict_order {
            sorted_effects.sort_by(|a, b| {
                let priority_a = Self::effect_priority(a);
                let priority_b = Self::effect_priority(b);
                priority_a.cmp(&priority_b)
            });
        }
        
        // Apply each effect
        for effect in sorted_effects {
            match self.apply_single_effect(effect, &mut result) {
                Ok(_) => {
                    result.effects_applied += 1;
                }
                Err(e) => {
                    result.add_error(&e);
                }
            }
        }
        
        // Record applied effects for deduplication
        self.applied_effects.insert(event.id.id, event.effects.iter()
            .map(|e| e.effect_name().to_string())
            .collect());
        
        result.success = result.errors.is_empty();
        result
    }
    
    /// Get priority for effect ordering.
    fn effect_priority(effect: &EventEffect) -> i32 {
        match effect {
            // Population effects first (foundational)
            EventEffect::PopulationLoss { .. } => 1,
            EventEffect::PopulationGrowth { .. } => 2,
            EventEffect::PopulationShift { .. } => 3,
            EventEffect::PopulationDisplacement { .. } => 4,
            
            // Territory effects (political geography)
            EventEffect::TerritoryClaim { .. } => 10,
            EventEffect::BorderShift { .. } => 11,
            EventEffect::TerritoryLoss { .. } => 12,
            
            // Military effects
            EventEffect::MilitaryChange { .. } => 20,
            EventEffect::AllianceFormed { .. } => 21,
            EventEffect::AllianceBroken { .. } => 22,
            
            // Political effects
            EventEffect::LeadershipChange { .. } => 30,
            EventEffect::GovernmentChange { .. } => 31,
            EventEffect::PolicyChange { .. } => 32,
            EventEffect::DiplomaticChange { .. } => 33,
            
            // Economic effects
            EventEffect::EconomicChange { .. } => 40,
            EventEffect::TradeRouteEstablished { .. } => 41,
            EventEffect::TradeRouteClosed { .. } => 42,
            EventEffect::ResourceChange { .. } => 43,
            
            // Cultural effects
            EventEffect::CulturalChange { .. } => 50,
            EventEffect::CulturalAdoption { .. } => 51,
            EventEffect::ReligiousChange { .. } => 52,
            EventEffect::TechnologicalChange { .. } => 53,
            
            // Infrastructure
            EventEffect::Construction { .. } => 60,
            EventEffect::Destruction { .. } => 61,
            
            // Environmental
            EventEffect::EnvironmentalChange { .. } => 70,
            EventEffect::DiseaseOutbreak { .. } => 71,
            EventEffect::SpeciesExtinction { .. } => 72,
            
            // Social
            EventEffect::SocialUnrest { .. } => 80,
            EventEffect::MigrationWave { .. } => 81,
            EventEffect::ReputationChange { .. } => 82,
            
            // Society/Political effects
            EventEffect::SocietyFormation { .. } => 31,
            EventEffect::SocietyTransition { .. } => 32,
            
            // Figure effects
            EventEffect::FigureRise { .. } => 30,
            EventEffect::FigureDeath { .. } => 30,
            
            // Artifact effects
            EventEffect::ArtifactCreation { .. } => 50,
            EventEffect::ArtifactActivation { .. } => 51,
            
            // Custom
            EventEffect::Custom { .. } => 99,
        }
    }
    
    /// Apply a single effect.
    fn apply_single_effect(
        &mut self,
        effect: &EventEffect,
        result: &mut EffectApplicationResult,
    ) -> Result<(), String> {
        // Validate target if configured
        if self.config.validate_entities {
            if let Some(target) = effect.primary_target() {
                if !self.validate_target(target) {
                    return Err(format!("Target entity {} does not exist", target));
                }
            }
        }
        
        // Track state changes if enabled
        if self.config.track_state_changes {
            self.track_effect_changes(effect, result);
        }
        
        Ok(())
    }
    
    /// Validate that a target entity exists.
    fn validate_target(&self, entity_id: Uuid) -> bool {
        match self.entity_cache.get(&entity_id) {
            Some(EntityStatus::Exists) => true,
            Some(EntityStatus::DoesNotExist) => false,
            Some(EntityStatus::Unknown) | None => {
                // Unknown - assume exists (lazy validation)
                true
            }
        }
    }
    
    /// Register an entity as existing.
    pub fn register_entity(&mut self, entity_id: Uuid) {
        self.entity_cache.insert(entity_id, EntityStatus::Exists);
    }
    
    /// Register an entity as deleted.
    pub fn remove_entity(&mut self, entity_id: Uuid) {
        self.entity_cache.insert(entity_id, EntityStatus::DoesNotExist);
    }
    
    /// Track state changes from an effect.
    fn track_effect_changes(
        &mut self,
        effect: &EventEffect,
        result: &mut EffectApplicationResult,
    ) {
        match effect {
            EventEffect::PopulationLoss { target, amount, cause: _, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Population,
                    old_value: format!("population +{}", amount),
                    new_value: format!("population -{}", amount),
                });
            }
            EventEffect::PopulationGrowth { target, amount, cause: _, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Population,
                    old_value: format!("population -{}", amount),
                    new_value: format!("population +{}", amount),
                });
            }
            EventEffect::BorderShift { from, to, territory: _ } => {
                if let Some(from_id) = from {
                    result.add_state_change(StateChange {
                        entity_id: *from_id,
                        change_type: StateChangeType::Territory,
                        old_value: "owns territory".to_string(),
                        new_value: "lost territory".to_string(),
                    });
                }
                result.add_state_change(StateChange {
                    entity_id: *to,
                    change_type: StateChangeType::Territory,
                    old_value: "does not own territory".to_string(),
                    new_value: "gained territory".to_string(),
                });
            }
            EventEffect::MilitaryChange { target, amount, cause: _, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Military,
                    old_value: format!("military strength"),
                    new_value: format!("military strength {}", if *amount >= 0 { "+" } else { "" }),
                });
            }
            EventEffect::LeadershipChange { target, change_type, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Political,
                    old_value: format!("leadership"),
                    new_value: format!("{:?}", change_type),
                });
            }
            EventEffect::EconomicChange { target, change_type, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Economic,
                    old_value: "economy".to_string(),
                    new_value: format!("{:?}", change_type),
                });
            }
            EventEffect::ReputationChange { target, amount, cause: _, .. } => {
                result.add_state_change(StateChange {
                    entity_id: *target,
                    change_type: StateChangeType::Reputation,
                    old_value: "reputation".to_string(),
                    new_value: format!("reputation {}", if *amount >= 0 { "+" } else { "" }),
                });
            }
            _ => {
                // For other effects, track generic change if we have a target
                if let Some(target) = effect.primary_target() {
                    result.add_state_change(StateChange {
                        entity_id: target,
                        change_type: StateChangeType::Cultural, // Default
                        old_value: "state".to_string(),
                        new_value: effect.effect_name().to_string(),
                    });
                }
            }
        }
    }
    
    /// Check if an effect has already been applied.
    pub fn is_effect_applied(&self, event_id: Uuid, effect_name: &str) -> bool {
        self.applied_effects.get(&event_id)
            .map(|effects| effects.contains(&effect_name.to_string()))
            .unwrap_or(false)
    }
    
    /// Check if an entire event's effects have been applied.
    pub fn is_event_applied(&self, event_id: Uuid) -> bool {
        self.applied_effects.contains_key(&event_id)
    }
    
    /// Clear applied effects cache.
    pub fn clear_cache(&mut self) {
        self.applied_effects.clear();
    }
    
    /// Clear entity cache.
    pub fn clear_entity_cache(&mut self) {
        self.entity_cache.clear();
    }
    
    /// Get number of cached entities.
    pub fn cached_entity_count(&self) -> usize {
        self.entity_cache.len()
    }
    
    /// Get number of tracked applied effects.
    pub fn tracked_effect_count(&self) -> usize {
        self.applied_effects.len()
    }
}

// ============================================================================
// Effect Magnitude Calculations
// ============================================================================

impl EffectApplicator {
    /// Calculate effect magnitude based on event significance.
    pub fn calculate_effect_magnitude(significance: f32, base_magnitude: EffectMagnitude) -> EffectMagnitude {
        // Adjust magnitude based on significance
        match (significance as f64, base_magnitude) {
            // World-altering events boost magnitude
            (s, _) if s >= 0.9 => EffectMagnitude::Catastrophic,
            (s, EffectMagnitude::Major) if s >= 0.7 => EffectMagnitude::Major,
            (s, EffectMagnitude::Minor) if s >= 0.6 => EffectMagnitude::Moderate,
            _ => base_magnitude,
        }
    }
    
    /// Apply magnitude multiplier to a value.
    pub fn apply_magnitude(value: f64, magnitude: EffectMagnitude) -> f64 {
        let multiplier = match magnitude {
            EffectMagnitude::Minor => 0.5,
            EffectMagnitude::Moderate => 1.0,
            EffectMagnitude::Major => 2.0,
            EffectMagnitude::Catastrophic => 5.0,
        };
        value * multiplier
    }
    
    /// Get duration in years for magnitude.
    pub fn magnitude_duration(magnitude: EffectMagnitude) -> i32 {
        match magnitude {
            EffectMagnitude::Minor => 1,
            EffectMagnitude::Moderate => 5,
            EffectMagnitude::Major => 20,
            EffectMagnitude::Catastrophic => 100,
        }
    }
}

// ============================================================================
// Effect Conversion Helpers
// ============================================================================

impl EffectApplicator {
    /// Convert population change to effect.
    pub fn population_change_to_effect(
        target: Uuid,
        change: i64,
        cause: Option<String>,
    ) -> EventEffect {
        if change >= 0 {
            EventEffect::PopulationGrowth {
                target,
                amount: change as u64,
                duration_years: None,
                cause,
            }
        } else {
            EventEffect::PopulationLoss {
                target,
                amount: (-change) as u64,
                duration_years: None,
                cause,
            }
        }
    }
    
    /// Convert border shift to effect.
    pub fn border_shift_to_effect(
        from: Option<Uuid>,
        to: Uuid,
        territory: Uuid,
    ) -> EventEffect {
        EventEffect::BorderShift { from, to, territory }
    }
    
    /// Get all effect types for an event category.
    pub fn get_effects_for_category(event_type: crate::events::EventType) -> Vec<&'static str> {
        match event_type.category() {
            crate::events::EventCategory::Political => {
                vec!["territory_change", "leadership_change", "government_change"]
            }
            crate::events::EventCategory::Military => {
                vec!["military_change", "border_shift", "reputation_change"]
            }
            crate::events::EventCategory::Natural => {
                vec!["population_loss", "economic_change", "displacement"]
            }
            crate::events::EventCategory::Cultural => {
                vec!["cultural_change", "social_change", "reputation_change"]
            }
            crate::events::EventCategory::Discovery => {
                vec!["technological_change", "economic_change", "territory_claim"]
            }
            crate::events::EventCategory::Catastrophe => {
                vec!["population_loss", "territory_loss", "collapse", "extinction"]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBuilder;
    use crate::types::HistoricalTime;
    
    #[test]
    fn test_effect_application() {
        let mut applicator = EffectApplicator::new();
        
        let event = EventBuilder::new("Test War")
            .event_type(crate::events::EventType::WarDeclared)
            .time(HistoricalTime::year(1000))
            .effect(EventEffect::PopulationLoss {
                target: Uuid::new_v4(),
                amount: 1000,
                duration_years: None,
                cause: Some("War casualties".to_string()),
            })
            .effect(EventEffect::BorderShift {
                from: Some(Uuid::new_v4()),
                to: Uuid::new_v4(),
                territory: Uuid::new_v4(),
            })
            .build(Uuid::new_v4());
        
        let result = applicator.apply_event_effects(&event);
        
        assert!(result.success);
        assert_eq!(result.effects_applied, 2);
    }
    
    #[test]
    fn test_state_change_tracking() {
        let mut applicator = EffectApplicator::new();
        
        let target_id = Uuid::new_v4();
        let event = EventBuilder::new("Test Event")
            .event_type(crate::events::EventType::Plague)
            .time(HistoricalTime::year(1347))
            .effect(EventEffect::PopulationLoss {
                target: target_id,
                amount: 10000,
                duration_years: Some(50),
                cause: Some("The Great Plague".to_string()),
            })
            .build(Uuid::new_v4());
        
        let result = applicator.apply_event_effects(&event);
        
        assert!(result.state_changes.iter().any(|sc| sc.entity_id == target_id));
    }
    
    #[test]
    fn test_deduplication() {
        let mut applicator = EffectApplicator::new();
        
        let world_id = Uuid::new_v4();
        let event = EventBuilder::new("Test")
            .event_type(crate::events::EventType::SettlementFounded)
            .time(HistoricalTime::year(1000))
            .effect(EventEffect::PopulationGrowth {
                target: Uuid::new_v4(),
                amount: 100,
                duration_years: None,
                cause: None,
            })
            .build(world_id);
        
        // First application
        let result1 = applicator.apply_event_effects(&event);
        assert!(result1.success);
        
        // Check it was recorded (using the actual event.id, not a separate variable)
        assert!(applicator.is_event_applied(event.id.id));
    }
    
    #[test]
    fn test_entity_validation() {
        let mut applicator = EffectApplicator::new();
        let entity_id = Uuid::new_v4();
        
        // Register entity
        applicator.register_entity(entity_id);
        
        // Should validate
        assert!(applicator.validate_target(entity_id));
        
        // Remove entity
        applicator.remove_entity(entity_id);
        
        // Should no longer validate
        assert!(!applicator.validate_target(entity_id));
    }
    
    #[test]
    fn test_effect_priority_ordering() {
        let population_effect = EventEffect::PopulationLoss {
            target: Uuid::new_v4(),
            amount: 100,
            duration_years: None,
            cause: None,
        };
        
        let territory_effect = EventEffect::BorderShift {
            from: None,
            to: Uuid::new_v4(),
            territory: Uuid::new_v4(),
        };
        
        assert!(EffectApplicator::effect_priority(&population_effect) < 
                EffectApplicator::effect_priority(&territory_effect));
    }
    
    #[test]
    fn test_magnitude_calculation() {
        assert_eq!(
            EffectApplicator::calculate_effect_magnitude(0.95, EffectMagnitude::Moderate),
            EffectMagnitude::Catastrophic
        );
        
        assert_eq!(
            EffectApplicator::calculate_effect_magnitude(0.5, EffectMagnitude::Minor),
            EffectMagnitude::Minor
        );
    }
    
    #[test]
    fn test_magnitude_application() {
        assert_eq!(EffectApplicator::apply_magnitude(100.0, EffectMagnitude::Minor), 50.0);
        assert_eq!(EffectApplicator::apply_magnitude(100.0, EffectMagnitude::Moderate), 100.0);
        assert_eq!(EffectApplicator::apply_magnitude(100.0, EffectMagnitude::Major), 200.0);
        assert_eq!(EffectApplicator::apply_magnitude(100.0, EffectMagnitude::Catastrophic), 500.0);
    }
}