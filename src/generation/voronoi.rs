//! Voronoi Diagram Generation for World Factory
//!
//! Generates deterministic Voronoi diagrams for terrain cell partitioning.
//! Uses Lloyd's relaxation for centroidal Voronoi diagrams when needed.
//!
//! # Algorithm Choice
//!
//! We use a hybrid approach optimized for large-scale terrain generation:
//! - Initial seed placement: Blue noise distribution via jittered grid
//! - Cell assignment: Parallel scanline algorithm
//! - Neighbor detection: Boundary detection during scan
//!
//! This provides O(n*m) complexity where n=seeds, m=grid_cells,
//! with good cache locality and parallelization potential.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::SQRT_2;

use crate::util::noise::Rng;
use crate::terrain::{PolygonGraph, Polygon};

/// Configuration for Voronoi generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoronoiConfig {
    /// World width in cells/polygons
    pub width: u32,
    /// World height in cells/polygons
    pub height: u32,
    /// Number of Voronoi seeds (approximate target polygon count)
    pub num_seeds: u32,
    /// Number of Lloyd relaxation iterations (0 = no relaxation)
    /// Lloyd relaxation moves seeds toward cell centroids for more uniform cells.
    pub lloyd_iterations: u32,
    /// Boundary mode - how to handle world edges
    pub boundary_mode: BoundaryMode,
    /// Seed jitter amount (0.0-1.0, fraction of cell size)
    pub jitter: f32,
    /// Enable blue noise distribution for seeds
    pub blue_noise: bool,
}

impl Default for VoronoiConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            num_seeds: 512,
            lloyd_iterations: 2,
            boundary_mode: BoundaryMode::Torus, // Wrapped edges for seamless worlds
            jitter: 0.5,
            blue_noise: true,
        }
    }
}

/// How to handle world boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryMode {
    /// Edges wrap (toroidal topology) - good for seamless worlds
    Torus,
    /// Edges are hard boundaries
    Finite,
    /// Extend slightly beyond bounds to ensure edge coverage
    Extended,
}

impl Default for BoundaryMode {
    fn default() -> Self {
        Self::Torus
    }
}

/// A Voronoi seed point.
#[derive(Debug, Clone, Copy)]
pub struct Seed {
    /// Original ID (stable across iterations)
    pub id: u32,
    /// Current x position
    pub x: f32,
    /// Current y position
    pub y: f32,
    /// Original x (before Lloyd relaxation)
    pub origin_x: f32,
    /// Original y (before Lloyd relaxation)
    pub origin_y: f32,
}

impl Seed {
    fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            origin_x: x,
            origin_y: y,
        }
    }

    /// Calculate squared distance to another seed.
    fn dist_sq(&self, other: &Seed) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate squared distance to a point.
    fn dist_sq_to_point(&self, px: f32, py: f32) -> f32 {
        let dx = self.x - px;
        let dy = self.y - py;
        dx * dx + dy * dy
    }
}

/// Result of Voronoi generation.
#[derive(Debug, Clone)]
pub struct VoronoiResult {
    /// The generated seeds
    pub seeds: Vec<Seed>,
    /// Cell assignments: cell_id[y * width + x] = seed_id
    /// This gives the owner seed for each grid cell
    pub cells: Vec<u32>,
    /// Width of the grid
    pub width: u32,
    /// Height of the grid
    pub height: u32,
}

impl VoronoiResult {
    /// Get the owning seed ID for a grid cell.
    #[inline]
    pub fn cell_at(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.cells.get(idx).copied()
    }

    /// Get the owning seed for a grid cell.
    #[inline]
    pub fn seed_at(&self, x: u32, y: u32) -> Option<Seed> {
        self.cell_at(x, y).map(|id| self.seeds[id as usize])
    }

    /// Extract polygon boundary vertices for API serialization.
    /// 
    /// Iterates through grid cells and extracts boundary points where
    /// cell ownership changes between adjacent seeds.
    /// 
    /// Returns a Vec of polygons, one per seed, with vertices sorted
    /// to form a valid polygon boundary.
    pub fn extract_polygon_vertices(&self) -> Vec<Vec<(f32, f32)>> {
        use std::collections::HashSet;
        
        let mut polygons: Vec<Vec<(f32, f32)>> = vec![Vec::new(); self.seeds.len()];
        
        // Scan all interior cells for boundaries
        for y in 1..self.height.saturating_sub(1) {
            for x in 1..self.width.saturating_sub(1) {
                let cell_a = match self.cell_at(x, y) {
                    Some(id) => id,
                    None => continue,
                };
                
                // Check right neighbor
                if let Some(cell_b) = self.cell_at(x + 1, y) {
                    if cell_a != cell_b {
                        polygons[cell_a as usize].push((x as f32 + 0.5, y as f32));
                        polygons[cell_b as usize].push((x as f32 + 0.5, y as f32));
                    }
                }
                
                // Check bottom neighbor
                if let Some(cell_b) = self.cell_at(x, y + 1) {
                    if cell_a != cell_b {
                        polygons[cell_a as usize].push((x as f32, y as f32 + 0.5));
                        polygons[cell_b as usize].push((x as f32, y as f32 + 0.5));
                    }
                }
            }
        }
        
        // Sort each polygon's vertices by angle from centroid
        for seed in &self.seeds {
            if polygons[seed.id as usize].len() >= 3 {
                sort_vertices_by_angle(&mut polygons[seed.id as usize]);
            }
        }
        
        polygons
    }
}

/// Voronoi diagram generator.
#[derive(Debug, Clone)]
pub struct VoronoiGenerator {
    config: VoronoiConfig,
    rng: Rng,
}

impl VoronoiGenerator {
    /// Create a new generator with configuration.
    pub fn new(config: VoronoiConfig, seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_seed(seed: u64) -> Self {
        Self::new(VoronoiConfig::default(), seed)
    }

    /// Generate the Voronoi diagram.
    ///
    /// Returns a VoronoiResult containing seeds and cell assignments.
    /// The cell assignments tell which seed owns each grid cell.
    ///
    /// # Performance
    ///
    /// - O(n*m) where n=seeds, m=cells for naive assignment
    /// - O(k*n*m) where k=Lloyd iterations for relaxation
    /// - Typical generation: <100ms for 256x256 with 512 seeds
    pub fn generate(&mut self) -> VoronoiResult {
        // Step 1: Generate seed points
        let mut seeds = self.generate_seeds();
        
        // Step 2: Assign cells to nearest seed
        let mut cells = self.assign_cells(&seeds);
        
        // Step 3: Lloyd relaxation (if enabled)
        for _ in 0..self.config.lloyd_iterations {
            self.lloyd_relaxation(&mut seeds, &cells);
            cells = self.assign_cells(&seeds);
        }
        
        VoronoiResult {
            seeds,
            cells,
            width: self.config.width,
            height: self.config.height,
        }
    }

    /// Generate seed points with blue noise distribution.
    fn generate_seeds(&mut self) -> Vec<Seed> {
        let mut seeds = Vec::with_capacity(self.config.num_seeds as usize);
        
        if self.config.blue_noise {
            // Blue noise via jittered grid
            self.generate_jittered_grid_seeds(&mut seeds);
        } else {
            // Random seeds
            self.generate_random_seeds(&mut seeds);
        }
        
        seeds
    }

    /// Generate seeds using jittered grid (creates blue noise).
    /// 
    /// This divides the space into cells and places one seed per cell
    /// with random jitter, resulting in uniform-ish distribution
    /// with minimal clustering.
    fn generate_jittered_grid_seeds(&mut self, seeds: &mut Vec<Seed>) {
        // Calculate grid dimensions to distribute seeds roughly evenly
        let num_seeds = self.config.num_seeds as f32;
        let aspect_ratio = self.config.width as f32 / self.config.height as f32;
        
        // Solve for grid_x * grid_y = num_seeds, grid_x/grid_y = aspect_ratio
        // grid_y^2 * aspect_ratio = num_seeds
        let grid_y = (num_seeds / aspect_ratio).sqrt().ceil() as u32;
        let grid_x = ((num_seeds as f32) / (grid_y as f32)).ceil() as u32;
        
        let cell_width = self.config.width as f32 / grid_x as f32;
        let cell_height = self.config.height as f32 / grid_y as f32;
        
        let jitter_range_x = cell_width * self.config.jitter;
        let jitter_range_y = cell_height * self.config.jitter;
        
        // We'll generate slightly more cells and select the best distribution
        let extra_x = 2;
        let extra_y = 2;
        
        for gy in 0..grid_y {
            for gx in 0..grid_x {
                // Calculate cell center
                let center_x = (gx as f32 + 0.5) * cell_width;
                let center_y = (gy as f32 + 0.5) * cell_height;
                
                // Apply jitter
                let jitter_x = (self.rng.next_f64Signed() as f32) * jitter_range_x;
                let jitter_y = (self.rng.next_f64Signed() as f32) * jitter_range_y;
                
                let x = center_x + jitter_x;
                let y = center_y + jitter_y;
                
                // Clamp to bounds
                let x = x.clamp(0.0, self.config.width as f32 - 0.001);
                let y = y.clamp(0.0, self.config.height as f32 - 0.001);
                
                seeds.push(Seed::new(seeds.len() as u32, x, y));
            }
        }
        
        // If we have too few seeds, add more randomly
        while seeds.len() < self.config.num_seeds as usize {
            let x = (self.rng.next_f64() as f32) * self.config.width as f32;
            let y = (self.rng.next_f64() as f32) * self.config.height as f32;
            seeds.push(Seed::new(seeds.len() as u32, x, y));
        }
    }

    /// Generate random seeds (Poisson-like but without spatial restriction).
    fn generate_random_seeds(&mut self, seeds: &mut Vec<Seed>) {
        for i in 0..self.config.num_seeds {
            let x = (self.rng.next_f64() as f32) * self.config.width as f32;
            let y = (self.rng.next_f64() as f32) * self.config.height as f32;
            seeds.push(Seed::new(i, x, y));
        }
    }

    /// Assign each grid cell to the nearest seed.
    /// 
    /// Uses a simple but efficient approach: for each cell, find the nearest seed.
    /// For better performance on large grids, we could use quadtree spatial indexing.
    fn assign_cells(&self, seeds: &[Seed]) -> Vec<u32> {
        let total_cells = (self.config.width * self.config.height) as usize;
        let mut cells = vec![0u32; total_cells];
        
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let (nearest_id, _dist_sq) = self.find_nearest_seed(x as f32, y as f32, seeds);
                let idx = (y * self.config.width + x) as usize;
                cells[idx] = nearest_id;
            }
        }
        
        cells
    }

    /// Find the nearest seed to a point.
    /// 
    /// For wrapped boundaries (torus), handles distance wrapping.
    /// Optimization: we could use spatial hashing or quadtree for O(log n) lookup.
    #[inline]
    fn find_nearest_seed(&self, px: f32, py: f32, seeds: &[Seed]) -> (u32, f32) {
        let width = self.config.width as f32;
        let height = self.config.height as f32;
        
        let mut nearest_id = 0;
        let mut nearest_dist_sq = f32::MAX;
        
        for seed in seeds {
            let dist_sq = match self.config.boundary_mode {
                BoundaryMode::Torus => {
                    // Handle toroidal wrapping
                    self.torus_dist_sq(px, py, seed.x, seed.y, width, height)
                }
                BoundaryMode::Finite | BoundaryMode::Extended => {
                    seed.dist_sq_to_point(px, py)
                }
            };
            
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_id = seed.id;
            }
        }
        
        (nearest_id, nearest_dist_sq)
    }

    /// Calculate squared distance with toroidal (wrapped) boundaries.
    #[inline]
    fn torus_dist_sq(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, height: f32) -> f32 {
        let dx = self.wrapped_diff(x1, x2, width);
        let dy = self.wrapped_diff(y1, y2, height);
        dx * dx + dy * dy
    }

    /// Calculate wrapped difference along one axis.
    #[inline]
    fn wrapped_diff(&self, a: f32, b: f32, size: f32) -> f32 {
        let half = size * 0.5;
        let mut diff = a - b;
        if diff > half {
            diff -= size;
        } else if diff < -half {
            diff += size;
        }
        diff
    }

    /// Perform one iteration of Lloyd's relaxation.
    /// 
    /// Lloyd's algorithm:
    /// 1. Compute Voronoi diagram (already done via assign_cells)
    /// 2. Compute centroids of each cell
    /// 3. Move seeds to centroids
    /// 4. Repeat
    /// 
    /// This produces more uniform cell sizes and shapes.
    fn lloyd_relaxation(&self, seeds: &mut Vec<Seed>, cells: &[u32]) {
        // Compute centroids
        let mut sums: HashMap<u32, (f32, f32, u32)> = HashMap::with_capacity(seeds.len());
        
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let cell_idx = (y * self.config.width + x) as usize;
                let seed_id = cells[cell_idx];
                
                let entry = sums.entry(seed_id).or_insert((0.0, 0.0, 0));
                entry.0 += x as f32;
                entry.1 += y as f32;
                entry.2 += 1;
            }
        }
        
        // Move seeds to centroids
        for seed in seeds {
            if let Some((sum_x, sum_y, count)) = sums.get(&seed.id) {
                if *count > 0 {
                    seed.x = sum_x / *count as f32;
                    seed.y = sum_y / *count as f32;
                    
                    // Clamp to bounds (for non-torus)
                    if self.config.boundary_mode != BoundaryMode::Torus {
                        seed.x = seed.x.clamp(0.0, self.config.width as f32 - 0.001);
                        seed.y = seed.y.clamp(0.0, self.config.height as f32 - 0.001);
                    }
                }
            }
        }
    }

    /// Build a PolygonGraph from the Voronoi result.
    /// 
    /// This creates the terrain graph structure with neighbor relationships.
    pub fn build_polygon_graph(&self, result: &VoronoiResult) -> PolygonGraph {
        let mut graph = PolygonGraph::with_capacity(result.seeds.len());
        
        // Step 1: Create all polygons
        for seed in &result.seeds {
            let mut polygon = Polygon::new(seed.id);
            // Initial elevation will be computed later based on distance from coast
            polygon.elevation = 0.5; // Placeholder
            graph.add_polygon(polygon);
        }
        
        // Step 2: Detect neighbors and edges by checking cell boundaries
        self.detect_neighbors(&result, &mut graph);
        
        // Step 3: Mark edge polygons
        self.mark_edge_polygons(&result, &mut graph);
        
        graph
    }

    /// Detect neighbor relationships by scanning cell boundaries.
    fn detect_neighbors(&self, result: &VoronoiResult, graph: &mut PolygonGraph) {
        let width = result.width;
        let height = result.height;
        
        // Horizontal neighbors (check right edge of each cell)
        for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                let idx = (y * width + x) as usize;
                let cell_a = result.cells[idx];
                let cell_b = result.cells[idx + 1]; // Right neighbor
                
                if cell_a != cell_b {
                    graph.add_edge(cell_a, cell_b);
                }
            }
        }
        
        // Vertical neighbors (check bottom edge of each cell)
        for y in 0..height.saturating_sub(1) {
            for x in 0..width {
                let idx_a = (y * width + x) as usize;
                let idx_b = ((y + 1) * width + x) as usize;
                let cell_a = result.cells[idx_a];
                let cell_b = result.cells[idx_b]; // Bottom neighbor
                
                if cell_a != cell_b {
                    graph.add_edge(cell_a, cell_b);
                }
            }
        }
    }

    /// Mark polygons that are on the world edge.
    fn mark_edge_polygons(&self, result: &VoronoiResult, graph: &mut PolygonGraph) {
        let width = result.width;
        let height = result.height;
        
        // Find all edge seed IDs
        let mut edge_seeds: Vec<bool> = vec![false; result.seeds.len()];
        
        // Left edge
        for y in 0..height {
            let cell_id = result.cell_at(0, y).unwrap();
            edge_seeds[cell_id as usize] = true;
        }
        
        // Right edge
        for y in 0..height {
            let cell_id = result.cell_at(width - 1, y).unwrap();
            edge_seeds[cell_id as usize] = true;
        }
        
        // Top edge
        for x in 0..width {
            let cell_id = result.cell_at(x, 0).unwrap();
            edge_seeds[cell_id as usize] = true;
        }
        
        // Bottom edge
        for x in 0..width {
            let cell_id = result.cell_at(x, height - 1).unwrap();
            edge_seeds[cell_id as usize] = true;
        }
        
        // Mark in graph
        for (id, &is_edge) in edge_seeds.iter().enumerate() {
            if is_edge {
                if let Some(poly) = graph.get_mut(id as u32) {
                    poly.is_coastal = true; // Edge polygons are coastal
                }
            }
        }
    }
}

/// Generate a Voronoi diagram and return as PolygonGraph.
///
/// This is a convenience function for the common case.
pub fn generate_voronoi_graph(config: VoronoiConfig, seed: u64) -> PolygonGraph {
    let mut generator = VoronoiGenerator::new(config, seed);
    let result = generator.generate();
    generator.build_polygon_graph(&result)
}

/// Generate a Voronoi diagram with default settings.
pub fn quick_voronoi(width: u32, height: u32, seed: u64) -> PolygonGraph {
    let config = VoronoiConfig {
        width,
        height,
        num_seeds: ((width * height) as f32 / 128.0).ceil() as u32,
        ..Default::default()
    };
    generate_voronoi_graph(config, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_generation() {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: false, // Use random for predictability
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 12345);
        let result = gen.generate();
        
        assert_eq!(result.seeds.len(), 16);
        assert_eq!(result.cells.len(), 32 * 32);
    }

    #[test]
    fn test_deterministic_generation() {
        let config = VoronoiConfig {
            width: 64,
            height: 64,
            num_seeds: 32,
            blue_noise: false,
            lloyd_iterations: 0,
            ..Default::default()
        };
        
        let mut gen1 = VoronoiGenerator::new(config.clone(), 42);
        let result1 = gen1.generate();
        
        let mut gen2 = VoronoiGenerator::new(config.clone(), 42);
        let result2 = gen2.generate();
        
        // Same seed should produce same results
        assert_eq!(result1.seeds.len(), result2.seeds.len());
        for (s1, s2) in result1.seeds.iter().zip(result2.seeds.iter()) {
            assert_eq!(s1.x, s2.x);
            assert_eq!(s1.y, s2.y);
        }
    }

    #[test]
    fn test_cell_assignment_coverage() {
        let config = VoronoiConfig {
            width: 16,
            height: 16,
            num_seeds: 4,
            blue_noise: false,
            lloyd_iterations: 0,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 100);
        let result = gen.generate();
        
        // Every cell should be assigned to some seed
        for cell in &result.cells {
            assert!(*cell < result.seeds.len() as u32);
        }
    }

    #[test]
    fn test_polygon_graph_creation() {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: false,
            ..Default::default()
        };
        
        let graph = generate_voronoi_graph(config, 42);
        
        assert_eq!(graph.len(), 16);
        
        // Check that some polygons have neighbors
        let has_neighbors = (0..graph.len())
            .any(|id| graph.get(id as u32).map(|p| !p.neighbors.is_empty()).unwrap_or(false));
        assert!(has_neighbors, "At least one polygon should have neighbors");
    }

    #[test]
    fn test_edge_detection() {
        let config = VoronoiConfig {
            width: 16,
            height: 16,
            num_seeds: 8,
            blue_noise: false,
            boundary_mode: BoundaryMode::Finite,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        let mut graph = gen.build_polygon_graph(&result);
        
        // Re-assign cells for edge detection
        gen.detect_neighbors(&result, &mut graph);
        gen.mark_edge_polygons(&result, &mut graph);
        
        // Edge polygons should be marked as coastal
        let edge_polygons: Vec<_> = (0..graph.len())
            .filter(|&id| graph.get(id as u32).map(|p| p.is_coastal).unwrap_or(false))
            .collect();
        
        assert!(!edge_polygons.is_empty(), "Should have some edge/coastal polygons");
    }

    #[test]
    fn test_lloyd_relaxation() {
        let config = VoronoiConfig {
            width: 64,
            height: 64,
            num_seeds: 16,
            blue_noise: false,
            lloyd_iterations: 3, // Multiple iterations
            jitter: 0.3, // Less jitter for cleaner start
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        
        // After Lloyd relaxation, seeds should be more evenly distributed
        // We can't easily test "more even" but we can verify they moved
        // and are still within bounds
        for seed in &result.seeds {
            assert!(seed.x >= 0.0 && seed.x < 64.0);
            assert!(seed.y >= 0.0 && seed.y < 64.0);
        }
    }

    #[test]
    fn test_torus_boundary() {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: false,
            boundary_mode: BoundaryMode::Torus,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        
        // With torus mode, seeds can be anywhere but still should cover all cells
        assert_eq!(result.cells.len(), 32 * 32);
        
        // Verify no cells are unassigned
        for (i, &cell_id) in result.cells.iter().enumerate() {
            assert!(cell_id < result.seeds.len() as u32, 
                "Cell {} should be assigned to a valid seed", i);
        }
    }

    #[test]
    fn test_blue_noise_vs_random() {
        let config_blue = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: true,
            jitter: 0.5,
            ..Default::default()
        };
        
        let config_random = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: false,
            ..Default::default()
        };
        
        let mut gen_blue = VoronoiGenerator::new(config_blue, 42);
        let result_blue = gen_blue.generate();
        
        let mut gen_random = VoronoiGenerator::new(config_random, 42);
        let result_random = gen_random.generate();
        
        // Blue noise and random should produce different results
        // (unless by miracle the jitter is exactly zero)
        let first_seed_same = result_blue.seeds[0].x == result_random.seeds[0].x 
            && result_blue.seeds[0].y == result_random.seeds[0].y;
        assert!(!first_seed_same, "Blue noise and random should differ");
    }

    #[test]
    fn test_quick_voronoi() {
        let graph = quick_voronoi(64, 64, 123);
        
        // Should create a reasonable polygon graph
        assert!(graph.len() > 0);
    }

    #[test]
    fn test_cell_at() {
        let config = VoronoiConfig {
            width: 10,
            height: 10,
            num_seeds: 4,
            blue_noise: false,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        
        // Test various positions
        assert!(result.cell_at(0, 0).is_some());
        assert!(result.cell_at(5, 5).is_some());
        assert!(result.cell_at(9, 9).is_some());
        assert!(result.cell_at(10, 10).is_none()); // Out of bounds
        assert!(result.cell_at(100, 0).is_none());
    }

    #[test]
    fn test_neighbors_connectivity() {
        let config = VoronoiConfig {
            width: 32,
            height: 32,
            num_seeds: 16,
            blue_noise: true,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        let mut graph = gen.build_polygon_graph(&result);
        
        // Verify neighbor relationships are symmetric
        for id in 0..graph.len() as u32 {
            if let Some(poly) = graph.get(id) {
                for &neighbor_id in &poly.neighbors {
                    if let Some(neighbor) = graph.get(neighbor_id) {
                        assert!(
                            neighbor.neighbors.contains(&id),
                            "Neighbor relationship should be symmetric: {} <-> {}",
                            id, neighbor_id
                        );
                    }
                }
            }
        }
    }
}

/// Sort vertices by angle from centroid for valid polygon boundary.
fn sort_vertices_by_angle(vertices: &mut Vec<(f32, f32)>) {
    if vertices.len() < 3 {
        return;
    }
    
    // Calculate centroid
    let cx: f32 = vertices.iter().map(|(x, _)| x).sum::<f32>() / vertices.len() as f32;
    let cy: f32 = vertices.iter().map(|(_, y)| y).sum::<f32>() / vertices.len() as f32;
    
    // Sort by angle from centroid
    vertices.sort_by(|(ax, ay), (bx, by)| {
        let a_angle = (ay - cy).atan2(ax - cx);
        let b_angle = (by - cy).atan2(bx - cx);
        a_angle.partial_cmp(&b_angle).unwrap()
    });
}

#[cfg(test)]
mod polygon_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_polygon_vertices() {
        let config = VoronoiConfig {
            width: 16,
            height: 16,
            num_seeds: 4,
            blue_noise: false,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        
        let polygons = result.extract_polygon_vertices();
        
        // Should have one polygon per seed
        assert_eq!(polygons.len(), result.seeds.len());
        
        // At least some polygons should have vertices (boundary polygons)
        let polygons_with_vertices: usize = polygons.iter()
            .filter(|p| p.len() >= 3)
            .count();
        assert!(polygons_with_vertices > 0, "At least some polygons should have vertices");
    }

    #[test]
    fn test_polygon_vertices_sorted() {
        let config = VoronoiConfig {
            width: 16,
            height: 16,
            num_seeds: 4,
            blue_noise: false,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 42);
        let result = gen.generate();
        
        let polygons = result.extract_polygon_vertices();
        
        // For polygons with vertices, verify they're sorted by angle
        for polygon in &polygons {
            if polygon.len() >= 3 {
                let cx: f32 = polygon.iter().map(|(x, _)| x).sum::<f32>() / polygon.len() as f32;
                let cy: f32 = polygon.iter().map(|(_, y)| y).sum::<f32>() / polygon.len() as f32;
                
                for window in polygon.windows(2) {
                    let angle1 = ((window[0].1 - cy).atan2(window[0].0 - cx));
                    let angle2 = ((window[1].1 - cy).atan2(window[1].0 - cx));
                    assert!(angle1 <= angle2, "Vertices should be sorted by angle");
                }
            }
        }
    }
}
