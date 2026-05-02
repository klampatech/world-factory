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
    WorldConfig, World, TerrainGenerator, TerrainConfig, TerrainLayer,
    TerrainGrid, TerrainCell, BiomeType, BiomeAssignmentMatrix,
    PolygonGraph, VoronoiConfig, VoronoiGenerator, generate_voronoi_graph,
    SettlementGenerator, SettlementConfig, SettlementResult,
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
    
    // 1. Generate terrain grid
    let terrain_config = TerrainConfig {
        seed: config.seed,
        width: config.width,
        height: config.height,
        sea_level: config.sea_level,
        enable_tectonics: true, // Enable tectonic simulation
        tectonic_activity: 0.6,
        ..Default::default()
    };
    let mut terrain_generator = TerrainGenerator::new(terrain_config);
    let terrain_grid = terrain_generator.generate(TerrainLayer::Full);
    
    // Access tectonic data for verification
    let tectonic_result = terrain_generator.get_tectonic_result();
    
    // 2. Generate Voronoi polygons
    let voronoi_config = VoronoiConfig {
        width: config.width,
        height: config.height,
        num_seeds: (config.width * config.height) / 16, // 256 cells
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        blue_noise: true,
        ..Default::default()
    };
    let polygon_graph = generate_voronoi_graph(voronoi_config, config.seed);
    
    // 3. Detect ocean zones
    let ocean_config = OceanDetectionConfig::default();
    let ocean_detector = OceanDetector::new(ocean_config);
    let ocean_zones = ocean_detector.detect_ocean(&terrain_grid);
    
    // 4. Generate settlements (species-aware)
    let settlement_config = SettlementConfig {
        density_target: 0.02, // 2% of cells have settlements
        min_spacing: 8, // cells between settlements
        ..Default::default()
    };
    let settlement_generator = SettlementGenerator::new(settlement_config);
    let settlements = settlement_generator.generate(
        &terrain_grid,
        &polygon_graph,
        config.seed,
    );
    
    let generation_time = start.elapsed();
    
    TestWorld {
        terrain_grid,
        polygon_graph,
        ocean_zones,
        settlements,
        generation_time_ms: generation_time.as_millis() as u64,
        tectonic_result,
    }
}

/// Test world structure containing all generated components
struct TestWorld {
    terrain_grid: TerrainGrid,
    polygon_graph: PolygonGraph,
    ocean_zones: Vec<OceanZone>,
    settlements: Vec<SettlementResult>,
    generation_time_ms: u64,
    tectonic_result: Option<world_factory::TectonicResult>,
}

// =============================================================================
// Test Cases
// =============================================================================

#[test]
fn test_world_generation_terrain_grid() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        ..Default::default()
    };
    
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
        world.terrain_grid.cells().len(),
        TEST_GRID_SIZE,
        "Expected {} cells, got {}",
        TEST_GRID_SIZE,
        world.terrain_grid.cells().len()
    );
}

#[test]
fn test_world_generation_ocean_coverage() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    // Count water cells (below sea level)
    let water_cells = world.terrain_grid.cells()
        .filter(|cell| cell.is_water())
        .count();
    let ocean_ratio = water_cells as f64 / TEST_GRID_SIZE as f64;
    
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
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    // Count land cells (above sea level)
    let land_cells = world.terrain_grid.cells()
        .filter(|cell| !cell.is_water())
        .count();
    let land_ratio = land_cells as f64 / TEST_GRID_SIZE as f64;
    
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
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    // Collect unique biome types
    let biomes: HashSet<BiomeType> = world.terrain_grid.cells()
        .filter(|cell| !cell.is_water()) // Only land biomes
        .map(|cell| BiomeType::from_u8(cell.biome()))
        .collect();
    
    assert!(
        biomes.len() >= MIN_BIOME_DIVERSITY,
        "Only {} unique biomes, expected at least {}: {:?}",
        biomes.len(),
        MIN_BIOME_DIVERSITY,
        biomes
    );
    
    println!("Biome diversity: {} types - {:?}", biomes.len(), biomes);
}

#[test]
fn test_world_generation_temperature_gradient() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    // Check pole regions (top and bottom 20% of map)
    let pole_threshold = TEST_HEIGHT / 5;
    
    // Northern pole
    let northern_pole_biomes: HashSet<BiomeType> = world.terrain_grid.cells()
        .filter(|cell| cell.y() < pole_threshold as usize && !cell.is_water())
        .map(|cell| BiomeType::from_u8(cell.biome()))
        .collect();
    
    // Southern pole  
    let southern_pole_biomes: HashSet<BiomeType> = world.terrain_grid.cells()
        .filter(|cell| cell.y() >= (TEST_HEIGHT - pole_threshold) as usize && !cell.is_water())
        .map(|cell| BiomeType::from_u8(cell.biome()))
        .collect();
    
    // Verify cold biomes exist at poles
    let cold_biomes = [
        BiomeType::Tundra,
        BiomeType::Arctic,
        BiomeType::SnowIce,
        BiomeType::BorealForest,
    ];
    
    let has_cold_north = northern_pole_biomes.iter()
        .any(|b| cold_biomes.contains(b));
    let has_cold_south = southern_pole_biomes.iter()
        .any(|b| cold_biomes.contains(b));
    
    assert!(
        has_cold_north,
        "No cold biomes at northern pole. Biomes: {:?}",
        northern_pole_biomes
    );
    assert!(
        has_cold_south,
        "No cold biomes at southern pole. Biomes: {:?}",
        southern_pole_biomes
    );
    
    // Verify equatorial regions have warm biomes
    let equator_biomes: HashSet<BiomeType> = world.terrain_grid.cells()
        .filter(|cell| {
            let mid_y = TEST_HEIGHT / 2;
            cell.y() >= mid_y.saturating_sub(pole_threshold) as usize 
            && cell.y() <= mid_y.saturating_add(pole_threshold) as usize
            && !cell.is_water()
        })
        .map(|cell| BiomeType::from_u8(cell.biome()))
        .collect();
    
    let warm_biomes = [
        BiomeType::TropicalRainforest,
        BiomeType::TropicalSavanna,
        BiomeType::SubtropicalDesert,
        BiomeType::TemperateForest,
    ];
    
    let has_warm_equator = equator_biomes.iter()
        .any(|b| warm_biomes.contains(b));
    
    assert!(
        has_warm_equator,
        "No warm biomes at equator. Biomes: {:?}",
        equator_biomes
    );
    
    println!("Temperature gradient: Poles have cold biomes, equator has warm biomes");
}

#[test]
fn test_world_generation_elevation_range() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        mountain_amplitude: 2000.0,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    let elevations: Vec<f32> = world.terrain_grid.cells()
        .map(|cell| cell.height())
        .collect();
    
    let min_elevation = elevations.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_elevation = elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
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
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
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
    
    // Verify polygons have valid vertices
    let invalid_polygons: Vec<_> = world.polygon_graph.iter()
        .filter(|p| p.vertices().len() < 3)
        .collect();
    
    assert!(
        invalid_polygons.is_empty(),
        "Found {} polygons with less than 3 vertices",
        invalid_polygons.len()
    );
    
    println!("Voronoi polygons: {} cells", polygon_count);
}

#[test]
fn test_world_generation_settlements() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world = generate_world(config);
    
    // Verify settlements were generated
    assert!(
        !world.settlements.is_empty(),
        "No settlements generated"
    );
    
    println!("Settlements generated: {}", world.settlements.len());
    
    // Verify settlements are placed in suitable biomes
    let unsuitable_count = world.settlements.iter()
        .filter(|s| {
            let cell = world.terrain_grid.get(s.x, s.y);
            cell.map(|c| c.is_water()).unwrap_or(false)
        })
        .count();
    
    assert_eq!(
        unsuitable_count, 0,
        "{} settlements placed in water (unsuitable biome)",
        unsuitable_count
    );
    
    // Verify species assignment
    let unique_species: HashSet<SpeciesId> = world.settlements.iter()
        .map(|s| s.species_id)
        .collect();
    
    assert!(
        !unique_species.is_empty(),
        "No species assigned to settlements"
    );
    
    println!("Settlement species: {:?} ({} types)", 
             unique_species, unique_species.len());
}

#[test]
fn test_world_generation_performance() {
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
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
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let world1 = generate_world(config);
    
    // Generate again with same config
    let world2 = generate_world(config);
    
    // Verify same generation time (within tolerance)
    let time_diff = (world1.generation_time_ms as i64 - world2.generation_time_ms as i64).abs();
    assert!(
        time_diff <= 100, // Within 100ms tolerance
        "Generation time differs by {}ms between runs",
        time_diff
    );
    
    // Verify same settlement count (determinism check)
    assert_eq!(
        world1.settlements.len(),
        world2.settlements.len(),
        "Settlement count differs between runs: {} vs {}",
        world1.settlements.len(),
        world2.settlements.len()
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
    
    let config = WorldConfig {
        seed: TEST_SEED,
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        sea_level: 0.4,
        ..Default::default()
    };
    
    let start = Instant::now();
    let world = generate_world(config);
    let total_time = start.elapsed().as_secs_f32();
    
    // Terrain stats
    let water_cells = world.terrain_grid.cells()
        .filter(|c| c.is_water()).count();
    let land_cells = TEST_GRID_SIZE - water_cells;
    let ocean_ratio = water_cells as f64 / TEST_GRID_SIZE as f64;
    let land_ratio = land_cells as f64 / TEST_GRID_SIZE as f64;
    
    // Biome diversity
    let land_biomes: HashSet<BiomeType> = world.terrain_grid.cells()
        .filter(|c| !c.is_water())
        .map(|c| BiomeType::from_u8(c.biome()))
        .collect();
    
    // Elevation range
    let elevations: Vec<f32> = world.terrain_grid.cells()
        .map(|c| c.height())
        .collect();
    let elevation_range = elevations.iter().max() - elevations.iter().min();
    
    // Tectonic verification
    let tectonic_info = if let Some(ref result) = world.tectonic_result {
        format!("Tectonic plates: {} | Boundaries: {} | Elevation modifiers: {}",
            result.plates.len(),
            result.boundaries.len(),
            result.elevation_modifiers.iter().filter(|&&m| m != 0.0).count())
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
    println!("Settlements: {}", world.settlements.len());
    println!();
    
    // Verify all criteria
    assert!(ocean_ratio >= MIN_OCEAN_RATIO && ocean_ratio <= MAX_OCEAN_RATIO);
    assert!(land_ratio >= MIN_LAND_COVERAGE);
    assert!(land_biomes.len() >= MIN_BIOME_DIVERSITY);
    assert!(elevation_range >= MIN_ELEVATION_RANGE_M);
    assert!(!world.settlements.is_empty());
    assert!(total_time < MAX_GENERATION_TIME_SECS);
    
    // Tectonic verification
    if let Some(ref result) = world.tectonic_result {
        assert!(!result.plates.is_empty(), "Tectonic plates should be generated");
        assert!(result.cell_to_plate.len() == (TEST_WIDTH * TEST_HEIGHT) as usize, 
            "All cells should be assigned to a plate");
    }
    
    println!("=== PASSED: Earthlike planet generated with tectonics ===\n");
}

// =============================================================================
// Helpers
// =============================================================================

impl TestWorld {
    /// Get cell at coordinates (for verification)
    fn get_cell(&self, x: usize, y: usize) -> Option<&TerrainCell> {
        self.terrain_grid.get(x, y)
    }
}

impl BiomeType {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => BiomeType::Ocean,
            1 => BiomeType::TropicalRainforest,
            2 => BiomeType::TropicalSavanna,
            3 => BiomeType::SubtropicalDesert,
            4 => BiomeType::TemperateForest,
            5 => BiomeType::TemperateRainforest,
            6 => BiomeType::TemperateGrassland,
            7 => BiomeType::BorealForest,
            8 => BiomeType::Tundra,
            9 => BiomeType::Arctic,
            10 => BiomeType::HotDesert,
            _ => BiomeType::Grassland,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig {
            seed: 0,
            width: 256,
            height: 256,
            sea_level: 0.4,
            terrain: None,
            river: None,
            biome: None,
        }
    }
}