//! World Generation Orchestrator
//! 
//! Coordinates terrain, hydrology, and other generators to produce complete worlds.
pub mod voronoi;
pub mod world_with_settlements;

// Re-export Voronoi types for convenience
pub use voronoi::{VoronoiConfig, VoronoiGenerator, VoronoiResult, BoundaryMode};
pub use voronoi::{generate_voronoi_graph, quick_voronoi};


use crate::terrain::{TerrainGenerator, TerrainConfig, ElevationGrid};
use crate::terrain::biome::BiomeType;
use crate::terrain::natural_wonders::{NaturalWonderSpawner, WonderSpawnConfig};
use crate::hydro::{RiverGenerator, RiverConfig, River};
use crate::util::Rng;

/// Configuration for world generation
#[derive(Debug, Clone)]
pub struct WorldGenConfig {
    pub width: usize,
    pub height: usize,
    pub sea_level: f32,
    pub terrain: TerrainConfig,
    pub rivers: RiverConfig,
    /// Natural wonder spawn configuration (None to disable wonders)
    pub wonders: Option<WonderSpawnConfig>,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            sea_level: 0.4,
            terrain: TerrainConfig::default(),
            rivers: RiverConfig::default(),
            wonders: Some(WonderSpawnConfig::default()),
        }
    }
}

/// Generated world state
#[derive(Debug, Clone)]
pub struct GeneratedWorld {
    pub width: usize,
    pub height: usize,
    pub sea_level: f32,
    pub elevation: ElevationGrid,
    pub rivers: Vec<River>,
    pub wonders: Vec<crate::terrain::natural_wonders::NaturalWonder>,
}

impl GeneratedWorld {
    /// Get all cells that are below sea level (water)
    pub fn water_cells(&self) -> Vec<crate::util::Vec2<i32>> {
        let mut cells = Vec::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.elevation.get_value_unchecked(x as i32, y as i32) < self.sea_level {
                    cells.push(crate::util::Vec2::new(x as i32, y as i32));
                }
            }
        }
        
        cells
    }
    
    /// Get all cells that are above sea level (land)
    pub fn land_cells(&self) -> Vec<crate::util::Vec2<i32>> {
        let mut cells = Vec::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.elevation.get_value_unchecked(x as i32, y as i32) >= self.sea_level {
                    cells.push(crate::util::Vec2::new(x as i32, y as i32));
                }
            }
        }
        
        cells
    }
    
    /// Get all cells that are below sea level (ocean)
    pub fn ocean_cells(&self) -> Vec<crate::util::Vec2<i32>> {
        self.water_cells()
    }
    
    /// Get land cells adjacent to ocean (coastline)
    pub fn coastline_cells(&self) -> Vec<crate::util::Vec2<i32>> {
        use crate::util::Direction;
        let mut coast = Vec::new();
        let sea_level = self.sea_level;
        
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let elev = self.elevation.get_value_unchecked(x, y);
                
                // Is land but adjacent to water
                if elev >= sea_level {
                    for dir in Direction::cardinal() {
                        let neighbor = crate::util::Vec2::new(x, y) + dir.delta();
                        if self.elevation.is_valid(neighbor.x, neighbor.y) {
                            let neighbor_elev = self.elevation.get_value_unchecked(neighbor.x, neighbor.y);
                            if neighbor_elev < sea_level {
                                coast.push(crate::util::Vec2::new(x, y));
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        coast
    }
    
    /// Calculate land percentage
    pub fn land_percentage(&self) -> f32 {
        let land_count = self.land_cells().len() as f32;
        let total = (self.width * self.height) as f32;
        land_count / total
    }
    
    /// Calculate ocean percentage
    pub fn ocean_percentage(&self) -> f32 {
        1.0 - self.land_percentage()
    }
    
    /// Get river cells (for rendering/collision)
    pub fn river_cells(&self) -> Vec<crate::util::Vec2<i32>> {
        let mut cells = Vec::new();
        for river in &self.rivers {
            cells.extend_from_slice(&river.cells);
        }
        cells
    }
    
    /// Check if a cell contains a river
    pub fn has_river_at(&self, x: i32, y: i32) -> bool {
        let pos = crate::util::Vec2::new(x, y);
        self.rivers.iter().any(|r| r.cells.contains(&pos))
    }
}

/// World generator orchestrator
#[derive(Debug)]
pub struct WorldGenerator {
    config: WorldGenConfig,
}

impl WorldGenerator {
    pub fn new(config: WorldGenConfig) -> Self {
        Self { config }
    }
    
    /// Get carrying capacity for a given biome type.
    /// Per WOR-95 2.2.1: population per polygon per year baseline.
    pub fn get_carrying_capacity(&self, biome: BiomeType) -> u64 {
        crate::types::Settlement::calculate_carrying_capacity(biome)
    }
    
    /// Generate a complete world from seed
    pub fn generate(&self, seed: u64) -> GeneratedWorld {
        let mut rng = Rng::new(seed);
        
        // Generate terrain - use elevation grid directly for river generation
        let mut terrain_config = self.config.terrain.clone();
        terrain_config.width = self.config.width as u32;
        terrain_config.height = self.config.height as u32;
        
        let mut terrain_gen = TerrainGenerator::new(terrain_config);
        let elevation = terrain_gen.generate_elevation_grid();
        
        // Generate rivers from elevation grid
        let mut river_gen = RiverGenerator::new(self.config.rivers.clone());
        let rivers = river_gen.generate_rivers(&elevation, self.config.sea_level, &mut rng);
        
        // Apply river erosion to elevation grid
        river_gen.apply_erosion(&mut elevation.clone());
        
        // Generate natural wonders if configured
        let wonders = if let Some(ref wonder_config) = self.config.wonders {
            let mut wonder_spawner = NaturalWonderSpawner::with_config(
                seed,
                self.config.width as f32,
                self.config.height as f32,
                wonder_config.clone(),
            );
            let terrain_data = crate::terrain::natural_wonders::TerrainDataForSpawning::from_elevation_grid(
                &elevation,
                self.config.width as u32,
                self.config.height as u32,
            );
            wonder_spawner.spawn_wonders(&terrain_data).wonders
        } else {
            Vec::new()
        };
        
        GeneratedWorld {
            width: self.config.width,
            height: self.config.height,
            sea_level: self.config.sea_level,
            elevation,
            rivers,
            wonders,
        }
    }
    
    /// Generate with separate terrain/river phases for streaming
    pub fn generate_phases(&self, seed: u64) -> (ElevationGrid, Vec<River>) {
        let mut rng = Rng::new(seed);
        
        let mut terrain_config = self.config.terrain.clone();
        terrain_config.width = self.config.width as u32;
        terrain_config.height = self.config.height as u32;
        
        let mut terrain_gen = TerrainGenerator::new(terrain_config);
        let elevation = terrain_gen.generate_elevation_grid();
        
        let mut river_gen = RiverGenerator::new(self.config.rivers.clone());
        let rivers = river_gen.generate_rivers(&elevation, self.config.sea_level, &mut rng);
        
        (elevation, rivers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_generation() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        
        assert_eq!(world.width, 256);
        assert_eq!(world.height, 256);
        assert!(world.elevation.width > 0);
    }

    #[test]
    fn test_deterministic_generation() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        let world1 = generator.generate(12345);
        let world2 = generator.generate(12345);
        
        // Same seed should produce same world
        assert_eq!(world1.rivers.len(), world2.rivers.len());
        
        for (r1, r2) in world1.rivers.iter().zip(world2.rivers.iter()) {
            assert_eq!(r1.path.len(), r2.path.len());
        }
    }

    #[test]
    fn test_land_water_classification() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        let land = world.land_cells();
        let water = world.water_cells();
        
        let total = land.len() + water.len();
        assert_eq!(total, world.width * world.height);
    }
    
    #[test]
    fn test_coastline_detection() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        let coast = world.coastline_cells();
        
        // Coastline should be a subset of land
        let land: Vec<_> = world.land_cells();
        for cell in &coast {
            assert!(land.contains(cell), "Coastline cell should be land");
        }
    }
    
    #[test]
    fn test_river_detection() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        let river_cells = world.river_cells();
        
        // Should be able to detect rivers
        if let Some(first_river) = world.rivers.first() {
            if let Some(cell) = first_river.cells.first() {
                assert!(world.has_river_at(cell.x, cell.y));
            }
        }
    }
    
    #[test]
    fn test_get_carrying_capacity() {
        // Per WOR-95 2.2.1: carrying capacity by biome
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config);
        
        use crate::terrain::biome::BiomeType;
        
        // High capacity biomes
        assert_eq!(generator.get_carrying_capacity(BiomeType::TropicalRainforest), 7000);
        assert_eq!(generator.get_carrying_capacity(BiomeType::TemperateRainforest), 6000);
        assert_eq!(generator.get_carrying_capacity(BiomeType::TemperateDeciduousForest), 5000);
        
        // Medium capacity
        assert_eq!(generator.get_carrying_capacity(BiomeType::TropicalSavanna), 3000);
        assert_eq!(generator.get_carrying_capacity(BiomeType::BorealForest), 1500);
        
        // Low capacity
        assert_eq!(generator.get_carrying_capacity(BiomeType::HotDesert), 200);
        assert_eq!(generator.get_carrying_capacity(BiomeType::Tundra), 300);
        
        // Uninhabitable
        assert_eq!(generator.get_carrying_capacity(BiomeType::OpenOcean), 0);
        assert_eq!(generator.get_carrying_capacity(BiomeType::Arctic), 0);
    }
    
    #[test]
    fn test_natural_wonders_generation() {
        // Use smaller config for faster testing
        let mut config = WorldGenConfig::default();
        config.width = 64;
        config.height = 64;
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        
        // World should have wonders (density is 0.3 by default)
        assert!(!world.wonders.is_empty(), "World should have natural wonders");
        
        // Each wonder should have valid properties
        for wonder in &world.wonders {
            assert!(!wonder.name.is_empty(), "Wonder should have a name");
            assert!(wonder.x >= 0.0 && wonder.x <= 64.0, "Wonder x should be within world bounds");
            assert!(wonder.y >= 0.0 && wonder.y <= 64.0, "Wonder y should be within world bounds");
            assert!(!wonder.bonuses.is_empty(), "Wonder should have at least one bonus");
        }
        
        // Test that same seed produces same wonders
        let world2 = generator.generate(42);
        assert_eq!(world.wonders.len(), world2.wonders.len(), "Same seed should produce same wonder count");
    }
    
    #[test]
    fn test_wonders_disabled() {
        let mut config = WorldGenConfig::default();
        config.width = 64;
        config.height = 64;
        config.wonders = None; // Disable wonders
        let generator = WorldGenerator::new(config);
        
        let world = generator.generate(42);
        
        // World should have no wonders when disabled
        assert!(world.wonders.is_empty(), "World should have no wonders when disabled");
    }
}
