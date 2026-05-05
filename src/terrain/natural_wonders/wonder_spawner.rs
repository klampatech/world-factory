//! Wonder Spawning Module for World Factory
//! 
//! Deterministic procedural generation of natural wonders based on terrain,
//! biome, and world parameters.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::util::noise::SimplexNoise;
use crate::terrain::{ElevationGrid, BiomeType};
use super::{
    NaturalWonder, WonderType, WonderBonus,
    WonderVisualProperties, WonderIconType,
    wonder_types::{WonderProperties, KNOWN_WONDERS, WonderCategory, BiomeConstraint},
};

/// Configuration for wonder spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderSpawnConfig {
    /// Base wonder density (0.0-1.0, affects total count)
    pub density: f32,
    /// Minimum distance between wonders (cells)
    pub min_wonder_distance: f32,
    /// Enable legendary/unique named wonders
    pub enable_legendary: bool,
    /// Enable magical wonder types
    pub enable_magical: bool,
    /// Enable atmospheric wonder types
    pub enable_atmospheric: bool,
    /// World seed for deterministic generation
    pub world_seed: u64,
    /// World dimensions for boundary checks
    pub world_width: f32,
    pub world_height: f32,
}

impl Default for WonderSpawnConfig {
    fn default() -> Self {
        Self {
            density: 0.3,
            min_wonder_distance: 15.0,
            enable_legendary: true,
            enable_magical: true,
            enable_atmospheric: true,
            world_seed: 0,
            world_width: 256.0,
            world_height: 256.0,
        }
    }
}

/// Result of wonder spawning for one world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderSpawnResult {
    /// All spawned wonders
    pub wonders: Vec<NaturalWonder>,
    /// Spawn statistics
    pub stats: WonderSpawnStats,
    /// Spawned positions (for distance checking)
    spawned_positions: Vec<(f32, f32)>,
}

impl WonderSpawnResult {
    /// Check if a position is far enough from all existing wonders.
    fn is_valid_position(&self, x: f32, y: f32, min_dist: f32) -> bool {
        for (sx, sy) in &self.spawned_positions {
            let dx = x - sx;
            let dy = y - sy;
            if (dx * dx + dy * dy).sqrt() < min_dist {
                return false;
            }
        }
        true
    }
    
    /// Add a wonder to the result.
    fn add_wonder(&mut self, wonder: NaturalWonder) {
        self.spawned_positions.push((wonder.x, wonder.y));
        self.wonders.push(wonder);
    }
}

/// Statistics about wonder spawning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WonderSpawnStats {
    /// Total wonders spawned
    pub total_wonders: usize,
    /// Wonders by category
    pub by_category: HashMap<String, usize>,
    /// Average distance between wonders
    pub avg_wonder_distance: f32,
    /// Region coverage (percentage of regions with wonders)
    pub region_coverage: f32,
}

/// Spawn parameters for a wonder type (used internally).
#[derive(Debug, Clone)]
pub struct WonderSpawnParams {
    pub wonder_type: WonderType,
    pub properties: WonderProperties,
    pub x: f32,
    pub y: f32,
    pub seed: u64,
}

/// Natural wonder spawner.
#[derive(Debug, Clone)]
pub struct NaturalWonderSpawner {
    config: WonderSpawnConfig,
    noise: SimplexNoise,
    world_seed: u64,
    width: f32,
    height: f32,
}

impl NaturalWonderSpawner {
    /// Create a new wonder spawner.
    pub fn new(world_seed: u64, width: f32, height: f32) -> Self {
        Self {
            config: WonderSpawnConfig::default(),
            noise: SimplexNoise::new(world_seed.wrapping_add(42)), // "WOND"
            world_seed,
            width,
            height,
        }
    }
    
    /// Create with custom config.
    pub fn with_config(world_seed: u64, width: f32, height: f32, config: WonderSpawnConfig) -> Self {
        Self {
            config,
            noise: SimplexNoise::new(world_seed.wrapping_add(42)), // "WOND"
            world_seed,
            width,
            height,
        }
    }
    
    /// Spawn all wonders for a world.
    pub fn spawn_wonders(
        &mut self,
        terrain_data: &TerrainDataForSpawning,
    ) -> WonderSpawnResult {
        let mut result = WonderSpawnResult {
            wonders: Vec::new(),
            stats: WonderSpawnStats::default(),
            spawned_positions: Vec::new(),
        };
        
        // First, spawn legendary/known wonders
        if self.config.enable_legendary {
            self.spawn_legendary_wonders(&mut result, terrain_data);
        }
        
        // Then spawn regular wonders based on density
        let wonder_count = self.calculate_wonder_count();
        
        for i in 0..wonder_count {
            if let Some(params) = self.select_wonder_type(i, terrain_data) {
                if let Some(wonder) = self.try_spawn_wonder(params, terrain_data, &mut result) {
                    result.add_wonder(wonder);
                }
            }
        }
        
        // Calculate stats
        result.stats.total_wonders = result.wonders.len();
        result.stats.region_coverage = result.wonders.len() as f32 / terrain_data.region_count as f32;
        
        // Count by category
        for wonder in &result.wonders {
            let cat_name = wonder.wonder_type.category().name().to_string();
            *result.stats.by_category.entry(cat_name).or_insert(0) += 1;
        }
        
        result
    }
    
    /// Calculate how many wonders to spawn based on density and world size.
    fn calculate_wonder_count(&self) -> usize {
        let base_count = ((self.width * self.height) / 10000.0) as usize;
        let density_adjusted = (base_count as f32 * self.config.density) as usize;
        
        // Clamp to reasonable range
        density_adjusted.max(2).min(50)
    }
    
    /// Spawn legendary/known wonders.
    fn spawn_legendary_wonders(
        &self,
        result: &mut WonderSpawnResult,
        terrain_data: &TerrainDataForSpawning,
    ) {
        let world_size = self.width.max(self.height);
        
        for known in KNOWN_WONDERS {
            // Check if world is large enough
            if world_size < known.min_world_size as f32 {
                continue;
            }
            
            // Only spawn unique wonders once per world
            if known.unique_per_world {
                let seed_offset = self.hash_combine(self.world_seed, known.name.len() as u64);
                
                // Find a valid position
                if let Some((x, y)) = self.find_valid_position(
                    seed_offset,
                    terrain_data,
                    result,
                    &known.wonder_type.properties(),
                ) {
                    let wonder = self.create_wonder_instance(
                        known.wonder_type,
                        x,
                        y,
                        seed_offset,
                        Some(known.name.to_string()),
                    );
                    result.add_wonder(wonder);
                }
            }
        }
    }
    
    /// Select a wonder type based on terrain and noise.
    fn select_wonder_type(
        &self,
        index: usize,
        _terrain_data: &TerrainDataForSpawning,
    ) -> Option<WonderSpawnParams> {
        // Collect viable wonder types with weights
        let mut candidates: Vec<(WonderType, f32)> = Vec::new();
        
        for wonder_type in super::wonder_types::WONDER_TYPES {
            let props = wonder_type.properties();
            
            // Check if this type is enabled
            let category = wonder_type.category();
            match category {
                super::wonder_types::WonderCategory::Magical if !self.config.enable_magical => continue,
                super::wonder_types::WonderCategory::Atmospheric if !self.config.enable_atmospheric => continue,
                _ => {}
            }
            
            candidates.push((wonder_type, props.spawn_weight));
        }
        
        if candidates.is_empty() {
            return None;
        }
        
        // Select based on weighted noise
        let seed = self.hash_combine(self.world_seed, index as u64);
        let selection = self.noise.get_seed_u64(seed) % candidates.len() as u64;
        
        let (wonder_type, _) = candidates[selection as usize];
        let props = wonder_type.properties();
        
        // Find a valid position for this type
        let x = self.noise.get_bounded_f32(seed.wrapping_add(1), 0.0, self.width);
        let y = self.noise.get_bounded_f32(seed.wrapping_add(2), 0.0, self.height);
        
        Some(WonderSpawnParams {
            wonder_type,
            properties: props.clone(),
            x,
            y,
            seed,
        })
    }
    
    /// Try to spawn a wonder at the given params.
    fn try_spawn_wonder(
        &self,
        params: WonderSpawnParams,
        terrain_data: &TerrainDataForSpawning,
        result: &mut WonderSpawnResult,
    ) -> Option<NaturalWonder> {
        let WonderSpawnParams {
            wonder_type,
            properties,
            x,
            y,
            seed,
        } = params;
        
        // Check elevation constraints
        let elevation = (terrain_data.get_elevation)(x, y);
        if elevation < properties.min_elevation || elevation > properties.max_elevation {
            return None;
        }
        
        // Check distance from other wonders
        if !result.is_valid_position(x, y, self.config.min_wonder_distance) {
            return None;
        }
        
        // Check biome constraints (if any) - skip if no valid biomes defined
        if !properties.valid_biomes.is_empty() {
            // For Phase 1: biome constraints use WonderType as a placeholder
            // In a full implementation, this would check actual BiomeType
            let mut biome_allowed = false;
            for constraint in &properties.valid_biomes {
                // Empty allowed list means any biome is allowed
                if constraint.allowed.is_empty() {
                    biome_allowed = true;
                    break;
                }
            }
            if !biome_allowed {
                return None;
            }
        }
        
        // Check water/mountain requirements
        if properties.requires_water && !(terrain_data.has_water_nearby)(x, y, 5.0) {
            return None;
        }
        if properties.requires_mountains && !(terrain_data.has_mountains_nearby)(x, y, 3.0) {
            return None;
        }
        
        // All checks passed - create the wonder
        Some(self.create_wonder_instance(wonder_type, x, y, seed, None))
    }
    
    /// Find a valid position for a wonder with specific properties.
    fn find_valid_position(
        &self,
        seed: u64,
        terrain_data: &TerrainDataForSpawning,
        result: &mut WonderSpawnResult,
        properties: &WonderProperties,
    ) -> Option<(f32, f32)> {
        // Try multiple positions using noise
        for attempt in 0..20 {
            let attempt_seed = seed.wrapping_add(attempt);
            let x = self.noise.get_bounded_f32(attempt_seed, 0.0, self.width);
            let y = self.noise.get_bounded_f32(attempt_seed.wrapping_add(1), 0.0, self.height);
            
            // Check constraints
            let elevation = (terrain_data.get_elevation)(x, y);
            if elevation < properties.min_elevation || elevation > properties.max_elevation {
                continue;
            }
            
            if !result.is_valid_position(x, y, self.config.min_wonder_distance) {
                continue;
            }
            
            if properties.requires_water && !(terrain_data.has_water_nearby)(x, y, 5.0) {
                continue;
            }
            if properties.requires_mountains && !(terrain_data.has_mountains_nearby)(x, y, 3.0) {
                continue;
            }
            
            return Some((x, y));
        }
        
        None
    }
    
    /// Create a wonder instance at a position.
    fn create_wonder_instance(
        &self,
        wonder_type: WonderType,
        x: f32,
        y: f32,
        seed: u64,
        override_name: Option<String>,
    ) -> NaturalWonder {
        // Generate unique name
        let name = override_name.unwrap_or_else(|| {
            self.generate_wonder_name(wonder_type, seed)
        });
        
        // Generate unique ID
        let id = self.hash_combine(seed, self.world_seed) as u32;
        
        // Get effects from type definition
        let effects = wonder_type.effects();
        let bonuses: Vec<WonderBonus> = effects.into_iter().map(|e| {
            WonderBonus {
                bonus_type: e.bonus_type,
                magnitude: e.magnitude,
                radius: e.radius,
                region_wide: e.region_wide,
            }
        }).collect();
        
        // Get visual properties
        let icon_type = wonder_type.icon_type();
        let visual = WonderVisualProperties {
            primary_color: icon_type.default_color(),
            secondary_color: None,
            icon_type,
            has_particles: matches!(
                icon_type,
                WonderIconType::Aurora | WonderIconType::Lightning | WonderIconType::Portal
            ),
            elevation_offset: if wonder_type.properties().min_elevation > 1000.0 {
                50.0
            } else {
                0.0
            },
        };
        
        NaturalWonder {
            id,
            wonder_type,
            name: name.to_string(),
            x,
            y,
            influence_radius: wonder_type.properties().influence_radius as f32,
            region_ids: vec![], // Will be filled by caller
            bonuses,
            description: wonder_type.description().to_string(),
            visual_properties: visual,
        }
    }
    
    /// Generate a unique name for a wonder based on type and seed.
    fn generate_wonder_name(&self, wonder_type: WonderType, seed: u64) -> String {
        let prefixes = [
            "Ancient", "Sacred", "Eternal", "Mystic", "Hidden",
            "Forgotten", "Enchanted", "Legendary", "Blessed", "Cursed",
        ];
        
        let base = wonder_type.name();
        let prefix_idx = (seed % prefixes.len() as u64) as usize;
        let use_prefix = seed % 3 != 0; // 2/3 chance of prefix
        
        if use_prefix {
            format!("{} {}", prefixes[prefix_idx], base)
        } else {
            base.to_string()
        }
    }
    
    /// Combine two seeds for deterministic variation.
    fn hash_combine(&self, a: u64, b: u64) -> u64 {
        // Simple but effective hash combination
        let mut hash = a.wrapping_mul(0x9e3779b97f4a7c15);
        hash ^= b.wrapping_add(0x9e3779b97f4a7c15).wrapping_mul(0x9e3779b9);
        hash = hash.rotate_left(32);
        hash.wrapping_mul(0xbf324aad)
    }
}

/// Terrain data needed for wonder spawning (passed by caller).
pub struct TerrainDataForSpawning<'a> {
    /// Get elevation at world position (normalized 0.0-1.0)
    pub get_elevation: Box<dyn Fn(f32, f32) -> f32 + 'a>,
    /// Get biome at world position
    pub get_biome: Box<dyn Fn(f32, f32) -> BiomeType + 'a>,
    /// Check if water exists within radius (cells)
    pub has_water_nearby: Box<dyn Fn(f32, f32, f32) -> bool + 'a>,
    /// Check if mountains exist within radius (cells)
    pub has_mountains_nearby: Box<dyn Fn(f32, f32, f32) -> bool + 'a>,
    /// Number of regions in the world
    pub region_count: usize,
    /// Get which region contains this position
    pub get_region_id: Box<dyn Fn(f32, f32) -> Option<u32> + 'a>,
}

// Default implementations for common use cases
impl<'a> TerrainDataForSpawning<'a> {
    /// Create from ElevationGrid - for use with WorldGenerator.
    /// 
    /// The elevation grid stores normalized values [0.0, 1.0] where:
    /// - 0.0 = sea level or below
    /// - 1.0 = highest elevation (e.g., 2500m or higher)
    /// 
    /// For wonder spawning, we convert to approximate meters using the scale factor.
    pub fn from_elevation_grid<'b: 'a>(
        elevation_grid: &'b ElevationGrid,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            get_elevation: Box::new(move |x, y| {
                let ix = (x as u32).min(width - 1);
                let iy = (y as u32).min(height - 1);
                elevation_grid.get(ix as usize, iy as usize).unwrap_or(0.0)
            }),
            get_biome: Box::new(|_, _| BiomeType::TemperateDeciduousForest), // Default biome
            has_water_nearby: Box::new(move |x, y, radius| {
                let ix = x as i32;
                let iy = y as i32;
                let r = radius as i32;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = ix + dx;
                        let ny = iy + dy;
                        if elevation_grid.is_valid(nx, ny) {
                            if elevation_grid.get_value_unchecked(nx, ny) < 0.5 {
                                return true;
                            }
                        }
                    }
                }
                false
            }),
            has_mountains_nearby: Box::new(move |x, y, radius| {
                let ix = x as i32;
                let iy = y as i32;
                let r = radius as i32;
                // Mountains: normalized elevation > 0.7 (roughly > 1750m)
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = ix + dx;
                        let ny = iy + dy;
                        if elevation_grid.is_valid(nx, ny) {
                            if elevation_grid.get_value_unchecked(nx, ny) > 0.7 {
                                return true;
                            }
                        }
                    }
                }
                false
            }),
            region_count: ((width * height) / 100) as usize, // Estimate: ~100 cells per region
            get_region_id: Box::new(move |x, y| {
                let ix = (x as u32 / 10).min(width / 10 - 1);
                let iy = (y as u32 / 10).min(height / 10 - 1);
                Some(iy * (width / 10) + ix)
            }),
        }
    }
}

/// Spawn parameters for specific wonder types (for documentation).
pub const WONDER_SPAWN_PARAMS: &[(&str, f32, f32, f32, bool, bool)] = &[
    // (name, min_elev, max_elev, weight, requires_water, requires_mountains)
    ("SacredMountain", 1500.0, 6000.0, 1.5, false, false),
    ("GrandCanyon", 200.0, 2500.0, 1.0, true, false),
    ("AncientTree", 0.0, 2000.0, 1.2, true, false),
    ("CrystalCavern", -200.0, 500.0, 0.8, false, true),
    ("ActiveVolcano", 500.0, 3000.0, 0.6, false, true),
    ("MagnificentWaterfall", 100.0, 1500.0, 1.3, true, false),
    ("GreatLake", -50.0, 3000.0, 1.4, true, false),
    ("AncientForest", 0.0, 1500.0, 1.1, true, false),
    ("AuroraBorealis", 0.0, 5000.0, 0.7, false, false),
    ("LeyLineNexus", 0.0, 2000.0, 0.5, false, false),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::BiomeType;
    
    #[test]
    fn test_wonder_spawner_creation() {
        let spawner = NaturalWonderSpawner::new(12345, 256.0, 256.0);
        assert_eq!(spawner.world_seed, 12345);
    }
    
    #[test]
    fn test_deterministic_naming() {
        let mut spawner1 = NaturalWonderSpawner::new(42, 100.0, 100.0);
        let mut spawner2 = NaturalWonderSpawner::new(42, 100.0, 100.0);
        
        // Same seed should produce same wonder count
        let terrain_data = TerrainDataForSpawning {
            get_elevation: Box::new(|_, _| 0.6), // Normalized elevation
            get_biome: Box::new(|_, _| BiomeType::TemperateDeciduousForest),
            has_water_nearby: Box::new(|_, _, _| true),
            has_mountains_nearby: Box::new(|_, _, _| true),
            region_count: 100,
            get_region_id: Box::new(|_, _| Some(0)),
        };
        
        let result1 = spawner1.spawn_wonders(&terrain_data);
        let result2 = spawner2.spawn_wonders(&terrain_data);
        
        assert_eq!(result1.wonders.len(), result2.wonders.len());
    }
    
    #[test]
    fn test_wonder_count_calculation() {
        // 256x256 = 65536 cells, / 10000 = ~6.5, * 0.3 density = ~2
        let spawner = NaturalWonderSpawner::new(0, 256.0, 256.0);
        assert!(spawner.calculate_wonder_count() >= 2);
        assert!(spawner.calculate_wonder_count() <= 10);
    }
    
    #[test]
    fn test_from_elevation_grid() {
        use crate::terrain::ElevationGrid;
        
        // Create a test elevation grid
        let mut grid = ElevationGrid::new(50, 50, 0.5);
        // Add some high elevation cells
        grid.set(25, 25, 0.8);
        grid.set(26, 25, 0.9);
        grid.set(25, 26, 0.85);
        
        // Create terrain data from grid
        let terrain_data = TerrainDataForSpawning::from_elevation_grid(&grid, 50, 50);
        
        // Test elevation access - use the closure directly
        let get_elev = terrain_data.get_elevation;
        assert_eq!(get_elev(0.0, 0.0), 0.5);
        assert_eq!(get_elev(25.0, 25.0), 0.8);
        
        // Test mountain detection (should find mountains in the high elevation area)
        let has_mountains = terrain_data.has_mountains_nearby;
        assert!(has_mountains(25.0, 25.0, 2.0));
        assert!(!has_mountains(0.0, 0.0, 2.0));
        
        // Test water detection (all cells are land in this test grid)
        let has_water = terrain_data.has_water_nearby;
        assert!(!has_water(25.0, 25.0, 2.0));
    }
}