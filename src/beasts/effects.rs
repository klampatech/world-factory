//! Beast Effects - Terrain modification from beast presence
//!
//! Each primal beast modifies terrain within its territory.
//! Effects can be blessings (positive) or curses (negative).
//!
//! ## Terrain Effects
//!
//! - Pyraxes: Volcanic activity, fertile volcanic soil
//! - Tidarth: Coastal erosion, abundant fisheries
//! - Terros: Mountain formation, mineral veins
//! - Lumina: Forest growth, healing springs

use super::{
    profiles::get_beast_profile, BeastElement, BeastState, PrimalBeast, PrimalBeastInstance,
};
use crate::events::EventBuilder;
use crate::terrain::{BiomeType, TerrainCell};
use crate::types::HistoricalTime;

/// Effect of a beast on a single terrain cell.
#[derive(Debug, Clone)]
pub struct BeastTerrainEffect {
    /// Cell index
    pub cell: u32,
    /// Primary effect type
    pub effect_type: BeastEffectType,
    /// Effect magnitude (0.0-1.0)
    pub magnitude: f32,
}

/// Types of terrain effects from beasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BeastEffectType {
    /// Increase vegetation
    VegetationBonus,
    /// Reduce vegetation
    VegetationPenalty,
    /// Increase rainfall
    RainfallBonus,
    /// Reduce rainfall
    Drought,
    /// Volcanic activity
    VolcanicActivity,
    /// Mineral enrichment
    MineralDeposit,
    /// Coastal modification
    CoastalShift,
    /// Healing waters
    SacredSpring,
    /// Earthquake risk
    SeismicInstability,
}

/// Calculate all terrain effects for a beast's territory.
/// Note: This is a simplified version that works with available terrain data.
pub fn calculate_terrain_effects(_beast: &PrimalBeastInstance) -> Vec<BeastTerrainEffect> {
    // TODO: Implement full terrain effect calculation
    // Requires access to terrain grid through world.state
    Vec::new()
}

/// Fire element effects (Pyraxes).
fn calculate_fire_effect(
    _cell: &TerrainCell,
    cell_idx: u32,
    strength: f32,
) -> Option<BeastTerrainEffect> {
    // Volcanic areas get bonus, forests get penalty
    // Note: BiomeType::VolcanicLandscape used instead of deprecated Volcanic
    if _cell.biome() > 0 {
        Some(BeastTerrainEffect {
            cell: cell_idx,
            effect_type: BeastEffectType::VolcanicActivity,
            magnitude: strength,
        })
    } else {
        None
    }
}

/// Water element effects (Tidarth).
fn calculate_water_effect(
    _cell: &TerrainCell,
    cell_idx: u32,
    strength: f32,
) -> Option<BeastTerrainEffect> {
    Some(BeastTerrainEffect {
        cell: cell_idx,
        effect_type: if _cell.is_water() {
            BeastEffectType::CoastalShift
        } else {
            BeastEffectType::RainfallBonus
        },
        magnitude: strength,
    })
}

/// Earth element effects (Terros).
fn calculate_earth_effect(
    _cell: &TerrainCell,
    cell_idx: u32,
    strength: f32,
) -> Option<BeastTerrainEffect> {
    Some(BeastTerrainEffect {
        cell: cell_idx,
        effect_type: BeastEffectType::MineralDeposit,
        magnitude: strength,
    })
}

/// Life element effects (Lumina).
fn calculate_life_effect(
    _cell: &TerrainCell,
    cell_idx: u32,
    strength: f32,
) -> Option<BeastTerrainEffect> {
    Some(BeastTerrainEffect {
        cell: cell_idx,
        effect_type: BeastEffectType::VegetationBonus,
        magnitude: strength,
    })
}

/// Apply effects to terrain cells.
/// Note: TerrainCell uses bit-packed representation, effects are limited.
pub fn apply_terrain_effects(
    _terrain: &mut crate::terrain::TerrainGrid,
    _effects: &[BeastTerrainEffect],
) {
    // TODO: Implement terrain modification
    // TerrainCell is bit-packed, direct field modification not available
}

/// Generate events from beast effects.
pub fn generate_beast_events(
    effects: &[BeastTerrainEffect],
    year: i32,
    world_id: crate::Uuid,
) -> Vec<crate::events::Event> {
    let mut events = Vec::new();

    for effect in effects {
        if effect.magnitude > 0.5 {
            let time = HistoricalTime::year(year);
            let event = match effect.effect_type {
                BeastEffectType::VolcanicActivity => EventBuilder::new("Pyraxes volcanic activity")
                    .event_type(crate::events::EventType::Volcano)
                    .time(time)
                    .description("Pyraxes stirs volcanic activity")
                    .build(world_id),
                BeastEffectType::SeismicInstability => EventBuilder::new("Terros seismic tremors")
                    .event_type(crate::events::EventType::Earthquake)
                    .time(time)
                    .description("Terros causes seismic tremors")
                    .build(world_id),
                _ => continue,
            };

            events.push(event);
        }
    }

    events
}
