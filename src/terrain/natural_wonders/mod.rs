//! Natural Wonders Module for World Factory
//!
//! Natural Wonders are unique geological, ecological, or magical formations that provide
//! special bonuses, historical significance, and visual landmarks for generated worlds.
//!
//! # Design Principles
//!
//! - **Deterministic**: Same seed produces same wonders at same locations
//! - **Sparse**: Wonders are rare and impactful, not common occurrences
//! - **Contextual**: Wonder placement respects terrain, biome, and elevation constraints
//! - **Categorized**: Wonders grouped by type (geological, hydrological, magical, etc.)
//!
//! # Wonder Categories
//!
//! - **Geological**: Mountains, canyons, rock formations, caves
//! - **Hydrological**: Waterfalls, lakes, oases, hot springs
//! - **Biological**: Ancient forests, crystal groves, unique ecosystems
//! - **Atmospheric**: Persistent weather phenomena, auroras, lightning storms
//! - **Magical**: ley lines, mana springs, portals, ancient groves
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use world_factory::terrain::natural_wonders::{NaturalWonderSpawner, WonderSpawnConfig};
//!
//! let seed = 42u64;
//! let width = 100.0f32;
//! let height = 100.0f32;
//! let mut spawner = NaturalWonderSpawner::new(seed, width, height);
//! let config = WonderSpawnConfig::default();
//!
//! // Note: Full spawning requires terrain and biome data setup
//! // for wonder in spawner.spawn_wonders(&terrain_data, &biome_data, config) {
//! //     println!("{} at ({}, {})", wonder.name, wonder.x, wonder.y);
//! // }
//! ```

mod wonder_effects;
mod wonder_spawner;
mod wonder_types;

pub use wonder_effects::{
    apply_wonder_effects, compute_wonder_bonuses, WonderBonus, WonderBonusSource, WonderBonusType,
};
pub use wonder_spawner::{
    NaturalWonderSpawner, TerrainDataForSpawning, WonderSpawnConfig, WonderSpawnResult,
    WonderSpawnStats, WONDER_SPAWN_PARAMS,
};
pub use wonder_types::{
    WonderCategory, WonderData, WonderEffect, WonderProperties, WonderType, KNOWN_WONDERS,
    WONDER_TYPES,
};

use serde::{Deserialize, Serialize};

/// Represents a spawned natural wonder in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalWonder {
    /// Unique identifier for this wonder
    pub id: u32,
    /// Type of wonder
    pub wonder_type: WonderType,
    /// Display name (may be unique per instance)
    pub name: String,
    /// World position
    pub x: f32,
    pub y: f32,
    /// Radius of influence in cells
    pub influence_radius: f32,
    /// Which region(s) this wonder belongs to
    pub region_ids: Vec<u32>,
    /// Bonuses provided by this wonder
    pub bonuses: Vec<WonderBonus>,
    /// Description for UI/rendering
    pub description: String,
    /// Visual style hints
    pub visual_properties: WonderVisualProperties,
}

/// Visual rendering hints for a natural wonder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderVisualProperties {
    /// Primary color for rendering
    pub primary_color: [u8; 3],
    /// Secondary/accent color
    pub secondary_color: Option<[u8; 3]>,
    /// Icon/marker type for map display
    pub icon_type: WonderIconType,
    /// Particle effect hint (for animated rendering)
    pub has_particles: bool,
    /// Vertical offset for rendering (some wonders are elevated)
    pub elevation_offset: f32,
}

/// Icon types for wonder rendering on maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderIconType {
    Mountain,
    Waterfall,
    Lake,
    Volcano,
    Geyser,
    AncientTree,
    Crystal,
    Aura,
    Portal,
    Forest,
    Canyon,
    Cave,
    Aurora,
    Lightning,
    HotSpring,
    Oasis,
    LeyLine,
    Ruins,
    Unknown,
}

impl WonderIconType {
    /// Get the default color for this icon type.
    pub fn default_color(&self) -> [u8; 3] {
        match self {
            WonderIconType::Mountain => [139, 90, 43],    // Brown
            WonderIconType::Waterfall => [64, 164, 223],  // Blue
            WonderIconType::Lake => [30, 144, 255],       // Dodger blue
            WonderIconType::Volcano => [255, 69, 0],      // Red-orange
            WonderIconType::Geyser => [200, 200, 200],    // Light gray
            WonderIconType::AncientTree => [34, 139, 34], // Forest green
            WonderIconType::Crystal => [186, 85, 211],    // Medium orchid
            WonderIconType::Aura => [148, 0, 211],        // Dark violet
            WonderIconType::Portal => [75, 0, 130],       // Indigo
            WonderIconType::Forest => [0, 128, 0],        // Green
            WonderIconType::Canyon => [210, 105, 30],     // Chocolate
            WonderIconType::Cave => [47, 79, 79],         // Dark slate gray
            WonderIconType::Aurora => [0, 255, 127],      // Spring green
            WonderIconType::Lightning => [255, 255, 0],   // Yellow
            WonderIconType::HotSpring => [255, 160, 122], // Light salmon
            WonderIconType::Oasis => [0, 191, 255],       // Deep sky blue
            WonderIconType::LeyLine => [148, 0, 211],     // Dark violet
            WonderIconType::Ruins => [128, 128, 128],     // Gray
            WonderIconType::Unknown => [105, 105, 105],   // Dim gray
        }
    }
}
