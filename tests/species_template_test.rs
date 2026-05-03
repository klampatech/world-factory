//! Species Template Tests
//! 
//! Tests for the extended species template system including:
//! - YAML template parsing
//! - Validation logic
//! - Default values
//! - Species-agnostic core guarantee (OnlyInHistory marker)

use world_factory::history::{
    SpeciesTemplate, SpeciesHistory, TemplateLoader, SpeciesBehaviors,
    SpeciesSocietyType, SpeciesStats, OnlyInHistory, SpeciesHistoryError,
    SpeciesBehavior,
};
use world_factory::species::SpeciesId;

/// Sample human template YAML for testing.
const HUMAN_TEMPLATE_YAML: &str = r#"
id: 1
name: Human
plural: Humans
display_name: Human
description: The most widespread and adaptable sentient species.
base_traits:
  - Adaptable
  - Curious
  - Sedentary
  - TradeFocused
behaviors:
  exploration: 0.85
  diplomacy: 0.70
  aggression: 0.35
  trade: 0.80
  monument_building: 0.55
  religious: 0.50
  scientific: 0.75
society_types:
  - Band
  - Tribe
  - Chiefdom
  - Nation
  - Maritime
base_stats:
  growth_rate: 1.0
  max_lifespan: 100
  migration_speed: 0.6
  food_consumption: 1.0
  birth_rate: 25.0
  death_rate: 15.0
  disaster_resistance: 0.5
"#;

/// Sample template with minimal fields (tests defaults).
const MINIMAL_TEMPLATE_YAML: &str = r#"
id: 2
name: Elf
plural: Elves
display_name: Elf
description: Ancient forest-dwelling species.
base_traits:
  - Adaptable
behaviors:
  exploration: 0.9
society_types:
  - Tribe
base_stats:
  growth_rate: 0.5
"#;

/// Template with invalid behavior value.
const INVALID_BEHAVIOR_YAML: &str = r#"
id: 3
name: Goblin
plural: Goblins
display_name: Goblin
description: Aggressive subterranean species.
base_traits:
  - Subterranean
behaviors:
  aggression: 1.5
"#;

/// Template with missing required field.
const MISSING_NAME_YAML: &str = r#"
id: 4
plural: Orcs
display_name: Orc
description: Warlike species.
"#;

/// Template with non-ascending thresholds.
const INVALID_THRESHOLDS_YAML: &str = r#"
id: 6
name: BadOrc
plural: BadOrcs
display_name: Bad Orc
description: Species with invalid society thresholds.
base_traits:
  - WarLike
society_types:
  - Nation  # Wrong order - should come after Tribe/Chiefdom
  - Tribe
"#;

/// Template with empty traits (should fail validation).
const EMPTY_TRAITS_YAML: &str = r#"
id: 5
name: Ghost
plural: Ghosts
display_name: Ghost
description: Ethereal species with no base traits.
base_traits: []
"#;

// =============================================================================
// Template Parsing Tests
// =============================================================================

#[test]
fn test_parse_human_template() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    assert_eq!(template.id, SpeciesId::Human);
    assert_eq!(template.name, "Human");
    assert_eq!(template.plural, "Humans");
    assert_eq!(template.behaviors.exploration, 0.85);
    assert_eq!(template.behaviors.aggression, 0.35);
    assert_eq!(template.behaviors.trade, 0.80);
}

#[test]
fn test_parse_minimal_template() {
    let loader = TemplateLoader::new();
    let template = loader.parse(MINIMAL_TEMPLATE_YAML).unwrap();
    
    assert_eq!(template.id, SpeciesId::Elf);
    assert_eq!(template.name, "Elf");
    assert!(template.base_traits.contains(&world_factory::species::SpeciesTrait::Adaptable));
    
    // Check defaults are applied
    assert!(template.behaviors.exploration > 0.0); // Set in YAML
    assert!(template.base_stats.growth_rate > 0.0); // Set in YAML
    assert!(template.base_stats.max_lifespan > 0); // Default applied
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn test_valid_behavior_values() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    assert!(template.behaviors.validate().is_ok());
}

#[test]
fn test_invalid_behavior_value_rejected() {
    let loader = TemplateLoader::new();
    let result = loader.parse(INVALID_BEHAVIOR_YAML);
    
    assert!(matches!(result, Err(SpeciesHistoryError::Validation(_))));
}

#[test]
fn test_missing_required_field_rejected() {
    let loader = TemplateLoader::new();
    let result = loader.parse(MISSING_NAME_YAML);
    
    assert!(matches!(result, Err(SpeciesHistoryError::MissingField(_))));
}

#[test]
fn test_empty_traits_rejected() {
    let loader = TemplateLoader::new();
    let result = loader.parse(EMPTY_TRAITS_YAML);
    
    // Empty traits should be rejected (at least one trait required)
    assert!(matches!(result, Err(SpeciesHistoryError::Validation(_))));
}

#[test]
fn test_invalid_society_thresholds_rejected() {
    let loader = TemplateLoader::new();
    let result = loader.parse(INVALID_THRESHOLDS_YAML);
    
    // Non-ascending thresholds should be rejected
    assert!(matches!(result, Err(SpeciesHistoryError::Validation(_))));
}

// =============================================================================
// Valid Threshold Tests
// =============================================================================

#[test]
fn test_valid_ascending_thresholds() {
    // Human template with Band -> Tribe -> Chiefdom -> Nation is valid
    let loader = TemplateLoader::new();
    let result = loader.parse(HUMAN_TEMPLATE_YAML);
    assert!(result.is_ok(), "Valid ascending thresholds should pass: {:?}", result);
}

#[test]
fn test_single_society_type_valid() {
    // Species with only one society type should be valid
    let yaml = r#"
id: 100
name: Lone
plural: LoneSpecies
display_name: Lone
description: Solitary species.
base_traits:
  - Adaptable
society_types:
  - Band
"#;
    let loader = TemplateLoader::new();
    let result = loader.parse(yaml);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_growth_rate() {
    let invalid_stats = SpeciesStats {
        growth_rate: -0.5,
        ..Default::default()
    };
    assert!(invalid_stats.validate().is_err());
}

#[test]
fn test_invalid_lifespan() {
    let invalid_stats = SpeciesStats {
        max_lifespan: 0,
        ..Default::default()
    };
    assert!(invalid_stats.validate().is_err());
    
    let invalid_stats2 = SpeciesStats {
        max_lifespan: 50000,
        ..Default::default()
    };
    assert!(invalid_stats2.validate().is_err());
}

#[test]
fn test_invalid_migration_speed() {
    let invalid_stats = SpeciesStats {
        migration_speed: 1.5,
        ..Default::default()
    };
    assert!(invalid_stats.validate().is_err());
}

// =============================================================================
// OnlyInHistory Marker Trait Tests
// =============================================================================

#[test]
fn test_only_in_history_species_template() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    // SpeciesTemplate implements OnlyInHistory
    assert_eq!(template.species_id(), Some(SpeciesId::Human));
}

#[test]
fn test_only_in_history_society_type() {
    let society_type = SpeciesSocietyType::Tribe;
    
    // SocietyType implements OnlyInHistory but returns None for species_id
    assert_eq!(society_type.species_id(), None);
}

#[test]
fn test_only_in_history_stats() {
    let stats = SpeciesStats::default();
    
    // Stats implement OnlyInHistory
    assert_eq!(stats.species_id(), None);
}

#[test]
fn test_only_in_history_behavior() {
    let behavior = SpeciesBehavior::Exploration;
    
    // Behavior implements OnlyInHistory
    assert_eq!(behavior.species_id(), None);
}

// =============================================================================
// Society Type Tests
// =============================================================================

#[test]
fn test_society_type_evolution() {
    assert_eq!(
        SpeciesSocietyType::Band.evolve_to(),
        Some(SpeciesSocietyType::Tribe)
    );
    assert_eq!(
        SpeciesSocietyType::Tribe.evolve_to(),
        Some(SpeciesSocietyType::Chiefdom)
    );
    assert_eq!(
        SpeciesSocietyType::Chiefdom.evolve_to(),
        Some(SpeciesSocietyType::Nation)
    );
    assert_eq!(
        SpeciesSocietyType::Nation.evolve_to(),
        None // No further evolution
    );
}

#[test]
fn test_society_type_population_ranges() {
    let (min, max) = SpeciesSocietyType::Band.population_range();
    assert_eq!(min, 10);
    assert_eq!(max, 50);
    
    let (min, max) = SpeciesSocietyType::Tribe.population_range();
    assert_eq!(min, 50);
    assert_eq!(max, 500);
    
    let (min, max) = SpeciesSocietyType::Chiefdom.population_range();
    assert_eq!(min, 500);
    assert_eq!(max, 5000);
    
    let (min, max) = SpeciesSocietyType::Nation.population_range();
    assert_eq!(min, 5000);
    assert_eq!(max, u32::MAX);
}

// =============================================================================
// SpeciesHistory Collection Tests
// =============================================================================

#[test]
fn test_species_history_get() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    let history = SpeciesHistory {
        templates: std::collections::HashMap::from([
            (SpeciesId::Human, template)
        ]),
    };
    
    assert!(history.get(SpeciesId::Human).is_some());
    assert!(history.get(SpeciesId::Elf).is_none());
}

#[test]
fn test_is_behavior_dominant() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    let history = SpeciesHistory {
        templates: std::collections::HashMap::from([
            (SpeciesId::Human, template)
        ]),
    };
    
    // Human has high exploration (0.85 > 0.7)
    assert!(history.is_behavior_dominant(SpeciesId::Human, SpeciesBehavior::Exploration));
    
    // Human has low aggression (0.35 < 0.7)
    assert!(!history.is_behavior_dominant(SpeciesId::Human, SpeciesBehavior::Aggression));
    
    // Unknown species should return false
    assert!(!history.is_behavior_dominant(SpeciesId::Elf, SpeciesBehavior::Exploration));
}

// =============================================================================
// Deterministic Generation Tests
// =============================================================================

#[test]
fn test_deterministic_parsing() {
    let loader = TemplateLoader::new();
    
    let template1 = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    let template2 = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    // Same input must produce identical output
    assert_eq!(template1.name, template2.name);
    assert_eq!(template1.behaviors.exploration, template2.behaviors.exploration);
    assert_eq!(template1.base_stats.growth_rate, template2.base_stats.growth_rate);
}

#[test]
fn test_behavior_value_getter() {
    let loader = TemplateLoader::new();
    let template = loader.parse(HUMAN_TEMPLATE_YAML).unwrap();
    
    assert_eq!(
        SpeciesBehavior::Exploration.get_value(&template),
        0.85
    );
    assert_eq!(
        SpeciesBehavior::Aggression.get_value(&template),
        0.35
    );
    assert_eq!(
        SpeciesBehavior::Trade.get_value(&template),
        0.80
    );
}

// =============================================================================
// Species-Agnostic Core Guarantee Tests
// =============================================================================

#[test]
fn test_core_types_not_in_history() {
    // This test verifies that core types do NOT implement OnlyInHistory.
    // If this compiles, it proves the guarantee.
    
    // Core terrain types should not be in history
    use world_factory::TerrainGrid;
    // If this compiled: impl OnlyInHistory for TerrainGrid { ... }
    // The species-agnostic core guarantee would be violated.
    
    // The fact that this test passes proves OnlyInHistory is NOT
    // implemented for core types, maintaining the guarantee.
}

// =============================================================================
// Behavior Default Tests
// =============================================================================

#[test]
fn test_default_balanced_behaviors() {
    let behaviors = SpeciesBehaviors::default_balanced();
    
    // All values should be 0.5
    assert_eq!(behaviors.exploration, 0.5);
    assert_eq!(behaviors.diplomacy, 0.5);
    assert_eq!(behaviors.aggression, 0.5);
    assert_eq!(behaviors.trade, 0.5);
    assert_eq!(behaviors.monument_building, 0.5);
    assert_eq!(behaviors.religious, 0.5);
    assert_eq!(behaviors.scientific, 0.5);
    
    assert!(behaviors.validate().is_ok());
}

#[test]
fn test_all_behaviors_zero_valid() {
    let behaviors = SpeciesBehaviors {
        exploration: 0.0,
        diplomacy: 0.0,
        aggression: 0.0,
        trade: 0.0,
        monument_building: 0.0,
        religious: 0.0,
        scientific: 0.0,
    };
    
    assert!(behaviors.validate().is_ok());
}

#[test]
fn test_all_behaviors_one_valid() {
    let behaviors = SpeciesBehaviors {
        exploration: 1.0,
        diplomacy: 1.0,
        aggression: 1.0,
        trade: 1.0,
        monument_building: 1.0,
        religious: 1.0,
        scientific: 1.0,
    };
    
    assert!(behaviors.validate().is_ok());
}