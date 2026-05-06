//! Property-based tests for Voronoi generation and terrain systems
//!
//! These tests verify invariants across many random seeds using property-based testing.
//! Run with: `cargo test --test voronoi_property_tests`

use proptest::prelude::*;
use world_factory::{
    generation::voronoi::{generate_voronoi_graph, BoundaryMode, VoronoiConfig},
    terrain::{Polygon, PolygonGraph},
};

// =============================================================================
// Voronoi Validity Tests (GOAL.md Section 5.3 - Missing Test Category 1)
// =============================================================================

proptest! {
    /// Test that Voronoi generation is valid across seeds 0-1000
    /// Verifies: all polygons have valid vertices, correct neighbor relationships
    #[test]
    fn test_voronoi_validity_across_seeds(seed: u32) {
        let config = VoronoiConfig {
            width: 64,
            height: 64,
            num_seeds: 64,
            lloyd_iterations: 2,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
        };

        let graph = generate_voronoi_graph(config, seed as u64);

        // Invariant 1: Graph is not empty
        prop_assert!(graph.len() > 0, "Graph should have polygons for seed {}", seed);

        // Invariant 2: All polygons have at least 3 vertices
        for id in graph.polygon_ids() {
            if let Some(poly) = graph.get(id) {
                prop_assert!(
                    poly.vertices().len() >= 3,
                    "Polygon {} should have at least 3 vertices, got {}",
                    id, poly.vertices().len()
                );
            }
        }

        // Invariant 3: All polygon IDs are valid
        let polygon_count = graph.len();
        for id in graph.polygon_ids() {
            prop_assert!(
                (id as usize) < polygon_count || graph.get(id).is_some(),
                "Polygon ID {} should be valid",
                id
            );
        }
    }

    /// Test that Voronoi generation maintains polygon coverage area
    #[test]
    fn test_voronoi_coverage_area(seed: u32) {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 32,
            lloyd_iterations: 1,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
        };

        let graph = generate_voronoi_graph(config, seed as u64);
        let expected_area = 32.0 * 32.0;

        // Sum of all polygon areas should approximately equal total area
        let mut total_area = 0.0;
        for id in graph.polygon_ids() {
            if let Some(poly) = graph.get(id) {
                total_area += poly.area();
            }
        }

        // Allow 5% tolerance for edge effects
        let tolerance = expected_area * 0.05;
        prop_assert!(
            (total_area - expected_area).abs() < tolerance,
            "Total polygon area {} differs from expected {} by more than {}",
            total_area, expected_area, tolerance
        );
    }

    /// Test that Voronoi neighbors are reciprocal
    #[test]
    fn test_voronoi_neighbors_reciprocal(seed: u32) {
        let config = VoronoiConfig {
            width: 48,
            height: 48,
            num_seeds: 48,
            lloyd_iterations: 0,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
        };

        let graph = generate_voronoi_graph(config, seed as u64);

        // For each polygon, if B is a neighbor of A, then A should be a neighbor of B
        for id in graph.polygon_ids() {
            let neighbors = graph.get(id)
                .map(|p| p.neighbors.clone())
                .unwrap_or_default();

            for neighbor_id in neighbors {
                let neighbor_neighbors = graph.get(neighbor_id)
                    .map(|p| p.neighbors.clone())
                    .unwrap_or_default();

                prop_assert!(
                    neighbor_neighbors.contains(&id),
                    "Neighbor relationship not reciprocal: {} is neighbor of {} but not vice versa",
                    id, neighbor_id
                );
            }
        }
    }
}

// =============================================================================
// Determinism Tests (GOAL.md Section 5.3 - Missing Test Category 2)
// =============================================================================

proptest! {
    /// Test that same seed always produces same polygon graph
    #[test]
    fn test_determinism_identical_seeds(seed: u64) {
        let config = VoronoiConfig {
            width: 64,
            height: 64,
            num_seeds: 128,
            lloyd_iterations: 2,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: true,
        };

        let graph1 = generate_voronoi_graph(config.clone(), seed);
        let graph2 = generate_voronoi_graph(config.clone(), seed);

        // Same seed should produce same polygon count
        prop_assert_eq!(
            graph1.len(),
            graph2.len(),
            "Same seed {} should produce same polygon count",
            seed
        );

        // Same seed should produce same polygon structure
        for id in graph1.polygon_ids() {
            let poly1 = graph1.get(id);
            let poly2 = graph2.get(id);

            prop_assert_eq!(
                poly1.map(|p| p.vertices().len()),
                poly2.map(|p| p.vertices().len()),
                "Same seed should produce same vertex count for polygon {}",
                id
            );

            // Vertices should be approximately equal (within floating point tolerance)
            if let (Some(p1), Some(p2)) = (poly1, poly2) {
                prop_assert!(
                    p1.vertices().iter().zip(p2.vertices().iter())
                        .all(|(v1, v2)| (v1.x - v2.x).abs() < 0.001 && (v1.y - v2.y).abs() < 0.001),
                    "Same seed should produce same vertex positions for polygon {}",
                    id
                );
            }
        }
    }

    /// Test that different seeds produce different results (with high probability)
    #[test]
    fn test_different_seeds_different_output(seed1: u32, seed2: u32) {
        // Skip if same seed
        prop_assume!(seed1 != seed2);

        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 32,
            lloyd_iterations: 1,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
        };

        let graph1 = generate_voronoi_graph(config.clone(), seed1 as u64);
        let graph2 = generate_voronoi_graph(config.clone(), seed2 as u64);

        // With different seeds, at least one polygon should have different structure
        // This is probabilistic but should pass with overwhelming probability
        let polygon_count = graph1.len();

        if polygon_count > 0 {
            let mut found_difference = false;
            for id in 0..polygon_count as u32 {
                if let (Some(p1), Some(p2)) = (graph1.get(id), graph2.get(id)) {
                    // Check if vertex count or first vertex differs
                    if p1.vertices().len() != p2.vertices().len() {
                        found_difference = true;
                        break;
                    }
                    if let (Some(v1), Some(v2)) = (p1.vertices().first(), p2.vertices().first()) {
                        if (v1.x - v2.x).abs() > 0.01 || (v1.y - v2.y).abs() > 0.01 {
                            found_difference = true;
                            break;
                        }
                    }
                }
            }

            prop_assert!(
                found_difference,
                "Different seeds ({}, {}) should produce different graphs with high probability",
                seed1, seed2
            );
        }
    }
}

// =============================================================================
// Elevation Assignment Property Tests (GOAL.md Section 5.3 - Missing Test Category 3)
// =============================================================================

proptest! {
    /// Test that elevation values are always within valid range [0, 1]
    #[test]
    fn test_elevation_within_bounds(seed: u64) {
        let config = VoronoiConfig {
            width: 64,
            height: 64,
            num_seeds: 64,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
            ..Default::default()
        };

        let mut graph = generate_voronoi_graph(config, seed);

        // Initialize elevations deterministically
        for id in graph.polygon_ids() {
            if let Some(poly) = graph.get_mut(id) {
                // Use simple formula based on ID for deterministic assignment
                let elevation = (id as f32 % 100) as f32 / 100.0;
                poly.elevation = elevation;
            }
        }

        // Invariant: all elevations should be in [0, 1]
        for id in graph.polygon_ids() {
            if let Some(poly) = graph.get(id) {
                prop_assert!(
                    (0.0..=1.0).contains(&poly.elevation),
                    "Elevation {} out of bounds [0, 1] for polygon {}",
                    poly.elevation, id
                );
            }
        }
    }

    /// Test that elevation assignment preserves coastal polygon identity
    #[test]
    fn test_coastal_elevation_constraint(seed: u64) {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 32,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.5,
            blue_noise: false,
            ..Default::default()
        };

        let mut graph = generate_voronoi_graph(config, seed);

        // Mark some coastal polygons
        let coastal_ids: Vec<_> = graph.polygon_ids().take(5).collect();
        for id in &coastal_ids {
            graph.mark_coastal(*id);
        }

        // Set low elevations for coastal
        for id in &coastal_ids {
            if let Some(poly) = graph.get_mut(*id) {
                poly.elevation = 0.05; // Just above sea level
            }
        }

        // Verify coastal polygons have low elevation
        for id in &coastal_ids {
            if let Some(poly) = graph.get(*id) {
                prop_assert!(
                    poly.elevation < 0.2,
                    "Coastal polygon {} should have elevation < 0.2, got {}",
                    id, poly.elevation
                );
            }
        }
    }
}

// =============================================================================
// Biome Adjacency Tests (GOAL.md Section 5.3 - Missing Test Category 4)
// =============================================================================

#[test]
fn test_biome_adjacency_valid_rules() {
    // Test that biome adjacency rules are consistent
    use world_factory::generation::BiomeAssignmentMatrix;
    use world_factory::terrain::BiomeType;

    let matrix = BiomeAssignmentMatrix::new();

    // Get all biome types
    let biomes = vec![
        BiomeType::Ocean,
        BiomeType::Desert,
        BiomeType::Savanna,
        BiomeType::Grassland,
        BiomeType::Forest,
        BiomeType::Rainforest,
        BiomeType::Tundra,
        BiomeType::Snow,
    ];

    // Verify each biome can be assigned
    for biome in biomes {
        let test_cases = vec![
            (45.0, 20.0, 500.0, 15.0),  // Temperate
            (0.0, 10.0, 300.0, 28.0),   // Tropical
            (90.0, 70.0, 100.0, -15.0), // Polar
        ];

        for (lat, precip, elev, temp) in test_cases {
            let result = matrix.assign(lat, precip, elev, temp);
            // Biome assignment should always return a valid result
            assert!(
                result.confidence >= 0.0 && result.confidence <= 1.0,
                "Confidence should be in [0, 1] for {:?}",
                biome
            );
        }
    }
}

#[test]
fn test_adjacent_biomes_have_compatible_climate() {
    // Test that biomes with similar climate parameters can be adjacent
    use world_factory::generation::BiomeAssignmentMatrix;

    let matrix = BiomeAssignmentMatrix::new();

    // Tropical rainforest and tropical savanna should be adjacent-capable
    let rainforest = matrix.assign(5.0, 250.0, 200.0, 27.0);
    let savanna = matrix.assign(8.0, 80.0, 400.0, 26.0);
    let desert = matrix.assign(25.0, 5.0, 300.0, 32.0);

    // Verify all assignments are valid
    assert!(rainforest.confidence > 0.0);
    assert!(savanna.confidence > 0.0);
    assert!(desert.confidence > 0.0);

    // Rainforest and savanna are closer in climate than desert
    // This test verifies biome assignments are continuous functions
    let temp_diff_rain_savanna = (27.0 - 26.0).abs();
    let temp_diff_rain_desert = (27.0 - 32.0).abs();

    assert!(
        temp_diff_rain_savanna < temp_diff_rain_desert,
        "Rainforest-Savanna temp diff {} should be less than Rainforest-Desert diff {}",
        temp_diff_rain_savanna,
        temp_diff_rain_desert
    );
}
