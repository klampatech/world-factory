//! Test to reproduce BUG-296-01: planet endpoint hangs
//!
//! This test exercises the get_world_planet handler logic directly
//! to identify the hanging component.

use world_factory::{
    generation::{WorldGenConfig, WorldGenerator},
    terrain::biome::BiomeType,
    terrain::biome_assignment::BiomeAssignmentMatrix,
    terrain::elevation_grid::ElevationGrid,
    util::Rng,
    world::{GeographyGenerator, World},
};
use std::time::{Duration, Instant};

/// Measure time for a closure
fn measure_time<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    println!("[TIMING] {} took {:?}", name, elapsed);
    result
}

#[test]
fn test_planet_generation_components() {
    println!("=== Testing planet generation components ===\n");

    // Use a known seed for reproducibility
    let seed = 12345u64;

    // Test 1: Terrain generation (ElevationGrid)
    println!("--- Test 1: Terrain generation ---");
    let terrain = measure_time("WorldGenerator::generate()", || {
        let mut config = WorldGenConfig::default();
        config.width = 256;
        config.height = 256;
        let gen = WorldGenerator::new(config);
        gen.generate(seed)
    });

    println!("  - Elevation grid size: {}x{}", terrain.width, terrain.height);
    println!("  - River count: {}", terrain.rivers.len());
    println!("  - Wonder count: {}", terrain.wonders.len());

    // Test 2: Biome generation
    println!("\n--- Test 2: Biome generation ---");
    let biome_count = measure_time("generate_biome_grid", || {
        generate_biome_grid_inner(&terrain, seed)
    });
    println!("  - Biome count: {}", biome_count.len());

    // Test 3: Geography generation
    println!("\n--- Test 3: Geography generation ---");
    let geo_count = measure_time("GeographyGenerator::generate_grid", || {
        let mut geo_gen = GeographyGenerator::new();
        let elevation_data = terrain.elevation.data();
        geo_gen.generate_grid(
            terrain.width,
            terrain.height,
            |x, y| elevation_data[y * terrain.width + x],
            &biome_count,
            &terrain.rivers,
            seed.wrapping_add(0xDEAD),
        )
    });
    println!("  - Geography count: {}", geo_count.len());

    println!("\n=== All tests completed successfully ===");
}

/// Inner biome grid generation (matches worlds.rs)
fn generate_biome_grid_inner(
    terrain: &world_factory::generation::GeneratedWorld,
    seed: u64,
) -> Vec<BiomeType> {
    use world_factory::terrain::biome_assignment::BiomeAssignmentMatrix;
    use world_factory::util::Rng;

    let mut rng = Rng::new(seed);
    let matrix = BiomeAssignmentMatrix::new();
    let mut biomes = Vec::with_capacity(terrain.width * terrain.height);

    println!("  Generating biomes for {}x{} grid...", terrain.width, terrain.height);

    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let elevation = terrain.elevation.get_value_unchecked(x as i32, y as i32);

            // Below sea level = open ocean
            if elevation < terrain.sea_level {
                biomes.push(BiomeType::OpenOcean);
                continue;
            }

            // Calculate latitude
            let latitude = (y as f32 / terrain.height as f32) * 90.0;

            // Estimate temperature and precipitation
            let base_temp = 30.0 - latitude * 0.6;
            let temperature = base_temp.max(-50.0).min(50.0);

            // Use RNG for pseudo-precipitation
            let base = ((rng.next_f64Signed() * 0.5 + 0.5) * 2000.0) as f32;
            let precipitation = base.max(0.0).min(4000.0);

            // Assign biome
            let assignment = matrix.assign(elevation, latitude, precipitation, temperature);
            biomes.push(assignment.biome);
        }
    }

    biomes
}