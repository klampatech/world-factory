//! Cataclysm System for World Factory
//!
//! Major world-altering events that reshape civilizations, geography, and history.
//! Cataclysms are the most significant events in world history, causing lasting
//! changes to terrain, cultures, and the course of civilization.
//!
//! ## Cataclysm Categories
//!
//! - **Natural**: Volcanic eruptions, meteor strikes, ice ages
//! - **Magical**: Magical disasters, planar breaches, divine interventions
//! - **Technological**: Magical/technological catastrophes
//! - **Social**: Civilizational collapses, great migrations
//!
//! ## Effects
//!
//! Cataclysms can reshape terrain (creating mountains, flattening forests),
//! wipe out civilizations, trigger mass migrations, and leave lasting
//! scars on the world that influence future history.

use crate::events::{Event, EventEffect, EventType};
use crate::types::{EntityId, EntityType, HistoricalTime, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of cataclysmic events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CataclysmType {
    /// Volcanic eruption of massive proportions
    VolcanicEruption,
    /// Meteor or asteroid impact
    MeteorStrike,
    /// Major earthquake
    GreatQuake,
    /// Rising sea levels or major flood
    GreatFlood,
    /// Extended period of drought
    Megadrought,
    /// Plague that devastates populations
    GreatPlague,
    /// Ice age or major climate cooling
    IceAge,
    /// Magical catastrophe
    MagicalCataclysm,
    /// Divine intervention or god's wrath
    DivineWrath,
    /// Planar breach or invasion
    PlanarInvasion,
    /// Civilizational collapse
    CivilizationalCollapse,
    /// Great migration that reshapes cultures
    GreatMigration,
    /// Poisoning of land or water
    Blight,
    /// Loss of cultural knowledge and artifacts
    CulturalLoss,
}

impl CataclysmType {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::VolcanicEruption => "Volcanic Eruption",
            Self::MeteorStrike => "Meteor Strike",
            Self::GreatQuake => "The Great Quake",
            Self::GreatFlood => "The Great Flood",
            Self::Megadrought => "Megadrought",
            Self::GreatPlague => "The Great Plague",
            Self::IceAge => "Ice Age",
            Self::MagicalCataclysm => "Magical Cataclysm",
            Self::DivineWrath => "Divine Wrath",
            Self::PlanarInvasion => "Planar Invasion",
            Self::CivilizationalCollapse => "Civilizational Collapse",
            Self::GreatMigration => "The Great Migration",
            Self::Blight => "The Blight",
            Self::CulturalLoss => "The Great Burning",
        }
    }

    /// Get the default severity (0.0 to 1.0)
    pub fn default_severity(&self) -> f32 {
        match self {
            Self::MeteorStrike => 1.0,
            Self::IceAge => 0.95,
            Self::VolcanicEruption => 0.85,
            Self::GreatPlague => 0.8,
            Self::PlanarInvasion => 0.85,
            Self::DivineWrath => 0.9,
            Self::CivilizationalCollapse => 0.75,
            Self::GreatFlood => 0.7,
            Self::GreatQuake => 0.7,
            Self::MagicalCataclysm => 0.8,
            Self::Megadrought => 0.65,
            Self::GreatMigration => 0.5,
            Self::Blight => 0.6,
            Self::CulturalLoss => 0.55,
        }
    }

    /// Map to corresponding EventType
    pub fn to_event_type(&self) -> EventType {
        match self {
            Self::VolcanicEruption => EventType::Volcano,
            Self::MeteorStrike => EventType::MeteorStrike,
            Self::GreatQuake => EventType::Earthquake,
            Self::GreatFlood => EventType::Flood,
            Self::Megadrought => EventType::Drought,
            Self::GreatPlague => EventType::Plague,
            Self::IceAge => EventType::EnvironmentalChange,
            Self::MagicalCataclysm => EventType::MagicalCatastrophe,
            Self::DivineWrath => EventType::ReligiousEvent,
            Self::PlanarInvasion => EventType::FirstContact,
            Self::CivilizationalCollapse => EventType::Collapse,
            Self::GreatMigration => EventType::Migration,
            Self::Blight => EventType::Extinction,
            Self::CulturalLoss => EventType::CulturalAchievement, // Maps to cultural events
        }
    }
}

/// Severity or magnitude of the cataclysm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CataclysmSeverity {
    /// Localized, affects a single region
    Local,
    /// Affects multiple regions or a small nation
    Regional,
    /// Affects multiple nations or a continent
    Continental,
    /// Affects the entire world
    Global,
}

impl CataclysmSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Regional => "Regional",
            Self::Continental => "Continental",
            Self::Global => "Global",
        }
    }
}

/// Recovery state of a region from a cataclysm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryState {
    /// Still suffering immediate effects
    Active,
    /// Recovering but not yet stable
    Recovering,
    /// Mostly recovered but scars remain
    Scarring,
    /// Fully recovered
    Recovered,
    /// Permanently altered, cannot fully recover
    PermanentlyAltered,
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self::Recovering
    }
}

/// Regional impact of a cataclysm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionImpact {
    /// Region affected
    pub region_id: Uuid,
    /// Severity of impact in this region (0.0 to 1.0)
    pub severity: f32,
    /// Current recovery state
    #[serde(default)]
    pub recovery_state: RecoveryState,
    /// Year when impact started
    pub start_year: i32,
    /// Year when full recovery expected (if ever)
    pub recovery_year: Option<i32>,
    /// Population loss percentage
    pub population_loss_pct: Option<f32>,
    /// Cultural damage (artifacts lost, knowledge destroyed)
    pub cultural_damage: Option<f32>,
    /// Terrain permanently altered
    pub terrain_altered: bool,
    /// Notes on specific effects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Cataclysm description and effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CataclysmEffect {
    /// Effect description
    pub description: String,
    /// Effect magnitude (0.0 to 1.0)
    pub magnitude: f32,
    /// Type of effect
    pub effect_type: CataclysmEffectType,
}

/// Types of cataclysmic effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CataclysmEffectType {
    /// Population effects
    Population,
    /// Terrain modification
    Terrain,
    /// Cultural effects (loss of knowledge, artifacts)
    Cultural,
    /// Economic effects
    Economic,
    /// Political effects
    Political,
    /// Magical effects
    Magical,
    /// Climate effects
    Climate,
    /// Religious/spiritual effects
    Religious,
}

/// A cataclysmic event in world history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cataclysm {
    /// Unique identifier
    pub id: EntityId,

    /// World this cataclysm belongs to
    pub world_id: Uuid,

    /// Type of cataclysm
    pub cataclysm_type: CataclysmType,

    /// Human-readable name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Year when it occurred
    pub year: i32,

    /// How long the cataclysm lasted
    pub duration_years: Option<i32>,

    /// Severity level
    pub severity: f32,

    /// Geographic scope
    pub scope: CataclysmSeverity,

    /// Regions affected
    pub impacts: Vec<RegionImpact>,

    /// Primary effects of the cataclysm
    pub effects: Vec<CataclysmEffect>,

    /// Entities that survived or were created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survivors: Option<Vec<Uuid>>,

    /// Total population lost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_population_lost: Option<u64>,

    /// Cultures that were destroyed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cultures_destroyed: Option<Vec<String>>,

    /// Cultures that emerged from the aftermath
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cultures_emerged: Option<Vec<String>>,

    /// Related events (precursors and consequences)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_events: Option<Vec<Uuid>>,

    /// Historical significance (0.0 to 1.0)
    pub significance: f32,

    /// Timestamp
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Cataclysm {
    /// Create a new cataclysm
    pub fn new(
        world_id: Uuid,
        cataclysm_type: CataclysmType,
        name: String,
        description: String,
        year: i32,
        severity: f32,
        scope: CataclysmSeverity,
        effects: Vec<CataclysmEffect>,
        significance: f32,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Event),
            world_id,
            cataclysm_type,
            name,
            description,
            year,
            duration_years: None,
            severity: severity.clamp(0.0, 1.0),
            scope,
            impacts: Vec::new(),
            effects,
            survivors: None,
            total_population_lost: None,
            cultures_destroyed: None,
            cultures_emerged: None,
            related_events: None,
            significance: significance.clamp(0.0, 1.0),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a regional impact
    pub fn add_impact(&mut self, impact: RegionImpact) {
        self.impacts.push(impact);
        self.updated_at = Timestamp::now();
    }

    /// Convert to a historical Event
    pub fn to_event(&self) -> Event {
        let mut event = Event::new(
            self.world_id,
            self.name.clone(),
            self.description.clone(),
            self.cataclysm_type.to_event_type(),
            HistoricalTime::year(self.year),
        );

        event.significance = Some(self.significance);

        if let Some(duration) = self.duration_years {
            event.end_time = Some(HistoricalTime::year(self.year + duration));
        }

        // Add effects based on cataclysm type
        for effect in &self.effects {
            let event_effect = match effect.effect_type {
                CataclysmEffectType::Population => EventEffect::PopulationLoss {
                    target: Uuid::new_v4(), // Would need proper target
                    amount: 0,
                    duration_years: self.duration_years,
                    cause: Some(effect.description.clone()),
                },
                CataclysmEffectType::Terrain => EventEffect::EnvironmentalChange {
                    region: Uuid::new_v4(),
                    change_type: crate::events::effect::EnvironmentalChangeType::ClimateShift,
                    duration_years: self.duration_years,
                    magnitude: crate::events::effect::EffectMagnitude::Major,
                },
                CataclysmEffectType::Cultural => EventEffect::CulturalChange {
                    target: Uuid::new_v4(),
                    change_type: crate::events::effect::CulturalChangeType::Decline,
                    duration_years: self.duration_years,
                },
                CataclysmEffectType::Economic => EventEffect::EconomicChange {
                    target: Uuid::new_v4(),
                    change_type: crate::events::effect::EconomicChangeType::Depression,
                    magnitude: crate::events::effect::EffectMagnitude::Major,
                    duration_years: self.duration_years,
                },
                CataclysmEffectType::Political => EventEffect::GovernmentChange {
                    target: Uuid::new_v4(),
                    from_government: None,
                    to_government: None,
                    cause: Some(effect.description.clone()),
                },
                CataclysmEffectType::Magical => EventEffect::Destruction {
                    destroyer: Uuid::new_v4(),
                    structure: "Magical barrier".to_string(),
                    location: Uuid::new_v4(),
                    cause: Some(effect.description.clone()),
                },
                CataclysmEffectType::Climate => EventEffect::EnvironmentalChange {
                    region: Uuid::new_v4(),
                    change_type: crate::events::effect::EnvironmentalChangeType::Warming,
                    duration_years: self.duration_years,
                    magnitude: crate::events::effect::EffectMagnitude::Major,
                },
                CataclysmEffectType::Religious => EventEffect::ReligiousChange {
                    target: Uuid::new_v4(),
                    change_type: crate::events::effect::ReligiousChangeType::Suppression,
                    from_religion: None,
                    to_religion: None,
                },
            };
            event.effects.push(event_effect);
        }

        event
    }

    /// Calculate the total impact severity
    pub fn total_impact(&self) -> f32 {
        if self.impacts.is_empty() {
            return self.severity;
        }

        let sum: f32 = self.impacts.iter().map(|i| i.severity).sum();
        sum / self.impacts.len() as f32
    }
}

/// Cataclysm store for managing all cataclysms in a world
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CataclysmStore {
    /// All cataclysms
    cataclysms: Vec<Cataclysm>,
}

impl CataclysmStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self {
            cataclysms: Vec::new(),
        }
    }

    /// Add a cataclysm
    pub fn add(&mut self, cataclysm: Cataclysm) {
        self.cataclysms.push(cataclysm);
    }

    /// Number of cataclysms
    pub fn len(&self) -> usize {
        self.cataclysms.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cataclysms.is_empty()
    }

    /// Get all cataclysms
    pub fn cataclysms(&self) -> &[Cataclysm] {
        &self.cataclysms
    }

    /// Get cataclysm by ID
    pub fn get(&self, id: &Uuid) -> Option<&Cataclysm> {
        self.cataclysms.iter().find(|c| c.id.to_uuid() == *id)
    }

    /// Get cataclysms by type
    pub fn by_type(&self, cataclysm_type: CataclysmType) -> Vec<&Cataclysm> {
        self.cataclysms
            .iter()
            .filter(|c| c.cataclysm_type == cataclysm_type)
            .collect()
    }

    /// Get cataclysms in a year range
    pub fn in_year_range(&self, start_year: i32, end_year: i32) -> Vec<&Cataclysm> {
        self.cataclysms
            .iter()
            .filter(|c| c.year >= start_year && c.year <= end_year)
            .collect()
    }

    /// Get cataclysms affecting a region
    pub fn affecting_region(&self, region_id: &Uuid) -> Vec<&Cataclysm> {
        self.cataclysms
            .iter()
            .filter(|c| c.impacts.iter().any(|i| i.region_id == *region_id))
            .collect()
    }

    /// Get most severe cataclysms
    pub fn most_severe(&self, n: usize) -> Vec<&Cataclysm> {
        let mut sorted: Vec<_> = self.cataclysms.iter().collect();
        sorted.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        sorted.into_iter().take(n).collect()
    }

    /// Get cataclysms by scope
    pub fn by_scope(&self, scope: CataclysmSeverity) -> Vec<&Cataclysm> {
        self.cataclysms
            .iter()
            .filter(|c| c.scope == scope)
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> CataclysmStats {
        let mut by_type: HashMap<CataclysmType, usize> = HashMap::new();
        let mut by_scope: HashMap<CataclysmSeverity, usize> = HashMap::new();
        let mut total_population_lost: u64 = 0;
        let mut global_count = 0;

        for cataclysm in &self.cataclysms {
            *by_type.entry(cataclysm.cataclysm_type).or_insert(0) += 1;
            *by_scope.entry(cataclysm.scope).or_insert(0) += 1;
            if let Some(pops) = cataclysm.total_population_lost {
                total_population_lost += pops;
            }
            if cataclysm.scope == CataclysmSeverity::Global {
                global_count += 1;
            }
        }

        CataclysmStats {
            total_cataclysms: self.cataclysms.len(),
            by_type,
            by_scope,
            total_population_lost,
            global_cataclysms: global_count,
        }
    }
}

/// Statistics about cataclysms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CataclysmStats {
    pub total_cataclysms: usize,
    pub by_type: HashMap<CataclysmType, usize>,
    pub by_scope: HashMap<CataclysmSeverity, usize>,
    pub total_population_lost: u64,
    pub global_cataclysms: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cataclysm_creation() {
        let world_id = Uuid::new_v4();
        let cataclysm = Cataclysm::new(
            world_id,
            CataclysmType::VolcanicEruption,
            "The Fire Mountain Erupts".to_string(),
            "The great volcano erupted, devastating the surrounding lands".to_string(),
            1200,
            0.85,
            CataclysmSeverity::Continental,
            vec![CataclysmEffect {
                description: "Widespread destruction of settlements".to_string(),
                magnitude: 0.8,
                effect_type: CataclysmEffectType::Population,
            }],
            0.9,
        );

        assert_eq!(cataclysm.name, "The Fire Mountain Erupts");
        assert_eq!(cataclysm.cataclysm_type, CataclysmType::VolcanicEruption);
        assert!((cataclysm.severity - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_cataclysm_to_event() {
        let world_id = Uuid::new_v4();
        let cataclysm = Cataclysm::new(
            world_id,
            CataclysmType::GreatPlague,
            "The Crimson Death".to_string(),
            "A devastating plague swept across the land".to_string(),
            1347,
            0.8,
            CataclysmSeverity::Global,
            vec![],
            0.95,
        );

        let event = cataclysm.to_event();
        assert_eq!(event.event_type, EventType::Plague);
        assert!((event.significance.unwrap() - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_cataclysm_store() {
        let mut store = CataclysmStore::new();
        let world_id = Uuid::new_v4();

        store.add(Cataclysm::new(
            world_id,
            CataclysmType::MeteorStrike,
            "The Impact".to_string(),
            "A massive meteor struck the world".to_string(),
            1500,
            1.0,
            CataclysmSeverity::Global,
            vec![],
            1.0,
        ));

        assert_eq!(store.len(), 1);

        let severe = store.most_severe(1);
        assert_eq!(severe[0].cataclysm_type, CataclysmType::MeteorStrike);
    }
}
