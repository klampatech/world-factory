//! Remnant artifacts created from slain primal beasts.
//!
//! Per SPEC.md §D.4.3: When a primal beast is killed, the triggering faction absorbs
//! the curse but also inherits the Remnant artifact (dropped by the beast on death).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{EntityId, GeoLocation};

use super::BeastElement;

/// A Remnant artifact dropped when a primal beast is slain.
/// 
/// The Remnant contains the beast's elemental essence, the curse it carried,
/// and any blessing it may have had. The killing faction inherits all of these.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemnantArtifact {
    /// Unique entity ID for this Remnant
    pub id: EntityId,
    /// The primal beast element that created this Remnant
    pub element: BeastElement,
    /// Position where the beast was slain
    pub death_location: u32,
    /// Geographic location of the death
    pub geo_location: GeoLocation,
    /// The curse this Remnant carries (faction inherits this)
    pub curse: Option<String>,
    /// The blessing this Remnant carries (faction inherits this)
    pub blessing: Option<String>,
    /// Power level of the Remnant (typically lower than living beast)
    pub power: f32,
    /// Decay state: 0.0 = fresh, 1.0 = fully decayed
    pub decay_state: f32,
    /// Year when the beast was slain
    pub death_year: i32,
}

impl RemnantArtifact {
    /// Create a new Remnant artifact from beast slaying.
    pub fn from_beast_slaying(
        element: BeastElement,
        position: u32,
        geo_location: GeoLocation,
        curse: Option<String>,
        blessing: Option<String>,
        year: i32,
    ) -> Self {
        Self {
            id: EntityId::new(crate::types::EntityType::Artifact),
            element,
            death_location: position,
            geo_location,
            curse,
            blessing,
            power: 0.5, // Remnants are less powerful than living beasts
            decay_state: 0.0,
            death_year: year,
        }
    }
    
    /// Apply annual decay to the Remnant.
    pub fn apply_decay(&mut self, years: i32, decay_rate: f32) {
        self.decay_state = (self.decay_state + (years as f32 * decay_rate)).min(1.0);
        // Power decreases as decay increases
        self.power = (self.power * (1.0 - self.decay_state * 0.5)).max(0.1);
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

    #[test]
    fn test_remnant_creation() {
        let remnant = RemnantArtifact::from_beast_slaying(
            BeastElement::Fire,
            42,
            GeoLocation::new(45.0, -93.0),
            Some("Curse of Burning".to_string()),
            None,
            1200,
        );
        
        assert_eq!(remnant.element, BeastElement::Fire);
        assert_eq!(remnant.death_location, 42);
        assert_eq!(remnant.power, 0.5);
        assert_eq!(remnant.decay_state, 0.0);
    }
    
    #[test]
    fn test_remnant_decay() {
        let mut remnant = RemnantArtifact::from_beast_slaying(
            BeastElement::Water,
            100,
            GeoLocation::new(50.0, 10.0),
            None,
            Some("Blessing of Tides".to_string()),
            1300,
        );
        
        remnant.apply_decay(10, 0.05); // 10 years at 5% per year
        assert!(remnant.decay_state > 0.0);
        assert!(remnant.power < 0.5);
        assert!(!remnant.is_decayed());
        
        remnant.apply_decay(200, 0.05); // Another 200 years
        assert!(remnant.is_decayed());
    }
    
    #[test]
    fn test_remnant_system() {
        let mut system = RemnantSystem::new();
        
        let remnant1 = RemnantArtifact::from_beast_slaying(
            BeastElement::Fire,
            1,
            GeoLocation::new(0.0, 0.0),
            None,
            None,
            1000,
        );
        let remnant2 = RemnantArtifact::from_beast_slaying(
            BeastElement::Water,
            2,
            GeoLocation::new(10.0, 10.0),
            None,
            None,
            1000,
        );
        
        system.add_remnant(remnant1);
        system.add_remnant(remnant2);
        
        assert_eq!(system.count(), 2);
        
        let fire_remnants = system.get_remnants_by_element(BeastElement::Fire);
        assert_eq!(fire_remnants.len(), 1);
    }
}