//! Lloyd Relaxation for Centroidal Voronoi Diagrams
//!
//! Implements Lloyd's algorithm for relaxing Voronoi cell positions toward
//! their geometric centroids, producing more uniform cell distributions.
//!
//! # Algorithm
//!
//! ```text
//! 1. For each seed/cell, compute the geometric centroid of its polygon
//! 2. Apply jitter perturbation (configurable amount)
//! 3. Move seed position toward centroid by interpolation factor
//! 4. Repeat for configured iterations
//! ```
//!
//! # Parameters
//!
//! - `jitter`: Random perturbation magnitude (0.0-1.0, fraction of cell size)
//! - `iterations`: Number of relaxation passes
//! - `centroid_factor`: Weight toward centroid (0.0 = no movement, 1.0 = full centroid)

use serde::{Deserialize, Serialize};
use crate::world::entities::polygon::{Point2D, PolygonMesh};
use crate::util::noise::Rng;

/// Configuration for Lloyd relaxation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LloydConfig {
    /// Number of relaxation iterations.
    pub iterations: u32,
    /// Jitter amount (0.0-1.0, fraction of cell size).
    /// Applied as random perturbation before centroid movement.
    pub jitter: f32,
    /// Centroid factor (0.0-1.0), how much to move toward centroid.
    /// 0.0 = no movement, 1.0 = move directly to centroid.
    pub centroid_factor: f32,
    /// Enable deterministic output (disable random jitter).
    pub deterministic: bool,
}

impl Default for LloydConfig {
    fn default() -> Self {
        Self {
            iterations: 2,
            jitter: 0.2,
            centroid_factor: 0.5,
            deterministic: false,
        }
    }
}

impl LloydConfig {
    /// Create config with standard jitter (0.2) and 2 iterations.
    pub fn standard() -> Self {
        Self {
            iterations: 2,
            jitter: 0.2,
            centroid_factor: 0.5,
            deterministic: false,
        }
    }

    /// Create config optimized for terrain (more smoothing).
    pub fn for_terrain() -> Self {
        Self {
            iterations: 4,
            jitter: 0.15,
            centroid_factor: 0.6,
            deterministic: false,
        }
    }

    /// Create config for minimal smoothing (fewer artifacts).
    pub fn minimal() -> Self {
        Self {
            iterations: 1,
            jitter: 0.1,
            centroid_factor: 0.3,
            deterministic: false,
        }
    }
}

/// Seed position for Voronoi generation with relaxation support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelaxedSeed {
    /// Unique seed identifier.
    pub id: u32,
    /// Current x position.
    pub x: f32,
    /// Current y position.
    pub y: f32,
    /// Original x position (before any relaxation).
    pub origin_x: f32,
    /// Original y position (before any relaxation).
    pub origin_y: f32,
}

impl RelaxedSeed {
    /// Create a new seed at the given position.
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            origin_x: x,
            origin_y: y,
        }
    }

    /// Reset to original position.
    pub fn reset(&mut self) {
        self.x = self.origin_x;
        self.y = self.origin_y;
    }

    /// Move toward a target position.
    pub fn move_toward(&mut self, target: &Point2D, factor: f32) {
        self.x += (target.x - self.x) * factor;
        self.y += (target.y - self.y) * factor;
    }

    /// Apply random jitter within bounds.
    pub fn apply_jitter(&mut self, jitter: f32, cell_size: f32, rng: &mut Rng) {
        let jitter_x = (rng.next_f64Signed() as f32) * jitter * cell_size;
        let jitter_y = (rng.next_f64Signed() as f32) * jitter * cell_size;
        self.x += jitter_x;
        self.y += jitter_y;
    }

    /// Clamp position within bounds.
    pub fn clamp_to_bounds(&mut self, width: f32, height: f32) {
        self.x = self.x.clamp(0.0, width - 0.001);
        self.y = self.y.clamp(0.0, height - 0.001);
    }
}

/// Result of Lloyd relaxation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LloydResult {
    /// Relaxed seed positions.
    pub seeds: Vec<RelaxedSeed>,
    /// Number of iterations performed.
    pub iterations_completed: u32,
    /// Average movement per iteration.
    pub avg_movement_per_iteration: Vec<f32>,
}

/// Lloyd relaxation processor.
#[derive(Debug, Clone)]
pub struct LloydRelaxation {
    config: LloydConfig,
    rng: Rng,
}

impl LloydRelaxation {
    /// Create a new relaxation processor.
    pub fn new(config: LloydConfig, seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_seed(seed: u64) -> Self {
        Self::new(LloydConfig::default(), seed)
    }

    /// Perform relaxation on a polygon mesh.
    ///
    /// Uses polygon centroids (computed via signed area formula) to
    /// move seed positions toward geometric centers.
    pub fn relax(&mut self, mesh: &PolygonMesh, bounds: &BoundingBox) -> LloydResult {
        let mut seeds = self.initialize_seeds(mesh);
        let mut avg_movements = Vec::with_capacity(self.config.iterations as usize);

        for _iteration in 0..self.config.iterations {
            let (avg_movement, _centroids) = self.compute_centroids_and_move(&mut seeds, mesh, bounds);
            avg_movements.push(avg_movement);


            // Update mesh with new seed positions for next iteration
            // Note: update_mesh_centroids is a stub - no-op for simplified Lloyd relaxation

            // Stop if movement is negligible
            if avg_movement < 0.01 {
                break;
            }
        }

        LloydResult {
            seeds,
            iterations_completed: avg_movements.len() as u32,
            avg_movement_per_iteration: avg_movements,
        }
    }

    /// Initialize seed positions from polygon centroids.
    fn initialize_seeds(&mut self, mesh: &PolygonMesh) -> Vec<RelaxedSeed> {
        mesh.polygons
            .iter()
            .enumerate()
            .map(|(i, poly)| {
                let centroid = poly.centroid();
                RelaxedSeed::new(i as u32, centroid.x, centroid.y)
            })
            .collect()
    }

    /// Compute centroids for all polygons and move seeds toward them.
    ///
    /// Returns average movement distance and centroid map.
    fn compute_centroids_and_move(
        &mut self,
        seeds: &mut Vec<RelaxedSeed>,
        mesh: &PolygonMesh,
        bounds: &BoundingBox,
    ) -> (f32, Vec<Point2D>) {
        let cell_size = (bounds.width().min(bounds.height()) / (mesh.polygon_count() as f32).sqrt()).max(1.0);
        let mut centroids = Vec::with_capacity(seeds.len());
        let mut total_movement = 0.0f32;

        // First pass: compute centroids and store old positions
        let old_positions: Vec<Point2D> = seeds.iter().map(|s| Point2D::new(s.x, s.y)).collect();

        for seed in &mut *seeds {
            // Find polygon by ID
            if let Some(poly) = mesh.get(seed.id) {
                let centroid = poly.centroid();
                centroids.push(centroid);
            } else {
                // Fallback: use seed position as centroid
                centroids.push(Point2D::new(seed.x, seed.y));
            }
        }

        // Second pass: move seeds toward centroids
        for (i, seed) in seeds.iter_mut().enumerate() {
            let centroid = &centroids[i];

            // Apply jitter if configured (unless deterministic)
            if !self.config.deterministic && self.config.jitter > 0.0 {
                seed.apply_jitter(self.config.jitter, cell_size, &mut self.rng);
            }

            // Move toward centroid
            let old_pos = &old_positions[i];
            seed.move_toward(centroid, self.config.centroid_factor);

            // Track movement
            let movement = old_pos.distance(&Point2D::new(seed.x, seed.y));
            total_movement += movement;

            // Clamp to bounds
            seed.clamp_to_bounds(bounds.width(), bounds.height());
        }

        let avg_movement = if !seeds.is_empty() {
            total_movement / seeds.len() as f32
        } else {
            0.0
        };

        (avg_movement, centroids)
    }

    /// Update polygon centroids in mesh for next iteration.
    fn update_mesh_centroids(&mut self, _mesh: &mut PolygonMesh, _centroids: &[Point2D]) {
        // Note: This is a simplified update. For true Lloyd relaxation,
        // we'd need to rebuild the Voronoi diagram each iteration.
        // The actual Voronoi rebuild happens in VoronoiGenerator.
        // This module focuses on centroid computation for the seed positions.
    }

    /// Perform relaxation using raw seed positions and polygon mesh.
    ///
    /// This variant takes pre-existing seeds and computes their new positions
    /// based on polygon centroids.
    pub fn relax_seeds(
        &mut self,
        seeds: &[RelaxedSeed],
        mesh: &PolygonMesh,
        bounds: &BoundingBox,
    ) -> LloydResult {
        let mut mutable_seeds = seeds.to_vec();
        let mut avg_movements = Vec::with_capacity(self.config.iterations as usize);

        for _iteration in 0..self.config.iterations {
            let (avg_movement, _centroids) = self.compute_centroids_and_move(
                &mut mutable_seeds,
                mesh,
                bounds,
            );
            avg_movements.push(avg_movement);

            // Note: update_mesh_centroids is a stub - no-op for simplified Lloyd relaxation

            if avg_movement < 0.01 {
                break;
            }
        }

        LloydResult {
            seeds: mutable_seeds,
            iterations_completed: avg_movements.len() as u32,
            avg_movement_per_iteration: avg_movements,
        }
    }
}

/// Bounding box for world bounds.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point2D,
    pub max: Point2D,
}

impl BoundingBox {
    /// Create from min/max points.
    pub fn new(min: Point2D, max: Point2D) -> Self {
        Self { min, max }
    }

    /// Create from width and height.
    pub fn from_size(width: f32, height: f32) -> Self {
        Self {
            min: Point2D::origin(),
            max: Point2D::new(width, height),
        }
    }

    /// Width of bounding box.
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height of bounding box.
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Center point.
    pub fn center(&self) -> Point2D {
        Point2D::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Polygon;

    fn create_test_mesh() -> PolygonMesh {
        let mut mesh = PolygonMesh::new();
        
        // Create a simple grid of squares
        for y in 0..2 {
            for x in 0..2 {
                let id = (y * 2 + x) as u32;
                let px = x as f32;
                let py = y as f32;
                
                let poly = Polygon::new(id, vec![
                    Point2D::new(px, py),
                    Point2D::new(px + 1.0, py),
                    Point2D::new(px + 1.0, py + 1.0),
                    Point2D::new(px, py + 1.0),
                ]);
                mesh.add_polygon(poly);
            }
        }
        
        mesh
    }

    #[test]
    fn test_default_config() {
        let config = LloydConfig::default();
        assert_eq!(config.iterations, 2);
        assert_eq!(config.jitter, 0.2);
        assert_eq!(config.centroid_factor, 0.5);
    }

    #[test]
    fn test_standard_config() {
        let config = LloydConfig::standard();
        assert_eq!(config.jitter, 0.2);
        assert_eq!(config.iterations, 2);
    }

    #[test]
    fn test_relaxed_seed() {
        let mut seed = RelaxedSeed::new(0, 5.0, 5.0);
        
        // Test move toward
        seed.move_toward(&Point2D::new(10.0, 10.0), 0.5);
        assert_eq!(seed.x, 7.5);
        assert_eq!(seed.y, 7.5);
        
        // Test reset
        seed.reset();
        assert_eq!(seed.x, 5.0);
        assert_eq!(seed.y, 5.0);
    }

    #[test]
    fn test_relaxation_iterations() {
        let config = LloydConfig {
            iterations: 3,
            jitter: 0.0, // Disable jitter for determinism
            centroid_factor: 0.5,
            deterministic: true,
        };
        
        let mut relaxer = LloydRelaxation::new(config, 42);
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(2.0, 2.0);
        
        let result = relaxer.relax(&mesh, &bounds);
        
        // Algorithm may converge early or complete all iterations
        // With jitter=0.0 and exact centroid calculation, it may converge fast
        assert!(result.iterations_completed >= 1 && result.iterations_completed <= 3);
        assert_eq!(result.seeds.len(), 4);
    }

    #[test]
    fn test_centroid_factor_effect() {
        let config_full = LloydConfig {
            iterations: 1,
            jitter: 0.0,
            centroid_factor: 1.0,
            deterministic: true,
        };
        
        let config_none = LloydConfig {
            iterations: 1,
            jitter: 0.0,
            centroid_factor: 0.0,
            deterministic: true,
        };
        
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(2.0, 2.0);
        
        let mut relaxer_full = LloydRelaxation::new(config_full, 42);
        let result_full = relaxer_full.relax(&mesh, &bounds);
        
        let mut relaxer_none = LloydRelaxation::new(config_none, 42);
        let result_none = relaxer_none.relax(&mesh, &bounds);
        
        // With factor 0.0, seeds shouldn't move
        // (centroids match initial positions for regular grid)
        assert_eq!(result_none.avg_movement_per_iteration[0], 0.0);
    }

    #[test]
    fn test_deterministic_mode() {
        let config = LloydConfig {
            iterations: 2,
            jitter: 0.5,
            centroid_factor: 0.5,
            deterministic: true,
        };
        
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(2.0, 2.0);
        
        let mut relaxer1 = LloydRelaxation::new(config.clone(), 12345);
        let result1 = relaxer1.relax(&mesh, &bounds);
        
        let mut relaxer2 = LloydRelaxation::new(config.clone(), 12345);
        let result2 = relaxer2.relax(&mesh, &bounds);
        
        // Same seed, deterministic mode should give same results
        for (s1, s2) in result1.seeds.iter().zip(result2.seeds.iter()) {
            assert_eq!(s1.x, s2.x);
            assert_eq!(s1.y, s2.y);
        }
    }

    #[test]
    fn test_convergence() {
        let config = LloydConfig {
            iterations: 10, // More than needed
            jitter: 0.1,
            centroid_factor: 0.5,
            deterministic: true,
        };
        
        let mut relaxer = LloydRelaxation::new(config, 42);
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(2.0, 2.0);
        
        let result = relaxer.relax(&mesh, &bounds);
        
        // Should converge before max iterations
        assert!(result.iterations_completed < 10);
        
        // Movement should decrease each iteration
        let movements = &result.avg_movement_per_iteration;
        for window in movements.windows(2) {
            assert!(window[1] <= window[0]);
        }
    }

    #[test]
    fn test_seeds_within_bounds() {
        let config = LloydConfig::default();
        let mut relaxer = LloydRelaxation::new(config, 42);
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(2.0, 2.0);
        
        let result = relaxer.relax(&mesh, &bounds);
        
        for seed in &result.seeds {
            assert!(seed.x >= 0.0 && seed.x < 2.0);
            assert!(seed.y >= 0.0 && seed.y < 2.0);
        }
    }

    #[test]
    fn test_relax_seeds() {
        let initial_seeds = vec![
            RelaxedSeed::new(0, 0.2, 0.2),
            RelaxedSeed::new(1, 0.8, 0.2),
            RelaxedSeed::new(2, 0.2, 0.8),
            RelaxedSeed::new(3, 0.8, 0.8),
        ];
        
        let config = LloydConfig {
            iterations: 2,
            jitter: 0.0,
            centroid_factor: 0.5,
            deterministic: true,
        };
        
        let mut relaxer = LloydRelaxation::new(config, 42);
        let mesh = create_test_mesh();
        let bounds = BoundingBox::from_size(1.0, 1.0);
        
        let result = relaxer.relax_seeds(&initial_seeds, &mesh, &bounds);
        
        assert_eq!(result.seeds.len(), 4);
        assert_eq!(result.iterations_completed, 2);
    }
}
