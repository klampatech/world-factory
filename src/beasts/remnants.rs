//! Remnant Artifact System
//!
//! When a primal beast is slain, it drops a physical piece of itself — its **Remnant**.
//! Remnants are artifacts of immense power containing the beast's residual essence.
//!
//! ## Remnant Properties
//!
//! - **Environmental Effect**: Continues to exert a weaker version of the beast's 
//!   environmental effects in its immediate vicinity
//! - **Crafting Material**: Can be used in crafting, construction, or rituals to 
//!   produce world-quality elemental goods
//! - **Curse Carrier**: The curse from slaying is carried in the Remnant, not the slayer
//! - **Decay**: Has a decay rate (100-500 years depending on world age)
//! - **Indestructible**: Cannot be destroyed — only re-used or sealed
//!
//! ## Beast Remnants
//!
//! | Beast | Remnant | Effect Radius | Crafting Use |
//! |-------|---------|---------------|--------------|
//! | Pyraxes | Heartstone | 10km heat radius | Forge core for fire-elemental gear |
//! | Tidarth | Storm Eye | 3m calm sphere | Hurricane generation, storm-resistant buildings |
//! | Terros | Primordial Core | Structural integrity | Earthquake-proof underground vaults |
//! | Lumina | Life Pearl | 50km ocean health | Purify water, restore fish populations |

use super::{BeastElement, PrimalBeast, PrimalBeastInstance, BeastState, profiles::get_beast_profile};
use crate::artifacts::{
    Artifact, ArtifactCategory, ArtifactCondition, ArtifactProperty, 
    ArtifactPropertyType, ArtifactRarity, EffectScope, 
};
use crate::types::{EntityId, EntityType};
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Remnant artifact created from a slain primal beast.
/// Contains the beast's residual essence and continues to exert environmental effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemnantArtifact {
    /// The underlying artifact
    pub artifact: Artifact,
    /// The beast this remnant came from
    pub source_beast: PrimalBeast,
    /// Element of the source beast
    pub element: BeastElement,
    /// Radius of environmental effect in kilometers
    pub effect_radius_km: f32,
    /// Original power level (1.0 = full power)
    pub original_power: f32,
    /// Current power level (decays over time)
    pub current_power: f32,
    /// Year when the beast was slain
    pub slaying_year: i32,
    /// Estimated decay completion year (when power reaches 0)
    pub decay_completion_year: i32,
    /// Whether the curse is active (transferred to possessor)
    pub curse_active: bool,
    /// Year when curse was last triggered
    pub last_curse_year: Option<i32>,
    /// Curse effect description
    pub curse_effect: String,
    /// Blessing effect description  
    pub blessing_effect: String,
    /// Crafting bonus multiplier
    pub crafting_bonus: f32,
}

impl RemnantArtifact {
    /// Create a new Remnant artifact from a slain beast.
    pub fn from_beast_slaying(
        world_id: Uuid,
        beast: PrimalBeast,
        slaying_year: i32,
        position: u32,
    ) -> Self {
        let profile = get_beast_profile(beast);
        
        // Remnants are always Mythic rarity due to their world-altering power
        let significance = 0.97;
        
        // Build the artifact
        let mut artifact = Artifact::new(
            world_id,
            format!("{}'s {}", beast.name(), profile.remnant),
            ArtifactCategory::Magical,
            slaying_year,
            format!(
                "The Remnant of {} - {} - containing the residual essence of the slain primal beast. \
                {} This artifact exerts {} environmental effects and {} used in elemental crafting. \
                Possession carries the beast's curse.",
                beast.name(),
                profile.remnant,
                Self::get_effect_description(beast),
                Self::get_crafting_description(beast),
                Self::get_curse_description(beast),
            ),
            significance,
        );
        
        // Set origin and location
        artifact.origin_event_id = Some(Uuid::new_v4()); // TODO: Link to actual slaying event
        artifact.current_location_id = Some(Uuid::from_u128(position as u128));
        artifact.condition = ArtifactCondition::Pristine;
        artifact.rarity = ArtifactRarity::Mythic;
        
        // Add artifact properties
        let mut properties = Vec::new();
        
        // Beast essence property
        properties.push(ArtifactProperty {
            name: format!("{} Essence", profile.element.name()),
            description: format!("Contains residual {} essence from {}", 
                profile.element.name(), beast.name()),
            property_type: ArtifactPropertyType::Magical,
        });
        
        // Blessing property
        properties.push(ArtifactProperty {
            name: "Beast's Blessing".to_string(),
            description: format!("Grants: {}", profile.blessing),
            property_type: ArtifactPropertyType::Blessed,
        });
        
        // Curse property (always present)
        properties.push(ArtifactProperty {
            name: "Beast's Curse".to_string(),
            description: format!("Curses the possessor: {}", profile.curse),
            property_type: ArtifactPropertyType::Cursed,
        });
        
        // Environmental effect property
        properties.push(ArtifactProperty {
            name: "Environmental Influence".to_string(),
            description: format!("Affects {} area within {}km radius", 
                profile.element.name().to_lowercase(), 
                Self::get_effect_radius(beast)),
            property_type: ArtifactPropertyType::Magical,
        });
        
        // Elemental alignment property
        properties.push(ArtifactProperty {
            name: format!("{} Alignment", profile.element.name()),
            description: "Aligned with primal elemental forces".to_string(),
            property_type: ArtifactPropertyType::Magical,
        });
        
        artifact.properties = Some(properties);
        
        // Calculate decay period (100-500 years depending on beast type)
        let decay_years = Self::calculate_decay_years(beast);
        let decay_completion_year = slaying_year + decay_years;
        
        RemnantArtifact {
            artifact,
            source_beast: beast,
            element: profile.element,
            effect_radius_km: Self::get_effect_radius(beast),
            original_power: 1.0,
            current_power: 1.0,
            slaying_year,
            decay_completion_year,
            curse_active: true,
            last_curse_year: None,
            curse_effect: profile.curse.clone(),
            blessing_effect: profile.blessing.clone(),
            crafting_bonus: Self::get_crafting_bonus(beast),
        }
    }
    
    /// Get the effect radius in km for each beast type.
    fn get_effect_radius(beast: PrimalBeast) -> f32 {
        match beast {
            PrimalBeast::Pyraxes => 10.0,   // Heat radius
            PrimalBeast::Tidarth => 0.003, // 3m calm sphere (converted to km)
            PrimalBeast::Terros => 0.5,    // Structural integrity zone
            PrimalBeast::Lumina => 50.0,   // Ocean health radius
        }
    }
    
    /// Calculate decay period in years based on beast type.
    /// Per spec: 100-500 years depending on world age/beast.
    fn calculate_decay_years(beast: PrimalBeast) -> i32 {
        match beast {
            PrimalBeast::Pyraxes => 500, // Fire burns longest
            PrimalBeast::Tidarth => 400, // Storms persist
            PrimalBeast::Terros => 300,  // Stone decays slowly
            PrimalBeast::Lumina => 100,  // Life essence fades fastest
        }
    }
    
    /// Get crafting bonus multiplier.
    fn get_crafting_bonus(beast: PrimalBeast) -> f32 {
        match beast {
            PrimalBeast::Pyraxes => 1.0,  // Fire elemental gear
            PrimalBeast::Tidarth => 0.9, // Storm-resistant construction
            PrimalBeast::Terros => 0.95,  // Earthquake-proof vaults
            PrimalBeast::Lumina => 1.0,  // Water purification, fish restoration
        }
    }
    
    /// Get effect description for artifact description.
    fn get_effect_description(beast: PrimalBeast) -> &'static str {
        match beast {
            PrimalBeast::Pyraxes => "Radiates intense heat within a 10km radius,",
            PrimalBeast::Tidarth => "Creates a perfectly calm sphere of air 3m in diameter,",
            PrimalBeast::Terros => "Provides immense structural integrity to nearby structures,",
            PrimalBeast::Lumina => "Maintains ocean health within a 50km radius,",
        }
    }
    
    /// Get crafting description for artifact description.
    fn get_crafting_description(beast: PrimalBeast) -> &'static str {
        match beast {
            PrimalBeast::Pyraxes => "can be used as a forge core for fire-elemental weapon/armor crafting with no fuel cost",
            PrimalBeast::Tidarth => "when broken over water, generates a hurricane; in construction, allows buildings to withstand any storm",
            PrimalBeast::Terros => "can be used to create underground vaults that cannot be collapsed by any earthquake",
            PrimalBeast::Lumina => "can purify any freshwater source and restore fish populations; if consumed, grants water breathing",
        }
    }
    
    /// Get curse description.
    fn get_curse_description(beast: PrimalBeast) -> &'static str {
        match beast {
            PrimalBeast::Pyraxes => "Scorched earth and endless wildfires affect the possessor's territory",
            PrimalBeast::Tidarth => "Tsunamis and endless storms target the possessor's coastal regions",
            PrimalBeast::Terros => "Avalanches and volcanic eruptions plague the possessor's mountain holdings",
            PrimalBeast::Lumina => "Pestilence and blighted crops affect the possessor's agricultural output",
        }
    }
    
    /// Update power level based on current year (apply decay).
    /// Returns the new power level.
    pub fn apply_decay(&mut self, current_year: i32) -> f32 {
        if current_year >= self.decay_completion_year {
            // Power has fully decayed (but remnant persists)
            self.current_power = 0.01; // Minimum 1% power (remnants are indestructible)
        } else {
            let years_since_slaying = current_year - self.slaying_year;
            let total_decay_years = self.decay_completion_year - self.slaying_year;
            let decay_ratio = years_since_slaying as f32 / total_decay_years as f32;
            // Power decays from 1.0 to 0.01 (never reaches 0)
            self.current_power = 1.0 - (decay_ratio * 0.99);
        }
        self.current_power
    }
    
    /// Get the current environmental effect strength based on power level.
    pub fn get_effect_strength(&self) -> f32 {
        // Effect strength scales with current power
        self.current_power * 0.25 // Max 25% of original beast's effect
    }
    
    /// Transfer the curse to a new owner.
    pub fn transfer_curse(&mut self, new_owner_id: Uuid, year: i32) {
        self.artifact.owner_id = Some(new_owner_id);
        self.curse_active = true;
        self.last_curse_year = Some(year);
    }
    
    /// Seal the remnant (disables curse but also disables blessing).
    pub fn seal(&mut self) {
        self.curse_active = false;
        self.artifact.condition = ArtifactCondition::Ruined;
    }
    
    /// Reactivate a sealed remnant.
    pub fn unseal(&mut self) {
        self.curse_active = true;
        self.artifact.condition = ArtifactCondition::Worn;
    }
    
    /// Check if position is within effect radius.
    pub fn affects_position(&self, distance_km: f32) -> bool {
        distance_km <= self.effect_radius_km
    }
    
    /// Get the environmental modifier for a given element at a position.
    /// Returns modifier values for terrain generation.
    pub fn get_terrain_modifier(&self, target_element: BeastElement) -> f32 {
        if target_element != self.element {
            return 0.0;
        }
        self.get_effect_strength()
    }
    
    /// Get the current blessing bonus.
    pub fn get_blessing_bonus(&self) -> f32 {
        if self.curse_active {
            // Curse negates blessing if active
            0.0
        } else {
            self.get_effect_strength() * 0.5
        }
    }
    
    /// Get the current curse penalty.
    pub fn get_curse_penalty(&self) -> f32 {
        if self.curse_active {
            self.get_effect_strength() * 0.5
        } else {
            0.0
        }
    }
}

/// Event fired when a beast is slain and a Remnant is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastSlainEvent {
    /// Event ID
    pub id: Uuid,
    /// The beast that was slain
    pub beast: PrimalBeast,
    /// Year of slaying
    pub year: i32,
    /// Position where beast was slain
    pub position: u32,
    /// The created remnant artifact
    pub remnant: RemnantArtifact,
    /// Factions that participated in the slaying
    pub participating_factions: Vec<Uuid>,
    /// Curse transferred to factions
    pub curse_transferred: bool,
}

impl BeastSlainEvent {
    /// Create a new beast slaying event.
    pub fn new(
        beast: PrimalBeast,
        year: i32,
        position: u32,
        remnant: RemnantArtifact,
        participating_factions: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            beast,
            year,
            position,
            remnant,
            participating_factions,
            curse_transferred: true,
        }
    }
}

/// System for managing Remnant artifacts across the world.
/// Tracks all Remnants and applies their effects during simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemnantSystem {
    /// All Remnant artifacts in the world
    remnants: Vec<RemnantArtifact>,
    /// History of beast slayings
    slain_events: Vec<BeastSlainEvent>,
}

impl RemnantSystem {
    /// Create a new empty Remnant system.
    pub fn new() -> Self {
        Self {
            remnants: Vec::new(),
            slain_events: Vec::new(),
        }
    }
    
    /// Add a new Remnant from a beast slaying.
    pub fn add_remnant(&mut self, remnant: RemnantArtifact) {
        self.remnants.push(remnant);
    }
    
    /// Record a beast slaying event.
    pub fn record_slaying(&mut self, event: BeastSlainEvent) {
        self.slain_events.push(event.clone());
        self.add_remnant(event.remnant);
    }
    
    /// Number of Remnants.
    pub fn len(&self) -> usize {
        self.remnants.len()
    }
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.remnants.is_empty()
    }
    
    /// Get all Remnants.
    pub fn remnants(&self) -> &[RemnantArtifact] {
        &self.remnants
    }
    
    /// Get Remnant by source beast type.
    pub fn get_by_beast(&self, beast: PrimalBeast) -> Option<&RemnantArtifact> {
        self.remnants.iter().find(|r| r.source_beast == beast)
    }
    
    /// Get Remnant by source beast type (mutable).
    pub fn get_by_beast_mut(&mut self, beast: PrimalBeast) -> Option<&mut RemnantArtifact> {
        self.remnants.iter_mut().find(|r| r.source_beast == beast)
    }
    
    /// Get all slain events.
    pub fn slain_events(&self) -> &[BeastSlainEvent] {
        &self.slain_events
    }
    
    /// Apply decay to all Remnants for the current year.
    pub fn apply_annual_decay(&mut self, year: i32) {
        for remnant in &mut self.remnants {
            remnant.apply_decay(year);
        }
    }
    
    /// Get Remnants affecting a position.
    pub fn get_affecting_remnants(&self, position: u32, distance_km: f32) -> Vec<&RemnantArtifact> {
        self.remnants
            .iter()
            .filter(|r| r.affects_position(distance_km))
            .collect()
    }
    
    /// Get combined terrain modifier from all Remnants.
    pub fn get_combined_terrain_modifier(&self, element: BeastElement) -> f32 {
        self.remnants
            .iter()
            .map(|r| r.get_terrain_modifier(element))
            .sum()
    }
    
    /// Get Remnants with active curses.
    pub fn get_cursed_remnants(&self) -> Vec<&RemnantArtifact> {
        self.remnants
            .iter()
            .filter(|r| r.curse_active)
            .collect()
    }
    
    /// Get Remnants owned by a specific faction.
    pub fn get_owned_by(&self, owner_id: Uuid) -> Vec<&RemnantArtifact> {
        self.remnants
            .iter()
            .filter(|r| r.artifact.owner_id == Some(owner_id))
            .collect()
    }
    
    /// Transfer a Remnant to a new owner.
    pub fn transfer_remnant(&mut self, remnant_id: &Uuid, new_owner_id: Uuid, year: i32) -> bool {
        if let Some(remnant) = self.remnants.iter_mut().find(|r| r.artifact.id.to_uuid() == *remnant_id) {
            remnant.transfer_curse(new_owner_id, year);
            true
        } else {
            false
        }
    }
    
    /// Seal a Remnant.
    pub fn seal_remnant(&mut self, remnant_id: &Uuid) -> bool {
        if let Some(remnant) = self.remnants.iter_mut().find(|r| r.artifact.id.to_uuid() == *remnant_id) {
            remnant.seal();
            true
        } else {
            false
        }
    }
    
    /// Unseal a Remnant.
    pub fn unseal_remnant(&mut self, remnant_id: &Uuid) -> bool {
        if let Some(remnant) = self.remnants.iter_mut().find(|r| r.artifact.id.to_uuid() == *remnant_id) {
            remnant.unseal();
            true
        } else {
            false
        }
    }
    
    /// Get total environmental effect strength for an element.
    pub fn total_effect_strength(&self, element: BeastElement) -> f32 {
        self.remnants
            .iter()
            .filter(|r| r.element == element)
            .map(|r| r.get_effect_strength())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remnant_creation() {
        let world_id = Uuid::new_v4();
        let remnant = RemnantArtifact::from_beast_slaying(
            world_id,
            PrimalBeast::Pyraxes,
            1000,
            42,
        );
        
        assert_eq!(remnant.source_beast, PrimalBeast::Pyraxes);
        assert_eq!(remnant.element, BeastElement::Fire);
        assert_eq!(remnant.effect_radius_km, 10.0);
        assert_eq!(remnant.current_power, 1.0);
        assert!(remnant.curse_active);
        assert!(remnant.artifact.rarity == ArtifactRarity::Mythic);
    }
    
    #[test]
    fn test_remnant_decay() {
        let world_id = Uuid::new_v4();
        let mut remnant = RemnantArtifact::from_beast_slaying(
            world_id,
            PrimalBeast::Pyraxes,
            1000,
            42,
        );
        
        // Pyraxes has 500 year decay period
        assert_eq!(remnant.decay_completion_year, 1500);
        
        // At year 1000, power is 1.0
        remnant.apply_decay(1000);
        assert!((remnant.current_power - 1.0).abs() < 0.001);
        
        // At year 1250 (halfway), power should be ~0.5
        remnant.apply_decay(1250);
        assert!(remnant.current_power < 0.6 && remnant.current_power > 0.4);
        
        // At year 1500 (completion), power should be minimum 0.01
        remnant.apply_decay(1500);
        assert_eq!(remnant.current_power, 0.01);
    }
    
    #[test]
    fn test_remnant_curse_transfer() {
        let world_id = Uuid::new_v4();
        let mut remnant = RemnantArtifact::from_beast_slaying(
            world_id,
            PrimalBeast::Lumina,
            1000,
            42,
        );
        
        let new_owner = Uuid::new_v4();
        remnant.transfer_curse(new_owner, 1050);
        
        assert_eq!(remnant.artifact.owner_id, Some(new_owner));
        assert!(remnant.curse_active);
        assert_eq!(remnant.last_curse_year, Some(1050));
    }
    
    #[test]
    fn test_remnant_seal_unseal() {
        let world_id = Uuid::new_v4();
        let mut remnant = RemnantArtifact::from_beast_slaying(
            world_id,
            PrimalBeast::Terros,
            1000,
            42,
        );
        
        // Seal disables curse
        remnant.seal();
        assert!(!remnant.curse_active);
        assert_eq!(remnant.artifact.condition, ArtifactCondition::Ruined);
        
        // Unseal re-enables curse
        remnant.unseal();
        assert!(remnant.curse_active);
        assert_eq!(remnant.artifact.condition, ArtifactCondition::Worn);
    }
    
    #[test]
    fn test_remnant_system() {
        let mut system = RemnantSystem::new();
        
        // Add Remnants for different beasts
        let pyraxes_remnant = RemnantArtifact::from_beast_slaying(
            Uuid::new_v4(),
            PrimalBeast::Pyraxes,
            1000,
            1,
        );
        
        let tidarth_remnant = RemnantArtifact::from_beast_slaying(
            Uuid::new_v4(),
            PrimalBeast::Tidarth,
            1200,
            2,
        );
        
        system.add_remnant(pyraxes_remnant);
        system.add_remnant(tidarth_remnant);
        
        assert_eq!(system.len(), 2);
        
        // Check we can find by beast type
        assert!(system.get_by_beast(PrimalBeast::Pyraxes).is_some());
        assert!(system.get_by_beast(PrimalBeast::Terros).is_none());
        
        // Check combined effect strength
        let fire_effect = system.total_effect_strength(BeastElement::Fire);
        let water_effect = system.total_effect_strength(BeastElement::Water);
        let earth_effect = system.total_effect_strength(BeastElement::Earth);
        
        assert!(fire_effect > 0.0);
        assert!(water_effect > 0.0);
        assert_eq!(earth_effect, 0.0);
    }
    
    #[test]
    fn test_all_beast_remnants() {
        for beast in PrimalBeast::all() {
            let world_id = Uuid::new_v4();
            let remnant = RemnantArtifact::from_beast_slaying(
                world_id,
                beast,
                1000,
                1,
            );
            
            assert_eq!(remnant.source_beast, beast);
            assert!(remnant.effect_radius_km > 0.0);
            assert!(remnant.current_power > 0.0);
            assert!(!remnant.curse_effect.is_empty());
            assert!(!remnant.blessing_effect.is_empty());
        }
    }
}
