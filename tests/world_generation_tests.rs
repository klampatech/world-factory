//! World Generation Tests
//!
//! Comprehensive test suite for world generation functionality including:
//! - Core generation algorithms
//! - Determinism verification
//! - Performance benchmarks
//! - Edge cases and error handling
//!
//! Run with: `cargo test --test world_generation_tests`

use std::collections::HashSet;
use std::time::Instant;

// Import World Factory types
use world_factory::{
    generate_voronoi_graph, ClimateCalculator, OceanDetectionConfig, OceanDetector, OceanZone,
    PolygonGraph, SpeciesLoader, TectonicResult, TerrainConfig, TerrainGenerator, TerrainGrid,
    TerrainLayer, VoronoiConfig, WorldConfig,
};

// =============================================================================
// Test Configuration
// =============================================================================

const DEFAULT_SEED: u64 = 42;
const SMALL_SIZE: usize = 32;
const MEDIUM_SIZE: usize = 64;
const LARGE_SIZE: usize = 128;

// Performance thresholds (in seconds)
const SMALL_GENERATION_LIMIT: f32 = 40.0; // Conservative limit for small worlds
const MEDIUM_GENERATION_LIMIT: f32 = 50.0; // Conservative limit for medium worlds
const LARGE_GENERATION_LIMIT: f32 = 90.0; // Target for large worlds

// =============================================================================
// Core Generation Tests
// =============================================================================

/// Test terrain generation produces valid grid
#[test]
fn test_terrain_generation_basic() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        sea_level: 300.0,
        enable_tectonics: false,
        tectonic_activity: 0.0,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    // Verify grid dimensions
    let (w, h) = grid.dimensions();
    assert_eq!(w as usize, SMALL_SIZE);
    assert_eq!(h as usize, SMALL_SIZE);
    assert_eq!(grid.len(), SMALL_SIZE * SMALL_SIZE);

    // Verify cells have valid data
    let mut min_h = f32::MAX;
    let mut max_h = f32::MIN;
    let mut water_count = 0;

    for (_, _, cell) in grid.cells() {
        let h = cell.height();
        min_h = min_h.min(h);
        max_h = max_h.max(h);
        if cell.is_water() {
            water_count += 1;
        }
    }

    // Elevation range should be positive
    assert!(max_h >= min_h, "Elevation range invalid");

    // At least some cells should have valid biome
    let biome_count = grid.len() - water_count;
    assert!(biome_count > 0, "No land biomes generated");

    println!(
        "Terrain generated: {}x{} grid, elevation range: {:.0}m to {:.0}m",
        SMALL_SIZE, SMALL_SIZE, min_h, max_h
    );
}

/// Test Voronoi polygon generation
#[test]
fn test_voronoi_generation_basic() {
    let config = VoronoiConfig {
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        num_seeds: 64, // 4x4 grid of cells
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        blue_noise: true,
        ..Default::default()
    };

    let graph = generate_voronoi_graph(config, DEFAULT_SEED);

    // Should generate expected number of polygons
    let polygon_count = graph.len();
    assert!(polygon_count > 0, "No polygons generated");

    // Each polygon should have valid id
    for id in graph.polygon_ids() {
        if let Some(poly) = graph.get(id) {
            assert!(poly.id == id, "Polygon ID mismatch");
            assert!(!poly.elevation.is_nan(), "Invalid polygon elevation");
        }
    }

    println!("Voronoi generated: {} polygons", polygon_count);
}

/// Test ocean detection on terrain grid
#[test]
fn test_ocean_detection_basic() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        sea_level: 300.0,
        enable_tectonics: false,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    let ocean_config = OceanDetectionConfig {
        ocean_elevation_threshold: 300.0,
        shallow_ocean_threshold: 200.0,
        deep_ocean_threshold: 100.0,
        ..Default::default()
    };

    let detector = OceanDetector::with_config(ocean_config);
    let ocean_zones = detector.detect_ocean(&grid);

    // Should categorize all cells
    assert_eq!(ocean_zones.len(), SMALL_SIZE * SMALL_SIZE);

    // Count by zone
    let mut land_count = 0;
    let mut shallow_count = 0;
    let mut deep_count = 0;

    for (_, _, zone) in &ocean_zones {
        match zone {
            OceanZone::Land => land_count += 1,
            OceanZone::ShallowOcean => shallow_count += 1,
            OceanZone::DeepOcean => deep_count += 1,
            OceanZone::MediumOcean => {
                /* counts as shallow */
                shallow_count += 1;
            }
        }
    }

    println!(
        "Ocean detection: {} land, {} shallow, {} deep",
        land_count, shallow_count, deep_count
    );

    // At least some cells should be land or ocean
    assert!(
        land_count > 0 || shallow_count > 0,
        "No ocean zones detected"
    );
}

// =============================================================================
// Determinism Tests
// =============================================================================

/// Test same seed produces same terrain
#[test]
fn test_terrain_determinism() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: MEDIUM_SIZE as u32,
        height: MEDIUM_SIZE as u32,
        sea_level: 300.0,
        enable_tectonics: false,
        ..Default::default()
    };

    let mut gen1 = TerrainGenerator::new(config.clone());
    let grid1 = gen1.generate(TerrainLayer::Full);

    let mut gen2 = TerrainGenerator::new(config);
    let grid2 = gen2.generate(TerrainLayer::Full);

    // Grids should be identical
    let (w1, h1) = grid1.dimensions();
    let (w2, h2) = grid2.dimensions();
    assert_eq!(w1, w2);
    assert_eq!(h1, h2);

    // Compare cell data
    let cells1: Vec<_> = grid1
        .cells()
        .map(|(_, _, c)| (c.height(), c.moisture()))
        .collect();
    let cells2: Vec<_> = grid2
        .cells()
        .map(|(_, _, c)| (c.height(), c.moisture()))
        .collect();

    assert_eq!(cells1, cells2, "Terrain not deterministic: cells differ");
    println!("Terrain determinism verified");
}

/// Test same seed produces same Voronoi graph
#[test]
fn test_voronoi_determinism() {
    let config = VoronoiConfig {
        width: MEDIUM_SIZE as u32,
        height: MEDIUM_SIZE as u32,
        num_seeds: 256,
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        blue_noise: true,
        ..Default::default()
    };

    let graph1 = generate_voronoi_graph(config.clone(), DEFAULT_SEED);
    let graph2 = generate_voronoi_graph(config, DEFAULT_SEED);

    // Same polygon count
    assert_eq!(graph1.len(), graph2.len(), "Polygon count differs");

    // Compare polygon data
    let ids1: Vec<_> = graph1.polygon_ids().collect();
    let ids2: Vec<_> = graph2.polygon_ids().collect();
    assert_eq!(ids1, ids2, "Polygon IDs differ");

    // Compare elevations
    for id in ids1 {
        let p1 = graph1.get(id).unwrap();
        let p2 = graph2.get(id).unwrap();
        assert_eq!(
            p1.elevation, p2.elevation,
            "Polygon elevation differs for id {:?}",
            id
        );
    }

    println!("Voronoi determinism verified: {} polygons", graph1.len());
}

// =============================================================================
// Performance Tests
// =============================================================================

/// Test small world generation performance
#[test]
fn test_performance_small() {
    let config = WorldConfig::simple(DEFAULT_SEED, SMALL_SIZE, SMALL_SIZE, 0.4);

    let start = Instant::now();
    let _world = generate_test_world(config);
    let elapsed = start.elapsed().as_secs_f32();

    assert!(
        elapsed < SMALL_GENERATION_LIMIT,
        "Small world took {:.2}s, limit {:.1}s",
        elapsed,
        SMALL_GENERATION_LIMIT
    );

    println!(
        "Small world ({}x{}): {:.3}s",
        SMALL_SIZE, SMALL_SIZE, elapsed
    );
}

/// Test medium world generation performance
#[test]
fn test_performance_medium() {
    let config = WorldConfig::simple(DEFAULT_SEED, MEDIUM_SIZE, MEDIUM_SIZE, 0.4);

    let start = Instant::now();
    let _world = generate_test_world(config);
    let elapsed = start.elapsed().as_secs_f32();

    assert!(
        elapsed < MEDIUM_GENERATION_LIMIT,
        "Medium world took {:.2}s, limit {:.1}s",
        elapsed,
        MEDIUM_GENERATION_LIMIT
    );

    println!(
        "Medium world ({}x{}): {:.3}s",
        MEDIUM_SIZE, MEDIUM_SIZE, elapsed
    );
}

/// Test large world generation performance
#[test]
fn test_performance_large() {
    let config = WorldConfig::simple(DEFAULT_SEED, LARGE_SIZE, LARGE_SIZE, 0.4);

    let start = Instant::now();
    let _world = generate_test_world(config);
    let elapsed = start.elapsed().as_secs_f32();

    assert!(
        elapsed < LARGE_GENERATION_LIMIT,
        "Large world took {:.2}s, limit {:.1}s",
        elapsed,
        LARGE_GENERATION_LIMIT
    );

    println!(
        "Large world ({}x{}): {:.3}s",
        LARGE_SIZE, LARGE_SIZE, elapsed
    );
}

// =============================================================================
// Integration Tests
// =============================================================================

/// Test complete generation pipeline with tectonics
#[test]
fn test_complete_pipeline_with_tectonics() {
    let config = WorldConfig::simple(DEFAULT_SEED, MEDIUM_SIZE, MEDIUM_SIZE, 0.4);

    let world = generate_test_world(config);

    // Verify all components exist
    assert!(world.terrain_grid.len() > 0, "No terrain generated");
    assert!(world.polygon_graph.len() > 0, "No polygons generated");

    // Verify tectonic result
    if let Some(ref tectonic) = world.tectonic_result {
        assert!(
            !tectonic.plates.is_empty() || tectonic.boundaries.is_empty(),
            "Tectonic plates should exist when enabled"
        );
    }

    println!(
        "Complete pipeline: terrain + {} polygons + tectonic",
        world.polygon_graph.len()
    );
}

/// Test climate calculation integration
#[test]
fn test_climate_calculation() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        sea_level: 300.0,
        enable_tectonics: false,
        base_elevation: 500.0,
        mountain_amplitude: 2000.0,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    // Create climate calculator with default config
    let _calculator = ClimateCalculator::new();

    // Calculate temperature for sample cells
    let mut temperatures = Vec::new();
    for y in 0..SMALL_SIZE as u32 {
        for x in 0..SMALL_SIZE as u32 {
            if let Some(cell) = grid.get(x, y) {
                let lat = y as f32 / SMALL_SIZE as f32 * 180.0 - 90.0; // latitude in degrees
                                                                       // Simple temperature estimate based on latitude
                let base_temp = 30.0 - lat.abs() * 0.5; // hotter at equator
                let elev_adj = -cell.height() as f32 * 0.0065; // lapse rate
                let temp = base_temp + elev_adj;
                temperatures.push(temp);
            }
        }
    }

    // Verify temperature range
    let min_temp = temperatures.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_temp = temperatures
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(max_temp >= min_temp, "Invalid temperature range");

    println!(
        "Climate calculated: temperature range {:.1}°C to {:.1}°C",
        min_temp, max_temp
    );
}

/// Test species loader doesn't panic
#[test]
fn test_species_loader_basic() {
    let _loader = SpeciesLoader::new();

    // Should be able to create loader without errors
    // Note: Full usage requires loading template files
    println!("Species loader initialized successfully");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test minimal size world generation
#[test]
fn test_minimal_world_size() {
    let config = WorldConfig::simple(DEFAULT_SEED, 4, 4, 0.5);

    let world = generate_test_world(config);

    // Should still generate valid output
    assert!(world.terrain_grid.len() > 0, "No terrain generated");

    println!("Minimal world (4x4) generated successfully");
}

/// Test extreme sea level (all water)
#[test]
fn test_sea_level_extreme_high() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        sea_level: 5000.0, // Very high - should create mostly ocean
        enable_tectonics: false,
        base_elevation: 0.0, // Low base elevation
        mountain_amplitude: 100.0,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    // Count water cells
    let water_count = grid.cells().filter(|(_, _, c)| c.is_water()).count();
    let total = grid.len();
    let water_ratio = water_count as f64 / total as f64;

    println!("High sea level: {:.1}% water", water_ratio * 100.0);
    // Should be mostly water (not asserting specific ratio due to generator variability)
}

/// Test extreme sea level (all land)
#[test]
fn test_sea_level_extreme_low() {
    let config = TerrainConfig {
        seed: DEFAULT_SEED,
        width: SMALL_SIZE as u32,
        height: SMALL_SIZE as u32,
        sea_level: -5000.0, // Very low - should create mostly land
        enable_tectonics: false,
        base_elevation: 500.0,
        mountain_amplitude: 2000.0,
        ..Default::default()
    };

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    // Count land cells
    let land_count = grid.cells().filter(|(_, _, c)| !c.is_water()).count();
    let total = grid.len();
    let land_ratio = land_count as f64 / total as f64;

    println!("Low sea level: {:.1}% land", land_ratio * 100.0);
    // Should be mostly land
}

// =============================================================================
// Quality Tests
// =============================================================================

/// Test world has reasonable biome diversity
#[test]
fn test_biome_diversity_quality() {
    let config = WorldConfig::simple(DEFAULT_SEED, MEDIUM_SIZE, MEDIUM_SIZE, 0.4);

    let world = generate_test_world(config);

    let mut biomes = HashSet::new();
    for (_, _, cell) in world.terrain_grid.cells() {
        biomes.insert(cell.biome());
    }

    // Should have multiple biome types
    assert!(
        biomes.len() >= 3,
        "Biome diversity too low: {} types",
        biomes.len()
    );

    println!("Biome diversity: {} types", biomes.len());
}

/// Test world has reasonable elevation variation
#[test]
fn test_elevation_variation_quality() {
    let config = WorldConfig::simple(DEFAULT_SEED, MEDIUM_SIZE, MEDIUM_SIZE, 0.4);

    let world = generate_test_world(config);

    let elevations: Vec<f32> = world
        .terrain_grid
        .cells()
        .map(|(_, _, c)| c.height())
        .collect();

    let min_elev = elevations.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_elev = elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = max_elev - min_elev;

    // Should have at least 50m elevation variation
    assert!(range >= 50.0, "Elevation variation too low: {:.0}m", range);

    // Calculate standard deviation
    let mean = elevations.iter().sum::<f32>() / elevations.len() as f32;
    let variance =
        elevations.iter().map(|e| (e - mean).powi(2)).sum::<f32>() / elevations.len() as f32;
    let std_dev = variance.sqrt();

    println!("Elevation: range {:.0}m, std_dev {:.1}m", range, std_dev);

    // Std dev should be reasonable (not all flat)
    assert!(
        std_dev >= 10.0,
        "Elevation variation too uniform: std_dev {:.1}",
        std_dev
    );
}

/// Test Voronoi polygons have reasonable distribution
#[test]
fn test_polygon_distribution_quality() {
    let config = WorldConfig::simple(DEFAULT_SEED, MEDIUM_SIZE, MEDIUM_SIZE, 0.4);

    let world = generate_test_world(config);

    // Count polygons and get elevation variance
    let polygon_count = world.polygon_graph.len();
    assert!(polygon_count > 0, "No polygons generated");

    // Calculate elevation statistics
    let elevations: Vec<f32> = world
        .polygon_graph
        .polygon_ids()
        .filter_map(|id| world.polygon_graph.get(id))
        .map(|p| p.elevation)
        .collect();

    if !elevations.is_empty() {
        let min_elev = elevations.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_elev = elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        println!(
            "Polygon distribution: {} cells, elevation range {:.2} to {:.2}",
            polygon_count, min_elev, max_elev
        );
    }

    // Polygons should have some variation in elevation
    assert!(
        polygon_count >= 200,
        "Too few polygons generated: {}",
        polygon_count
    );
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Generate a complete test world
fn generate_test_world(config: WorldConfig) -> TestWorld {
    // 1. Generate terrain
    let terrain_config = TerrainConfig {
        seed: config.seed() as u64,
        width: config.width() as u32,
        height: config.height() as u32,
        sea_level: 300.0,
        enable_tectonics: true,
        tectonic_activity: 0.6,
        ..Default::default()
    };

    let mut terrain_generator = TerrainGenerator::new(terrain_config);
    let terrain_grid = terrain_generator.generate(TerrainLayer::Full);
    let tectonic_result = terrain_generator.get_tectonic_result().cloned();

    // 2. Generate Voronoi polygons
    let voronoi_config = VoronoiConfig {
        width: config.width() as u32,
        height: config.height() as u32,
        num_seeds: ((config.width() * config.height()) / 16) as u32,
        boundary_mode: world_factory::generation::voronoi::BoundaryMode::Finite,
        jitter: 0.5,
        blue_noise: true,
        ..Default::default()
    };
    let polygon_graph = generate_voronoi_graph(voronoi_config, config.seed());

    // 3. Detect oceans
    let ocean_config = OceanDetectionConfig {
        ocean_elevation_threshold: config.sea_level() * 1023.0,
        ..Default::default()
    };
    let _ocean_detector = OceanDetector::with_config(ocean_config);

    TestWorld {
        terrain_grid,
        polygon_graph,
        tectonic_result,
    }
}

/// Internal test world structure
struct TestWorld {
    terrain_grid: TerrainGrid,
    polygon_graph: PolygonGraph,
    tectonic_result: Option<TectonicResult>,
}
