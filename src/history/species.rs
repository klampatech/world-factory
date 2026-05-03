//! History Module - Species Data Model
//! 
//! Phase 2 implementation of extended species definitions.
//! Includes behaviors, society types, base stats, and plugin loading.
//! 
//! ## Architecture
//! 
//! This module provides the species-agnostic core guarantee via the
//! `OnlyInHistory` marker trait. Types that implement this trait are
//! guaranteed to only exist in the history layer, ensuring species-agnostic
//! core systems don't depend on species-specific logic.
//! 
//! ## SpeciesTemplate vs Species
//! 
//! - `SpeciesTemplate`: Extended definition with behaviors, stats, society types
//! - `Species`: Core species definition (biomes, climate, traits)
//! 
//! Templates are loaded from YAML/JSON and converted to Species at runtime.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

// Re-export core species types for convenience
use crate::species::{SpeciesId, SpeciesTrait};

/// Errors from species history operations.
#[derive(Debug, Error)]
pub enum SpeciesHistoryError {
    #[error("Failed to read template file: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to parse template: {0}")]
    Parse(#[from] serde_yaml::Error),
    
    #[error("Species '{0}' not found")]
    NotFound(String),
    
    #[error("Invalid species ID: {0}")]
    InvalidId(String),
    
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    
    #[error("Template validation failed: {0}")]
    Validation(String),
}

// =============================================================================
// OnlyInHistory Marker Trait
// =============================================================================

/// Marker trait for types that only exist in the history layer.
/// 
/// This trait provides the species-agnostic core guarantee:
/// - Types implementing `OnlyInHistory` are guaranteed to only be created
///   or used within the history/timeline system
/// - Core engine types should NOT implement this trait
/// - This allows static analysis to verify species-agnosticism
/// 
/// # Example
/// 
/// ```ignore
/// impl OnlyInHistory for HistoricalEvent { ... }
/// 
/// // Core types should NOT implement this:
/// // impl OnlyInHistory for TerrainGrid { ... } // Compile error!
/// ```
pub trait OnlyInHistory: private::Sealed {
    /// Returns the species ID if this type is species-specific.
    fn species_id(&self) -> Option<SpeciesId>;
}

mod private {
    /// Sealed trait to prevent external implementors.
    pub trait Sealed {}
    
    // Species history types that implement OnlyInHistory
    impl Sealed for super::SpeciesTemplate {}
    impl Sealed for super::SpeciesBehavior {}
    impl Sealed for super::SpeciesStat {}
    impl Sealed for super::SpeciesSocietyType {}
    impl Sealed for super::SocietyEvolution {}
    impl Sealed for super::SpeciesStats {}
}

// =============================================================================
// Extended SpeciesTemplate
// =============================================================================

/// Extended species template with Phase 2 additions.
/// 
/// This template includes:
/// - Base name and plural
/// - Base traits from core species system
/// - Behaviors affecting historical events
/// - Society types the species can form
/// - Base statistics for simulation
/// 
/// # YAML Format
/// 
/// ```yaml
/// name: Human
/// plural: Humans
/// base_traits:
///   - Adaptable
///   - Curious
///   - Sedentary
///   - TradeFocused
/// behaviors:
///   exploration: 0.8
///   diplomacy: 0.7
///   aggression: 0.3
/// society_types:
///   - Tribe
///   - Chiefdom
///   - Nation
/// base_stats:
///   growth_rate: 1.0
///   max_lifespan: 100
///   migration_speed: 0.5
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesTemplate {
    /// Unique species identifier.
    pub id: SpeciesId,
    
    /// Base name (singular).
    pub name: String,
    
    /// Plural name for display.
    pub plural: String,
    
    /// Display name for UI.
    pub display_name: String,
    
    /// Description of the species.
    pub description: String,
    
    /// Base traits from core species system.
    #[serde(default)]
    pub base_traits: Vec<SpeciesTrait>,
    
    /// Behaviors affecting historical events (0.0 - 1.0).
    #[serde(default)]
    pub behaviors: SpeciesBehaviors,
    
    /// Society types this species can form.
    #[serde(default)]
    pub society_types: Vec<SpeciesSocietyType>,
    
    /// Base statistics for simulation.
    #[serde(default)]
    pub base_stats: SpeciesStats,
}

impl OnlyInHistory for SpeciesTemplate {
    fn species_id(&self) -> Option<SpeciesId> {
        Some(self.id)
    }
}

/// Species behaviors affecting historical events.
/// 
/// Each behavior is a value from 0.0 to 1.0:
/// - 0.0: Never exhibits this behavior
/// - 0.5: Sometimes exhibits this behavior
/// - 1.0: Always exhibits this behavior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesBehaviors {
    /// Tendency to explore and discover new territories.
    /// High: More exploration events, faster map coverage.
    #[serde(default)]
    pub exploration: f32,
    
    /// Tendency to negotiate and form alliances.
    /// High: More treaties, fewer wars.
    #[serde(default)]
    pub diplomacy: f32,
    
    /// Tendency toward violent conflict.
    /// High: More wars, faster territorial expansion through conquest.
    #[serde(default)]
    pub aggression: f32,
    
    /// Tendency to establish trade networks.
    /// High: More trade events, faster economic development.
    #[serde(default)]
    pub trade: f32,
    
    /// Tendency to build monumental structures.
    /// High: More construction events, larger cities.
    #[serde(default)]
    pub monument_building: f32,
    
    /// Tendency toward religious/spiritual pursuits.
    /// High: More cultural events, sacred site construction.
    #[serde(default)]
    pub religious: f32,
    
    /// Tendency toward scientific/technological advancement.
    /// High: More innovation events, faster tech progression.
    #[serde(default)]
    pub scientific: f32,
}

impl SpeciesBehaviors {
    /// Create behaviors with all defaults (0.5).
    pub fn default_balanced() -> Self {
        Self {
            exploration: 0.5,
            diplomacy: 0.5,
            aggression: 0.5,
            trade: 0.5,
            monument_building: 0.5,
            religious: 0.5,
            scientific: 0.5,
        }
    }
    
    /// Validate behavior values are in range.
    pub fn validate(&self) -> Result<(), SpeciesHistoryError> {
        let behaviors = [
            ("exploration", self.exploration),
            ("diplomacy", self.diplomacy),
            ("aggression", self.aggression),
            ("trade", self.trade),
            ("monument_building", self.monument_building),
            ("religious", self.religious),
            ("scientific", self.scientific),
        ];
        
        for (name, value) in behaviors {
            if !(0.0..=1.0).contains(&value) {
                return Err(SpeciesHistoryError::Validation(
                    format!("Species behavior '{}' value {} out of range [0.0, 1.0]", name, value)
                ));
            }
        }
        Ok(())
    }
}

// =============================================================================
// Society Types
// =============================================================================

/// Society types a species can form.
/// 
/// Each society type has evolution prerequisites and characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeciesSocietyType {
    /// Small family groups (10-50 people).
    Band,
    /// Small, family-based groups (50-500 people).
    Tribe,
    /// Larger groups with leadership (500-5000 people).
    Chiefdom,
    /// Complex societies with institutions (5000+ people).
    Nation,
    /// Maritime-focused societies.
    Maritime,
    /// Religious/theocratic societies.
    Theocracy,
    /// Nomadic traveling societies.
    Nomadic,
}

impl SpeciesSocietyType {
    /// Get population range for this society type.
    pub fn population_range(&self) -> (u32, u32) {
        match self {
            SpeciesSocietyType::Band => (10, 50),
            SpeciesSocietyType::Tribe => (50, 500),
            SpeciesSocietyType::Chiefdom => (500, 5000),
            SpeciesSocietyType::Nation => (5000, u32::MAX),
            SpeciesSocietyType::Maritime => (200, 20000),
            SpeciesSocietyType::Theocracy => (500, 10000),
            SpeciesSocietyType::Nomadic => (100, 5000),
        }
    }
    
    /// Get the next society type in evolution.
    pub fn evolve_to(&self) -> Option<SpeciesSocietyType> {
        match self {
            SpeciesSocietyType::Band => Some(SpeciesSocietyType::Tribe),
            SpeciesSocietyType::Tribe => Some(SpeciesSocietyType::Chiefdom),
            SpeciesSocietyType::Chiefdom => Some(SpeciesSocietyType::Nation),
            _ => None,
        }
    }
}

impl OnlyInHistory for SpeciesSocietyType {
    fn species_id(&self) -> Option<SpeciesId> {
        None
    }
}

/// Evolution of a society over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocietyEvolution {
    /// Society type after evolution.
    pub new_type: SpeciesSocietyType,
    /// Years required for evolution.
    pub years_required: u32,
    /// Population threshold for evolution.
    pub population_threshold: u32,
    /// Events that can trigger early evolution.
    #[serde(default)]
    pub triggers: Vec<String>,
}

impl OnlyInHistory for SocietyEvolution {
    fn species_id(&self) -> Option<SpeciesId> {
        None
    }
}

// =============================================================================
// Base Statistics
// =============================================================================

/// Base statistics for species simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesStats {
    /// Population growth rate multiplier (1.0 = baseline).
    #[serde(default)]
    pub growth_rate: f32,
    
    /// Maximum lifespan in years.
    #[serde(default)]
    pub max_lifespan: u32,
    
    /// Migration speed (0.0 = stationary, 1.0 = instant).
    #[serde(default)]
    pub migration_speed: f32,
    
    /// Food consumption per person per year (arbitrary units).
    #[serde(default)]
    pub food_consumption: f32,
    
    /// Base birth rate per 1000 people per year.
    #[serde(default)]
    pub birth_rate: f32,
    
    /// Base death rate per 1000 people per year.
    #[serde(default)]
    pub death_rate: f32,
    
    /// Natural disaster resistance (0.0 = none, 1.0 = complete).
    #[serde(default)]
    pub disaster_resistance: f32,
}

impl SpeciesStats {
    /// Validate stats are within reasonable ranges.
    pub fn validate(&self) -> Result<(), SpeciesHistoryError> {
        if self.growth_rate <= 0.0 {
            return Err(SpeciesHistoryError::Validation(
                format!("growth_rate must be positive, got {}", self.growth_rate)
            ));
        }
        if self.max_lifespan == 0 || self.max_lifespan > 10000 {
            return Err(SpeciesHistoryError::Validation(
                format!("max_lifespan must be 1-10000, got {}", self.max_lifespan)
            ));
        }
        if !(0.0..=1.0).contains(&self.migration_speed) {
            return Err(SpeciesHistoryError::Validation(
                format!("migration_speed {} out of range [0.0, 1.0]", self.migration_speed)
            ));
        }
        Ok(())
    }
}

impl OnlyInHistory for SpeciesStats {
    fn species_id(&self) -> Option<SpeciesId> {
        None
    }
}

/// Individual species statistic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeciesStat {
    GrowthRate,
    MaxLifespan,
    MigrationSpeed,
    FoodConsumption,
    BirthRate,
    DeathRate,
    DisasterResistance,
}

impl OnlyInHistory for SpeciesStat {
    fn species_id(&self) -> Option<SpeciesId> {
        None
    }
}

// =============================================================================
// SpeciesBehavior (for events)
// =============================================================================

/// Behavior affecting historical events.
/// 
/// This is a simplified enum for event system integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeciesBehavior {
    Exploration,
    Diplomacy,
    Aggression,
    Trade,
    MonumentBuilding,
    Religious,
    Scientific,
}

impl SpeciesBehavior {
    /// Get the behavior value from species template.
    pub fn get_value(&self, template: &SpeciesTemplate) -> f32 {
        match self {
            SpeciesBehavior::Exploration => template.behaviors.exploration,
            SpeciesBehavior::Diplomacy => template.behaviors.diplomacy,
            SpeciesBehavior::Aggression => template.behaviors.aggression,
            SpeciesBehavior::Trade => template.behaviors.trade,
            SpeciesBehavior::MonumentBuilding => template.behaviors.monument_building,
            SpeciesBehavior::Religious => template.behaviors.religious,
            SpeciesBehavior::Scientific => template.behaviors.scientific,
        }
    }
}

impl OnlyInHistory for SpeciesBehavior {
    fn species_id(&self) -> Option<SpeciesId> {
        None
    }
}

// =============================================================================
// Template Loading
// =============================================================================

/// Loader for species templates from YAML files.
pub struct TemplateLoader {
    /// Trait name mappings for parsing.
    trait_map: HashMap<String, SpeciesTrait>,
}

impl TemplateLoader {
    /// Create a new template loader.
    pub fn new() -> Self {
        Self {
            trait_map: Self::build_trait_map(),
        }
    }
    
    /// Build trait name → SpeciesTrait mapping.
    fn build_trait_map() -> HashMap<String, SpeciesTrait> {
        HashMap::from([
            ("Aquatic".to_string(), SpeciesTrait::Aquatic),
            ("Flying".to_string(), SpeciesTrait::Flying),
            ("Subterranean".to_string(), SpeciesTrait::Subterranean),
            ("Nocturnal".to_string(), SpeciesTrait::Nocturnal),
            ("PackHunter".to_string(), SpeciesTrait::PackHunter),
            ("Nomadic".to_string(), SpeciesTrait::Nomadic),
            ("Sedentary".to_string(), SpeciesTrait::Sedentary),
            ("TradeFocused".to_string(), SpeciesTrait::TradeFocused),
            ("WarLike".to_string(), SpeciesTrait::WarLike),
            ("Peaceful".to_string(), SpeciesTrait::Peaceful),
            ("Adaptable".to_string(), SpeciesTrait::Adaptable),
            ("Curious".to_string(), SpeciesTrait::Curious),
        ])
    }
    
    /// Load a single species template from a YAML file.
    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<SpeciesTemplate, SpeciesHistoryError> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        self.parse(&contents)
    }
    
    /// Parse species template from YAML string.
    pub fn parse(&self, yaml: &str) -> Result<SpeciesTemplate, SpeciesHistoryError> {
        let mut template: SpeciesTemplate = serde_yaml::from_str(yaml)?;
        
        // Validate
        self.validate_template(&template)?;
        
        // Set defaults for optional fields
        if template.behaviors.exploration == 0.0 
            && template.behaviors.diplomacy == 0.0 
            && template.behaviors.aggression == 0.0 {
            template.behaviors = SpeciesBehaviors::default_balanced();
        }
        if template.base_stats.growth_rate == 0.0 {
            template.base_stats.growth_rate = 1.0;
        }
        if template.base_stats.max_lifespan == 0 {
            template.base_stats.max_lifespan = 100;
        }
        
        Ok(template)
    }
    
    /// Load all species templates from a directory.
    /// 
    /// Returns a map of SpeciesId → SpeciesTemplate.
    pub fn load_all<P: AsRef<Path>>(&self, dir: P) -> Result<HashMap<SpeciesId, SpeciesTemplate>, SpeciesHistoryError> {
        let mut templates = HashMap::new();
        
        let path = dir.as_ref();
        if !path.is_dir() {
            return Err(SpeciesHistoryError::Io(
                std::io::Error::new(std::io::ErrorKind::NotFound, 
                    format!("Template directory not found: {:?}", path))
            ));
        }
        
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                match self.load(&path) {
                    Ok(template) => {
                        templates.insert(template.id, template);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load template {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(templates)
    }
    
    /// Validate a species template.
    fn validate_template(&self, template: &SpeciesTemplate) -> Result<(), SpeciesHistoryError> {
        if template.name.is_empty() {
            return Err(SpeciesHistoryError::MissingField("name"));
        }
        if template.plural.is_empty() {
            return Err(SpeciesHistoryError::MissingField("plural"));
        }
        if template.id == SpeciesId::Undefined {
            return Err(SpeciesHistoryError::InvalidId("UNDEFINED".to_string()));
        }
        
        // Validate at least one trait is present
        if template.base_traits.is_empty() {
            return Err(SpeciesHistoryError::Validation(
                "At least one base trait is required".to_string()
            ));
        }
        
        // Validate traits exist in trait map
        for trait_name in &template.base_traits.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>() {
            if !self.trait_map.contains_key(trait_name) {
                return Err(SpeciesHistoryError::Validation(
                    format!("Unknown trait: {}", trait_name)
                ));
            }
        }
        
        // Validate society thresholds are ascending (each must have higher min pop than previous)
        if template.society_types.len() > 1 {
            for window in template.society_types.windows(2) {
                let current = window[0].population_range();
                let next = window[1].population_range();
                if current.0 >= next.0 {
                    return Err(SpeciesHistoryError::Validation(
                        format!("Society thresholds must be ascending: {:?} min pop {} >= {:?} min pop {}",
                            window[0], current.0, window[1], next.0)
                    ));
                }
            }
        }
        
        // Validate behaviors
        template.behaviors.validate()?;
        
        // Validate stats
        template.base_stats.validate()?;
        
        Ok(())
    }
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Species History Collection
// =============================================================================

/// Collection of all species templates with history extensions.
#[derive(Debug, Clone, Default)]
pub struct SpeciesHistory {
    /// Species templates indexed by ID.
    pub templates: HashMap<SpeciesId, SpeciesTemplate>,
}

impl SpeciesHistory {
    /// Load all templates from a directory.
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self, SpeciesHistoryError> {
        let loader = TemplateLoader::new();
        let templates = loader.load_all(dir)?;
        Ok(Self { templates })
    }
    
    /// Get a template by ID.
    pub fn get(&self, id: SpeciesId) -> Option<&SpeciesTemplate> {
        self.templates.get(&id)
    }
    
    /// Get all template IDs.
    pub fn ids(&self) -> Vec<SpeciesId> {
        self.templates.keys().copied().collect()
    }
    
    /// Check if a behavior is dominant for a species.
    pub fn is_behavior_dominant(&self, id: SpeciesId, behavior: SpeciesBehavior) -> bool {
        self.get(id)
            .map(|t| behavior.get_value(t) > 0.7)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const HUMANS_YAML: &str = r#"
id: 1
name: Human
plural: Humans
display_name: Human
description: The most common sentient species, known for adaptability and curiosity.
base_traits:
  - Adaptable
  - Curious
  - Sedentary
  - TradeFocused
behaviors:
  exploration: 0.8
  diplomacy: 0.7
  aggression: 0.3
  trade: 0.8
society_types:
  - Tribe
  - Chiefdom
  - Nation
base_stats:
  growth_rate: 1.0
  max_lifespan: 100
  migration_speed: 0.6
  food_consumption: 1.0
  birth_rate: 25.0
  death_rate: 15.0
  disaster_resistance: 0.5
"#;
    
    #[test]
    fn test_parse_humans_template() {
        let loader = TemplateLoader::new();
        let template = loader.parse(HUMANS_YAML).unwrap();
        
        assert_eq!(template.id, SpeciesId::Human);
        assert_eq!(template.name, "Human");
        assert_eq!(template.plural, "Humans");
        assert!(template.base_traits.contains(&SpeciesTrait::Adaptable));
        assert!(template.base_traits.contains(&SpeciesTrait::Curious));
    }
    
    #[test]
    fn test_behaviors_validation() {
        let behaviors = SpeciesBehaviors::default_balanced();
        behaviors.validate().unwrap();
        
        let bad_behaviors = SpeciesBehaviors {
            exploration: 1.5,
            ..Default::default()
        };
        assert!(bad_behaviors.validate().is_err());
    }
    
    #[test]
    fn test_only_in_history_marker() {
        let template = SpeciesTemplate {
            id: SpeciesId::Human,
            name: "Human".to_string(),
            plural: "Humans".to_string(),
            display_name: "Human".to_string(),
            description: "Test".to_string(),
            base_traits: vec![],
            behaviors: SpeciesBehaviors::default_balanced(),
            society_types: vec![SpeciesSocietyType::Tribe],
            base_stats: SpeciesStats::default(),
        };
        
        assert_eq!(template.species_id(), Some(SpeciesId::Human));
    }
    
    #[test]
    fn test_society_evolution() {
        assert_eq!(
            SpeciesSocietyType::Tribe.evolve_to(),
            Some(SpeciesSocietyType::Chiefdom)
        );
        assert_eq!(
            SpeciesSocietyType::Nation.evolve_to(),
            None
        );
    }
}
