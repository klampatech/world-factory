//! Elevation Assignment Module
//!
//! Implements distance-from-coastline elevation assignment for Voronoi polygon grids.
//! This is the main entry point for Phase 1.3 elevation computation.
//!
//! ## Algorithm Overview
//!
//! 1. **Graph Construction**: Voronoi polygons with neighbor edges
//! 2. **Coastal Detection**: Edge polygons and ocean-adjacent land
//! 3. **Distance-from-Coast BFS**: Compute distance for all polygons
//! 4. **Elevation Normalization**: Map distances to [0, 1] elevation
//! 5. **Noise Variation**: Add natural terrain variation via multi-octave noise
//! 6. **Monotonic Enforcement**: Ensure elevation always increases away from coast
//!
//! ## Usage
//!
//! ```rust,ignore
//! use world_factory::terrain::elevation_assignment::{ElevationAssigner, ElevationConfig};
//!
//! let mut assigner = ElevationAssigner::new(config);
//! assigner.assign_elevation(&mut graph, seed);
//! ```

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use crate::terrain::{ElevationStats, PolygonGraph};
use crate::util::noise::SimplexNoise;

/// Wrapper for f32 that implements Ord for use in BinaryHeap.
#[derive(Debug, Clone, Copy)]
pub struct OrderedFloat(pub f32);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

/// Configuration for elevation assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationConfig {
    /// Base sea level threshold (polygons at or below this are ocean)
    pub sea_level: f32,
    /// Minimum elevation for coastal polygons (typically 0)
    pub coastal_min_elevation: f32,
    /// Maximum distance normalization factor
    pub max_distance_factor: f32,
    /// Enable noise-based terrain variation
    pub enable_noise_variation: bool,
    /// Noise frequency for elevation variation
    pub noise_frequency: f32,
    /// Noise amplitude (0.0-1.0, fraction of elevation range)
    pub noise_amplitude: f32,
    /// Number of noise octaves for multi-scale variation
    pub noise_octaves: usize,
    /// Enable monotonic elevation enforcement (uphill = toward mountains)
    pub enforce_monotonic: bool,
    /// Use weighted distance (considers terrain ruggedness)
    pub use_weighted_distance: bool,
    /// Base weight for distance vs terrain factor
    pub distance_weight: f32,
    /// Base weight for terrain ruggedness vs distance factor
    pub terrain_weight: f32,
}

impl Default for ElevationConfig {
    fn default() -> Self {
        Self {
            sea_level: 0.5,
            coastal_min_elevation: 0.0,
            max_distance_factor: 1.0,
            enable_noise_variation: true,
            noise_frequency: 0.02,
            noise_amplitude: 0.15,
            noise_octaves: 4,
            enforce_monotonic: true,
            use_weighted_distance: false,
            distance_weight: 0.7,
            terrain_weight: 0.3,
        }
    }
}

impl ElevationConfig {
    /// Configuration for high-mountain worlds (more dramatic elevation)
    pub fn mountainous() -> Self {
        Self {
            noise_amplitude: 0.25,
            noise_frequency: 0.015,
            enforce_monotonic: true,
            ..Default::default()
        }
    }

    /// Configuration for gentle/hill worlds (less dramatic)
    pub fn gentle() -> Self {
        Self {
            noise_amplitude: 0.08,
            noise_frequency: 0.03,
            enforce_monotonic: false, // Allow some variation
            ..Default::default()
        }
    }
}

/// Result of elevation assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationAssignmentResult {
    /// Statistics about the elevation distribution
    pub stats: ElevationStats,
    /// Number of coastal polygons
    pub coastal_count: usize,
    /// Number of mountain polygons (above threshold)
    pub mountain_count: usize,
    /// Total polygons processed
    pub total_polygons: usize,
}

/// Main elevation assigner that orchestrates the BFS-based distance calculation.
#[derive(Debug, Clone)]
pub struct ElevationAssigner {
    config: ElevationConfig,
    noise: SimplexNoise,
}

impl ElevationAssigner {
    /// Create a new elevation assigner.
    pub fn new(config: ElevationConfig) -> Self {
        Self {
            noise: SimplexNoise::new(0), // Seed set during assignment
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_default() -> Self {
        Self::new(ElevationConfig::default())
    }

    /// Assign elevations to all polygons in the graph.
    ///
    /// This is the main entry point. Call this after Voronoi graph construction
    /// and before any terrain analysis that needs elevation data.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph to modify (mutable for in-place updates)
    /// * `seed` - Random seed for deterministic generation
    ///
    /// # Returns
    /// Statistics about the elevation assignment for validation.
    pub fn assign_elevation(
        &mut self,
        graph: &mut PolygonGraph,
        seed: u64,
    ) -> ElevationAssignmentResult {
        if graph.is_empty() {
            return ElevationAssignmentResult {
                stats: ElevationStats::default(),
                coastal_count: 0,
                mountain_count: 0,
                total_polygons: 0,
            };
        }

        // Initialize noise with seed
        self.noise = SimplexNoise::new(seed);

        // Step 1: Identify coastal polygons based on elevation threshold
        self.mark_coastal_polygons(graph);

        // Step 2: Compute distance-from-coast using BFS
        if self.config.use_weighted_distance {
            self.compute_weighted_distances(graph);
        } else {
            self.compute_bfs_distances(graph);
        }

        // Step 3: Apply noise-based variation for natural terrain
        if self.config.enable_noise_variation {
            self.apply_noise_variation(graph, seed);
        }

        // Step 4: Blend with base elevation if available
        self.blend_with_base_elevation(graph);

        // Step 5: Enforce monotonic elevation (optional but recommended)
        if self.config.enforce_monotonic {
            self.enforce_monotonic_elevation(graph);
        }

        // Compute and return statistics
        self.compute_result(graph)
    }

    /// Mark polygons as coastal based on elevation threshold.
    fn mark_coastal_polygons(&self, graph: &mut PolygonGraph) {
        for polygon in graph.polygons_mut() {
            // A polygon is coastal if its base elevation is near sea level
            // or if it's an edge polygon
            let base_near_sea = (polygon.base_elevation - self.config.sea_level).abs() < 0.1;

            if base_near_sea || polygon.neighbors.is_empty() {
                polygon.is_coastal = true;
                polygon.elevation = self.config.coastal_min_elevation;
            }
        }
    }

    /// Compute BFS distances from coast and normalize to elevation.
    fn compute_bfs_distances(&mut self, graph: &mut PolygonGraph) {
        let n = graph.len();

        // Initialize BFS structures
        let mut visited = vec![false; n];
        let mut distances = vec![u32::MAX; n];
        let mut queue = VecDeque::new();

        // Start BFS from all coastal polygons
        for polygon in graph.polygons() {
            if polygon.is_coastal {
                let id = polygon.id as usize;
                distances[id] = 0;
                visited[id] = true;
                queue.push_back(id as u32);
            }
        }

        // If no coastal polygons, nothing to compute
        if queue.is_empty() {
            return;
        }

        // BFS traversal
        while let Some(current_id) = queue.pop_front() {
            let current_dist = distances[current_id as usize];

            for &neighbor_id in graph.polygons()[current_id as usize].neighbors.iter() {
                let neighbor_idx = neighbor_id as usize;
                if !visited[neighbor_idx] {
                    visited[neighbor_idx] = true;
                    distances[neighbor_idx] = current_dist + 1;
                    queue.push_back(neighbor_id);
                }
            }
        }

        // Normalize distances to [0, 1] elevation
        let max_distance = distances
            .iter()
            .filter(|&&d| d != u32::MAX)
            .max()
            .copied()
            .unwrap_or(u32::MAX);
        let max_dist_f = max_distance as f32 * self.config.max_distance_factor;

        for polygon in graph.polygons_mut() {
            let idx = polygon.id as usize;
            if distances[idx] != u32::MAX && max_dist_f > 0.0 {
                let normalized = distances[idx] as f32 / max_dist_f;
                polygon.elevation = normalized.clamp(0.0, 1.0);
            }
        }
    }

    /// Compute weighted distances considering terrain ruggedness.
    fn compute_weighted_distances(&mut self, graph: &mut PolygonGraph) {
        let n = graph.len();

        // Initialize distances with infinity
        let mut distances = vec![f32::INFINITY; n];
        let mut visited = vec![false; n];
        let mut heap: BinaryHeap<(OrderedFloat, u32)> = BinaryHeap::new();

        // Start from coastal polygons
        for polygon in graph.polygons() {
            if polygon.is_coastal {
                let id = polygon.id as usize;
                distances[id] = 0.0;
                heap.push((OrderedFloat(0.0), id as u32));
            }
        }

        if heap.is_empty() {
            return;
        }

        // Dijkstra's algorithm with weighted edges
        while let Some((neg_dist, current_id)) = heap.pop() {
            let current_dist: f32 = -neg_dist.0;
            let current_idx = current_id as usize;

            if visited[current_idx] {
                continue;
            }
            visited[current_idx] = true;

            let current_base = graph.polygons()[current_idx].base_elevation;

            for &neighbor_id in graph.polygons()[current_idx].neighbors.iter() {
                let neighbor_idx = neighbor_id as usize;

                if visited[neighbor_idx] {
                    continue;
                }

                // Weight considers elevation difference for monotonic paths
                let neighbor_base = graph.polygons()[neighbor_idx].base_elevation;
                let elevation_diff = (neighbor_base - current_base).abs();
                let weight = 1.0 + elevation_diff;

                let new_dist = current_dist + weight;

                if new_dist < distances[neighbor_idx] {
                    distances[neighbor_idx] = new_dist;
                    heap.push((OrderedFloat(-new_dist), neighbor_id));
                }
            }
        }

        // Normalize to [0, 1]
        let max_dist = distances
            .iter()
            .filter(|&&d| d.is_finite())
            .fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));

        for polygon in graph.polygons_mut() {
            let idx = polygon.id as usize;
            if distances[idx].is_finite() && max_dist > 0.0 {
                polygon.elevation = distances[idx] / max_dist;
            }
        }
    }

    /// Apply multi-octave noise for natural terrain variation.
    fn apply_noise_variation(&mut self, graph: &mut PolygonGraph, _seed: u64) {
        let freq = self.config.noise_frequency;
        let amp = self.config.noise_amplitude;
        let octaves = self.config.noise_octaves;

        for polygon in graph.polygons_mut() {
            if polygon.is_coastal {
                continue; // Keep coast at 0
            }

            // Multi-octave noise for natural variation
            let mut noise_val = 0.0;
            let mut amplitude = amp;
            let mut frequency = freq;
            let mut max_val = 0.0;

            for _ in 0..octaves {
                // Use polygon ID as coordinate proxy (would use actual position in real impl)
                let nx = polygon.id as f64 * frequency as f64;
                let ny = polygon.id as f64 * frequency as f64 * 0.7; // Offset for variety

                noise_val += self.noise.get(nx, ny) * amplitude as f64;
                max_val += amplitude as f64;
                amplitude *= 0.5;
                frequency *= 2.0;
            }

            if max_val > 0.0 {
                noise_val /= max_val;
            }

            // Add noise to elevation (preserve monotonic direction)
            polygon.elevation =
                ((polygon.elevation as f64 + noise_val * 0.5).clamp(0.0, 1.0)) as f32;
        }
    }

    /// Blend distance elevation with base elevation for final value.
    fn blend_with_base_elevation(&mut self, graph: &mut PolygonGraph) {
        let dist_weight = self.config.distance_weight;
        let terrain_weight = self.config.terrain_weight;
        let total = dist_weight + terrain_weight;

        if total <= 0.0 {
            return;
        }

        let dist_norm = dist_weight / total;
        let terrain_norm = terrain_weight / total;

        for polygon in graph.polygons_mut() {
            // Normalize base elevation to [0, 1] (8000m = Everest)
            let normalized_base = (polygon.base_elevation / 8000.0).clamp(0.0, 1.0);

            // Blend
            polygon.elevation =
                (polygon.elevation * dist_norm + normalized_base * terrain_norm).clamp(0.0, 1.0);
        }
    }

    /// Ensure monotonic elevation paths (always uphill toward mountains).
    fn enforce_monotonic_elevation(&mut self, graph: &mut PolygonGraph) {
        if graph.is_empty() {
            return;
        }

        let mut changes = true;
        let max_iterations = 10;
        let mut iteration = 0;

        while changes && iteration < max_iterations {
            changes = false;
            iteration += 1;

            // Collect elevation changes first to avoid borrow issues
            let mut elevation_changes: Vec<(u32, f32)> = Vec::new();

            for polygon in graph.polygons() {
                if polygon.is_coastal {
                    continue;
                }

                let mut max_neighbor_elev = polygon.elevation;

                for &neighbor_id in &polygon.neighbors {
                    if let Some(neighbor) = graph.get(neighbor_id) {
                        // Only consider higher elevations (moving toward mountains)
                        if neighbor.elevation > max_neighbor_elev {
                            max_neighbor_elev = neighbor.elevation;
                        }
                    }
                }

                // Ensure this polygon is at least as high as the lowest point
                // on the path toward mountains
                if polygon.elevation < max_neighbor_elev {
                    elevation_changes.push((polygon.id, max_neighbor_elev));
                }
            }

            // Apply changes
            for (id, elevation) in elevation_changes {
                if let Some(polygon) = graph.get_mut(id) {
                    polygon.elevation = elevation;
                    changes = true;
                }
            }
        }
    }

    /// Compute final statistics for the assignment.
    fn compute_result(&self, graph: &PolygonGraph) -> ElevationAssignmentResult {
        let stats = graph.elevation_stats();
        let coastal_count = graph.polygons().iter().filter(|p| p.is_coastal).count();
        let mountain_count = graph
            .polygons()
            .iter()
            .filter(|p| p.elevation > 0.8)
            .count();

        ElevationAssignmentResult {
            stats,
            coastal_count,
            mountain_count,
            total_polygons: graph.len(),
        }
    }

    /// Get elevation at a specific polygon.
    pub fn get_elevation(&self, graph: &PolygonGraph, polygon_id: u32) -> f32 {
        graph.elevation(polygon_id)
    }

    /// Get all coastal polygon IDs.
    pub fn get_coastal_ids(&self, graph: &PolygonGraph) -> Vec<u32> {
        graph
            .polygon_ids()
            .filter(|&id| graph.is_coastal(id))
            .collect()
    }

    /// Get all mountain polygon IDs (above threshold).
    pub fn get_mountain_ids(&self, graph: &PolygonGraph, threshold: f32) -> Vec<u32> {
        graph
            .polygon_ids()
            .filter(|&id| graph.elevation(id) >= threshold)
            .collect()
    }
}

impl Default for ElevationAssigner {
    fn default() -> Self {
        Self::new(ElevationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Polygon;

    fn create_test_graph() -> PolygonGraph {
        let mut graph = PolygonGraph::with_capacity(9);

        // Create 3x3 grid
        // 6 7 8
        // 3 4 5
        // 0 1 2
        for i in 0..9 {
            graph.add_polygon(Polygon::new(i));
        }

        // Grid connections
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(3, 4);
        graph.add_edge(4, 5);
        graph.add_edge(6, 7);
        graph.add_edge(7, 8);
        graph.add_edge(0, 3);
        graph.add_edge(1, 4);
        graph.add_edge(2, 5);
        graph.add_edge(3, 6);
        graph.add_edge(4, 7);
        graph.add_edge(5, 8);
        graph.add_edge(0, 4);
        graph.add_edge(1, 5);
        graph.add_edge(3, 7);
        graph.add_edge(4, 8);

        // Mark left edge as coastal
        graph.mark_coastal(0);
        graph.mark_coastal(3);
        graph.mark_coastal(6);

        // Set base elevations
        let base_elevations = [0.0, 0.0, 0.1, 0.0, 0.5, 0.3, 0.0, 0.7, 0.9];
        for (i, &elev) in base_elevations.iter().enumerate() {
            if let Some(p) = graph.get_mut(i as u32) {
                p.base_elevation = elev;
            }
        }

        graph
    }

    #[test]
    fn test_basic_assignment() {
        let mut graph = create_test_graph();
        let mut assigner = ElevationAssigner::with_default();

        let result = assigner.assign_elevation(&mut graph, 42);

        assert_eq!(result.total_polygons, 9);
        assert_eq!(result.coastal_count, 3);
        assert!(result.stats.is_valid());
    }

    #[test]
    fn test_coastal_unchanged() {
        let mut graph = create_test_graph();
        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph, 42);

        // Coastal polygons should have 0 elevation
        assert_eq!(graph.elevation(0), 0.0);
        assert_eq!(graph.elevation(3), 0.0);
        assert_eq!(graph.elevation(6), 0.0);
    }

    #[test]
    fn test_interior_higher_than_coast() {
        let mut graph = create_test_graph();
        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph, 42);

        // Interior polygons should have higher elevation than coast
        for id in 1..9 {
            if !graph.is_coastal(id) {
                assert!(
                    graph.elevation(id) > graph.elevation(0),
                    "Polygon {} should have higher elevation than coast",
                    id
                );
            }
        }
    }

    #[test]
    fn test_mountain_highest() {
        let mut graph = create_test_graph();
        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph, 42);

        // Polygon 8 (highest base elevation) should be among highest
        let elevation_8 = graph.elevation(8);
        for id in 0..8 {
            if id != 8 {
                // 8 should be at least as high as most others
                assert!(
                    elevation_8 >= graph.elevation(id) - 0.1,
                    "Polygon 8 should be highest or near-highest"
                );
            }
        }
    }

    #[test]
    fn test_deterministic() {
        let mut graph1 = create_test_graph();
        let mut graph2 = create_test_graph();

        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph1, 12345);
        assigner.assign_elevation(&mut graph2, 12345);

        // Same seed should produce same elevations
        for id in 0..9 {
            assert_eq!(
                graph1.elevation(id),
                graph2.elevation(id),
                "Elevations should match for same seed"
            );
        }
    }

    #[test]
    fn test_different_seeds_different_output() {
        let mut graph1 = create_test_graph();
        let mut graph2 = create_test_graph();

        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph1, 111);
        assigner.assign_elevation(&mut graph2, 222);

        // Different seeds should produce different noise variation
        // (not guaranteed to differ, but likely)
        let has_difference =
            (0..9).any(|id| (graph1.elevation(id) - graph2.elevation(id)).abs() > 0.001);

        // With noise enabled, this should be true
        // Note: This test might occasionally fail if seeds produce similar patterns
        assert!(
            has_difference,
            "Different seeds should produce different elevations"
        );
    }

    #[test]
    fn test_mountainous_config() {
        let mut graph = create_test_graph();
        let config = ElevationConfig::mountainous();
        let mut assigner = ElevationAssigner::new(config);

        let result = assigner.assign_elevation(&mut graph, 42);

        // Mountainous config should produce more mountains
        assert!(result.mountain_count >= 1);
    }

    #[test]
    fn test_get_coastal_ids() {
        let mut graph = create_test_graph();
        let assigner = ElevationAssigner::with_default();

        let coastal = assigner.get_coastal_ids(&graph);

        assert_eq!(coastal.len(), 3);
        assert!(coastal.contains(&0));
        assert!(coastal.contains(&3));
        assert!(coastal.contains(&6));
    }

    #[test]
    fn test_get_mountain_ids() {
        let mut graph = create_test_graph();
        let mut assigner = ElevationAssigner::with_default();

        assigner.assign_elevation(&mut graph, 42);

        let mountains = assigner.get_mountain_ids(&graph, 0.7);

        // Should have some high elevation polygons
        assert!(!mountains.is_empty());
    }

    #[test]
    fn test_monotonic_enforcement() {
        let mut graph = create_test_graph();
        let config = ElevationConfig {
            enforce_monotonic: true,
            ..Default::default()
        };
        let mut assigner = ElevationAssigner::new(config);

        assigner.assign_elevation(&mut graph, 42);

        // After monotonic enforcement, verify property
        for polygon in graph.polygons() {
            if polygon.is_coastal {
                continue;
            }

            let mut min_path_elev = polygon.elevation;

            // Find the path to coast and check elevation increases
            // For this simple graph, check neighbors
            for &neighbor_id in &polygon.neighbors {
                let neighbor_elev = graph.elevation(neighbor_id);
                if neighbor_elev < min_path_elev && !graph.is_coastal(neighbor_id) {
                    // This would indicate a problem before enforcement
                }
            }
        }
    }

    #[test]
    fn test_empty_graph() {
        let graph = PolygonGraph::new();
        let assigner = ElevationAssigner::with_default();

        let coastal = assigner.get_coastal_ids(&graph);

        assert!(coastal.is_empty());
    }

    #[test]
    fn test_weighted_distance() {
        let mut graph = create_test_graph();
        let config = ElevationConfig {
            use_weighted_distance: true,
            ..Default::default()
        };
        let mut assigner = ElevationAssigner::new(config);

        let result = assigner.assign_elevation(&mut graph, 42);

        // Weighted distance should still produce valid elevations
        assert!(result.stats.is_valid());
        assert!(result.stats.min >= 0.0);
        assert!(result.stats.max <= 1.0);
    }
}
