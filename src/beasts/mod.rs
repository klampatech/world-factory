//! Primal Beasts Module
//!
//! Implements the four primal beasts: Pyraxes, Tidarth, Terros, Lumina.
//! These are legendary creatures that shape terrain, grant blessings/curses,
//! and can be slain through faction cooperation with legendary artifacts.
//!
//! ## Primal Beasts
//!
//! - **Pyraxes** (Fire) - Volcanic dragon of flame
//! - **Tidarth** (Water) - Great serpent of storms  
//! - **Terros** (Earth) - Ancient golem of mountains
//! - **Lumina** (Life) - Winged spirit of renewal
//!
//! ## Systems
//!
//! - BeastProfile: Static data per beast
//! - BeastMovement: Elemental gradient seeking
//! - BeastEffect: Terrain modification from presence
//! - BeastSlaying: Multi-faction cooperation requirements
//! - BeastDeath: Remnant drops + curse transfer

pub mod effects;
pub mod movement;
pub mod profiles;
pub mod remnants;
pub mod slaying;

// Re-export slaying types
pub use slaying::{
    BeastSlayingRequirements, BeastSlayingResult, SlayingAttemptError, SlayingParticipant,
};
// Re-export remnants types
pub use remnants::{BeastSlainEvent, EffectIntensity, RemnantArtifact, RemnantSystem};

use crate::types::EntityId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Element type for primal beasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeastElement {
    /// Fire element - Pyraxes
    Fire,
    /// Water element - Tidarth
    Water,
    /// Earth element - Terros
    Earth,
    /// Life element - Lumina
    Life,
}

impl BeastElement {
    /// Get the element name.
    pub fn name(&self) -> &'static str {
        match self {
            BeastElement::Fire => "Fire",
            BeastElement::Water => "Water",
            BeastElement::Earth => "Earth",
            BeastElement::Life => "Life",
        }
    }

    /// Get the opposing element (for weakness targeting).
    pub fn opposing(&self) -> Self {
        match self {
            BeastElement::Fire => BeastElement::Water,
            BeastElement::Water => BeastElement::Fire,
            BeastElement::Earth => BeastElement::Life,
            BeastElement::Life => BeastElement::Earth,
        }
    }
}

/// Physical form of a primal beast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeastForm {
    /// Dragon-like form
    Dragon,
    /// Serpentine form
    Serpent,
    /// Golem/stone form
    Golem,
    /// Spirit/wisp form
    Spirit,
}

/// The four named primal beasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalBeast {
    /// Pyraxes - The Flame Wyrm
    Pyraxes,
    /// Tidarth - The Storm Serpent
    Tidarth,
    /// Terros - The Mountain Titan
    Terros,
    /// Lumina - The Life Wing
    Lumina,
}

impl PrimalBeast {
    /// Get the element of this beast.
    pub fn element(&self) -> BeastElement {
        match self {
            PrimalBeast::Pyraxes => BeastElement::Fire,
            PrimalBeast::Tidarth => BeastElement::Water,
            PrimalBeast::Terros => BeastElement::Earth,
            PrimalBeast::Lumina => BeastElement::Life,
        }
    }

    /// Get the beast's form.
    pub fn form(&self) -> BeastForm {
        match self {
            PrimalBeast::Pyraxes => BeastForm::Dragon,
            PrimalBeast::Tidarth => BeastForm::Serpent,
            PrimalBeast::Terros => BeastForm::Golem,
            PrimalBeast::Lumina => BeastForm::Spirit,
        }
    }

    /// Get the beast's name.
    pub fn name(&self) -> &'static str {
        match self {
            PrimalBeast::Pyraxes => "Pyraxes",
            PrimalBeast::Tidarth => "Tidarth",
            PrimalBeast::Terros => "Terros",
            PrimalBeast::Lumina => "Lumina",
        }
    }

    /// Get the beast's title/epithet.
    pub fn title(&self) -> &'static str {
        match self {
            PrimalBeast::Pyraxes => "The Flame Wyrm",
            PrimalBeast::Tidarth => "The Storm Serpent",
            PrimalBeast::Terros => "The Mountain Titan",
            PrimalBeast::Lumina => "The Life Wing",
        }
    }

    /// Get all primal beasts.
    pub fn all() -> [PrimalBeast; 4] {
        [
            PrimalBeast::Pyraxes,
            PrimalBeast::Tidarth,
            PrimalBeast::Terros,
            PrimalBeast::Lumina,
        ]
    }
}

/// Current state of a primal beast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeastState {
    /// Beast is active and affecting the world
    Active,
    /// Beast is dormant/sleeping
    Dormant,
    /// Beast is weakened (near death)
    Weakened,
    /// Beast has been slain
    Slain,
}

/// Active primal beast instance in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalBeastInstance {
    /// Unique entity ID
    pub id: EntityId,
    /// Which primal beast type
    pub beast: PrimalBeast,
    /// Current state
    pub state: BeastState,
    /// Current position (cell index or location)
    pub position: u32,
    /// Territory center
    pub territory_center: u32,
    /// Territory radius in km
    pub territory_radius: f32,
    /// Current power level (grows over time)
    pub power_level: f32,
    /// Years of existence
    pub age_years: i32,
    /// Last territory expansion year
    pub last_expansion_year: i32,
}

impl PrimalBeastInstance {
    /// Create a new primal beast instance.
    pub fn new(beast: PrimalBeast, position: u32, year: i32) -> Self {
        let profile = profiles::get_beast_profile(beast);
        Self {
            id: EntityId::new(crate::types::EntityType::Event),
            beast,
            state: BeastState::Active,
            position,
            territory_center: position,
            territory_radius: profile.territory_radius,
            power_level: 1.0,
            age_years: 0,
            last_expansion_year: year,
        }
    }

    /// Grow power over time.
    pub fn grow_power(&mut self, years: i32) {
        let profile = profiles::get_beast_profile(self.beast);
        let growth = profile.power_growth_rate * years as f32;
        self.power_level = (self.power_level + growth).min(10.0);
    }

    /// Check if position is within territory.
    pub fn is_in_territory(&self, position: u32, distance_km: f32) -> bool {
        distance_km <= self.territory_radius
    }
}
