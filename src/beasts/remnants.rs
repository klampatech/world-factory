//! Remnants Module
//!
//! Manages Remnant artifacts created when primal beasts are slain.
//! Per SPEC.md §D.4.3, a Remnant is dropped at the beast's death location.
//!
//! ## Components
//!
//! - `RemnantArtifact`: Individual remnant dropped by a beast
//! - `BeastSlainEvent`: Record of a beast slaying event
//! - `RemnantSystem`: Manages all remnants in a world

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use uuid::Uuid;

use super::{BeastElement, PrimalBeast};

/// Effect intensity levels for remnants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectIntensity {
    /// Full effect strength (recently created)
    Strong,
    /// Moderate effect (some decay)
    Moderate,
    /// Weak effect (significant decay)
    Weak,
    /// Minimal effect (near expiration)
    Fading,
}

impl Default for EffectIntensity {
    fn default() -> Self {
        EffectIntensity::Strong
    }
}

/// Remnant artifact created when a beast is slain.
/// Dropped at the beast's death location and affects surrounding area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemnantArtifact {
    /// Unique ID for this remnant
    pub id: Uuid,
    /// World ID where the beast was slain
    pub world_id: Uuid,
    /// Beast type that created this remnant
    pub beast: PrimalBeast,
    /// Element of the beast
    pub element: BeastElement,
    /// Year the beast was slain
    pub slaying_year: i32,
    /// Position where the beast was slain (cell index)
    pub position: u32,
    /// Curse effect description
    pub curse_effect: String,
    /// Blessing effect description
    pub blessing_effect: String,
    /// Effect radius in km
    pub effect_radius_km: f32,
    /// Whether the curse is currently active
    pub curse_active: bool,
    /// Effect intensity (decays over time)
    pub intensity: EffectIntensity,
    /// Remaining years before remnant fades completely
    pub remaining_years: i32,
}

impl RemnantArtifact {
    /// Create a new remnant from beast slaying data.
    pub fn new(
        world_id: Uuid,
        beast: PrimalBeast,
        slaying_year: i32,
        position: u32,
        curse_effect: String,
        blessing_effect: String,
    ) -> Self {
        let element = beast.element();
        Self {
            id: Uuid::new_v4(),
            world_id,
            beast,
            element,
            slaying_year,
            position,
            curse_effect,
            blessing_effect,
            effect_radius_km: 10.0, // Default radius
            curse_active: true,
            intensity: EffectIntensity::Strong,
            remaining_years: 100, // Default lifespan
        }
    }

    /// Apply annual decay to this remnant.
    /// Returns true if the remnant should be removed.
    pub fn apply_decay(&mut self) -> bool {
        self.remaining_years -= 1;

        // Update intensity based on remaining years
        self.intensity = match self.remaining_years {
            y if y > 75 => EffectIntensity::Strong,
            y if y > 50 => EffectIntensity::Moderate,
            y if y > 25 => EffectIntensity::Weak,
            _ => EffectIntensity::Fading,
        };

        // Curse fades as remnant ages
        if self.remaining_years < 50 {
            self.curse_active = self.remaining_years > 10;
        }

        // Remove when fully decayed
        self.remaining_years <= 0
    }

    /// Check if a position is within this remnant's effect radius.
    pub fn affects_position(&self, position: u32, distance_km: f32) -> bool {
        distance_km <= self.effect_radius_km && self.curse_active
    }

    /// Create a remnant from a destroyed faction asset.
    /// Faction remnants provide faction-specific bonuses when faction assets are destroyed.
    pub fn from_faction_asset(
        asset: &crate::faction::FactionAsset,
        faction_id: Uuid,
        year: i32,
    ) -> Self {
        let (curse_effect, blessing_effect) = match asset.category {
            crate::faction::AssetCategory::Force => (
                "Military forces have abandoned this territory".to_string(),
                "Strategic advantage remains from past campaigns".to_string(),
            ),
            crate::faction::AssetCategory::Cunning => (
                "Subtle influence has faded from this area".to_string(),
                "Wisdom from past dealings lingers here".to_string(),
            ),
            crate::faction::AssetCategory::Wealth => (
                "Economic hardship grips what was once wealthy ground".to_string(),
                "Riches from the past remain buried here".to_string(),
            ),
        };

        Self {
            id: Uuid::new_v4(),
            world_id: faction_id, // Use faction_id as world_id for faction remnants
            beast: PrimalBeast::Pyraxes, // Default, not applicable for faction remnants
            slaying_year: year,
            position: asset.location.unwrap_or(0),
            element: crate::beasts::BeastElement::Fire, // Default
            curse_effect,
            blessing_effect,
            effect_radius_km: 5.0, // Smaller radius for faction remnants
            curse_active: true,
            intensity: EffectIntensity::Strong,
            remaining_years: 50, // Shorter lifespan for faction remnants
        }
    }
}

/// Record of a beast being slain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastSlainEvent {
    /// Unique ID for this event
    pub id: Uuid,
    /// World ID
    pub world_id: Uuid,
    /// Beast that was slain
    pub beast: PrimalBeast,
    /// Year of slaying
    pub year: i32,
    /// Position where slain
    pub position: u32,
    /// Factions that participated in the slaying
    pub participating_factions: Vec<Uuid>,
    /// The remnant artifact created
    pub remnant_id: Uuid,
}

impl BeastSlainEvent {
    /// Create a new beast slain event.
    pub fn new(
        world_id: Uuid,
        beast: PrimalBeast,
        year: i32,
        position: u32,
        participating_factions: Vec<Uuid>,
        remnant_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            world_id,
            beast,
            year,
            position,
            participating_factions,
            remnant_id,
        }
    }
}

/// System managing all remnants in a world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemnantSystem {
    /// All active remnants keyed by ID
    remnants: HashMap<Uuid, RemnantArtifact>,
    /// History of beast slaying events
    events: Vec<BeastSlainEvent>,
    /// Index of remnants by position for fast lookup
    position_index: HashMap<u32, Vec<Uuid>>,
}

impl RemnantSystem {
    /// Create a new empty remnant system.
    pub fn new() -> Self {
        Self {
            remnants: HashMap::new(),
            events: Vec::new(),
            position_index: HashMap::new(),
        }
    }

    /// Add a new remnant to the system.
    pub fn add_remnant(&mut self, remnant: RemnantArtifact) -> Uuid {
        let id = remnant.id;
        let position = remnant.position;

        self.remnants.insert(id, remnant);

        // Update position index
        self.position_index
            .entry(position)
            .or_insert_with(Vec::new)
            .push(id);

        id
    }

    /// Remove a remnant from the system.
    pub fn remove_remnant(&mut self, id: Uuid) -> Option<RemnantArtifact> {
        if let Some(remnant) = self.remnants.remove(&id) {
            // Remove from position index
            if let Some(ids) = self.position_index.get_mut(&remnant.position) {
                ids.retain(|&i| i != id);
            }
            Some(remnant)
        } else {
            None
        }
    }

    /// Get a remnant by ID.
    pub fn get_remnant(&self, id: Uuid) -> Option<&RemnantArtifact> {
        self.remnants.get(&id)
    }

    /// Get a mutable remnant by ID.
    pub fn get_remnant_mut(&mut self, id: Uuid) -> Option<&mut RemnantArtifact> {
        self.remnants.get_mut(&id)
    }

    /// Get all remnants.
    pub fn get_all_remnants(&self) -> Vec<&RemnantArtifact> {
        self.remnants.values().collect()
    }

    /// Get all remnants as mutable.
    pub fn get_all_remnants_mut(&mut self) -> Vec<&mut RemnantArtifact> {
        self.remnants.values_mut().collect()
    }

    /// Get remnants at a specific position.
    pub fn get_remnants_at_position(&self, position: u32) -> Vec<&RemnantArtifact> {
        self.position_index
            .get(&position)
            .map(|ids| ids.iter().filter_map(|id| self.remnants.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get remnants within a radius of a position (approximate).
    /// This is a simplified check - assumes position correlates to distance.
    pub fn get_remnants_in_radius(&self, position: u32, radius: u32) -> Vec<&RemnantArtifact> {
        self.remnants
            .values()
            .filter(|r| {
                // Simple distance check - assumes contiguous positions
                let distance = ((r.position as i32 - position as i32).abs()) as u32;
                distance <= radius && r.curse_active
            })
            .collect()
    }

    /// Add a beast slain event to history.
    pub fn record_beast_slain(&mut self, event: BeastSlainEvent) {
        self.events.push(event);
    }

    /// Get all beast slain events.
    pub fn get_slain_events(&self) -> &Vec<BeastSlainEvent> {
        &self.events
    }

    /// Apply annual decay to all remnants.
    /// Removes fully decayed remnants.
    /// Returns count of removed remnants.
    pub fn apply_annual_decay(&mut self, _year: i32) -> usize {
        let mut removed_count = 0;

        // Collect IDs to remove (can't remove while iterating)
        let to_remove: Vec<Uuid> = self
            .remnants
            .values_mut()
            .filter_map(|r| if r.apply_decay() { Some(r.id) } else { None })
            .collect();

        // Remove decayed remnants
        for id in &to_remove {
            if let Some(remnant) = self.remnants.remove(id) {
                if let Some(ids) = self.position_index.get_mut(&remnant.position) {
                    ids.retain(|&i| i != *id);
                }
                removed_count += 1;
            }
        }

        removed_count
    }

    /// Get count of active remnants.
    pub fn remnant_count(&self) -> usize {
        self.remnants.len()
    }

    /// Get count of active curses.
    pub fn active_curse_count(&self) -> usize {
        self.remnants.values().filter(|r| r.curse_active).count()
    }

    /// Check if any remnant affects a given position.
    pub fn is_affected_by_remnant(&self, position: u32) -> bool {
        self.remnants
            .values()
            .any(|r| r.position == position && r.curse_active)
    }

    /// Get remnants by element type.
    pub fn get_remnants_by_element(&self, element: BeastElement) -> Vec<&RemnantArtifact> {
        self.remnants
            .values()
            .filter(|r| r.element == element)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_remnant(beast: PrimalBeast, position: u32) -> RemnantArtifact {
        let profile = super::super::profiles::get_beast_profile(beast);
        RemnantArtifact::new(
            Uuid::new_v4(),
            beast,
            1000,
            position,
            profile.curse.clone(),
            profile.blessing.clone(),
        )
    }

    #[test]
    fn test_remnant_creation() {
        let world_id = Uuid::new_v4();
        let remnant = create_test_remnant(PrimalBeast::Pyraxes, 42);

        assert_eq!(remnant.beast, PrimalBeast::Pyraxes);
        assert_eq!(remnant.element, BeastElement::Fire);
        assert_eq!(remnant.position, 42);
        assert!(remnant.curse_active);
        assert_eq!(remnant.intensity, EffectIntensity::Strong);
        assert_eq!(remnant.remaining_years, 100);
    }

    #[test]
    fn test_remnant_decay() {
        let mut remnant = create_test_remnant(PrimalBeast::Tidarth, 100);

        // Initial state
        assert_eq!(remnant.intensity, EffectIntensity::Strong);
        assert_eq!(remnant.remaining_years, 100);

        // Apply decay many times
        for _ in 0..30 {
            let should_remove = remnant.apply_decay();
            assert!(!should_remove);
        }

        // Should be in Moderate state now (50-75 years remain)
        assert_eq!(remnant.intensity, EffectIntensity::Moderate);

        // Apply more decay to reach Weak (25-50 years remain)
        for _ in 0..25 {
            let should_remove = remnant.apply_decay();
            if remnant.remaining_years <= 0 {
                break;
            }
        }

        // Should be in Weak state now (25-50 years remain)
        assert_eq!(remnant.intensity, EffectIntensity::Weak);

        // Apply more decay to fully deplete years
        while remnant.remaining_years > 0 {
            remnant.apply_decay();
        }

        // Should be fully decayed now
        assert!(remnant.remaining_years <= 0);
    }

    #[test]
    fn test_remnant_system_add_remove() {
        let mut system = RemnantSystem::new();

        let remnant = create_test_remnant(PrimalBeast::Terros, 50);
        let id = remnant.id;

        // Add remnant
        system.add_remnant(remnant);
        assert_eq!(system.remnant_count(), 1);

        // Get back
        let retrieved = system.get_remnant(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().beast, PrimalBeast::Terros);

        // Remove
        let removed = system.remove_remnant(id);
        assert!(removed.is_some());
        assert_eq!(system.remnant_count(), 0);
    }

    #[test]
    fn test_remnant_system_decay() {
        let mut system = RemnantSystem::new();

        // Add multiple remnants
        for i in 0..5 {
            let remnant = create_test_remnant(PrimalBeast::Lumina, i * 10);
            system.add_remnant(remnant);
        }

        assert_eq!(system.remnant_count(), 5);

        // Apply decay multiple times to fully decay some
        for _ in 0..101 {
            system.apply_annual_decay(1000);
        }

        // All should be removed after 100 years
        assert_eq!(system.remnant_count(), 0);
    }

    #[test]
    fn test_beast_slain_event() {
        let world_id = Uuid::new_v4();
        let faction_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        let event = BeastSlainEvent::new(
            world_id,
            PrimalBeast::Pyraxes,
            1200,
            42,
            faction_ids.clone(),
            Uuid::new_v4(),
        );

        assert_eq!(event.beast, PrimalBeast::Pyraxes);
        assert_eq!(event.year, 1200);
        assert_eq!(event.participating_factions, faction_ids);
    }

    #[test]
    fn test_remnants_by_element() {
        let mut system = RemnantSystem::new();

        system.add_remnant(create_test_remnant(PrimalBeast::Pyraxes, 10)); // Fire
        system.add_remnant(create_test_remnant(PrimalBeast::Tidarth, 20)); // Water
        system.add_remnant(create_test_remnant(PrimalBeast::Terros, 30)); // Earth
        system.add_remnant(create_test_remnant(PrimalBeast::Pyraxes, 40)); // Fire

        let fire_remnants = system.get_remnants_by_element(BeastElement::Fire);
        assert_eq!(fire_remnants.len(), 2);

        let water_remnants = system.get_remnants_by_element(BeastElement::Water);
        assert_eq!(water_remnants.len(), 1);
    }

    #[test]
    fn test_affects_position() {
        let mut remnant = create_test_remnant(PrimalBeast::Lumina, 50);
        remnant.effect_radius_km = 10.0;

        // Position in radius
        assert!(remnant.affects_position(50, 5.0));
        assert!(remnant.affects_position(50, 10.0));

        // Position outside radius
        assert!(!remnant.affects_position(50, 15.0));

        // Deactivate curse
        remnant.curse_active = false;
        assert!(!remnant.affects_position(50, 5.0));
    }
}
