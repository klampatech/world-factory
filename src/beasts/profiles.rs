//! Beast Profiles - Static data for each primal beast
//!
//! Contains BeastProfile data for Pyraxes, Tidarth, Terros, Lumina.
//! Profiles define form, habitat, territory radius, power growth, and weakness.

use super::{BeastForm, BeastElement, PrimalBeast, BeastState};
use crate::types::EntityId;

/// Profile data for a primal beast.
#[derive(Debug, Clone)]
pub struct BeastProfile {
    /// Beast type
    pub beast: PrimalBeast,
    /// Physical form
    pub form: BeastForm,
    /// Element
    pub element: BeastElement,
    /// Preferred biome/habitat
    pub habitat: String,
    /// Territory radius in km (50-200)
    pub territory_radius: f32,
    /// Annual power growth rate
    pub power_growth_rate: f32,
    /// Elemental weakness (opposing element)
    pub weakness: BeastElement,
    /// Base terrain modifier strength
    pub terrain_modifier_strength: f32,
    /// Blessing this beast grants
    pub blessing: String,
    /// Curse when angered
    pub curse: String,
    /// Remnant item dropped on death
    pub remnant: String,
}

/// Get the profile for a primal beast.
pub fn get_beast_profile(beast: PrimalBeast) -> BeastProfile {
    match beast {
        PrimalBeast::Pyraxes => pyraxes_profile(),
        PrimalBeast::Tidarth => tidarth_profile(),
        PrimalBeast::Terros => terros_profile(),
        PrimalBeast::Lumina => lumina_profile(),
    }
}

/// Pyraxes - The Flame Wyrm (Fire element)
fn pyraxes_profile() -> BeastProfile {
    BeastProfile {
        beast: PrimalBeast::Pyraxes,
        form: BeastForm::Dragon,
        element: BeastElement::Fire,
        habitat: "volcanic".to_string(),
        territory_radius: 100.0,
        power_growth_rate: 0.15,
        weakness: BeastElement::Water,
        terrain_modifier_strength: 0.8,
        blessing: "Fertile volcanic soil and abundant metal ores".to_string(),
        curse: "Scorched earth and endless wildfires".to_string(),
        remnant: "Heartstone".to_string(),
    }
}

/// Tidarth - The Storm Serpent (Water element)
fn tidarth_profile() -> BeastProfile {
    BeastProfile {
        beast: PrimalBeast::Tidarth,
        form: BeastForm::Serpent,
        element: BeastElement::Water,
        habitat: "coastal".to_string(),
        territory_radius: 150.0,
        power_growth_rate: 0.12,
        weakness: BeastElement::Fire,
        terrain_modifier_strength: 0.7,
        blessing: "Bountiful fisheries and fair weather".to_string(),
        curse: "Tsunamis and endless storms".to_string(),
        remnant: "Storm Eye".to_string(),
    }
}

/// Terros - The Mountain Titan (Earth element)
fn terros_profile() -> BeastProfile {
    BeastProfile {
        beast: PrimalBeast::Terros,
        form: BeastForm::Golem,
        element: BeastElement::Earth,
        habitat: "mountain".to_string(),
        territory_radius: 200.0,
        power_growth_rate: 0.08,
        weakness: BeastElement::Life,
        terrain_modifier_strength: 0.9,
        blessing: "Rich mineral veins and earthquake resistance".to_string(),
        curse: "Avalanches and volcanic eruptions".to_string(),
        remnant: "Primordial Core".to_string(),
    }
}

/// Lumina - The Life Wing (Life element)
fn lumina_profile() -> BeastProfile {
    BeastProfile {
        beast: PrimalBeast::Lumina,
        form: BeastForm::Spirit,
        element: BeastElement::Life,
        habitat: "forest".to_string(),
        territory_radius: 80.0,
        power_growth_rate: 0.18,
        weakness: BeastElement::Earth,
        terrain_modifier_strength: 0.6,
        blessing: "Abundant wildlife and healing waters".to_string(),
        curse: "Pestilence and blighted crops".to_string(),
        remnant: "Life Pearl".to_string(),
    }
}

/// Get all beast profiles.
pub fn all_profiles() -> [BeastProfile; 4] {
    [
        pyraxes_profile(),
        tidarth_profile(),
        terros_profile(),
        lumina_profile(),
    ]
}
