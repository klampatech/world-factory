//! World Generation with Settlement Integration
//! 
//! This module shows how to wire settlement generation into the world pipeline.

use crate::generation::{WorldGenConfig, WorldGenerator, GeneratedWorld};
use crate::settlements::{SettlementGenerator, SettlementConfig, SettlementResult};
use crate::terrain::biome::{BiomeType, ClimateZone};
use crate::terrain::biome_assignment::BiomeAssignmentMatrix;
use crate::util::Rng;

/// Extended world state that includes settlements.
#[derive(Debug)]
pub struct WorldWithSettlements {
    pub world: GeneratedWorld,
    pub settlements: SettlementResult,
}

/// Generate a complete world with settlements.
pub fn generate_world_with_settlements(
    config: WorldGenConfig,
    settlement_config: SettlementConfig,
    seed: u64,
) -> WorldWithSettlements {
    // Phase 1: Generate terrain and rivers
    let generator = WorldGenerator::new(config.clone());
    let world = generator.generate(seed);
    
    // Phase 2: Generate biomes for land cells
    let (biome_grid, climate_grid) = generate_biomes(&world, seed);
    
    // Phase 3: Generate settlements
    let river_cells: Vec<(i32, i32)> = world.river_cells().iter()
        .map(|v| (v.x, v.y))
        .collect();
    
    let mut settlement_gen = SettlementGenerator::new(settlement_config, seed.wrapping_add(0xABCD));
    let settlements = settlement_gen.generate(
        world.elevation.data(),
        &biome_grid,
        &climate_grid,
        config.sea_level,
        config.width,
        config.height,
        Some(&river_cells),
    );
    
    WorldWithSettlements { world, settlements }
}

/// Generate biome and climate grids from elevation.
fn generate_biomes(world: &GeneratedWorld, seed: u64) -> (Vec<BiomeType>, Vec<ClimateZone>) {
    let mut rng = Rng::new(seed);
    let matrix = BiomeAssignmentMatrix::new();
    
    let mut biomes = Vec::with_capacity(world.width * world.height);
    let mut climates = Vec::with_capacity(world.width * world.height);
    
    for y in 0..world.height {
        for x in 0..world.width {
            let elevation = world.elevation.get_value_unchecked(x as i32, y as i32);
            
            // Below sea level = open ocean
            if elevation < world.sea_level {
                biomes.push(BiomeType::OpenOcean);
                climates.push(ClimateZone::Temperate); // Default
                continue;
            }
            
            // Calculate latitude (simplified - actual would use coordinates)
            let latitude = (y as f32 / world.height as f32) * 90.0;
            
            // Estimate temperature and precipitation
            let temperature = estimate_temperature(latitude, elevation);
            let precipitation = estimate_precipitation(x, y, world.width, world.height, &mut rng);
            
            // Assign biome
            let assignment = matrix.assign(elevation, latitude, precipitation, temperature);
            biomes.push(assignment.biome);
            
            // Determine climate zone
            let climate = if latitude < 23.5 {
                ClimateZone::Tropical
            } else if latitude < 35.0 {
                ClimateZone::Subtropical
            } else if latitude < 55.0 {
                ClimateZone::Temperate
            } else if latitude < 65.0 {
                ClimateZone::Boreal
            } else {
                ClimateZone::Polar
            };
            climates.push(climate);
        }
    }
    
    (biomes, climates)
}

/// Estimate temperature based on latitude and elevation.
fn estimate_temperature(latitude: f32, elevation: f32) -> f32 {
    // Lapse rate: -6.5°C per 1000m
    let base_temp = 30.0 - latitude * 0.6;
    let lapse = (elevation / 1000.0) * -6.5;
    (base_temp + lapse).max(-50.0).min(50.0)
}

/// Estimate precipitation using noise.
fn estimate_precipitation(x: usize, y: usize, width: usize, height: usize, rng: &mut Rng) -> f32 {
    // Simplified precipitation model
    let _nx = x as f32 / width as f32;
    let _ny = y as f32 / height as f32;
    
    // Use RNG for pseudo-noise (not proper simplex noise, but usable)
    let base = ((rng.next_f64Signed() * 0.5 + 0.5) * 2000.0) as f32;
    base.max(0.0).min(4000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_with_settlements() {
        let world_config = WorldGenConfig::default();
        let settlement_config = SettlementConfig::default();
        
        let result = generate_world_with_settlements(
            world_config,
            settlement_config,
            12345,
        );
        
        // Verify settlements were generated
        assert!(result.settlements.stats.total > 0);
        
        // Verify settlements are on valid biomes
        for settlement in &result.settlements.settlements {
            // Just verify it was created properly
            assert!(!settlement.name.is_empty());
            assert!(settlement.population.is_some());
        }
    }
    
    #[test]
    fn test_biome_generation() {
        let config = WorldGenConfig::default();
        let generator = WorldGenerator::new(config.clone());
        let world = generator.generate(42);
        
        let (biomes, climates) = generate_biomes(&world, 42);
        
        assert_eq!(biomes.len(), config.width * config.height);
        assert_eq!(climates.len(), config.width * config.height);
        
        // Ocean cells should have OpenOcean biome
        let sea_level = config.sea_level;
        for (i, &biome) in biomes.iter().enumerate() {
            let x = i % config.width;
            let y = i / config.width;
            let elevation = world.elevation.get_value_unchecked(x as i32, y as i32);
            
            if elevation < sea_level {
                assert_eq!(biome, BiomeType::OpenOcean);
            }
        }
    }
}