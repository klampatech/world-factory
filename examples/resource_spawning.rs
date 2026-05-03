//! Example: Resource Spawning Integration with World Generation
//!
//! This example demonstrates how to integrate the ResourceSpawner
//! with the Voronoi-based world generation pipeline.

use world_factory::{
    // Voronoi generation
    quick_voronoi, PolygonGraph,
    // Terrain types
    BiomeType,
    // Resource spawning
    ResourceSpawner, ResourceSpawnConfig,
};

/// Example: Generate a world with resources
pub fn generate_world_with_resources(
    width: u32,
    height: u32,
    seed: u64,
) -> WorldWithResources {
    // Step 1: Generate Voronoi polygons
    let mut graph = quick_voronoi(width, height, seed);
    
    // Step 2: For each polygon, determine biome (simplified)
    // In a full implementation, this would use the BiomeAssignmentMatrix
    let regions = generate_region_data(&graph, seed);
    
    // Step 3: Create resource spawner
    let config = ResourceSpawnConfig {
        enable_fantasy: true,
        enable_legendary: true,
        density: 0.5,
        max_per_region: 8,
        cluster_min_distance: 100.0,
        clustering: 0.3,
        base_rate: 1.0,
    };
    let mut spawner = ResourceSpawner::with_config(seed, config);
    
    // Step 4: Spawn resources for each region
    let resource_spawns = spawner.spawn_regions(&regions);
    
    // Step 5: Calculate statistics
    let stats = spawner.calculate_stats(&resource_spawns);
    
    WorldWithResources {
        polygon_graph: graph,
        resource_spawns,
        stats,
    }
}

/// Generate region data from polygon graph
fn generate_region_data(graph: &PolygonGraph, seed: u64) -> Vec<(u32, BiomeType, f32, f32, f32)> {
    use world_factory::Polygon;
    
    graph.polygon_ids()
        .enumerate()
        .filter_map(|(i, id)| {
            graph.get(id).map(|poly| {
                let elevation = poly.elevation;
                // Simplified biome assignment based on position
                let biome = BiomeType::TemperateDeciduousForest;
                (i as u32, biome, elevation, 0.0, 0.0)
            })
        })
        .collect()
}

/// World data with resources
pub struct WorldWithResources {
    pub polygon_graph: PolygonGraph,
    pub resource_spawns: Vec<world_factory::RegionResourceSpawn>,
    pub stats: world_factory::ResourceSpawnStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_with_resources() {
        let result = generate_world_with_resources(64, 64, 42);
        
        assert!(!result.resource_spawns.is_empty());
        assert!(result.stats.total_deposits > 0);
        assert!(result.stats.total_world_value > 0.0);
    }
    
    #[test]
    fn test_deterministic_resource_spawning() {
        let result1 = generate_world_with_resources(32, 32, 12345);
        let result2 = generate_world_with_resources(32, 32, 12345);
        
        assert_eq!(result1.resource_spawns.len(), result2.resource_spawns.len());
    }
}

fn main() {
    // Generate world with resources
    let result = generate_world_with_resources(64, 64, 42);
    
    println!("World Factory - Resource Spawning Demo");
    println!("======================================");
    println!("Generated {} resource deposits", result.resource_spawns.len());
    println!("Total world value: {:.2}", result.stats.total_world_value);
    println!("Total deposits: {}", result.stats.total_deposits);
}
