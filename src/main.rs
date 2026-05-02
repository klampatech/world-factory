//! World Factory CLI
//! 
//! Command-line interface for world generation.

use world_factory::terrain::{TerrainGenerator, TerrainConfig, TerrainLayer};

fn main() {
    println!("World Factory - Procedural World Generator");
    println!("=========================================\n");
    
    // Create terrain generator with default configuration
    let config = TerrainConfig {
        seed: 42,
        width: 128,
        height: 128,
        ..Default::default()
    };
    
    println!("Generating world with seed {}...", config.seed);
    
    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);
    
    println!("Generated terrain grid: {}x{} cells", grid.dimensions().0, grid.dimensions().1);
    println!("Memory usage: {} bytes", grid.memory_usage());
    
    // Sample some cells
    println!("\nSample biomes:");
    for y in [0, 32, 64, 96, 128] {
        for x in [0, 32, 64, 96] {
            if let Some(cell) = grid.get(x, y) {
                let biome = match cell.biome() {
                    0 => "Tropical Rainforest",
                    1 => "Tropical Seasonal Forest",
                    2 => "Tropical Savanna",
                    3 => "Subtropical Desert",
                    4 => "Temperate Forest",
                    5 => "Boreal Taiga",
                    6 => "Tundra",
                    7 => "Arctic",
                    8 => "Hot Desert",
                    9 => "Ocean",
                    _ => "Unknown",
                };
                println!("  ({:3}, {:3}): {} at {}m", x, y, biome, cell.height() as u32);
            }
        }
    }
    
    println!("\nWorld generation complete!");
}
