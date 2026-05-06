//! Integration test for elevation assignment module
//!
//! Verifies that ElevationAssigner integrates correctly with the terrain system.

#[cfg(test)]
mod integration_tests {
    use world_factory::{
        generation::voronoi::{generate_voronoi_graph, BoundaryMode, VoronoiConfig},
        terrain::{ElevationAssigner, ElevationConfig, Polygon, PolygonGraph},
    };

    #[test]
    fn test_voronoi_with_elevation_assignment() {
        // Create a Voronoi graph
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 64,
            lloyd_iterations: 2,
            boundary_mode: BoundaryMode::Torus,
            jitter: 0.5,
            blue_noise: true,
        };

        let mut graph = generate_voronoi_graph(config, 42);

        // Assign elevations
        let mut assigner = ElevationAssigner::with_default();
        let result = assigner.assign_elevation(&mut graph, 42);

        // Verify results
        assert_eq!(result.total_polygons, 64);
        assert!(result.stats.is_valid());
        assert!(result.stats.min >= 0.0);
        assert!(result.stats.max <= 1.0);
        assert!(result.coastal_count > 0);
    }

    #[test]
    fn test_elevation_assigner_config_presets() {
        let default_config = ElevationConfig::default();
        let mountainous = ElevationConfig::mountainous();
        let gentle = ElevationConfig::gentle();

        // Verify configs are different
        assert_ne!(default_config.noise_amplitude, mountainous.noise_amplitude);
        assert_ne!(default_config.noise_amplitude, gentle.noise_amplitude);

        // Verify mountainous has higher amplitude
        assert!(mountainous.noise_amplitude > default_config.noise_amplitude);

        // Verify gentle has lower amplitude
        assert!(gentle.noise_amplitude < default_config.noise_amplitude);
    }

    #[test]
    fn test_elevation_assignment_with_weighted_distance() {
        let config = ElevationConfig {
            use_weighted_distance: true,
            distance_weight: 0.6,
            terrain_weight: 0.4,
            ..Default::default()
        };

        let mut graph = PolygonGraph::with_capacity(9);
        for i in 0..9 {
            let poly = Polygon::with_base_elevation(i, i as f32 * 500.0);
            graph.add_polygon(poly);
        }

        // Add edges
        for i in 0..8 {
            graph.add_edge(i, i + 1);
        }

        // Mark first as coastal
        graph.mark_coastal(0);

        let mut assigner = ElevationAssigner::new(config);
        let result = assigner.assign_elevation(&mut graph, 42);

        // Should produce valid elevations
        assert!(result.stats.is_valid());
        assert!(result.coastal_count >= 1);
    }

    #[test]
    fn test_get_mountain_ids() {
        let mut graph = PolygonGraph::with_capacity(5);
        for i in 0..5 {
            graph.add_polygon(Polygon::new(i));
        }

        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);

        // Mark 0 as coastal and set increasing elevations
        graph.mark_coastal(0);

        for i in 0..5 {
            if let Some(p) = graph.get_mut(i) {
                p.elevation = i as f32 / 4.0;
            }
        }

        let assigner = ElevationAssigner::with_default();
        let mountains = assigner.get_mountain_ids(&graph, 0.7);

        // Polygon 4 (elevation 1.0) should be above 0.7 threshold
        assert!(mountains.contains(&4));
    }

    #[test]
    fn test_empty_graph_handling() {
        let graph = PolygonGraph::new();
        let assigner = ElevationAssigner::with_default();

        // Should not panic on empty graph
        let coastal = assigner.get_coastal_ids(&graph);
        assert!(coastal.is_empty());

        let mountains = assigner.get_mountain_ids(&graph, 0.5);
        assert!(mountains.is_empty());
    }

    #[test]
    fn test_deterministic_elevation_assignment() {
        let config = VoronoiConfig {
            width: 16,
            height: 16,
            num_seeds: 32,
            lloyd_iterations: 1,
            boundary_mode: BoundaryMode::Finite,
            jitter: 0.3,
            blue_noise: false,
        };

        // Generate two identical graphs
        let mut graph1 = generate_voronoi_graph(config.clone(), 12345);
        let mut graph2 = generate_voronoi_graph(config.clone(), 12345);

        // Assign elevations with same seed
        let mut assigner = ElevationAssigner::with_default();
        assigner.assign_elevation(&mut graph1, 99999);
        assigner.assign_elevation(&mut graph2, 99999);

        // Verify elevations match
        for id in graph1.polygon_ids() {
            assert_eq!(
                graph1.elevation(id),
                graph2.elevation(id),
                "Elevations should be identical for same graph+seed"
            );
        }
    }
}
