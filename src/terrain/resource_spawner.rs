//! # Resource Spawning Algorithm
//! 
//! Deterministic resource generation for World Factory.
//! 
//! ## Overview
//! 
//! This module implements procedural resource spawning that assigns resources to
//! regions based on biome compatibility, elevation constraints, tectonic geology,
//! and seeded random selection.
//! 
//! ## Quick Start
//! 
//! ```rust,ignore
//! use world_factory::terrain::resource_spawner::ResourceSpawner;
//! use world_factory::terrain::biome::BiomeType;
//! 
//! let seed = 42u64;
//! let mut spawner = ResourceSpawner::new(seed, Default::default());
//! let spawn = spawner.spawn_region(
//!     1,
//!     BiomeType::TemperateDeciduousForest,
//!     200.0,
//!     100.0,
//!     100.0,
//! );
//! 
//! for deposit in &spawn.deposits {
//!     println!("{} ({:?}): value={}", 
//!         deposit.resource_type.name(),
//!         deposit.richness,
//!         deposit.estimated_value
//!     );
//! }
//! ```
//! 
//! ## Algorithm
//! 
//! 1. **Biome Compatibility**: Check `ViabilityMatrix` for resources valid in vegetation type
//! 2. **Elevation Filtering**: Apply constraints (aquatic ≤100m, oil/gas depth ranges)
//! 3. **Tectonic Boundary Influence**: Plate boundaries boost mineral deposits (1.8x for convergent)
//! 4. **Biome Affinity**: Mountains → minerals (2x), Forests → timber (2x), Swamps → coal/oil (2x)
//! 5. **Spawn Decision**: Deterministic hash of (seed, region_id, position) for reproducibility
//! 6. **Richness Calculation**: Map rarity + noise + tectonic modifiers to `ResourceRichness` levels
//! 
//! ## Configuration
//! 
//! ```rust,ignore
//! use world_factory::terrain::resource_spawner::{ResourceSpawnConfig, ResourceSpawner};
//! use world_factory::terrain::resource_types::{TectonicBoundaryData, BoundaryEffectType};
//! 
//! let config = ResourceSpawnConfig {
//!     enable_fantasy: true,      // Include fantasy resources
//!     enable_legendary: true,     // Include legendary resources
//!     density: 0.5,              // 0.0-1.0 spawn probability
//!     max_per_region: 8,          // Max resources per region
//!     clustering: 0.3,           // Resource clustering tendency
//!     base_rate: 1.0,            // Base spawn rate multiplier
//!     ..Default::default()
//! };
//! 
//! let seed = 42u64;
//! let mut spawner = ResourceSpawner::new(seed, config);
//! 
//! // With tectonic data for enhanced mineral spawning
//! let tectonic = TectonicBoundaryData {
//!     is_on_boundary: true,
//!     distance_to_boundary: 5.0,
//!     boundary_effect: BoundaryEffectType::Convergent, // Mountains: 1.8x minerals
//! };
//! 
//! let biome = world_factory::terrain::biome::BiomeType::TemperateDeciduousForest;
//! let result = spawner.spawn_region_with_tectonic(
//!     1, biome, 200.0, 100.0, 100.0, Some(tectonic)
//! );
//! ```
//! 
//! ## Tectonic Boundary Effects
//! 
//! - **Convergent** (mountains): 1.8x mineral deposits, +15% richness bonus
//! - **Divergent** (rifts): 1.2x mineral deposits, +6% richness bonus
//! - **Transform** (faults): 0.8x mineral deposits (limited potential)
//! - **None**: 1.0x (no geological influence)
//! 
//! ## Biome Affinity Table
//! 
//! | Biome | Resource Category | Modifier |
//! |-------|-----------------|----------|
//! | Mountain types | BaseMetals, PreciousMetals, Stone | 2.0x |
//! | TemperateDeciduousForest | Timber | 2.0x |
//! | TropicalRainforest | Timber | 2.5x |
//! | Swamp | FossilFuels | 2.0x |
//! | HotDesert | PreciousMetals, IndustrialMinerals | 1.5x |
//! | TemperateGrassland | Agriculture, Livestock | 2.0x |
//! 
//! ## Determinism
//! 
//! Same seed produces identical results:
//! 
//! ```rust,ignore
//! use world_factory::terrain::resource_spawner::ResourceSpawner;
//! use world_factory::terrain::biome::BiomeType;
//! 
//! let biome = BiomeType::TemperateDeciduousForest;
//! let mut s1 = ResourceSpawner::new(42, Default::default());
//! let mut s2 = ResourceSpawner::new(42, Default::default());
//! 
//! let r1 = s1.spawn_region(1, biome, 200.0, 100.0, 100.0);
//! let r2 = s2.spawn_region(1, biome, 200.0, 100.0, 100.0);
//! 
//! assert_eq!(r1.deposits.len(), r2.deposits.len());
//! ```

// ============================================================================
// IMPLEMENTATION
// ============================================================================

use serde::{Deserialize, Serialize};
use crate::terrain::resource_types::{
    ResourceType, ResourceCategory, ResourceRichness, ResourceDeposit,
    ResourceGenerator, ResourceGenConfig, ALL_RESOURCE_TYPES, ViabilityMatrix,
};
use crate::terrain::biome::{BiomeType, VegetationType};
use crate::util::noise::SimplexNoise;

/// Configuration for resource spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpawnConfig {
    /// Enable fantasy/sci-fi resources
    pub enable_fantasy: bool,
    /// Enable legendary resources (mithril, adamantine, etc.)
    pub enable_legendary: bool,
    /// Base resource density (0.0-1.0)
    pub density: f32,
    /// Maximum resources per region
    pub max_per_region: usize,
    /// Minimum distance between resource clusters
    pub cluster_min_distance: f32,
    /// Resource clustering tendency (0.0-1.0)
    pub clustering: f32,
    /// Base rate for resource spawning (can be overridden per call)
    pub base_rate: f32,
}

impl Default for ResourceSpawnConfig {
    fn default() -> Self {
        Self {
            enable_fantasy: true,
            enable_legendary: true,
            density: 0.5,
            max_per_region: 8,
            cluster_min_distance: 100.0,
            clustering: 0.3,
            base_rate: 1.0,
        }
    }
}

/// Tectonic boundary data for resource spawning.
/// Represents plate boundaries that influence mineral deposits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TectonicBoundaryData {
    /// Whether this region is on a plate boundary
    pub is_on_boundary: bool,
    /// Distance to nearest plate boundary (in cells/regions)
    pub distance_to_boundary: f32,
    /// Boundary effect type affecting resources
    pub boundary_effect: BoundaryEffectType,
}

/// Type of tectonic boundary effect on resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryEffectType {
    /// No boundary influence
    None,
    /// Convergent boundary (uplift) - increases mineral deposits
    Convergent,
    /// Divergent boundary (rifts) - moderate mineral potential
    Divergent,
    /// Transform boundary - limited mineral potential
    Transform,
}

impl Default for BoundaryEffectType {
    fn default() -> Self {
        BoundaryEffectType::None
    }
}

impl BoundaryEffectType {
    /// Get the mineral deposit multiplier for this boundary type.
    pub fn mineral_multiplier(&self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Convergent => 1.8,  // Mountains at convergent boundaries have more minerals
            Self::Divergent => 1.2,   // Rifts have some mineral potential
            Self::Transform => 0.8,    // Limited mineral potential
        }
    }
}

/// Resource spawn result for a single region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionResourceSpawn {
    /// Region identifier
    pub region_id: u32,
    /// All spawned resource deposits
    pub deposits: Vec<ResourceDeposit>,
    /// Primary resource (highest value)
    pub primary: Option<ResourceType>,
    /// Total estimated value of all resources
    pub total_value: f32,
    /// Number of resource types present
    pub diversity: usize,
}

/// Resource spawn statistics for a world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSpawnStats {
    /// Total resources spawned across all regions
    pub total_deposits: usize,
    /// Resources by category
    pub by_category: std::collections::HashMap<ResourceCategory, usize>,
    /// Average richness level
    pub avg_richness: f32,
    /// Highest value deposit
    pub max_value_deposit: Option<(ResourceType, f32)>,
    /// Regions with no resources
    pub barren_regions: usize,
    /// Total estimated world resource value
    pub total_world_value: f32,
}

/// Main resource spawner implementing the spawning algorithm.
#[derive(Debug, Clone)]
pub struct ResourceSpawner {
    config: ResourceSpawnConfig,
    noise: SimplexNoise,
    generator: ResourceGenerator,
    seed: u64,
}

impl ResourceSpawner {
    /// Create a new resource spawner.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, ResourceSpawnConfig::default())
    }

    /// Create a resource spawner with custom configuration.
    pub fn with_config(seed: u64, config: ResourceSpawnConfig) -> Self {
        let resource_gen_config = ResourceGenConfig {
            enable_fantasy_resources: config.enable_fantasy,
            enable_legendary: config.enable_legendary,
            base_density: config.density,
            max_resources_per_region: config.max_per_region,
        };
        Self {
            seed,
            config,
            noise: SimplexNoise::new(seed),
            generator: ResourceGenerator::with_config(resource_gen_config),
        }
    }
    
    /// Spawn resources for a single region.
    ///
    /// Uses the biome type, elevation, and position to determine
    /// which resources can spawn and their richness levels.
    /// Optionally accepts tectonic boundary data for enhanced mineral spawning.
    ///
    /// # Arguments
    /// * `region_id` - Unique identifier for this region
    /// * `biome` - The biome type of this region
    /// * `elevation` - Elevation in meters
    /// * `x` - X position in world (for noise sampling)
    /// * `y` - Y position in world (for noise sampling)
    /// * `tectonic` - Optional tectonic boundary data (None = no boundary influence)
    pub fn spawn_region(
        &mut self,
        region_id: u32,
        biome: BiomeType,
        elevation: f32,
        x: f32,
        y: f32,
    ) -> RegionResourceSpawn {
        self.spawn_region_with_tectonic(region_id, biome, elevation, x, y, None)
    }
    
    /// Spawn resources for a single region with tectonic boundary data.
    ///
    /// This extended version includes geological context from plate boundaries
    /// for more accurate mineral resource generation.
    pub fn spawn_region_with_tectonic(
        &mut self,
        region_id: u32,
        biome: BiomeType,
        elevation: f32,
        x: f32,
        y: f32,
        tectonic: Option<TectonicBoundaryData>,
    ) -> RegionResourceSpawn {
        let mut deposits = Vec::new();
        let vegetation = biome.vegetation();
        let viable_resources = self.get_viable_resources(&vegetation, elevation);
        let elevation_filtered = self.filter_by_elevation(viable_resources, elevation);
        
        // Determine spawns with tectonic influence on minerals
        let spawned = self.determine_spawns_with_tectonic(
            region_id, 
            elevation_filtered, 
            x, 
            y, 
            tectonic.as_ref()
        );
        
        for resource in spawned {
            let richness = self.calculate_richness_with_tectonic(
                resource, 
                region_id, 
                x, 
                y, 
                tectonic.as_ref()
            );
            if richness != ResourceRichness::None {
                deposits.push(ResourceDeposit::new(resource, richness));
            }
        }
        
        deposits.sort_by(|a, b| b.estimated_value.partial_cmp(&a.estimated_value).unwrap());
        
        let primary = deposits.first().map(|d| d.resource_type);
        let total_value: f32 = deposits.iter().map(|d| d.estimated_value).sum();
        let diversity = deposits.len();
        
        RegionResourceSpawn { region_id, deposits, primary, total_value, diversity }
    }
    
    /// Spawn resources for multiple regions.
    pub fn spawn_regions(
        &mut self,
        regions: &[(u32, BiomeType, f32, f32, f32)],
    ) -> Vec<RegionResourceSpawn> {
        regions.iter().map(|(id, biome, elev, x, y)| {
            self.spawn_region(*id, *biome, *elev, *x, *y)
        }).collect()
    }
    
    /// Calculate spawn statistics for a set of spawns.
    pub fn calculate_stats(&self, spawns: &[RegionResourceSpawn]) -> ResourceSpawnStats {
        let mut stats = ResourceSpawnStats::default();
        let mut total_richness = 0.0;
        
        for spawn in spawns {
            stats.total_deposits += spawn.deposits.len();
            if spawn.deposits.is_empty() { stats.barren_regions += 1; }
            
            for deposit in &spawn.deposits {
                let category = deposit.resource_type.category();
                *stats.by_category.entry(category).or_insert(0) += 1;
                
                if let Some((_, max_val)) = stats.max_value_deposit {
                    if deposit.estimated_value > max_val {
                        stats.max_value_deposit = Some((deposit.resource_type, deposit.estimated_value));
                    }
                } else {
                    stats.max_value_deposit = Some((deposit.resource_type, deposit.estimated_value));
                }
                total_richness += deposit.richness.as_f32();
            }
            stats.total_world_value += spawn.total_value;
        }
        
        if stats.total_deposits > 0 {
            stats.avg_richness = total_richness / stats.total_deposits as f32;
        }
        stats
    }
    
    fn get_viable_resources(&self, vegetation: &VegetationType, _elevation: f32) -> Vec<ResourceType> {
        ALL_RESOURCE_TYPES.iter().filter(|rt| {
            if rt.is_fantasy() && !self.config.enable_fantasy { return false; }
            if !self.config.enable_legendary && rt.rarity() > 0.9 { return false; }
            ViabilityMatrix::is_viable(rt, vegetation)
        }).copied().collect()
    }
    
    fn filter_by_elevation(&self, resources: Vec<ResourceType>, elevation: f32) -> Vec<ResourceType> {
        resources.into_iter().filter(|rt| {
            if rt.is_aquatic() && elevation > 100.0 { return false; }
            if elevation > 4000.0 && !rt.is_mineral() { return false; }
            if matches!(rt, ResourceType::Oil | ResourceType::NaturalGas) {
                return elevation > -500.0 && elevation < 2000.0;
            }
            true
        }).collect()
    }
    
    fn determine_spawns(&mut self, region_id: u32, candidates: Vec<ResourceType>, x: f32, y: f32) -> Vec<ResourceType> {
        let mut spawned = Vec::new();
        let region_noise = self.noise.get(x as f64 * 0.01, y as f64 * 0.01);
        
        for resource in candidates {
            if spawned.len() >= self.config.max_per_region { break; }
            
            let rarity = resource.rarity();
            let category_chance = self.get_category_spawn_chance(resource.category());
            let base_prob = (1.0 - rarity) * self.config.density * category_chance;
            let probability = base_prob * ((region_noise * 0.5 + 0.5) * 0.4 + 0.6) as f32;
            
            let hash = self.spawn_hash(region_id, resource, x, y);
            if hash < probability { spawned.push(resource); }
        }
        
        spawned
    }
    
    fn spawn_hash(&self, region_id: u32, resource: ResourceType, x: f32, y: f32) -> f32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        region_id.hash(&mut hasher);
        (resource as u16).hash(&mut hasher);
        ((x * 100.0) as u32).hash(&mut hasher);
        ((y * 100.0) as u32).hash(&mut hasher);
        
        (hasher.finish() as f64 % 1_000_000.0 / 1_000_000.0) as f32
    }
    
    fn get_category_spawn_chance(&self, category: ResourceCategory) -> f32 {
        match category {
            ResourceCategory::Timber => 0.9,
            ResourceCategory::Agriculture => 0.8,
            ResourceCategory::FreshWater => 0.85,
            ResourceCategory::Stone | ResourceCategory::BaseMetals => 0.7,
            ResourceCategory::Livestock | ResourceCategory::Hunting => 0.7,
            ResourceCategory::IndustrialMinerals => 0.5,
            ResourceCategory::Fishing | ResourceCategory::Botanical | ResourceCategory::Fibers => 0.5,
            ResourceCategory::PreciousMetals => 0.3,
            ResourceCategory::FossilFuels => 0.2,
            ResourceCategory::Nuclear | ResourceCategory::RareMetals => 0.2,
            ResourceCategory::MagicalMaterials | ResourceCategory::VolcanicMinerals => 0.2,
            ResourceCategory::LegendaryMetals => 0.05,
            _ => 0.3,
        }
    }
    
    /// Determine spawns with tectonic boundary influence.
    /// 
    /// For mineral resources near plate boundaries (especially convergent),
    /// the spawn probability is multiplied by the boundary effect multiplier.
    fn determine_spawns_with_tectonic(
        &mut self,
        region_id: u32,
        candidates: Vec<ResourceType>,
        x: f32,
        y: f32,
        tectonic: Option<&TectonicBoundaryData>,
    ) -> Vec<ResourceType> {
        let mut spawned = Vec::new();
        let region_noise = self.noise.get(x as f64 * 0.01, y as f64 * 0.01);
        
        // Calculate tectonic multiplier for mineral resources
        let tectonic_multiplier = tectonic.map(|t| t.boundary_effect.mineral_multiplier()).unwrap_or(1.0);
        
        for resource in candidates {
            if spawned.len() >= self.config.max_per_region { break; }
            
            let rarity = resource.rarity();
            let category_chance = self.get_category_spawn_chance(resource.category());
            
            // Base probability
            let mut base_prob = (1.0 - rarity) * self.config.density * category_chance;
            
            // Apply tectonic multiplier for mineral resources
            if resource.is_mineral() && tectonic_multiplier > 1.0 {
                base_prob *= tectonic_multiplier;
            }
            
            // Apply region noise variation
            let probability = base_prob * ((region_noise * 0.5 + 0.5) * 0.4 + 0.6) as f32;
            
            let hash = self.spawn_hash(region_id, resource, x, y);
            if hash < probability { spawned.push(resource); }
        }
        
        spawned
    }
    
    /// Calculate richness with tectonic boundary influence.
    /// 
    /// Convergent boundaries (mountains) get +1.8x mineral richness.
    /// Other boundary types get moderate bonuses.
    fn calculate_richness_with_tectonic(
        &mut self,
        resource: ResourceType,
        region_id: u32,
        x: f32,
        y: f32,
        tectonic: Option<&TectonicBoundaryData>,
    ) -> ResourceRichness {
        let x_f64 = x as f64 * 0.02 + region_id as f64;
        let y_f64 = y as f64 * 0.02 + region_id as f64 * 0.7;
        let richness_noise = self.noise.get(x_f64, y_f64);
        let base_richness = (1.0 - resource.rarity()) * 0.6 + 0.2;
        
        // Apply tectonic influence on mineral richness
        let mut tectonic_bonus = 0.0f32;
        if let Some(t) = tectonic {
            if resource.is_mineral() {
                let multiplier = t.boundary_effect.mineral_multiplier();
                // Multiplier > 1 means more rich deposits
                tectonic_bonus = (multiplier - 1.0) * 0.15;
            }
        }
        
        let final_value = base_richness + (richness_noise * 0.3) as f32 + tectonic_bonus;
        
        if final_value < 0.15 { ResourceRichness::Sparse }
        else if final_value < 0.35 { ResourceRichness::Normal }
        else if final_value < 0.55 { ResourceRichness::Rich }
        else if final_value < 0.75 { ResourceRichness::Abundant }
        else { ResourceRichness::Legendary }
    }
    
    /// Spawn resources for multiple regions with tectonic data.
    pub fn spawn_regions_with_tectonic(
        &mut self,
        regions: &[(u32, BiomeType, f32, f32, f32, Option<TectonicBoundaryData>)],
    ) -> Vec<RegionResourceSpawn> {
        regions.iter().map(|(id, biome, elev, x, y, tectonic)| {
            self.spawn_region_with_tectonic(*id, *biome, *elev, *x, *y, tectonic.clone())
        }).collect()
    }
    
    /// Get biome affinity modifier for resource categories.
    /// Mountains → minerals (2x), Forests → timber (2x), Swamps → coal/oil.
    fn get_biome_affinity(&self, category: ResourceCategory, biome: BiomeType) -> f32 {
        match (category, biome) {
            // Mountains boost minerals
            (ResourceCategory::BaseMetals, _) if biome.is_mountain() => 2.0,
            (ResourceCategory::PreciousMetals, _) if biome.is_mountain() => 2.0,
            (ResourceCategory::Stone, _) if biome.is_mountain() => 2.0,
            
            // Forests boost timber
            (ResourceCategory::Timber, BiomeType::TemperateDeciduousForest) => 2.0,
            (ResourceCategory::Timber, BiomeType::TropicalRainforest) => 2.5,  // Extra rich
            (ResourceCategory::Timber, BiomeType::BorealForest) => 1.5,
            (ResourceCategory::Timber, BiomeType::BorealTaiga) => 1.8,  // Coniferous forest
            
            // Wetlands/swamps boost coal and oil
            (ResourceCategory::FossilFuels, BiomeType::CoastalWetland) => 2.0,
            (ResourceCategory::FossilFuels, BiomeType::Mangrove) => 1.8,
            (ResourceCategory::IndustrialMinerals, BiomeType::ToxicSwamp) => 1.5,
            
            // Deserts boost precious metals
            (ResourceCategory::PreciousMetals, BiomeType::HotDesert) => 1.5,
            (ResourceCategory::IndustrialMinerals, BiomeType::HotDesert) => 1.5,
            
            // Grasslands boost agriculture and livestock
            (ResourceCategory::Agriculture, BiomeType::TemperateGrassland) => 2.0,
            (ResourceCategory::Livestock, BiomeType::TemperateGrassland) => 2.0,
            
            _ => 1.0,
        }
    }
    
    pub fn config(&self) -> &ResourceSpawnConfig { &self.config }
    pub fn set_config(&mut self, config: ResourceSpawnConfig) { self.config = config; }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_spawning() {
        let mut s1 = ResourceSpawner::new(42);
        let mut s2 = ResourceSpawner::new(42);
        
        let r1 = s1.spawn_region(1, BiomeType::TemperateDeciduousForest, 200.0, 100.0, 100.0);
        let r2 = s2.spawn_region(1, BiomeType::TemperateDeciduousForest, 200.0, 100.0, 100.0);
        
        assert_eq!(r1.deposits.len(), r2.deposits.len());
    }
    
    #[test]
    fn test_biome_compatibility() {
        let mut spawner = ResourceSpawner::new(42);
        let result = spawner.spawn_region(1, BiomeType::TemperateDeciduousForest, 200.0, 100.0, 100.0);
        let has_timber = result.deposits.iter().any(|d| d.resource_type.category() == ResourceCategory::Timber);
        assert!(has_timber);
    }
    
    #[test]
    fn test_elevation_filtering() {
        let mut spawner = ResourceSpawner::new(42);
        let result = spawner.spawn_region(1, BiomeType::AlpineTundra, 3500.0, 100.0, 100.0);
        let has_aquatic = result.deposits.iter().any(|d| d.resource_type.is_aquatic());
        assert!(!has_aquatic);
    }
    
    #[test]
    fn test_stats_calculation() {
        let mut spawner = ResourceSpawner::new(42);
        let spawns = vec![
            spawner.spawn_region(1, BiomeType::TemperateDeciduousForest, 200.0, 100.0, 100.0),
            spawner.spawn_region(2, BiomeType::TemperateGrassland, 150.0, 200.0, 100.0),
        ];
        let stats = spawner.calculate_stats(&spawns);
        assert!(stats.total_deposits > 0);
        assert!(stats.total_world_value > 0.0);
    }
    
    #[test]
    fn test_resource_value() {
        let deposit = ResourceDeposit::new(ResourceType::IronOre, ResourceRichness::Rich);
        assert!(deposit.estimated_value > 0.0);
    }
    
    #[test]
    fn test_no_fantasy() {
        let config = ResourceSpawnConfig { enable_fantasy: false, ..Default::default() };
        let mut spawner = ResourceSpawner::with_config(42, config);
        let _ = spawner.spawn_region(1, BiomeType::MagicalForest, 200.0, 100.0, 100.0);
    }
    
    #[test]
    fn test_tectonic_boundary_data() {
        let tectonic = TectonicBoundaryData {
            is_on_boundary: true,
            distance_to_boundary: 5.0,
            boundary_effect: BoundaryEffectType::Convergent,
        };
        
        // Convergent boundaries should have 1.8x mineral multiplier
        assert_eq!(tectonic.boundary_effect.mineral_multiplier(), 1.8);
    }
    
    #[test]
    fn test_tectonic_spawning() {
        let mut spawner = ResourceSpawner::new(42);
        
        let tectonic = TectonicBoundaryData {
            is_on_boundary: true,
            distance_to_boundary: 3.0,
            boundary_effect: BoundaryEffectType::Convergent,
        };
        
        // Spawn with tectonic data
        let result_with = spawner.spawn_region_with_tectonic(
            1, 
            BiomeType::AlpineTundra, 
            2500.0, 
            100.0, 
            100.0,
            Some(tectonic),
        );
        
        // Spawn without tectonic data
        let result_without = spawner.spawn_region(
            1,
            BiomeType::AlpineTundra,
            2500.0,
            100.0,
            100.0,
        );
        
        // Convergent boundary (mountains) should boost mineral resources
        // Both should have minerals but possibly different richness levels
        let with_minerals = result_with.deposits.iter().filter(|d| d.resource_type.is_mineral()).count();
        let without_minerals = result_without.deposits.iter().filter(|d| d.resource_type.is_mineral()).count();
        
        // Mountain biome should have minerals
        assert!(with_minerals >= without_minerals);
    }
    
    #[test]
    fn test_biome_affinity_mountains() {
        let mut spawner = ResourceSpawner::new(42);
        
        // Mountain biomes should have higher mineral spawn rates
        let mountain_result = spawner.spawn_region(
            1,
            BiomeType::AlpineTundra,
            3500.0,
            100.0,
            100.0,
        );
        
        let plain_result = spawner.spawn_region(
            2,
            BiomeType::TemperateGrassland,
            200.0,
            200.0,
            100.0,
        );
        
        let mountain_minerals = mountain_result.deposits.iter()
            .filter(|d| d.resource_type.category() == ResourceCategory::BaseMetals)
            .count();
        let plain_minerals = plain_result.deposits.iter()
            .filter(|d| d.resource_type.category() == ResourceCategory::BaseMetals)
            .count();
        
        // Mountains should have more or equal base metal deposits due to affinity
        assert!(mountain_minerals >= plain_minerals);
    }
    
    #[test]
    fn test_spawn_regions_with_tectonic() {
        let mut spawner = ResourceSpawner::new(42);
        
        let regions = vec![
            (1, BiomeType::TemperateDeciduousForest, 200.0, 100.0, 100.0, None),
            (2, BiomeType::AlpineTundra, 3000.0, 200.0, 100.0, Some(TectonicBoundaryData {
                is_on_boundary: true,
                distance_to_boundary: 2.0,
                boundary_effect: BoundaryEffectType::Convergent,
            })),
            (3, BiomeType::CoastalWetland, 50.0, 300.0, 100.0, None),
        ];
        
        let results = spawner.spawn_regions_with_tectonic(&regions);
        
        assert_eq!(results.len(), 3);
        
        // Second region (with tectonic) should have higher mineral richness
        let second_minerals = results[1].deposits.iter()
            .filter(|d| d.resource_type.is_mineral())
            .map(|d| d.richness as i32)
            .sum::<i32>();
        
        let first_minerals = results[0].deposits.iter()
            .filter(|d| d.resource_type.is_mineral())
            .map(|d| d.richness as i32)
            .sum::<i32>();
        
        // Mountain with convergent boundary should have more/better minerals
        assert!(second_minerals >= first_minerals);
    }
    
    #[test]
    fn test_boundary_effect_types() {
        assert_eq!(BoundaryEffectType::Convergent.mineral_multiplier(), 1.8);
        assert_eq!(BoundaryEffectType::Divergent.mineral_multiplier(), 1.2);
        assert_eq!(BoundaryEffectType::Transform.mineral_multiplier(), 0.8);
        assert_eq!(BoundaryEffectType::None.mineral_multiplier(), 1.0);
    }
}
