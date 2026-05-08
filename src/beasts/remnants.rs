//! Remnant artifacts created from slain primal beasts.
//!
//! Per SPEC.md §D.4.3: When a primal beast is killed, the triggering faction absorbs
//! the curse but also inherits the Remnant artifact (dropped by the beast on death).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{EntityId, GeoLocation};

use super::{BeastElement, PrimalBeast};
use crate::artifacts::{Artifact, ArtifactCategory, ArtifactRarity};

/// A Remnant artifact dropped when a primal beast is slain.
/// 
/// The Remnant contains the beast's elemental essence, the curse it carried,
/// and any blessing it may have had. The killing faction inherits all of these.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemnantArtifact {
    /// Unique entity ID for this Remnant
    pub id: EntityId,
    /// The primal beast type that created this Remnant
    pub source_beast: PrimalBeast,
    /// The primal beast element that created this Remnant
    pub element: BeastElement,
    /// Whether the curse is currently active
    pub curse_active: bool,
    /// Radius of environmental effects in kilometers
    pub effect_radius_km: f32,
    /// Position where the beast was slain
    pub death_location: u32,
    /// Geographic location of the death
    pub geo_location: GeoLocation,
    /// The embedded artifact properties
    pub artifact: Artifact,
    /// Description of the curse effect
    pub curse_effect: String,
    /// Description of the blessing effect
    pub blessing_effect: String,
    /// Decay state: 0.0 = fresh, 1.0 = fully decayed
    pub decay_state: f32,
    /// Year when the beast was slain
    pub death_year: i32,
}

impl RemnantArtifact {
    /// Create a new Remnant artifact from beast slaying.
    pub fn from_beast_slaying(
        source_beast: PrimalBeast,
        element: BeastElement,
        position: u32,
        geo_location: GeoLocation,
        curse_effect: String,
        blessing_effect: Option<String>,
        effect_radius_km: f32,
        year: i32,
    ) -> Self {
        Self {
            id: EntityId::new(crate::types::EntityType::Artifact),
            source_beast,
            element,
            curse_active: true,
            effect_radius_km,
            death_location: position,
            geo_location,
            artifact: Artifact::new(
                Uuid::new_v4(),
                format!("{:?} Remnant", element),
                ArtifactCategory::Magical,
                1000,
                format!("Remnant of slain {} containing {} essence", source_beast.name(), element.name()),
                0.95, // High significance for beast remnants
            ),
            curse_effect,
            blessing_effect: blessing_effect.unwrap_or_default(),
            decay_state: 0.0,
            death_year: year,
        }
    }
    
    /// Apply annual decay to the Remnant.
    pub fn apply_decay(&mut self, years: i32, decay_rate: f32) {
        self.decay_state = (self.decay_state + (years as f32 * decay_rate)).min(1.0);
        // Curse becomes inactive as decay progresses
        if self.decay_state > 0.5 {
            self.curse_active = false;
        }
    }
    
    /// Check if the Remnant has fully decayed.
    pub fn is_decayed(&self) -> bool {
        self.decay_state >= 1.0
    }
}

/// Event emitted when a primal beast is slain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastSlainEvent {
    /// Year when the beast was slain
    pub year: i32,
    /// Which beast was slain
    pub beast_name: String,
    /// The element of the slain beast
    pub element: BeastElement,
    /// Where the beast was slain
    pub location: u32,
    /// Faction(s) responsible for the slaying
    pub slaying_factions: Vec<EntityId>,
    /// The Remnant created from this slaying
    pub remnant_id: EntityId,
}

/// System for managing all Remnants in a world.
/// Tracks decay, location, and ownership.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemnantSystem {
    /// All active Remnants
    remnants: Vec<RemnantArtifact>,
    /// Decay rate per year (default 0.01 = 1% per year)
    #[serde(default = "default_decay_rate")]
    decay_rate: f32,
}

fn default_decay_rate() -> f32 {
    0.01
}

impl RemnantSystem {
    /// Create a new empty Remnant system.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a Remnant to the system.
    pub fn add_remnant(&mut self, remnant: RemnantArtifact) {
        self.remnants.push(remnant);
    }
    
    /// Apply annual decay to all Remnants.
    pub fn apply_annual_decay(&mut self, years: i32) {
        for remnant in &mut self.remnants {
            remnant.apply_decay(years, self.decay_rate);
        }
        // Remove fully decayed remnants
        self.remnants.retain(|r| !r.is_decayed());
    }
    
    /// Get all active Remnants.
    pub fn get_remnants(&self) -> &[RemnantArtifact] {
        &self.remnants
    }
    
    /// Get Remnants by element.
    pub fn get_remnants_by_element(&self, element: BeastElement) -> Vec<&RemnantArtifact> {
        self.remnants.iter().filter(|r| r.element == element).collect()
    }
    
    /// Get Remnant count.
    pub fn count(&self) -> usize {
        self.remnants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityId;
    use crate::beasts::BeastElement;
    use crate::beasts::PrimalBeast;

    #[test]
    fn test_remnant_creation() {
        let remnant = RemnantArtifact::from_beast_slaying(
            PrimalBeast::Pyraxes,
            BeastElement::Fire,
            42,
            GeoLocation::new(45.0, -93.0),
            "Curse of Burning".to_string(),
            None,
            10.0,
            1200,
        );
        
        assert_eq!(remnant.source_beast, PrimalBeast::Pyraxes);
        assert_eq!(remnant.element, BeastElement::Fire);
        assert_eq!(remnant.death_location, 42);
        assert!(remnant.curse_active);
        assert_eq!(remnant.decay_state, 0.0);
        assert_eq!(remnant.curse_effect, "Curse of Burning");
    }
    
    #[test]
    fn test_remnant_decay() {
        let mut remnant = RemnantArtifact::from_beast_slaying(
            PrimalBeast::Tidarth,
            BeastElement::Water,
            100,
            GeoLocation::new(50.0, 10.0),
            "Curse of Tides".to_string(),
            Some("Blessing of Tides".to_string()),
            15.0,
            1300,
        );
        
        remnant.apply_decay(10, 0.05); // 10 years at 5% per year
        assert!(remnant.decay_state > 0.0);
        assert!(!remnant.curse_active); // Curse inactive after > 50% decay
        assert!(!remnant.is_decayed());
        
        remnant.apply_decay(200, 0.05); // Another 200 years
        assert!(remnant.is_decayed());
    }
    
    #[test]
    fn test_remnant_system() {
        let mut system = RemnantSystem::new();
        
        let remnant1 = RemnantArtifact::from_beast_slaying(
            PrimalBeast::Pyraxes,
            BeastElement::Fire,
            1,
            GeoLocation::new(0.0, 0.0),
            "Fire Curse".to_string(),
            None,
            10.0,
            1000,
        );
        let remnant2 = RemnantArtifact::from_beast_slaying(
            PrimalBeast::Tidarth,
            BeastElement::Water,
            2,
            GeoLocation::new(10.0, 10.0),
            "Water Curse".to_string(),
            None,
            10.0,
            1000,
        );
        
        system.add_remnant(remnant1);
        system.add_remnant(remnant2);
        
        assert_eq!(system.count(), 2);
        
        let fire_remnants = system.get_remnants_by_element(BeastElement::Fire);
        assert_eq!(fire_remnants.len(), 1);
    }
}