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

/// Context required to evaluate artifact creation conditions
#[derive(Debug, Clone, Default)]
pub struct ArtifactCreationContext {
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
    /// Destroys terrain (volcanoes, earthquakes, etc.)
    TerrainDestruction,
    /// Mass population devastation
    PopulationDevastation,
    /// Technological advancement boost
    TechnologyBoost,
    /// Climate shifts and environmental changes
    ClimateShift,
    /// Territory expansion/claiming
    TerritoryGain,
    /// Society transformation
    SocietyTransform,
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
            Self::TerrainDestruction => "Terrain Destruction",
            Self::PopulationDevastation => "Population Devastation",
            Self::TechnologyBoost => "Technology Boost",
            Self::ClimateShift => "Climate Shift",
            Self::TerritoryGain => "Territory Gain",
            Self::SocietyTransform => "Society Transform",
        }
    }

    /// Whether this is a positive or negative effect
    pub fn is_positive(&self) -> bool {
        !matches!(
            self,
            Self::Cursed | Self::Doom | Self::TerrainDestruction | Self::PopulationDevastation
        )
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCondition {
    /// Pristine, never used
    Pristine,
    /// Normal wear and tear
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

impl Default for ArtifactCondition {
    fn default() -> Self {
        Self::Worn
    }
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

    /// Number of activations used (max 3)
    #[serde(default)]
    pub activations_used: u8,

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
            activations_used: 0,
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

    /// Maximum allowed activations for an artifact
    pub const MAX_ACTIVATIONS: u8 = 3;

    /// Whether the artifact has activations remaining
    pub fn can_activate(&self) -> bool {
        self.activations_used < Self::MAX_ACTIVATIONS
    }

    /// Activate the artifact (consumes one activation).
    /// Returns true if activation succeeded, false if max reached.
    pub fn activate(&mut self) -> bool {
        if self.can_activate() {
            self.activations_used += 1;
            self.updated_at = Timestamp::now();
            true
        } else {
            false
        }
    }

    /// Get the number of remaining activations
    pub fn remaining_activations(&self) -> u8 {
        Self::MAX_ACTIVATIONS.saturating_sub(self.activations_used)
    }
}

// ============================================================================
// Artifact Name Generator
// ============================================================================

/// Generator for artifact names using template "The {Title} {Material} {Type}"
///
/// Names are procedurally generated based on artifact category, rarity, and
/// historical context. Templates follow the pattern: "The {Title} {Material}
/// {Type}" e.g., "The Ancient Iron Crown", "The Forgotten Mithril Blade"
pub struct ArtifactNameGenerator {
    /// Random number generator
    rng: Rng,
}

impl ArtifactNameGenerator {
    /// Create a new name generator with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
        }
    }

    /// Generate a random index in range [0, max)
    fn random_index(&mut self, max: u32) -> usize {
        (self.rng.next() % max as u64) as usize
    }

    /// Generate a name for an artifact based on its category and rarity
    ///
    /// Template: "The {Title} {Material} {Type}"
    /// Example output: "The Ancient Mithril Crown", "The Cursed Obsidian Blade"
    pub fn generate_name(&mut self, category: ArtifactCategory, rarity: ArtifactRarity) -> String {
        let title = self.title_for_rarity(rarity);
        let material = self.material_for_category();
        let type_name = self.type_name_for_category(category);

        format!("The {} {} {}", title, material, type_name)
    }

    /// Get title adjective based on rarity
    fn title_for_rarity(&mut self, rarity: ArtifactRarity) -> String {
        let rarity_idx = rarity as u8 as usize;
        let titles: Vec<&str> = match rarity_idx {
            0 => vec!["Ancient", "Eternal", "Forgotten", "Lost", "Hidden"], // Common
            1 => vec!["Blessed", "Holy", "Sacred", "Divine", "Pure"],       // Uncommon
            2 => vec!["Cursed", "Dark", "Shadow", "Doomed", "Vile"],        // Rare
            3 => vec!["Legendary", "Mythical", "Epic", "Heroic", "Noble"],  // Legendary
            4 => vec!["World", "Cosmic", "Primordial", "Infinite", "Absolute"], // Mythic
            _ => vec!["Mysterious", "Unknown", "Strange", "Peculiar", "Odd"],
        };
        titles[self.random_index(titles.len() as u32)].to_string()
    }

    /// Get material based on category
    fn material_for_category(&mut self) -> String {
        let materials: Vec<&str> = vec![
            "Gold",
            "Silver",
            "Bronze",
            "Iron",
            "Steel",
            "Mithril",
            "Adamantine",
            "Orichalcum",
            "Crystal",
            "Obsidian",
            "Wooden",
            "Stone",
            "Bone",
            "Ivory",
            "Jade",
            "Celestial",
            "Ethereal",
            "Void",
            "Primal",
            "Arcane",
            "Phoenix",
            "Dragon",
            "Serpent",
            "Phoenix Feather",
            "Dragon Scale",
            "Mystic",
            "Enchanted",
            "Blessed",
            "Cursed",
            "Ancient",
        ];
        materials[self.random_index(materials.len() as u32)].to_string()
    }

    /// Get the type name for the category
    fn type_name_for_category(&mut self, category: ArtifactCategory) -> String {
        let types: Vec<&str> = match category {
            ArtifactCategory::CrownJewel => vec!["Crown", "Diadem", "Tiara", "Coronet", "Scepter"],
            ArtifactCategory::Weapon => vec!["Blade", "Sword", "Axe", "Spear", "Dagger"],
            ArtifactCategory::Magical => vec!["Orb", "Staff", "Wand", "Tome", "Amulet"],
            ArtifactCategory::Relic | ArtifactCategory::Sacred => {
                vec!["Reliquary", "Icon", "Chalice", "Scripture", "Holy relic"]
            }
            ArtifactCategory::Monument => vec!["Monument", "Statue", "Obelisk", "Pillar", "Tower"],
            ArtifactCategory::Document => vec!["Scroll", "Codex", "Map", "Treatise", "Chronicle"],
            ArtifactCategory::Trophy => vec!["Trophy", "Spoils", "Banner", "Helmet", "Shield"],
        };
        types[self.random_index(types.len() as u32)].to_string()
    }
}

// ============================================================================
// Artifact Creation Conditions
// ============================================================================

/// Context for checking if an artifact can be created
#[derive(Debug, Clone)]
pub struct ArtifactCreationCheck {
    /// Impact score of the associated figure (> 20 for creation)
    pub figure_impact: Option<f32>,
    /// Number of rare resources used in creation
    pub rare_resources_used: usize,
    /// Years since last artifact of this type
    pub years_since_last_artifact: i32,
    /// Whether a significant event occurred
    pub significant_event_occurred: bool,
}

impl ArtifactCreationCheck {
    /// Minimum figure impact required for artifact creation
    pub const MIN_FIGURE_IMPACT: f32 = 20.0;

    /// Minimum rare resources required
    pub const MIN_RARE_RESOURCES: usize = 3;

    /// Minimum year gap between artifacts
    pub const MIN_YEAR_GAP: i32 = 200;

    /// Check if an artifact can be created based on conditions
    ///
    /// An artifact can be created if ANY of the following is true:
    /// - Figure impact > 20
    /// - 3+ rare resources used
    /// - 200 year gap since last artifact AND significant event occurred
    pub fn can_create_artifact(&self) -> bool {
        // Option 1: High-impact figure
        if let Some(impact) = self.figure_impact {
            if impact > Self::MIN_FIGURE_IMPACT {
                return true;
            }
        }

        // Option 2: Rare resources
        if self.rare_resources_used >= Self::MIN_RARE_RESOURCES {
            return true;
        }

        // Option 3: Time gap + significant event
        if self.years_since_last_artifact >= Self::MIN_YEAR_GAP && self.significant_event_occurred {
            return true;
        }

        false
    }

    /// Get the reason why artifact can/cannot be created
    pub fn creation_reason(&self) -> &'static str {
        if let Some(impact) = self.figure_impact {
            if impact > Self::MIN_FIGURE_IMPACT {
                return "Created by high-impact figure";
            }
        }

        if self.rare_resources_used >= Self::MIN_RARE_RESOURCES {
            return "Created using rare resources";
        }

        if self.years_since_last_artifact >= Self::MIN_YEAR_GAP && self.significant_event_occurred {
            return "Created after significant event following long gap";
        }

        "Creation requirements not met"
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
}

// Import EventBuilder for tests
