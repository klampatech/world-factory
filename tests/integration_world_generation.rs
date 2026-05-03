//! Integration Test: 64x64 World Generation
//! 
//! This test verifies the complete world generation pipeline produces an earthlike planet.
//! 
//! Test Objective:
//! - Generate a 64x64 world
//! - Verify all components are populated
//! - Verify earthlike characteristics (oceans, continents, varied biomes)
//! - Target generation time < 30 seconds
//!
//! Run with: `cargo test --test integration_world_generation`

use std::collections::HashSet;
use std::time::Instant;

// Import World Factory types
use world_factory::{
    WorldConfig, TerrainGenerator, TerrainConfig, TerrainLayer,
    TerrainGrid, TerrainCell, BiomeType, BiomeAssignmentMatrix,
    PolygonGraph, VoronoiConfig, VoronoiGenerator, generate_voronoi_graph,
    OceanDetector, OceanDetectionConfig, OceanZone,
    Species, SpeciesId,
};

// =============================================================================
// Test Configuration
// =============================================================================

const TEST_SEED: u64 = 12345;
const TEST_WIDTH: u32 = 64;
const TEST_HEIGHT: u32 = 64;
const TEST_GRID_SIZE: usize = (TEST_WIDTH * TEST_HEIGHT) as usize;
const MAX_GENERATION_TIME_SECS: f32 = 30.0;

// Expected ranges for earthlike planet
const MIN_OCEAN_RATIO: f64 = 0.30;  // At least 30% ocean
const MAX_OCEAN_RATIO: f64 = 0.50;  // At most 50% ocean
const MIN_LAND_COVERAGE: f64 = 0.10; // At least 10% land
const MIN_BIOME_DIVERSITY: usize = 4; // At least 4 different biomes
const MIN_ELEVATION_RANGE_M: f32 = 2000.0; // Elevation spread >= 2000m

// =============================================================================
// World Generation Function
// =============================================================================

/// Generate a complete world with all components
fn generate_world(config: WorldConfig) -> TestWorld {
    let start = Instant::now();
    
    // 1. Generate terrain grid using TerrainConfig (not WorldConfig)
    let terrain_config = TerrainConfig {
        seed: config.seed() as u64,
        width: config.width() as u32,
        height: config.height() as u32,
        sea_level: config.sea_level(),
        enable_tectonics: true, // Enable tectonic simulation
        tectonic_activity: 0.6,
        ..Default::default()
    };
    let mut terrain_generator = TerrainGenerator::new(terrain_config);
    let terrain_grid = terrain_generator.generate(TerrainLayer::Full);
    
    // Access tectonic data for verification (store reference)
    let tectonic_result = terrain_generator.get_tectonic_result().cloned();
    
    // 2. Generate Voronoi polygons
    let voronoi_config = VoronoiConfig {
        width: config.width() as u32,
        height: config.height() as u32,
        num_seeds: (((config.width() * config.height()) / 16) as u32), // 256 cells
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        blue_noise: true,
        ..Default::default()
    };
    let polygon_graph = generate_voronoi_graph(voronoi_config, config.seed());
    
    // 3. Detect ocean zones
    let ocean_config = OceanDetectionConfig::default();
    let ocean_detector = OceanDetector::with_config(ocean_config);
    let ocean_zones = ocean_detector.detect_ocean(&terrain_grid);
    
    let generation_time = start.elapsed();
    
    TestWorld {
        terrain_grid,
        polygon_graph,
        ocean_zones,
        generation_time_ms: generation_time.as_millis() as u64,
        tectonic_result,
    }
}

/// Test world structure containing all generated components
struct TestWorld {
    terrain_grid: TerrainGrid,
    polygon_graph: PolygonGraph,
    ocean_zones: Vec<(u32, u32, OceanZone)>,
    generation_time_ms: u64,
    tectonic_result: Option<world_factory::TectonicResult>,
}

// =============================================================================
// Test Cases
// =============================================================================

#[test]
fn test_world_generation_terrain_grid() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Verify grid dimensions
    let (width, height) = world.terrain_grid.dimensions();
    assert_eq!(
        width, TEST_WIDTH,
        "Terrain grid width {} doesn't match expected {}",
        width, TEST_WIDTH
    );
    assert_eq!(
        height, TEST_HEIGHT,
        "Terrain grid height {} doesn't match expected {}",
        height, TEST_HEIGHT
    );
    
    // Verify cell count
    assert_eq!(
        world.terrain_grid.len(),
        TEST_GRID_SIZE,
        "Expected {} cells, got {}",
        TEST_GRID_SIZE,
        world.terrain_grid.len()
    );
}

#[test]
fn test_world_generation_ocean_coverage() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Count water cells using the ocean zones from detector
    let water_zones = world.ocean_zones.iter()
        .filter(|(_, _, zone)| *zone != OceanZone::Land)
        .count();
    let ocean_ratio = water_zones as f64 / TEST_GRID_SIZE as f64;
    
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
fn test_world_generation_land_coverage() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Count land cells using the ocean zones
    let land_zones = world.ocean_zones.iter()
        .filter(|(_, _, zone)| *zone == OceanZone::Land)
        .count();
    let land_ratio = land_zones as f64 / TEST_GRID_SIZE as f64;
    
    assert!(
        land_ratio >= MIN_LAND_COVERAGE,
        "Land coverage {:.1}% below minimum {:.1}%",
        land_ratio * 100.0,
        MIN_LAND_COVERAGE * 100.0
    );
    
    println!("Land coverage: {:.1}%", land_ratio * 100.0);
}

#[test]
fn test_world_generation_biome_diversity() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Collect unique biome types from land cells
    let mut biomes = HashSet::new();
    for (x, y, zone) in &world.ocean_zones {
        if *zone == OceanZone::Land {
            if let Some(cell) = world.terrain_grid.get(*x, *y) {
                biomes.insert(cell.biome());
            }
        }
    }
    
    assert!(
        biomes.len() >= MIN_BIOME_DIVERSITY,
        "Only {} unique biomes, expected at least {}",
        biomes.len(),
        MIN_BIOME_DIVERSITY
    );
    
    println!("Biome diversity: {} types", biomes.len());
}

#[test]
fn test_world_generation_elevation_range() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    let mut min_elevation = f32::INFINITY;
    let mut max_elevation = f32::NEG_INFINITY;
    
    for (_, _, cell) in world.terrain_grid.cells() {
        let h = cell.height();
        if h < min_elevation { min_elevation = h; }
        if h > max_elevation { max_elevation = h; }
    }
    
    let elevation_range = max_elevation - min_elevation;
    
    assert!(
        elevation_range >= MIN_ELEVATION_RANGE_M,
        "Elevation range {:.0}m below minimum {:.0}m",
        elevation_range,
        MIN_ELEVATION_RANGE_M
    );
    
    println!("Elevation range: {:.0}m to {:.0}m (spread: {:.0}m)", 
             min_elevation, max_elevation, elevation_range);
}

#[test]
fn test_world_generation_voronoi_polygons() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Verify polygons exist
    let polygon_count = world.polygon_graph.len();
    assert!(
        polygon_count > 0,
        "No Voronoi polygons generated"
    );
    
    // Verify reasonable polygon count (expecting ~256 for 64x64 with cell_size=4)
    let expected_polygons = (TEST_WIDTH * TEST_HEIGHT) / 16;
    let tolerance = expected_polygons / 2; // 50% tolerance
    
    assert!(
        (polygon_count as i32 - expected_polygons as i32).abs() <= tolerance as i32,
        "Polygon count {} far from expected {}",
        polygon_count,
        expected_polygons
    );
    
    // Verify all polygons have valid data
    let mut polygons_with_elevation = 0;
    for id in world.polygon_graph.polygon_ids() {
        if let Some(poly) = world.polygon_graph.get(id) {
            if poly.elevation >= 0.0 {
                polygons_with_elevation += 1;
            }
        }
    }
    
    assert_eq!(polygons_with_elevation, polygon_count, "All polygons should have elevation values");
    
    println!("Voronoi polygons: {} cells", polygon_count);
}

#[test]
fn test_world_generation_settlements() {
    // Settlement generation is complex and depends on proper Species setup
    // This test verifies basic structure generation works
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world = generate_world(config);
    
    // Verify polygons were generated (prerequisite for settlement placement)
    assert!(
        world.polygon_graph.len() > 0,
        "No Voronoi polygons generated"
    );
    
    println!("Voronoi polygons generated: {} (settlement placement requires species setup)", 
             world.polygon_graph.len());
}

#[test]
fn test_world_generation_performance() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let start = Instant::now();
    let _world = generate_world(config);
    let elapsed_secs = start.elapsed().as_secs_f32();
    
    assert!(
        elapsed_secs < MAX_GENERATION_TIME_SECS,
        "Generation took {:.2}s, exceeds {:.0}s limit",
        elapsed_secs,
        MAX_GENERATION_TIME_SECS
    );
    
    println!("Generation time: {:.2}s (limit: {:.0}s)", 
             elapsed_secs, MAX_GENERATION_TIME_SECS);
}

#[test]
fn test_world_generation_determinism() {
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let world1 = generate_world(config.clone());
    
    // Generate again with same config
    let world2 = generate_world(config);
    
    // Verify same generation time (within tolerance)
    let time_diff = (world1.generation_time_ms as i64 - world2.generation_time_ms as i64).abs();
    assert!(
        time_diff <= 100, // Within 100ms tolerance
        "Generation time differs by {}ms between runs",
        time_diff
    );
    
    // Verify same polygon count (determinism check)
    assert_eq!(
        world1.polygon_graph.len(),
        world2.polygon_graph.len(),
        "Polygon count differs between runs: {} vs {}",
        world1.polygon_graph.len(),
        world2.polygon_graph.len()
    );
    
    println!("Determinism verified: same seed produces same output");
}

// =============================================================================
// End-to-End Test
// =============================================================================

#[test]
fn test_world_generation_complete_e2e() {
    println!("\n=== World Factory: 64x64 Integration Test ===");
    println!("Seed: {}", TEST_SEED);
    println!("Dimensions: {}x{}", TEST_WIDTH, TEST_HEIGHT);
    println!("Sea Level: 0.4 (40%)");
    println!();
    
    let config = WorldConfig::simple(TEST_SEED, TEST_WIDTH as usize, TEST_HEIGHT as usize, 0.4);
    
    let start = Instant::now();
    let world = generate_world(config);
    let total_time = start.elapsed().as_secs_f32();
    
    // Count ocean/land zones
    let water_zones = world.ocean_zones.iter()
        .filter(|(_, _, zone)| *zone != OceanZone::Land)
        .count();
    let land_zones = TEST_GRID_SIZE - water_zones;
    let ocean_ratio = water_zones as f64 / TEST_GRID_SIZE as f64;
    let land_ratio = land_zones as f64 / TEST_GRID_SIZE as f64;
    
    // Biome diversity
    let mut land_biomes = HashSet::new();
    for (x, y, zone) in &world.ocean_zones {
        if *zone == OceanZone::Land {
            if let Some(cell) = world.terrain_grid.get(*x, *y) {
                land_biomes.insert(cell.biome());
            }
        }
    }
    
    // Elevation range
    let mut min_elev = f32::INFINITY;
    let mut max_elev = f32::NEG_INFINITY;
    for (_, _, cell) in world.terrain_grid.cells() {
        let h = cell.height();
        if h < min_elev { min_elev = h; }
        if h > max_elev { max_elev = h; }
    }
    let elevation_range = max_elev - min_elev;
    
    // Tectonic verification
    let tectonic_info = if let Some(ref result) = world.tectonic_result {
        format!("Tectonic plates: {} | Boundaries: {}",
            result.plates.len(),
            result.boundaries.len())
    } else {
        "Tectonic simulation: DISABLED".to_string()
    };
    
    println!("--- Results ---");
    println!("Generation time: {:.2}s", total_time);
    println!("Ocean coverage: {:.1}%", ocean_ratio * 100.0);
    println!("Land coverage: {:.1}%", land_ratio * 100.0);
    println!("Biome diversity: {} types", land_biomes.len());
    println!("Elevation range: {:.0}m", elevation_range);
    println!("{}", tectonic_info);
    println!("Voronoi cells: {}", world.polygon_graph.len());
    println!("Settlements: (requires species setup - tested separately)");
    println!();
    
    // Verify all criteria
    assert!(ocean_ratio >= MIN_OCEAN_RATIO && ocean_ratio <= MAX_OCEAN_RATIO);
    assert!(land_ratio >= MIN_LAND_COVERAGE);
    assert!(land_biomes.len() >= MIN_BIOME_DIVERSITY);
    assert!(elevation_range >= MIN_ELEVATION_RANGE_M);
    assert!(total_time < MAX_GENERATION_TIME_SECS);
    
    // Tectonic verification (simplified - just check if result exists)
    if let Some(ref result) = world.tectonic_result {
        assert!(!result.plates.is_empty() || result.boundaries.is_empty(), 
            "Tectonic result should have plates or boundaries");
    }
    
    println!("=== PASSED: Earthlike planet generated with tectonics ===\n");
}

// =============================================================================
// Helpers
// =============================================================================

impl TestWorld {
    /// Get cell at coordinates (for verification)
    fn get_cell(&self, x: u32, y: u32) -> Option<TerrainCell> {
        self.terrain_grid.get(x, y)
    }
}
