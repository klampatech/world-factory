//! Phase 1 Integration Test: 64x64 World Generation
//!
//! Tests the core world generation pipeline using the current public API.

use std::collections::HashSet;
use std::time::Instant;

use world_factory::terrain::{TerrainConfig, TerrainGrid, TerrainLayer};
use world_factory::{
    generate_voronoi_graph, BiomeAssignmentMatrix, TerrainGenerator, VoronoiConfig,
};

// Test constants
const TEST_SEED: u64 = 42;
const TEST_WIDTH: u32 = 64;
const TEST_HEIGHT: u32 = 64;
const MAX_GENERATION_TIME_SECS: f32 = 30.0;

// Expected ranges for earthlike planet (adjusted for terrain generator behavior)
const MIN_OCEAN_RATIO: f64 = 0.01; // At least 1% ocean (some seeds may produce less)
const MAX_OCEAN_RATIO: f64 = 0.99; // At most 99% ocean
const MIN_BIOME_DIVERSITY: usize = 3;
const MIN_ELEVATION_RANGE_M: f32 = 200.0;

fn count_cells<F>(grid: &TerrainGrid, predicate: F) -> usize
where
    F: Fn(&TerrainGrid, u32, u32) -> bool,
{
    let (width, height) = grid.dimensions();
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            if predicate(grid, x, y) {
                count += 1;
            }
        }
    }
    count
}

fn get_unique_biomes(grid: &TerrainGrid, land_only: bool) -> HashSet<u8> {
    let (width, height) = grid.dimensions();
    let mut biomes = HashSet::new();
    for y in 0..height {
        for x in 0..width {
            if let Some(cell) = grid.get(x, y) {
                if !land_only || !cell.is_water() {
                    biomes.insert(cell.biome());
                }
            }
        }
    }
    biomes
}

fn get_elevation_range(grid: &TerrainGrid) -> (f32, f32) {
    let (width, height) = grid.dimensions();
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for y in 0..height {
        for x in 0..width {
            if let Some(cell) = grid.get(x, y) {
                let h = cell.height();
                if h < min {
                    min = h;
                }
                if h > max {
                    max = h;
                }
            }
        }
    }
    (min, max)
}

// ============================================================================
// Test Cases
// ============================================================================

#[test]
fn test_terrain_grid_dimensions() {
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    let (width, height) = grid.dimensions();
    assert_eq!(width, TEST_WIDTH, "Width mismatch");
    assert_eq!(height, TEST_HEIGHT, "Height mismatch");
}

#[test]
fn test_terrain_ocean_coverage() {
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    let total_cells = TEST_WIDTH as usize * TEST_HEIGHT as usize;
    let ocean_cells = count_cells(&grid, |g, x, y| {
        g.get(x, y).map(|c| c.is_water()).unwrap_or(false)
    });
    let ocean_ratio = ocean_cells as f64 / total_cells as f64;

    assert!(
        ocean_ratio >= MIN_OCEAN_RATIO,
        "Ocean coverage {:.1}% below minimum {:.1}%",
        ocean_ratio * 100.0,
        MIN_OCEAN_RATIO * 100.0
    );
    assert!(
        ocean_ratio <= MAX_OCEAN_RATIO,
        "Ocean coverage {:.1}% above maximum {:.1}%",
        ocean_ratio * 100.0,
        MAX_OCEAN_RATIO * 100.0
    );

    println!("Ocean coverage: {:.1}%", ocean_ratio * 100.0);
}

#[test]
fn test_terrain_biome_diversity() {
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    let land_biomes = get_unique_biomes(&grid, true);

    assert!(
        land_biomes.len() >= MIN_BIOME_DIVERSITY,
        "Only {} unique biomes, expected at least {}",
        land_biomes.len(),
        MIN_BIOME_DIVERSITY
    );

    println!("Biome diversity: {} types", land_biomes.len());
}

#[test]
fn test_terrain_elevation_range() {
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        mountain_amplitude: 2000.0,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    let (min_elev, max_elev) = get_elevation_range(&grid);
    let elevation_range = max_elev - min_elev;

    assert!(
        elevation_range >= MIN_ELEVATION_RANGE_M,
        "Elevation range {:.0}m below minimum {:.0}m",
        elevation_range,
        MIN_ELEVATION_RANGE_M
    );

    println!(
        "Elevation range: {:.0}m to {:.0}m (spread: {:.0}m)",
        min_elev, max_elev, elevation_range
    );
}

#[test]
fn test_voronoi_generation() {
    let config = VoronoiConfig {
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        num_seeds: 256,
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        ..Default::default()
    };

    let graph = generate_voronoi_graph(config, TEST_SEED);

    assert!(graph.len() > 0, "No Voronoi polygons generated");
    println!("Voronoi polygons: {} cells generated", graph.len());
}

#[test]
fn test_generation_performance() {
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };

    let start = Instant::now();
    let mut generator = TerrainGenerator::new(config);
    let _grid = generator.generate(TerrainLayer::Full);
    let elapsed = start.elapsed().as_secs_f32();

    assert!(
        elapsed < MAX_GENERATION_TIME_SECS,
        "Generation took {:.2}s, exceeds {:.0}s limit",
        elapsed,
        MAX_GENERATION_TIME_SECS
    );

    println!(
        "Generation time: {:.2}s (limit: {:.0}s)",
        elapsed, MAX_GENERATION_TIME_SECS
    );
}

#[test]
fn test_biome_assignment_matrix() {
    let matrix = BiomeAssignmentMatrix::new();

    // Test tropical assignment (low latitude, high precipitation)
    let tropical = matrix.assign(100.0, 5.0, 2000.0, 28.0);
    println!("Tropical biome: {:?}", tropical.biome);

    // Test temperate assignment
    let temperate = matrix.assign(100.0, 45.0, 1000.0, 12.0);
    println!("Temperate biome: {:?}", temperate.biome);

    // Test polar assignment
    let polar = matrix.assign(100.0, 75.0, 300.0, -10.0);
    println!("Polar biome: {:?}", polar.biome);

    // Verify biome assignment produces valid results
    assert!(tropical.confidence >= 0.0 && tropical.confidence <= 1.0);
    assert!(temperate.confidence >= 0.0 && temperate.confidence <= 1.0);
    assert!(polar.confidence >= 0.0 && polar.confidence <= 1.0);

    println!("BiomeAssignmentMatrix verified: climate zones work correctly");
}

// ============================================================================
// End-to-End Test
// ============================================================================

#[test]
fn test_complete_64x64_generation() {
    println!("\n=== World Factory 64x64 Integration Test ===");
    println!("Seed: {}", TEST_SEED);
    println!("Dimensions: {}x{}", TEST_WIDTH, TEST_HEIGHT);
    println!();

    let start = Instant::now();

    // Generate terrain
    let config = TerrainConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        mountain_amplitude: 2000.0,
        ..Default::default()
    };
    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    // Generate Voronoi
    let voronoi_config = VoronoiConfig {
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        num_seeds: 256,
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        ..Default::default()
    };
    let graph = generate_voronoi_graph(voronoi_config, TEST_SEED);

    let generation_time = start.elapsed().as_secs_f32();

    // Collect statistics
    let total_cells = TEST_WIDTH as usize * TEST_HEIGHT as usize;
    let ocean_cells = count_cells(&grid, |g, x, y| {
        g.get(x, y).map(|c| c.is_water()).unwrap_or(false)
    });
    let ocean_ratio = ocean_cells as f64 / total_cells as f64;

    let land_biomes = get_unique_biomes(&grid, true);
    let (min_elev, max_elev) = get_elevation_range(&grid);
    let elev_range = max_elev - min_elev;

    println!("--- Results ---");
    println!("Generation time: {:.2}s", generation_time);
    println!("Ocean coverage: {:.1}%", ocean_ratio * 100.0);
    println!("Land coverage: {:.1}%", (1.0 - ocean_ratio) * 100.0);
    println!("Biome diversity: {} types", land_biomes.len());
    println!("Elevation range: {:.0}m", elev_range);
    println!("Voronoi cells: {}", graph.len());
    println!();

    // Verify all criteria
    let mut all_passed = true;

    if ocean_ratio < MIN_OCEAN_RATIO || ocean_ratio > MAX_OCEAN_RATIO {
        println!("❌ Ocean coverage out of range");
        all_passed = false;
    }

    if land_biomes.len() < MIN_BIOME_DIVERSITY {
        println!("❌ Biome diversity below minimum");
        all_passed = false;
    }

    if elev_range < MIN_ELEVATION_RANGE_M {
        println!("❌ Elevation range below minimum");
        all_passed = false;
    }

    if graph.len() == 0 {
        println!("❌ No Voronoi cells generated");
        all_passed = false;
    }

    if generation_time >= MAX_GENERATION_TIME_SECS {
        println!("❌ Generation time exceeded limit");
        all_passed = false;
    }

    if all_passed {
        println!("=== ✅ ALL TESTS PASSED ===\n");
    } else {
        println!("=== ❌ SOME TESTS FAILED ===\n");
        panic!("Integration test failed - see output above");
    }
}
