//! Artifact System for World Factory
//!
//! Provides historical artifact and cultural heritage management.
//! Artifacts are significant objects created during world history that
//! hold cultural, magical, or historical significance.
//!
//! Artifacts are linked to events, figures, and civilizations that created them.
//!
//! ## Artifact Types
//!
//! - **Relic**: Religious or sacred objects of cultural significance
//! - **Weapon**: Legendary weapons with historical importance
//! - **Artifact**: Magical or technological objects of power
//! - **Monument**: Buildings, statues, and structures commemorating history
//! - **Document**: Historical texts, maps, and records
//! - **Trophy**: Battle prizes and conquest spoils
//! - **CrownJewel**: Royal treasures and regalia
//! - **Sacred**: Objects of spiritual or magical significance

use crate::events::{Event, EventType};
use crate::types::{EntityId, EntityType, Timestamp};
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

// ============================================================================
// Artifact Rarity & Cataclysm Triggering
// ============================================================================

/// Rarity levels for artifacts, affecting their value and cataclysmic potential.
/// Higher rarity artifacts have stronger effects but also higher chances of
/// triggering cataclysmic events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRarity {
    /// Common artifacts - everyday items of historical interest
    Common = 0,
    /// Uncommon artifacts - significant items with some historical value
    Uncommon = 1,
    /// Rare artifacts - powerful items tied to important historical moments
    Rare = 2,
    /// Legendary artifacts - world-changing items tied to major events
    Legendary = 3,
    /// Mythic artifacts - artifacts of legend that can reshape reality
    Mythic = 4,
}

impl ArtifactRarity {
    /// Base cataclysm probability per year (0.0 to 1.0)
    /// Per requirement: < 0.1% per year per artifact
    pub fn cataclysm_probability(&self) -> f64 {
        match self {
            Self::Common => 0.00001,   // 0.001% - essentially zero
            Self::Uncommon => 0.0001,  // 0.01% - very rare
            Self::Rare => 0.0005,      // 0.05% - rare
            Self::Legendary => 0.0008, // 0.08% - approaching limit
            Self::Mythic => 0.001,     // 0.1% - maximum per artifact
        }
    }

    /// Get the maximum cap for combined cataclysm probability
    /// When multiple high-rarity artifacts exist, cap the total probability
    pub fn cataclysm_cap() -> f64 {
        0.05 // 5% maximum per year for all artifacts combined
    }

    /// Calculate rarity from significance value (0.0 to 1.0)
    pub fn from_significance(significance: f32) -> Self {
        if significance >= 0.95 {
            Self::Mythic
        } else if significance >= 0.85 {
            Self::Legendary
        } else if significance >= 0.7 {
            Self::Rare
        } else if significance >= 0.5 {
            Self::Uncommon
        } else {
            Self::Common
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Legendary => "Legendary",
            Self::Mythic => "Mythic",
        }
    }
}

/// System for calculating cataclysmic event probability from artifacts.
/// Implements the requirement: "< 0.1% per year per artifact, capped"
pub struct CataclysmTriggerSystem {
    /// Probability modifier based on artifact concentration
    concentration_modifier: f64,
}

impl CataclysmTriggerSystem {
    /// Create a new trigger system
    pub fn new() -> Self {
        Self {
            concentration_modifier: 1.0,
        }
    }

    /// Returns the maximum cataclysm probability cap (5%).
    pub fn cataclysm_cap() -> f64 {
        0.05
    }

    /// Calculate the annual probability of a cataclysm triggered by artifacts.
    /// Returns (probability, triggered_rarity) if triggered, None otherwise.
    pub fn calculate_annual_probability(
        &self,
        artifacts: &[&Artifact],
        rng: &mut Rng,
    ) -> Option<(f64, ArtifactRarity, Uuid)> {
        if artifacts.is_empty() {
            return None;
        }

        // Calculate total raw probability
        let mut total_probability = 0.0;
        let mut highest_rarity = ArtifactRarity::Common;
        let mut triggering_artifact_id = None;

        for artifact in artifacts {
            let rarity = ArtifactRarity::from_significance(artifact.significance);
            let prob = rarity.cataclysm_probability();
            total_probability += prob;

            if rarity > highest_rarity {
                highest_rarity = rarity;
            }

            // Track first artifact that would trigger (for reporting)
            if triggering_artifact_id.is_none() && prob > 0.0 {
                triggering_artifact_id = Some(artifact.id.to_uuid());
            }
        }

        // Apply concentration modifier if many artifacts exist
        let artifact_count_modifier = if artifacts.len() > 10 {
            (artifacts.len() as f64).sqrt() / 10.0
        } else {
            1.0
        };

        total_probability *= artifact_count_modifier;

        // Apply the cap
        let capped_probability = total_probability.min(0.001); // 0.1% cap

        // Roll for cataclysm
        let roll = rng.next_f64();
        if roll < capped_probability {
            Some((
                capped_probability,
                highest_rarity,
                triggering_artifact_id.unwrap_or(Uuid::nil()),
            ))
        } else {
            None
        }
    }

    /// Get artifacts that could trigger cataclysms (rare or higher)
    pub fn get_triggering_artifacts<'a>(&self, artifacts: &'a [Artifact]) -> Vec<&'a Artifact> {
        artifacts
            .iter()
            .filter(|a| ArtifactRarity::from_significance(a.significance) >= ArtifactRarity::Rare)
            .collect()
    }
}

impl Default for CataclysmTriggerSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Determines the type of cataclysm triggered by an artifact based on its nature.
impl Artifact {
    /// Get the name of the cataclysm type this artifact might trigger.
    /// Returns a string identifier that can be resolved to CataclysmType when needed.
    pub fn potential_cataclysm_type_name(&self) -> &'static str {
        match self.category {
            ArtifactCategory::Sacred => "divine_wrath",
            ArtifactCategory::Magical => "magical_cataclysm",
            ArtifactCategory::Relic => "divine_wrath",
            ArtifactCategory::Weapon => "civilizational_collapse",
            ArtifactCategory::CrownJewel => "civilizational_collapse",
            ArtifactCategory::Monument => "blight",
            ArtifactCategory::Document => "cultural_loss",
            ArtifactCategory::Trophy => "great_migration",
        }
    }
}

// ============================================================================
// Artifact Creation Conditions
// ============================================================================

/// Conditions required for artifact creation.
/// Artifacts don't simply appear - they require specific circumstances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCreationCondition {
    /// Minimum significance threshold for this artifact to be created
    pub min_significance: f32,
    /// Required condition type
    pub condition_type: ArtifactCreationConditionType,
    /// Optional required figure type (if associated with a notable figure)
    pub required_figure_type: Option<String>,
    /// Optional required rarity level
    pub min_rarity: Option<ArtifactRarity>,
}

/// Types of conditions that can lead to artifact creation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCreationConditionType {
    /// Created during a significant event (battle, coronation, etc.)
    SignificantEvent,
    /// Created by a notable figure (monarch, hero, etc.)
    NotableFigure,
    /// Created using rare resources
    RareResources,
    /// Created in a sacred/important location
    SacredLocation,
    /// Created as a memorial/monument
    Memorial,
    /// Won as a prize in competition
    Competition,
    /// Cursed artifact created through dark means
    DarkRitual,
    /// Inherited through bloodline
    Bloodline,
}

impl ArtifactCreationCondition {
    /// Check if a condition is satisfied
    pub fn is_satisfied(&self, context: &ArtifactCreationContext) -> bool {
        // Check significance threshold
        if context.significance < self.min_significance {
            return false;
        }

        match self.condition_type {
            ArtifactCreationConditionType::SignificantEvent => context.related_event.is_some(),
            ArtifactCreationConditionType::NotableFigure => {
                context.creator_figure_id.is_some()
                    && self
                        .required_figure_type
                        .as_ref()
                        .map_or(true, |t| context.creator_figure_type.as_ref() == Some(t))
            }
            ArtifactCreationConditionType::RareResources => context.uses_rare_resources,
            ArtifactCreationConditionType::SacredLocation => {
                context.location_sacred || context.location_id.is_some()
            }
            ArtifactCreationConditionType::Memorial => {
                context.is_memorial || context.related_event.is_some()
            }
            ArtifactCreationConditionType::Competition => context.was_competition_winner,
            ArtifactCreationConditionType::DarkRitual => {
                context.used_dark_ritual || context.has_cursed_property
            }
            ArtifactCreationConditionType::Bloodline => context.in_bloodline,
        }
    }

    /// Get default conditions for each artifact category
    pub fn default_for_category(category: ArtifactCategory) -> Vec<Self> {
        let mut conditions = Vec::new();

        conditions.push(ArtifactCreationCondition {
            min_significance: 0.5,
            condition_type: ArtifactCreationConditionType::SignificantEvent,
            required_figure_type: None,
            min_rarity: None,
        });

        match category {
            ArtifactCategory::CrownJewel => {
                conditions.push(ArtifactCreationCondition {
                    min_significance: 0.7,
                    condition_type: ArtifactCreationConditionType::NotableFigure,
                    required_figure_type: Some("Monarch".to_string()),
                    min_rarity: Some(ArtifactRarity::Rare),
                });
            }
            ArtifactCategory::Weapon => {
                conditions.push(ArtifactCreationCondition {
                    min_significance: 0.6,
                    condition_type: ArtifactCreationConditionType::NotableFigure,
                    required_figure_type: Some("MilitaryLeader".to_string()),
                    min_rarity: Some(ArtifactRarity::Uncommon),
                });
            }
            ArtifactCategory::Magical => {
                conditions.push(ArtifactCreationCondition {
                    min_significance: 0.7,
                    condition_type: ArtifactCreationConditionType::RareResources,
                    required_figure_type: None,
                    min_rarity: Some(ArtifactRarity::Rare),
                });
            }
            ArtifactCategory::Sacred | ArtifactCategory::Relic => {
                conditions.push(ArtifactCreationCondition {
                    min_significance: 0.6,
                    condition_type: ArtifactCreationConditionType::SacredLocation,
                    required_figure_type: None,
                    min_rarity: Some(ArtifactRarity::Uncommon),
                });
            }
            ArtifactCategory::Monument => {
                conditions.push(ArtifactCreationCondition {
                    min_significance: 0.7,
                    condition_type: ArtifactCreationConditionType::Memorial,
                    required_figure_type: None,
                    min_rarity: Some(ArtifactRarity::Rare),
                });
            }
            _ => {}
        }

        conditions
    }
}

// ============================================================================
// Causal Chain Validation (SPEC.md §D.3)
// ============================================================================

/// Validation result for artifact spawning, including reasons for failure.
#[derive(Debug, Clone, Default)]
pub struct CausalChainValidation {
    /// Whether the artifact can spawn
    pub can_spawn: bool,
    /// List of missing conditions (for debugging/logging)
    pub missing_conditions: Vec<String>,
}

impl CausalChainValidation {
    /// Create a passing validation
    pub fn success() -> Self {
        Self {
            can_spawn: true,
            missing_conditions: Vec::new(),
        }
    }

    /// Create a failing validation with reasons
    pub fn failure(reasons: Vec<String>) -> Self {
        Self {
            can_spawn: false,
            missing_conditions: reasons,
        }
    }

    /// Add a missing condition reason
    pub fn add_reason(&mut self, reason: impl Into<String>) {
        self.missing_conditions.push(reason.into());
        self.can_spawn = false;
    }

    /// Get formatted reason string for logging
    pub fn reasons_summary(&self) -> String {
        if self.missing_conditions.is_empty() {
            "All conditions met".to_string()
        } else {
            format!("Missing: {}", self.missing_conditions.join(", "))
        }
    }
}

/// Validates artifact spawning based on causal chain requirements per SPEC.md §D.3.
/// Artifacts cannot spawn without proper causal chain conditions being met.
pub struct CausalChainValidator;


impl CausalChainValidator {
    /// Validate whether an artifact can spawn based on its category and context.
    ///
    /// Per SPEC.md §D.3:
    /// - Legendary weapon: Iron/gold deposit + notable warrior figure
    /// - Ancient tome: Civilized biome + scholar figure
    /// - Sacred relic: Religious site + religious figure
    /// - Magical artifact: Gem deposit + historical event + magical tradition
    /// - Crown/regalia: Gold deposit + centralized government
    /// - Map to treasure: Rare resource + secrecy event
    /// - Ancient artifact: Pre-history civilization + survived ruin
    /// - Remnant artifact: Primal beast slain — dropped on death
    pub fn can_spawn(category: ArtifactCategory, context: &ArtifactCreationContext) -> CausalChainValidation {
        let mut validation = CausalChainValidation::default();


        match category {
            ArtifactCategory::Weapon => {
                // Legendary weapon: Iron/gold deposit + notable warrior figure
                if !context.iron_or_gold_deposit_nearby && !context.uses_rare_resources {
                    validation.add_reason("Iron or gold deposit nearby");
                }
                if !context.warrior_figure_exists {
                    validation.add_reason("Notable warrior figure");
                }
                if !context.capital_city_nearby && !context.related_event.is_some() {
                    validation.add_reason("Capital city or related event (battlefield)");
                }
            }
            ArtifactCategory::Document => {
                // Ancient tome: Civilized biome + scholar figure
                if !context.civilized_biome {
                    validation.add_reason("Civilized biome (not wilderness)");
                }
                if !context.scholar_figure_exists {
                    validation.add_reason("Scholar figure");
                }
            }
            ArtifactCategory::Sacred | ArtifactCategory::Relic => {
                // Sacred relic: Religious site + religious figure
                if !context.religious_site_nearby && !context.location_sacred {
                    validation.add_reason("Religious site or sacred location");
                }
                if !context.religious_figure_exists {
                    validation.add_reason("Religious figure");
                }
            }
            ArtifactCategory::Magical => {
                // Magical artifact: Gem deposit + historical event + magical tradition
                if !context.gem_deposit_nearby && !context.uses_rare_resources {
                    validation.add_reason("Gem deposit or rare resources");
                }
                if !context.historical_event && context.related_event.is_none() {
                    validation.add_reason("Historical event");
                }
                if !context.magical_tradition_exists {
                    validation.add_reason("Magical tradition exists");
                }
            }
            ArtifactCategory::CrownJewel => {
                // Crown/regalia: Gold deposit + centralized government
                if !context.gold_deposit_nearby && !context.uses_rare_resources {
                    validation.add_reason("Gold deposit or rare resources");
                }
                if !context.has_centralized_government {
                    validation.add_reason("Centralized government");
                }
                if !context.capital_city_nearby {
                    validation.add_reason("Capital city nearby");
                }
            }
            ArtifactCategory::Trophy => {
                // Map to treasure: Rare resource + secrecy event
                // Or battlefield trophy (less strict)
                if context.secrecy_event {
                    if !context.uses_rare_resources {
                        validation.add_reason("Rare resource for treasure map");
                    }
                }
                // Regular trophies can spawn from battle events
                if context.related_event.is_none() {
                    validation.add_reason("Related event (battle/conquest)");
                }
            }
            ArtifactCategory::Monument => {
                // Ancient artifact: Pre-history civilization + survived ruin
                if context.pre_history_civilization {
                    // Very strict requirements for pre-history artifacts
                    if !context.civilized_biome {
                        validation.add_reason("Civilized biome for ruins");
                    }
                }
                // Regular monuments need a memorial event or significant event
                if !context.is_memorial && context.related_event.is_none() {
                    validation.add_reason("Memorial or significant event");
                }
            }
        }

        // Apply significance threshold (must meet minimum for category)
        let min_significance = match category {
            ArtifactCategory::CrownJewel => 0.7,
            ArtifactCategory::Magical => 0.7,
            ArtifactCategory::Monument => 0.7,
            ArtifactCategory::Sacred | ArtifactCategory::Relic => 0.6,
            ArtifactCategory::Weapon => 0.6,
            _ => 0.5,
        };


        if context.significance < min_significance {
            validation.add_reason(format!("Minimum significance ({})", min_significance));
        }

        validation
    }

    /// Log artifact spawning decision for debugging.
    pub fn log_spawn_decision(
        category: ArtifactCategory,
        context: &ArtifactCreationContext,
        validation: &CausalChainValidation,
        artifact_name: &str,
    ) {
        if validation.can_spawn {
            debug!(
                artifact = artifact_name,
                category = ?category,
                significance = context.significance,
                "Artifact spawned with valid causal chain"
            );
        } else {
            debug!(
                artifact = artifact_name,
                category = ?category,
                significance = context.significance,
                reasons = validation.reasons_summary(),
                "Artifact skipped: missing causal chain conditions"
            );
        }
    }
}

// ============================================================================
// Artifact Creation Context (Extended for Causal Chains per SPEC.md §D.3)
// ============================================================================

/// Context required to evaluate artifact creation conditions.
/// Extended with causal chain fields per SPEC.md §D.3 requirements.
#[derive(Debug, Clone, Default)]
pub struct ArtifactCreationContext {
    // =========================================================================
    // Basic Context Fields
    // =========================================================================
    /// Significance of the potential artifact (0.0 to 1.0)
    pub significance: f32,
    /// Related event that might have created this artifact
    pub related_event: Option<Uuid>,
    /// Creator figure ID
    pub creator_figure_id: Option<Uuid>,
    /// Creator figure type name
    pub creator_figure_type: Option<String>,
    /// Whether rare resources were used
    pub uses_rare_resources: bool,
    /// Whether the location is sacred
    pub location_sacred: bool,
    /// Location ID
    pub location_id: Option<Uuid>,
    /// Whether this is a memorial artifact
    pub is_memorial: bool,
    /// Whether created through competition
    pub was_competition_winner: bool,
    /// Whether created through dark ritual
    pub used_dark_ritual: bool,
    /// Whether artifact has cursed properties
    pub has_cursed_property: bool,
    /// Whether in a noble bloodline
    pub in_bloodline: bool,

    // =========================================================================
    // Causal Chain Fields (SPEC.md §D.3)
    // =========================================================================
    /// Whether an iron or gold deposit exists nearby (for weapons, crowns)
    #[serde(default)]
    pub iron_or_gold_deposit_nearby: bool,
    /// Whether a gold deposit specifically exists nearby (for crowns/regalia)
    #[serde(default)]
    pub gold_deposit_nearby: bool,
    /// Whether an iron deposit exists nearby (for weapons)
    #[serde(default)]
    pub iron_deposit_nearby: bool,
    /// Whether a gem deposit exists nearby (for magical artifacts)
    #[serde(default)]
    pub gem_deposit_nearby: bool,
    /// Whether the location is in a civilized biome (not wilderness)
    #[serde(default)]
    pub civilized_biome: bool,
    /// Whether a capital city exists nearby
    #[serde(default)]
    pub capital_city_nearby: bool,
    /// Whether a religious site exists nearby
    #[serde(default)]
    pub religious_site_nearby: bool,
    /// Whether the world/nation has a centralized government
    #[serde(default)]
    pub has_centralized_government: bool,
    /// Whether a magical tradition exists in this world
    #[serde(default)]
    pub magical_tradition_exists: bool,
    /// Whether a scholar figure exists
    #[serde(default)]
    pub scholar_figure_exists: bool,
    /// Whether a warrior figure exists
    #[serde(default)]
    pub warrior_figure_exists: bool,
    /// Whether a religious figure exists
    #[serde(default)]
    pub religious_figure_exists: bool,
    /// Whether a primal beast was slain nearby (for remnant artifacts)
    #[serde(default)]
    pub primal_beast_slain_nearby: bool,
    /// Whether this is from a pre-history civilization
    #[serde(default)]
    pub pre_history_civilization: bool,
    /// Whether there's a secrecy event associated
    #[serde(default)]
    pub secrecy_event: bool,
    /// Whether there's a historical event associated
    #[serde(default)]
    pub historical_event: bool,
}

impl ArtifactCreationContext {
    /// Create a context from basic fields for backward compatibility
    pub fn from_basic(
        significance: f32,
        related_event: Option<Uuid>,
        creator_figure_id: Option<Uuid>,
        creator_figure_type: Option<String>,
        uses_rare_resources: bool,
    ) -> Self {
        Self {
            significance,
            related_event,
            creator_figure_id,
            creator_figure_type,
            uses_rare_resources,
            location_sacred: false,
            location_id: None,
            is_memorial: false,
            was_competition_winner: false,
            used_dark_ritual: false,
            has_cursed_property: false,
            in_bloodline: false,
            iron_or_gold_deposit_nearby: false,
            gold_deposit_nearby: false,
            iron_deposit_nearby: false,
            gem_deposit_nearby: false,
            civilized_biome: false,
            capital_city_nearby: false,
            religious_site_nearby: false,
            has_centralized_government: false,
            magical_tradition_exists: false,
            scholar_figure_exists: false,
            warrior_figure_exists: false,
            religious_figure_exists: false,
            primal_beast_slain_nearby: false,
            pre_history_civilization: false,
            secrecy_event: false,
            historical_event: related_event.is_some(),
        }
    }
}

// ============================================================================
// Artifact Effects
// ============================================================================

/// Effect that an artifact has on the world or its bearer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEffect {
    /// Effect identifier
    pub id: Uuid,
    /// Effect name
    pub name: String,
    /// Effect description
    pub description: String,
    /// Type of effect
    pub effect_type: ArtifactEffectType,
    /// Effect magnitude (0.0 to 1.0)
    pub magnitude: f32,
    /// Whether this is a passive or triggered effect
    pub is_passive: bool,
    /// Scope of the effect (who/what it affects)
    pub scope: EffectScope,
}

impl ArtifactEffect {
    /// Create a new effect
    pub fn new(
        name: String,
        description: String,
        effect_type: ArtifactEffectType,
        magnitude: f32,
        is_passive: bool,
        scope: EffectScope,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            effect_type,
            magnitude: magnitude.clamp(0.0, 1.0),
            is_passive,
            scope,
        }
    }

    /// Get effects for a specific category
    pub fn default_for_category(category: ArtifactCategory, rarity: ArtifactRarity) -> Vec<Self> {
        let magnitude = match rarity {
            ArtifactRarity::Common => 0.2,
            ArtifactRarity::Uncommon => 0.4,
            ArtifactRarity::Rare => 0.6,
            ArtifactRarity::Legendary => 0.8,
            ArtifactRarity::Mythic => 1.0,
        };

        match category {
            ArtifactCategory::Weapon => vec![
                Self::new(
                    "Combat Bonus".to_string(),
                    "Grants combat advantages in battle".to_string(),
                    ArtifactEffectType::MilitaryPower,
                    magnitude,
                    true,
                    EffectScope::Bearer,
                ),
                Self::new(
                    "Fear".to_string(),
                    "Strikes fear into enemies".to_string(),
                    ArtifactEffectType::Morale,
                    magnitude * 0.5,
                    true,
                    EffectScope::Enemies,
                ),
            ],
            ArtifactCategory::CrownJewel => vec![
                Self::new(
                    "Legitimacy".to_string(),
                    "Grants political legitimacy to rulers".to_string(),
                    ArtifactEffectType::PoliticalPower,
                    magnitude,
                    true,
                    EffectScope::Bearer,
                ),
                Self::new(
                    "Nation Prosperity".to_string(),
                    "Brings prosperity to the realm".to_string(),
                    ArtifactEffectType::Economic,
                    magnitude * 0.5,
                    true,
                    EffectScope::Nation,
                ),
            ],
            ArtifactCategory::Magical => vec![Self::new(
                "Mana Amplification".to_string(),
                "Amplifies magical abilities".to_string(),
                ArtifactEffectType::MagicalPower,
                magnitude,
                true,
                EffectScope::Bearer,
            )],
            ArtifactCategory::Sacred | ArtifactCategory::Relic => vec![
                Self::new(
                    "Divine Favor".to_string(),
                    "Grants divine blessing".to_string(),
                    ArtifactEffectType::Religious,
                    magnitude,
                    true,
                    EffectScope::Bearer,
                ),
                Self::new(
                    "Healing".to_string(),
                    "Can heal wounds and cure ailments".to_string(),
                    ArtifactEffectType::Healing,
                    magnitude * 0.7,
                    false,
                    EffectScope::Area,
                ),
            ],
            ArtifactCategory::Monument => vec![Self::new(
                "Cultural Unity".to_string(),
                "Unites the culture".to_string(),
                ArtifactEffectType::CulturalStability,
                magnitude,
                true,
                EffectScope::Culture,
            )],
            ArtifactCategory::Document => vec![Self::new(
                "Knowledge".to_string(),
                "Contains valuable knowledge".to_string(),
                ArtifactEffectType::KnowledgeBonus,
                magnitude,
                true,
                EffectScope::Culture,
            )],
            ArtifactCategory::Trophy => vec![Self::new(
                "Victory Symbol".to_string(),
                "Symbol of past victories".to_string(),
                ArtifactEffectType::Morale,
                magnitude * 0.5,
                true,
                EffectScope::Nation,
            )],
        }
    }
}

/// Types of effects artifacts can have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactEffectType {
    /// Military combat bonuses
    MilitaryPower,
    /// Political influence
    PoliticalPower,
    /// Magical ability enhancement
    MagicalPower,
    /// Economic benefits
    Economic,
    /// Religious/spiritual power
    Religious,
    /// Healing properties
    Healing,
    /// Morale effects
    Morale,
    /// Cultural stability
    CulturalStability,
    /// Knowledge/information
    KnowledgeBonus,
    /// Cursed effect (negative)
    Cursed,
    /// Population growth modifier
    PopulationGrowth,
    /// Defense bonus
    Defense,
    /// Speed/movement bonus
    Speed,
    /// Curse that brings doom
    Doom,
}

impl ArtifactEffectType {
    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::MilitaryPower => "Military Power",
            Self::PoliticalPower => "Political Power",
            Self::MagicalPower => "Magical Power",
            Self::Economic => "Economic",
            Self::Religious => "Religious",
            Self::Healing => "Healing",
            Self::Morale => "Morale",
            Self::CulturalStability => "Cultural Stability",
            Self::KnowledgeBonus => "Knowledge",
            Self::Cursed => "Cursed",
            Self::PopulationGrowth => "Population Growth",
            Self::Defense => "Defense",
            Self::Speed => "Speed",
            Self::Doom => "Doom",
        }
    }

    /// Whether this is a positive or negative effect
    pub fn is_positive(&self) -> bool {
        !matches!(self, Self::Cursed | Self::Doom)
    }
}

/// Who/what the effect applies to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectScope {
    /// Only affects the artifact's bearer/owner
    Bearer,
    /// Affects the bearer's nation/kingdom
    Nation,
    /// Affects the bearer's culture
    Culture,
    /// Affects enemies
    Enemies,
    /// Affects an area around the artifact
    Area,
    /// Affects everyone
    Global,
}

/// Categories of artifacts based on their nature and use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCategory {
    /// Religious or ceremonial objects
    Relic,
    /// Legendary weapons and armor
    Weapon,
    /// Magical or technological objects
    Magical,
    /// Buildings and structures
    Monument,
    /// Written records and documents
    Document,
    /// Prizes and spoils of conquest
    Trophy,
    /// Royal treasures and regalia
    CrownJewel,
    /// Objects of spiritual significance
    Sacred,
}

impl ArtifactCategory {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Relic => "Relic",
            Self::Weapon => "Weapon",
            Self::Magical => "Magical Artifact",
            Self::Monument => "Monument",
            Self::Document => "Document",
            Self::Trophy => "Trophy",
            Self::CrownJewel => "Crown Jewel",
            Self::Sacred => "Sacred Object",
        }
    }

    /// Determine artifact category from creation event type
    pub fn from_event(event_type: EventType) -> Option<Self> {
        match event_type {
            EventType::CulturalAchievement => Some(Self::Magical),
            EventType::MonumentCompleted => Some(Self::Monument),
            EventType::ReligiousEvent => Some(Self::Sacred),
            EventType::Battle | EventType::Victory => Some(Self::Trophy),
            EventType::Conquest => Some(Self::CrownJewel),
            EventType::ReligiousReformation => Some(Self::Relic),
            EventType::Invention => Some(Self::Magical),
            EventType::Discovery => Some(Self::Document),
            _ => None,
        }
    }
}

/// Condition or state of an artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCondition {
    /// Pristine, never used
    Pristine,
    /// Normal wear and tear
    #[default]
    Worn,
    /// Damaged but functional
    Damaged,
    /// Partially destroyed
    Ruined,
    /// Only fragments remain
    Fragment,
    /// Lost to history
    Lost,
    /// Hidden or secret
    Hidden,
}

/// Historical artifact with cultural significance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier
    pub id: EntityId,

    /// World this artifact belongs to
    pub world_id: Uuid,

    /// Name of the artifact
    pub name: String,

    /// Category/type of artifact
    pub category: ArtifactCategory,

    /// Era when it was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era: Option<String>,

    /// Year when created
    pub created_year: i32,

    /// Creator figure ID (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<Uuid>,

    /// Culture that created it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,

    /// Current location (settlement, region, or entity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_location_id: Option<Uuid>,

    /// Current owner entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,

    /// Description and history
    pub description: String,

    /// Significance (0.0 to 1.0)
    pub significance: f32,

    /// Rarity level (derived from significance, but stored for quick access)
    pub rarity: ArtifactRarity,

    /// Condition/state
    #[serde(default)]
    pub condition: ArtifactCondition,

    /// Origin event ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_event_id: Option<Uuid>,

    /// Related figures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_figures: Option<Vec<Uuid>>,

    /// Related events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_events: Option<Vec<Uuid>>,

    /// Special properties (magical, historical, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<ArtifactProperty>>,

    /// Timestamp
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(
        world_id: Uuid,
        name: String,
        category: ArtifactCategory,
        created_year: i32,
        description: String,
        significance: f32,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Event), // Uses Event type ID space
            world_id,
            name,
            category,
            era: None,
            created_year,
            creator_id: None,
            culture: None,
            current_location_id: None,
            owner_id: None,
            description,
            significance: significance.clamp(0.0, 1.0),
            rarity: ArtifactRarity::from_significance(significance),
            condition: ArtifactCondition::default(),
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create from a creation event
    pub fn from_event(event: &Event, category: ArtifactCategory) -> Self {
        let mut artifact = Self::new(
            event.world_id,
            format!("The {} of {}", category.name(), event.name),
            category,
            event.time.get_year(),
            format!("Created during: {}", event.description),
            event.significance.unwrap_or(0.5),
        );
        artifact.origin_event_id = Some(event.id.to_uuid());
        if let Some(participants) = &event.participants {
            if !participants.is_empty() {
                artifact.creator_id = Some(participants[0]);
            }
        }
        if let Some(location) = event.location_id {
            artifact.current_location_id = Some(location);
        }
        artifact
    }

    /// Add a related figure
    pub fn add_figure(&mut self, figure_id: Uuid) {
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

    /// Add a property
    pub fn add_property(&mut self, property: ArtifactProperty) {
        match &mut self.properties {
            Some(props) => props.push(property),
            None => self.properties = Some(vec![property]),
        }
        self.updated_at = Timestamp::now();
    }
}

/// Special property of an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactProperty {
    /// Property name
    pub name: String,
    /// Property description
    pub description: String,
    /// Property type
    pub property_type: ArtifactPropertyType,
}

/// Types of special properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactPropertyType {
    /// Magical enchantment
    Magical,
    /// Historical significance
    Historical,
    /// Cultural value
    Cultural,
    /// Economic value
    Economic,
    /// Religious significance
    Religious,
    /// Political power
    Political,
    /// Military power
    Military,
    /// Healing properties
    Healing,
    /// Cursed
    Cursed,
    /// Blessing
    Blessed,
}

// ============================================================================
// Artifact Store
// ============================================================================

/// In-memory storage for artifacts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactStore {
    /// All artifacts
    artifacts: Vec<Artifact>,
}

impl ArtifactStore {
    /// Create a new empty artifact store
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }

    /// Add an artifact
    pub fn add(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Number of artifacts
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// Get all artifacts
    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }

    /// Get artifact by ID
    pub fn get(&self, id: &Uuid) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id.to_uuid() == *id)
    }

    /// Get artifact by ID (mutable)
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Artifact> {
        self.artifacts.iter_mut().find(|a| a.id.to_uuid() == *id)
    }

    /// Get artifacts by category
    pub fn by_category(&self, category: ArtifactCategory) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.category == category)
            .collect()
    }

    /// Get artifacts by era
    pub fn by_era(&self, era: &str) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.era.as_ref().map(|e| e == era).unwrap_or(false))
            .collect()
    }

    /// Get artifacts by creator
    pub fn by_creator(&self, creator_id: &Uuid) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.creator_id == Some(*creator_id))
            .collect()
    }

    /// Get artifacts by current owner
    pub fn by_owner(&self, owner_id: &Uuid) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.owner_id == Some(*owner_id))
            .collect()
    }

    /// Get most significant artifacts
    pub fn top_significant(&self, n: usize) -> Vec<&Artifact> {
        let mut sorted: Vec<_> = self.artifacts.iter().collect();
        sorted.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap());
        sorted.into_iter().take(n).collect()
    }

    /// Get artifacts created in a year range
    pub fn in_year_range(&self, start_year: i32, end_year: i32) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.created_year >= start_year && a.created_year <= end_year)
            .collect()
    }

    /// Iterate over all artifacts
    pub fn iter(&self) -> impl Iterator<Item = &Artifact> {
        self.artifacts.iter()
    }

    /// Iterate over all artifacts (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Artifact> {
        self.artifacts.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBuilder;
    use crate::HistoricalTime;

    #[test]
    fn test_artifact_creation() {
        let world_id = Uuid::new_v4();
        let artifact = Artifact::new(
            world_id,
            "The Crown of Valdoria".to_string(),
            ArtifactCategory::CrownJewel,
            1250,
            "A golden crown worn by the first king of Valdoria".to_string(),
            0.85,
        );

        assert_eq!(artifact.name, "The Crown of Valdoria");
        assert_eq!(artifact.category, ArtifactCategory::CrownJewel);
        assert_eq!(artifact.created_year, 1250);
        assert!((artifact.significance - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_artifact_store() {
        let mut store = ArtifactStore::new();
        let world_id = Uuid::new_v4();

        store.add(Artifact::new(
            world_id,
            "Sword of Heroes".to_string(),
            ArtifactCategory::Weapon,
            1000,
            "A legendary blade".to_string(),
            0.7,
        ));

        store.add(Artifact::new(
            world_id,
            "Sacred Relic".to_string(),
            ArtifactCategory::Relic,
            1100,
            "A holy relic".to_string(),
            0.6,
        ));

        assert_eq!(store.len(), 2);

        let weapons = store.by_category(ArtifactCategory::Weapon);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].name, "Sword of Heroes");
    }

    #[test]
    fn test_artifact_from_event() {
        let world_id = Uuid::new_v4();
        let event = EventBuilder::new("The Great Victory")
            .event_type(EventType::Battle)
            .time(HistoricalTime::year(1200))
            .build(world_id);

        let artifact = Artifact::from_event(&event, ArtifactCategory::Trophy);
        assert_eq!(artifact.origin_event_id, Some(event.id.to_uuid()));
        assert!(artifact.significance >= 0.5);
    }

    #[test]
    fn test_artifact_rarity() {
        let world_id = Uuid::new_v4();

        // Common (significance < 0.5)
        let common = Artifact::new(
            world_id,
            "Old Coin".to_string(),
            ArtifactCategory::Document,
            100,
            "An old coin".to_string(),
            0.3,
        );
        assert_eq!(common.rarity, ArtifactRarity::Common);

        // Uncommon (0.5 <= significance < 0.7)
        let uncommon = Artifact::new(
            world_id,
            "Silver Cup".to_string(),
            ArtifactCategory::CrownJewel,
            500,
            "A silver cup".to_string(),
            0.55,
        );
        assert_eq!(uncommon.rarity, ArtifactRarity::Uncommon);

        // Rare (0.7 <= significance < 0.85)
        let rare = Artifact::new(
            world_id,
            "Gold Ring".to_string(),
            ArtifactCategory::CrownJewel,
            800,
            "A gold ring".to_string(),
            0.75,
        );
        assert_eq!(rare.rarity, ArtifactRarity::Rare);

        // Legendary (0.85 <= significance < 0.95)
        let legendary = Artifact::new(
            world_id,
            "Crown of Kings".to_string(),
            ArtifactCategory::CrownJewel,
            1000,
            "The crown of all kings".to_string(),
            0.9,
        );
        assert_eq!(legendary.rarity, ArtifactRarity::Legendary);

        // Mythic (significance >= 0.95)
        let mythic = Artifact::new(
            world_id,
            "World Ender".to_string(),
            ArtifactCategory::Magical,
            1500,
            "A weapon that can end worlds".to_string(),
            0.98,
        );
        assert_eq!(mythic.rarity, ArtifactRarity::Mythic);
    }

    #[test]
    fn test_rarity_cataclysm_probability() {
        // Common has very low probability
        assert!(ArtifactRarity::Common.cataclysm_probability() < 0.0001);
        // Mythic has maximum probability (< 0.1%)
        assert!(ArtifactRarity::Mythic.cataclysm_probability() <= 0.001);
        // Each tier has increasing probability
        assert!(
            ArtifactRarity::Rare.cataclysm_probability()
                > ArtifactRarity::Uncommon.cataclysm_probability()
        );
        assert!(
            ArtifactRarity::Legendary.cataclysm_probability()
                > ArtifactRarity::Rare.cataclysm_probability()
        );
        assert!(
            ArtifactRarity::Mythic.cataclysm_probability()
                > ArtifactRarity::Legendary.cataclysm_probability()
        );
    }

    #[test]
    fn test_cataclysm_cap() {
        // Test that cap is applied
        let cap = CataclysmTriggerSystem::cataclysm_cap();
        assert!(cap <= 0.05); // 5% maximum
        assert!(cap > 0.0);
    }

    #[test]
    fn test_artifact_effects() {
        let world_id = Uuid::new_v4();
        let artifact = Artifact::new(
            world_id,
            "Sword of Heroes".to_string(),
            ArtifactCategory::Weapon,
            1000,
            "A legendary blade".to_string(),
            0.85,
        );

        // Get default effects for this artifact
        let effects = ArtifactEffect::default_for_category(artifact.category, artifact.rarity);
        assert!(!effects.is_empty());

        // Verify effect properties
        for effect in &effects {
            assert!(effect.magnitude > 0.0 && effect.magnitude <= 1.0);
            if effect.effect_type != ArtifactEffectType::Cursed
                && effect.effect_type != ArtifactEffectType::Doom
            {
                assert!(effect.effect_type.is_positive());
            }
        }
    }

    #[test]
    fn test_creation_conditions() {
        let world_id = Uuid::new_v4();

        // Test CrownJewel conditions
        let crown_conditions =
            ArtifactCreationCondition::default_for_category(ArtifactCategory::CrownJewel);
        assert!(!crown_conditions.is_empty());

        // Test Weapon conditions
        let weapon_conditions =
            ArtifactCreationCondition::default_for_category(ArtifactCategory::Weapon);
        assert!(!weapon_conditions.is_empty());

        // Test condition satisfaction
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.8;
        context.related_event = Some(Uuid::new_v4());

        let condition = ArtifactCreationCondition {
            min_significance: 0.5,
            condition_type: ArtifactCreationConditionType::SignificantEvent,
            required_figure_type: None,
            min_rarity: None,
        };

        assert!(condition.is_satisfied(&context));

        // Should fail with low significance
        context.significance = 0.2;
        assert!(!condition.is_satisfied(&context));
    }

    #[test]
    fn test_artifact_potential_cataclysm_type() {
        let world_id = Uuid::new_v4();

        let sacred = Artifact::new(
            world_id,
            "Holy Relic".to_string(),
            ArtifactCategory::Sacred,
            1000,
            "A sacred relic".to_string(),
            0.8,
        );
        assert_eq!(sacred.potential_cataclysm_type_name(), "divine_wrath");

        let magical = Artifact::new(
            world_id,
            "Magic Orb".to_string(),
            ArtifactCategory::Magical,
            1000,
            "A magical orb".to_string(),
            0.8,
        );
        assert_eq!(magical.potential_cataclysm_type_name(), "magical_cataclysm");

        let weapon = Artifact::new(
            world_id,
            "Doom Blade".to_string(),
            ArtifactCategory::Weapon,
            1000,
            "A cursed weapon".to_string(),
            0.8,
        );
        assert_eq!(
            weapon.potential_cataclysm_type_name(),
            "civilizational_collapse"
        );
    }

    #[test]
    fn test_causal_chain_validator_weapon() {
        // Legendary weapon requires: iron/gold deposit + warrior figure
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.7;
        context.warrior_figure_exists = true;
        context.iron_or_gold_deposit_nearby = true;
        context.capital_city_nearby = true;

        let validation = CausalChainValidator::can_spawn(ArtifactCategory::Weapon, &context);
        assert!(validation.can_spawn, "Weapon should spawn: {:?}", validation.missing_conditions);
    }

    #[test]
    fn test_causal_chain_validator_sacred_relic() {
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.7;
        context.religious_figure_exists = true;
        context.religious_site_nearby = true;

        let validation = CausalChainValidator::can_spawn(ArtifactCategory::Sacred, &context);
        assert!(validation.can_spawn, "Sacred should spawn: {:?}", validation.missing_conditions);
    }

    #[test]
    fn test_causal_chain_validator_crown_jewel() {
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.8;
        context.gold_deposit_nearby = true;
        context.has_centralized_government = true;
        context.capital_city_nearby = true;

        let validation = CausalChainValidator::can_spawn(ArtifactCategory::CrownJewel, &context);
        assert!(validation.can_spawn, "Crown should spawn: {:?}", validation.missing_conditions);
    }

    #[test]
    fn test_causal_chain_validator_magical() {
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.75;
        context.gem_deposit_nearby = true;
        context.historical_event = true;
        context.magical_tradition_exists = true;
        context.related_event = Some(Uuid::new_v4());

        let validation = CausalChainValidator::can_spawn(ArtifactCategory::Magical, &context);
        assert!(validation.can_spawn, "Magical should spawn: {:?}", validation.missing_conditions);
    }

    #[test]
    fn test_causal_chain_validator_document() {
        let mut context = ArtifactCreationContext::default();
        context.significance = 0.6;
        context.civilized_biome = true;
        context.scholar_figure_exists = true;


        let validation = CausalChainValidator::can_spawn(ArtifactCategory::Document, &context);
        assert!(validation.can_spawn, "Document should spawn: {:?}", validation.missing_conditions);
    }

    #[test]
    fn test_causal_chain_validation_failure() {
        let context = ArtifactCreationContext::default();
        let validation = CausalChainValidator::can_spawn(ArtifactCategory::Weapon, &context);
        assert!(!validation.can_spawn);
        assert!(!validation.missing_conditions.is_empty());
    }
}

// Import EventBuilder for tests
