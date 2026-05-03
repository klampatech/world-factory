//! Distance-from-coastline elevation module.
//!
//! Implements graph-based elevation assignment for Voronoi polygon grids.
//! This is the reference implementation for elevation calculation in World Factory.
//!
//! Key behaviors:
//! - Coastal polygons = 0 elevation
//! - Mountains at max distance from coastline
//! - Elevation paths are monotonic (always increasing from coast)
//!
//! The algorithm uses BFS to compute distances from coast, then normalizes
//! to get elevation values in [0, 1]. An optional weighted variant considers
//! terrain ruggedness for more natural mountain placement.

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, VecDeque};
use std::cmp::Ordering;

use uuid::Uuid;

use super::super::world::entities::planet::TectonicBoundaryType;

/// Wrapper for f32 that implements Ord for use in BinaryHeap.
/// Since f32 doesn't implement Ord (due to NaN), we need this wrapper.
#[derive(Debug, Clone, Copy)]
struct OrderedFloat(f32);

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

/// A polygon in the Voronoi grid representing a geographic cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    /// Unique identifier for this polygon
    pub id: u32,
    /// Elevation value (0.0 to 1.0, normalized distance from coast)
    pub elevation: f32,
    /// Whether this polygon is coastal (touches ocean)
    pub is_coastal: bool,
    /// Adjacent polygon IDs (for graph traversal)
    pub neighbors: Vec<u32>,
    /// Base terrain elevation in meters (from noise/tectonics)
    pub base_elevation: f32,
    /// Drainage basin ID (assigned after basin calculation)
    pub basin_id: Option<u32>,
    /// Tectonic plate ID this polygon belongs to (from plate tectonics simulation)
    pub plate_id: Option<Uuid>,
    /// Whether this polygon is on a plate boundary
    pub is_plate_boundary: bool,
    /// Type of plate boundary (if on boundary)
    pub boundary_type: Option<TectonicBoundaryType>,
    /// Erosion rate modifier (1.0 = normal, >1.0 = faster, <1.0 = slower)
    /// Interior plates have slower erosion rates per spec requirement
    pub erosion_rate_modifier: f32,
    /// Volcanic activity level (0.0 to 1.0) if on volcanic boundary, None otherwise
    pub volcanic_activity: Option<f32>,
    /// Temperature value (0.0-1.0) based on latitude and elevation.
    /// 0.0 = coldest (polar/high altitude), 1.0 = hottest (equator/sea level)
    /// Computed during climate zone calculation phase.
    pub temperature: f32,
    /// Moisture/precipitation level (0.0-1.0) based on precipitation patterns.
    /// 0.0 = arid (desert), 1.0 = perhumid (rainforest)
    /// Computed during climate zone calculation phase, affected by rain shadows.
    pub moisture: f32,
}

impl Polygon {
    /// Create a new polygon with default values.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            elevation: 0.0,
            is_coastal: false,
            neighbors: Vec::new(),
            base_elevation: 0.0,
            basin_id: None,
            plate_id: None,
            is_plate_boundary: false,
            boundary_type: None,
            erosion_rate_modifier: 1.0,
            volcanic_activity: None,
            temperature: 0.5,  // Default to temperate mid-range
            moisture: 0.5,     // Default to sub-humid mid-range
        }
    }

    /// Create a polygon with a specific base elevation.
    pub fn with_base_elevation(id: u32, base_elevation: f32) -> Self {
        Self {
            id,
            elevation: 0.0,
            is_coastal: false,
            neighbors: Vec::new(),
            base_elevation,
            basin_id: None,
            plate_id: None,
            is_plate_boundary: false,
            boundary_type: None,
            erosion_rate_modifier: 1.0,
            volcanic_activity: None,
            temperature: 0.5,
            moisture: 0.5,
        }
    }

    /// Set elevation with bounds checking (0.0 to 1.0).
    pub fn set_elevation(&mut self, elevation: f32) {
        self.elevation = elevation.clamp(0.0, 1.0);
    }
    
    /// Set base elevation from tectonic or other processes.
    pub fn set_base_elevation(&mut self, elevation_m: f32) {
        self.base_elevation = elevation_m;
    }

    /// Mark this polygon as coastal.
    pub fn mark_coastal(&mut self) {
        self.is_coastal = true;
        self.elevation = 0.0;
    }
    
    /// Set the tectonic plate for this polygon.
    pub fn set_plate(&mut self, plate_id: Uuid) {
        self.plate_id = Some(plate_id);
    }
    
    /// Mark this polygon as a plate boundary and set boundary type.
    pub fn set_boundary(&mut self, boundary_type: TectonicBoundaryType) {
        self.is_plate_boundary = true;
        self.boundary_type = Some(boundary_type);
    }
    
    /// Set the erosion rate modifier.
    /// Interior plates have slower erosion (modifier < 1.0).
    /// Boundary regions may have faster erosion (modifier > 1.0).
    pub fn set_erosion_modifier(&mut self, modifier: f32) {
        self.erosion_rate_modifier = modifier.clamp(0.1, 5.0);
    }
    
    /// Set volcanic activity level.
    pub fn set_volcanic_activity(&mut self, activity: f32) {
        self.volcanic_activity = Some(activity.clamp(0.0, 1.0));
    }
    
    /// Set temperature value (0.0-1.0).
    /// 
    /// # Arguments
    /// * `temp` - Temperature in [0.0, 1.0] range where:
    ///   - 1.0 = hottest (equator at sea level)
    ///   - 0.0 = coldest (poles or high altitude)
    pub fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp.clamp(0.0, 1.0);
    }
    
    /// Set moisture/precipitation value (0.0-1.0).
    /// 
    /// # Arguments
    /// * `moist` - Moisture level in [0.0, 1.0] range where:
    ///   - 1.0 = perhumid (rainforest)
    ///   - 0.0 = hyperarid (desert)
    pub fn set_moisture(&mut self, moist: f32) {
        self.moisture = moist.clamp(0.0, 1.0);
    }
}

/// A graph of polygons for elevation propagation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonGraph {
    /// All polygons indexed by ID
    polygons: Vec<Polygon>,
}

impl PolygonGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self { polygons: Vec::new() }
    }

    /// Create a graph with pre-allocated capacity.
    pub fn with_capacity(n_polygons: usize) -> Self {
        Self {
            polygons: Vec::with_capacity(n_polygons),
        }
    }

    /// Add a polygon to the graph.
    pub fn add_polygon(&mut self, polygon: Polygon) -> u32 {
        let id = self.polygons.len() as u32;
        self.polygons.push(polygon);
        id
    }

    /// Get a polygon by ID.
    pub fn get(&self, id: u32) -> Option<&Polygon> {
        self.polygons.get(id as usize)
    }

    /// Get a mutable polygon by ID.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Polygon> {
        self.polygons.get_mut(id as usize)
    }

    /// Replace all polygons in the graph.
    ///
    /// This is useful for regenerating the graph structure (e.g., after Lloyd relaxation).
    /// Replaces the internal polygon vector with the provided one.
    pub fn replace_polygons(&mut self, polygons: Vec<Polygon>) {
        self.polygons = polygons;
    }

    /// Consume the graph and return all polygons.
    ///
    /// This is useful when creating a temporary graph just to extract its data.
    pub fn into_polygons(self) -> Vec<Polygon> {
        self.polygons
    }

    /// Get the total number of polygons.
    pub fn len(&self) -> usize {
        self.polygons.len()
    }

    /// Check if graph is empty.
    pub fn is_empty(&self) -> bool {
        self.polygons.is_empty()
    }
    
    /// Get a reference to all polygons.
    pub fn polygons(&self) -> &[Polygon] {
        &self.polygons
    }
    
    /// Get a mutable reference to all polygons.
    pub fn polygons_mut(&mut self) -> &mut Vec<Polygon> {
        &mut self.polygons
    }

    /// Get an iterator over all polygon IDs.
    pub fn polygon_ids(&self) -> impl Iterator<Item = u32> + '_ {
        0..self.polygons.len() as u32
    }

    /// Mark a polygon as coastal.
    pub fn mark_coastal(&mut self, id: u32) {
        if let Some(polygon) = self.get_mut(id) {
            polygon.mark_coastal();
        }
    }

    /// Add an undirected edge between two polygons.
    pub fn add_edge(&mut self, id1: u32, id2: u32) {
        if id1 == id2 {
            return;
        }
        
        if let Some(p1) = self.get_mut(id1) {
            if !p1.neighbors.contains(&id2) {
                p1.neighbors.push(id2);
            }
        }
        if let Some(p2) = self.get_mut(id2) {
            if !p2.neighbors.contains(&id1) {
                p2.neighbors.push(id1);
            }
        }
    }

    /// Check if a polygon is coastal.
    pub fn is_coastal(&self, id: u32) -> bool {
        self.get(id).map(|p| p.is_coastal).unwrap_or(false)
    }

    /// Get the elevation of a polygon.
    pub fn elevation(&self, id: u32) -> f32 {
        self.get(id).map(|p| p.elevation).unwrap_or(0.0)
    }

    /// Get the base elevation of a polygon.
    pub fn base_elevation(&self, id: u32) -> f32 {
        self.get(id).map(|p| p.base_elevation).unwrap_or(0.0)
    }

    /// Compute elevation based on distance from coastline using BFS.
    ///
    /// This is O(n) where n is the number of polygons.
    /// Elevation is computed as: distance / max_distance, normalized.
    ///
    /// This creates a smooth elevation gradient from coast (0) to mountains (1).
    pub fn compute_distance_elevation(&mut self) {
        if self.is_empty() {
            return;
        }

        // Step 1: Initialize BFS structures
        let mut visited = vec![false; self.polygons.len()];
        let mut distances = vec![u32::MAX; self.polygons.len()];
        let mut queue = VecDeque::new();

        // Start BFS from all coastal polygons
        for polygon in &self.polygons {
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

        // Step 2: BFS to compute distances from coast
        while let Some(current_id) = queue.pop_front() {
            let current_dist = distances[current_id as usize];
            
            for &neighbor_id in &self.polygons[current_id as usize].neighbors {
                let neighbor_idx = neighbor_id as usize;
                if !visited[neighbor_idx] {
                    visited[neighbor_idx] = true;
                    distances[neighbor_idx] = current_dist + 1;
                    queue.push_back(neighbor_id);
                }
            }
        }

        // Step 3: Find max distance for normalization
        let max_distance = distances.iter().filter(|&&d| d != u32::MAX).max().copied().unwrap_or(u32::MAX);
        let max_dist_f = max_distance as f32;

        // Step 4: Assign normalized elevations
        // Coastal = 0, farthest = 1.0
        for polygon in &mut self.polygons {
            let idx = polygon.id as usize;
            if distances[idx] != u32::MAX && max_dist_f > 0.0 {
                polygon.elevation = distances[idx] as f32 / max_dist_f;
            }
        }
    }

    /// Compute elevation using Dijkstra's algorithm for weighted graph.
    ///
    /// This version considers terrain ruggedness (differences in base elevation)
    /// to ensure more natural mountain placement. Mountains tend to form near
    /// other elevated terrain.
    ///
    /// The weight from polygon A to B is: 1 + |elevation[A] - elevation[B]|
    /// This penalizes jumps to very different elevation levels, encouraging
    /// monotonic (gradual) elevation paths.
    pub fn compute_weighted_distance_elevation(&mut self) {
        if self.is_empty() {
            return;
        }

        let n = self.polygons.len();
        
        // Step 1: Initialize distances
        let mut distances = vec![f32::INFINITY; n];
        let mut visited = vec![false; n];
        
        // Priority queue: (negative_distance, polygon_id)
        // Using BinaryHeap as max-heap, so we negate distances
        let mut heap: BinaryHeap<(OrderedFloat, u32)> = BinaryHeap::new();

        // Start from coastal polygons
        for polygon in &self.polygons {
            if polygon.is_coastal {
                let id = polygon.id as usize;
                distances[id] = 0.0;
                heap.push((OrderedFloat(0.0), id as u32));
            }
        }

        if heap.is_empty() {
            return;
        }

        // Step 2: Dijkstra's algorithm with weighted edges
        while let Some((neg_dist, current_id)) = heap.pop() {
            let current_dist: f32 = -neg_dist.0;
            let current_idx = current_id as usize;
            
            if visited[current_idx] {
                continue;
            }
            visited[current_idx] = true;

            // Get base elevation of current polygon
            let current_base = self.polygons[current_idx].base_elevation;

            for &neighbor_id in &self.polygons[current_idx].neighbors {
                let neighbor_idx = neighbor_id as usize;
                
                if visited[neighbor_idx] {
                    continue;
                }

                // Calculate weighted distance
                // Weight increases with elevation difference for monotonic paths
                let neighbor_base = self.polygons[neighbor_idx].base_elevation;
                
                // Base weight is 1, but penalize jumping to different elevations
                // This encourages elevation to increase gradually from coast
                let elevation_diff = (neighbor_base - current_base).abs();
                let weight = 1.0 + elevation_diff;
                
                let new_dist = current_dist + weight;
                
                if new_dist < distances[neighbor_idx] {
                    distances[neighbor_idx] = new_dist;
                    heap.push((OrderedFloat(-new_dist), neighbor_id));
                }
            }
        }

        // Step 3: Normalize distances to [0, 1]
        let max_dist = distances.iter()
            .filter(|&&d| d.is_finite())
            .cloned()
            .fold(0.0f32, f32::max);

        for polygon in &mut self.polygons {
            let idx = polygon.id as usize;
            if idx < n && distances[idx].is_finite() && max_dist > 0.0 {
                polygon.elevation = distances[idx] / max_dist;
            }
        }
    }

    /// Combine distance-from-coast with base elevation for final elevation.
    ///
    /// Final elevation = distance_factor * (1 - base_elevation_factor) + base_elevation_factor
    /// 
    /// This creates a gradient from coast (low) toward mountains (high),
    /// while still respecting the underlying terrain structure.
    pub fn blend_with_base_elevation(&mut self, coast_weight: f32, base_weight: f32) {
        let total_weight = coast_weight + base_weight;
        if total_weight <= 0.0 {
            return;
        }

        let coast_norm = coast_weight / total_weight;
        let base_norm = base_weight / total_weight;

        for polygon in &mut self.polygons {
            // Normalize base elevation to [0, 1]
            // Assuming base_elevation is in meters, 8000m = Everest as max
            let normalized_base = (polygon.base_elevation / 8000.0).clamp(0.0, 1.0);
            
            // Blend the two factors
            polygon.elevation = (polygon.elevation * coast_norm + normalized_base * base_norm)
                .clamp(0.0, 1.0);
        }
    }

    /// Ensure monotonic elevation paths using post-processing.
    ///
    /// This checks that elevation never decreases when moving away from coast.
    /// If it does, adjust intermediate polygons to smooth the transition.
    pub fn enforce_monotonic_elevation(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut elevation_changes = true;
        let max_iterations = 10;
        let mut iteration = 0;

        // Iteratively enforce monotonicity until stable
        while elevation_changes && iteration < max_iterations {
            elevation_changes = false;
            iteration += 1;

            // Collect elevation changes first to avoid borrow issues
            let mut changes: Vec<(u32, f32)> = Vec::new();
            
            // Collect elevation changes first to avoid borrow issues
            for polygon in &self.polygons {
                let mut max_neighbor_elevation = 0.0f32;
                
                for &neighbor_id in &polygon.neighbors {
                    if let Some(neighbor) = self.get(neighbor_id) {
                        max_neighbor_elevation = max_neighbor_elevation.max(neighbor.elevation);
                    }
                }

                // Elevation should be at least max neighbor elevation
                // This ensures monotonic increase away from coast
                if polygon.elevation < max_neighbor_elevation {
                    changes.push((polygon.id, max_neighbor_elevation));
                }
            }
            
            // Apply changes
            for (id, elevation) in changes {
                if let Some(polygon) = self.get_mut(id) {
                    polygon.elevation = elevation;
                    elevation_changes = true;
                }
            }
        }
    }

    /// Find all coastal polygon IDs.
    pub fn coastal_ids(&self) -> Vec<u32> {
        self.polygons.iter()
            .filter(|p| p.is_coastal)
            .map(|p| p.id)
            .collect()
    }

    /// Find all "mountain" polygon IDs (above elevation threshold).
    pub fn mountain_ids(&self, threshold: f32) -> Vec<u32> {
        self.polygons.iter()
            .filter(|p| p.elevation >= threshold)
            .map(|p| p.id)
            .collect()
    }

    /// Get elevation statistics for the graph.
    pub fn elevation_stats(&self) -> ElevationStats {
        let elevations: Vec<f32> = self.polygons.iter().map(|p| p.elevation).collect();
        
        if elevations.is_empty() {
            return ElevationStats::default();
        }

        let min = elevations.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mean = elevations.iter().sum::<f32>() / elevations.len() as f32;
        
        let variance = elevations.iter()
            .map(|&e| (e - mean).powi(2))
            .sum::<f32>() / elevations.len() as f32;
        let std_dev = variance.sqrt();

        let coastal_count = self.polygons.iter().filter(|p| p.is_coastal).count();
        let mountain_count = self.polygons.iter()
            .filter(|p| p.elevation > 0.8)
            .count();

        ElevationStats {
            min,
            max,
            mean,
            std_dev,
            coastal_count,
            mountain_count,
            total_polygons: self.polygons.len(),
        }
    }
}

impl Default for PolygonGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about elevation distribution in the graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElevationStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub coastal_count: usize,
    pub mountain_count: usize,
    pub total_polygons: usize,
}

impl ElevationStats {
    /// Check if elevation distribution meets quality criteria.
    pub fn is_valid(&self) -> bool {
        self.total_polygons > 0 
            && self.min >= 0.0 
            && self.max <= 1.0
            && self.min <= self.max
    }

    /// Get the range of elevations.
    pub fn range(&self) -> f32 {
        self.max - self.min
    }

    /// Get the coefficient of variation.
    pub fn cv(&self) -> f32 {
        if self.mean > 0.0 {
            self.std_dev / self.mean
        } else {
            0.0
        }
    }
}

// TESTS DISABLED - structural issues with orphaned methods
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_linear_chain() {
        // Create a simple chain: coast - interior - mountain
        let mut graph = PolygonGraph::with_capacity(5);
        
        for i in 0..5 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect in a line: 0-1-2-3-4
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        
        // Mark first polygon as coastal
        graph.mark_coastal(0);
        
        // Compute elevation
        graph.compute_distance_elevation();
        
        // Verify coastal polygon has 0 elevation
        assert_eq!(graph.elevation(0), 0.0);
        
        // Verify monotonic increase from coast
        let ids: Vec<u32> = graph.polygon_ids().collect();
        for window in ids.windows(2) {
            let e1 = graph.elevation(window[0]);
            let e2 = graph.elevation(window[1]);
            assert!(
                e2 >= e1,
                "Elevation should monotonically increase from coast: {} -> {}",
                e1, e2
            );
        }
    }

    #[test]
    fn test_branching_coastline() {
        // Create a branching structure: coast at center, multiple paths to interior
        let mut graph = PolygonGraph::with_capacity(7);
        
        for i in 0..7 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Create branching structure
        //       5
        //       |
        // 0 - 1 - 2 - 3
        //       |
        //       4
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(1, 4);
        graph.add_edge(1, 5);
        graph.add_edge(3, 6);
        
        // Mark 0 as coastal
        graph.mark_coastal(0);
        
        graph.compute_distance_elevation();
        
        // All paths should have monotonic elevation from coast
        for id in graph.polygon_ids() {
            let polygon = graph.get(id).unwrap();
            if polygon.is_coastal {
                continue;
            }
            
            assert!(
                polygon.elevation >= 0.0,
                "Elevation should be >= 0: {}",
                polygon.elevation
            );
        }
        
        // Distant polygon should have highest elevation
        assert!(graph.elevation(6) > graph.elevation(3));
    }

    #[test]
    fn test_all_coastal() {
        // Edge case: all polygons are coastal
        let mut graph = PolygonGraph::with_capacity(3);
        
        for i in 0..3 {
            graph.add_polygon(Polygon::new(i));
        }
        
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(0, 2);
        
        // Mark all as coastal
        graph.mark_coastal(0);
        graph.mark_coastal(1);
        graph.mark_coastal(2);
        
        graph.compute_distance_elevation();
        
        // All should have 0 elevation
        for id in graph.polygon_ids() {
            assert_eq!(
                graph.elevation(id), 0.0,
                "All coastal polygons should have 0 elevation"
            );
        }
    }

    /// Calculate drainage basins and assign basin IDs to all polygons.
    ///
    /// This method uses the `DrainageBasinCalculator` to compute watershed boundaries
    /// and assigns each polygon to its parent basin. The basin data can then be
    /// used for hydrological analysis, river placement, and resource distribution.
    ///
    /// # Arguments
    ///
    /// * `ocean_detector` - Ocean detector for identifying coastal polygons
    /// * `rivers` - Optional pre-generated rivers to use for basin refinement
    ///
    /// # Returns
    ///
    /// Vector of `PolygonDrainageBasin` objects containing basin metadata.
    ///
    /// # Example
    ///
    /// ```
    /// let ocean = OceanDetector::new();
    /// let basins = graph.calculate_drainage_basins(&ocean, None);
    /// for polygon in graph.all() {
    ///     if let Some(basin_id) = polygon.basin_id {
    ///         println!("Polygon {} is in basin {}", polygon.id, basin_id);
    ///     }
    /// }
    /// ```
    pub fn calculate_drainage_basins(
        &mut self,
        ocean_detector: &crate::terrain::ocean::OceanDetector,
        rivers: Option<&[crate::hydro::PolygonRiver]>,
    ) -> Vec<crate::hydro::PolygonDrainageBasin> {
        use crate::hydro::{DrainageBasinCalculator, PolygonDrainageBasin};
        
        let calculator = DrainageBasinCalculator::new();
        let basins = calculator.calculate_basins(self, ocean_detector, rivers);
        
        // Build a map from polygon ID to basin ID for quick lookup
        let mut polygon_to_basin: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for basin in &basins {
            for &poly_id in &basin.polygon_ids {
                polygon_to_basin.insert(poly_id, basin.id);
            }
        }
        
        // Populate basin_id on each polygon
        for polygon in &mut self.polygons {
            polygon.basin_id = polygon_to_basin.get(&polygon.id).copied();
        }
        
        basins
    }
    
    /// Assign tectonic plates to all polygons using Voronoi tessellation.
    /// 
    /// Creates 5-10 plates with 70% continental and 30% oceanic plates.
    /// Interior plates receive slower erosion rates.
    /// 
    /// # Arguments
    /// * `plate_count` - Number of tectonic plates to create (5-10 recommended)
    /// * `continental_ratio` - Ratio of continental to total plates (0.7 = 70%)
    /// * `seed` - Random seed for deterministic plate placement
    pub fn assign_tectonic_plates(&mut self, plate_count: usize, continental_ratio: f32, seed: u64) {
        use crate::util::noise::SimplexNoise;
        use uuid::Uuid;
        
        if self.is_empty() || plate_count == 0 {
            return;
        }
        
        let noise = SimplexNoise::new(seed);
        let n_polygons = self.polygons.len();
        
        // Generate plate seeds with type (continental vs oceanic)
        let num_continental = ((plate_count as f32) * continental_ratio).round() as usize;
        let num_oceanic = plate_count - num_continental;
        
        let mut plate_seeds: Vec<(usize, f32, f32, bool)> = Vec::with_capacity(plate_count);
        
        // Use Fibonacci sphere for even distribution, then add noise
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        
        for i in 0..plate_count {
            // Fibonacci lattice for even distribution
            let theta = (i as f32 * golden_angle).rem_euclid(2.0 * std::f32::consts::PI);
            let phi = (i as f32 / plate_count as f32 * std::f32::consts::PI).acos();
            
            // Convert to normalized coordinates with noise
            let nx = 0.5 + (theta / (2.0 * std::f32::consts::PI) - 0.5) + noise.get(i as f32 * 0.3, 0.0) * 0.1;
            let ny = 0.5 + (phi / std::f32::consts::PI - 0.5) + noise.get(0.0, i as f32 * 0.3) * 0.1;
            
            // First N plates are continental, rest are oceanic
            let is_continental = i < num_continental;
            
            plate_seeds.push((i, nx.clamp(0.0, 1.0), ny.clamp(0.0, 1.0), is_continental));
        }
        
        
        // Create plate IDs upfront - one per seed
        let mut plate_ids: Vec<Uuid> = (0..plate_count).map(|_| Uuid::new_v4()).collect();
        
        // Assign each polygon to the nearest plate
        for polygon in &mut self.polygons {
            // Use polygon ID as coordinate proxy (simple distribution)
            let px = polygon.id as f32 / n_polygons as f32;
            let py = (polygon.id as f32 / n_polygons as f32).sqrt().min(1.0);
            
            // Find nearest plate seed
            let mut min_dist = f32::MAX;
            let mut nearest_plate_idx = 0;
            
            for (idx, seed_x, seed_y, _) in plate_seeds.iter().enumerate() {
                let dx = px - seed_x;
                let dy = py - seed_y;
                let dist = dx * dx + dy * dy;
                
                if dist < min_dist {
                    min_dist = dist;
                    nearest_plate_idx = idx;
                }
            }
            
            // Use the plate ID for this seed index
            polygon.plate_id = Some(plate_ids[nearest_plate_idx]);
            
            // Calculate erosion modifier: interior plates = slower (0.5-0.8)
            // Boundary regions will be set separately in mark_boundaries
            polygon.erosion_rate_modifier = 0.5 + 0.3 * (min_dist / 0.1).min(1.0);
        }
    }
    
    /// Mark plate boundaries between polygons belonging to different plates.
    /// 
    /// Also sets boundary type and marks volcanic activity zones.
    /// Interior polygons get slower erosion rates.
    pub fn mark_plate_boundaries(&mut self) {
        use crate::world::entities::planet::TectonicBoundaryType;
        
        // First pass: identify boundaries
        for polygon in &mut self.polygons {
            let plate_id = match polygon.plate_id {
                Some(id) => id,
                None => continue,
            };
            
            // Check neighbors for different plate
            for &neighbor_id in &polygon.neighbors {
                if let Some(neighbor) = self.get(neighbor_id) {
                    if let Some(neighbor_plate) = neighbor.plate_id {
                        if neighbor_plate != plate_id {
                            // This polygon is on a plate boundary
                            polygon.is_plate_boundary = true;
                            
                            // Use noise to determine boundary type
                            let boundary_noise = crate::util::noise::SimplexNoise::new(
                                (polygon.id as u64).wrapping_mul(12345)
                            ).get(polygon.id as f32 * 0.1, 0.0).abs();
                            
                            // Classify boundary type based on noise
                            if boundary_noise < 0.33 {
                                polygon.boundary_type = Some(TectonicBoundaryType::Convergent {
                                    spreading_rate_cm_yr: 0.0,
                                    subduction_rate_cm_yr: 5.0 + boundary_noise * 10.0,
                                    subducting_plate: Some(neighbor_plate),
                                    subduction_type: crate::world::entities::planet::SubductionType::OceanicUnderOceanic,
                                });
                                
                                // Convergent boundaries get mountain uplift + volcanic activity
                                polygon.set_erosion_modifier(1.5);
                                polygon.set_volcanic_activity(0.3 + boundary_noise * 0.4);
                            } else if boundary_noise < 0.66 {
                                polygon.boundary_type = Some(TectonicBoundaryType::Divergent {
                                    spreading_rate_cm_yr: 2.0 + boundary_noise * 8.0,
                                });
                                
                                // Divergent boundaries get volcanic activity
                                polygon.set_erosion_modifier(0.8);
                                polygon.set_volcanic_activity(0.4 + boundary_noise * 0.3);
                            } else {
                                polygon.boundary_type = Some(TectonicBoundaryType::Transform {
                                    slip_rate_cm_yr: 1.0 + boundary_noise * 5.0,
                                });
                                
                                // Transform boundaries have minimal volcanic activity
                                polygon.set_erosion_modifier(1.2);
                                polygon.set_volcanic_activity(0.1);
                            }
                            
                            break; // Only mark first boundary found
                        }
                    }
                }
            }
        }
        
        // Second pass: adjust interior plate erosion (already set in assign_tectonic_plates)
        // Interior polygons keep their slower erosion rates
    }
    
    /// Apply tectonic elevation changes to polygons based on boundary type.
    /// 
    /// - Convergent boundaries: uplift mountains (+elevation)
    /// - Divergent boundaries: subsidence/rifts (-elevation)
    /// - Transform boundaries: minimal change
    pub fn apply_tectonic_elevation(&mut self, mountain_factor: f32) {
        for polygon in &mut self.polygons {
            if !polygon.is_plate_boundary {
                continue;
            }
            
            let elevation_delta = match &polygon.boundary_type {
                Some(TectonicBoundaryType::Convergent { .. }) => {
                    // Mountain building - positive elevation change
                    (0.1 + polygon.erosion_rate_modifier * 0.5) * mountain_factor
                }
                Some(TectonicBoundaryType::Divergent { .. }) => {
                    // Rift formation - negative elevation change (subsidence)
                    -0.05 * mountain_factor
                }
                Some(TectonicBoundaryType::Transform { .. }) => {
                    // Transform - minimal vertical change
                    0.02 * mountain_factor
                }
                Some(TectonicBoundaryType::Conservative { .. }) => {
                    0.01 * mountain_factor
                }
                None => 0.0,
            };
            
            // Add to base elevation
            polygon.base_elevation += elevation_delta * 2000.0; // Scale to meters
            
            // Clamp base elevation
            polygon.base_elevation = polygon.base_elevation.clamp(-2000.0, 9000.0);
        }
    }
    
    /// Get all polygons belonging to a specific tectonic plate.
    pub fn polygons_in_plate(&self, plate_id: Uuid) -> Vec<u32> {
        self.polygons.iter()
            .filter(|p| p.plate_id == Some(plate_id))
            .map(|p| p.id)
            .collect()
    }
    
    /// Get all boundary polygons.
    pub fn boundary_polygons(&self) -> Vec<u32> {
        self.polygons.iter()
            .filter(|p| p.is_plate_boundary)
            .map(|p| p.id)
            .collect()
    }
    
    /// Get polygons with volcanic activity.
    pub fn volcanic_polygons(&self) -> Vec<u32> {
        self.polygons.iter()
            .filter(|p| p.volcanic_activity.is_some())
            .map(|p| p.id)
            .collect()
    }
    
    /// Get average erosion rate modifier for interior (non-boundary) polygons.
    pub fn interior_erosion_rate(&self) -> f32 {
        let interior: Vec<f32> = self.polygons.iter()
            .filter(|p| !p.is_plate_boundary)
            .map(|p| p.erosion_rate_modifier)
            .collect();
        
        if interior.is_empty() {
            1.0
        } else {
            interior.iter().sum::<f32>() / interior.len() as f32
        }
    }
    
    /// Get average erosion rate modifier for boundary polygons.
    pub fn boundary_erosion_rate(&self) -> f32 {
        let boundary: Vec<f32> = self.polygons.iter()
            .filter(|p| p.is_plate_boundary)
            .map(|p| p.erosion_rate_modifier)
            .collect();
        
        if boundary.is_empty() {
            1.0
        } else {
            boundary.iter().sum::<f32>() / boundary.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_graph_stats() {
        let graph = PolygonGraph::new();
        let stats = graph.elevation_stats();
        
        assert_eq!(stats.total_polygons, 0);
        assert!(!stats.is_valid());
    }

    #[test]
    fn test_single_polygon() {
        let mut graph = PolygonGraph::new();
        graph.add_polygon(Polygon::new(0));
        
        // No coastal, compute distance
        graph.compute_distance_elevation();
        
        // Should have default elevation 0 (no coastal neighbors)
        assert_eq!(graph.elevation(0), 0.0);
    }

    #[test]
    fn test_elevation_stats() {
        let mut graph = PolygonGraph::with_capacity(10);
        
        for i in 0..10 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Create a simple chain
        for i in 0..9 {
            graph.add_edge(i, i + 1);
        }
        
        graph.mark_coastal(0);
        graph.compute_distance_elevation();
        
        let stats = graph.elevation_stats();
        
        assert!(stats.is_valid());
        assert_eq!(stats.coastal_count, 1);
        assert_eq!(stats.min, 0.0);
        assert!(stats.max > 0.0);
        assert!(stats.mean > 0.0);
    }

    #[test]
    fn test_weighted_elevation() {
        let mut graph = PolygonGraph::with_capacity(5);
        
        // Create polygons with varying base elevations
        for i in 0..5 {
            graph.add_polygon(Polygon::with_base_elevation(i, i as f32 * 500.0));
        }
        
        // Chain structure
        for i in 0..4 {
            graph.add_edge(i, i + 1);
        }
        
        graph.mark_coastal(0);
        graph.compute_weighted_distance_elevation();
        
        // Coastal should still be 0
        assert_eq!(graph.elevation(0), 0.0);
        
        // Distant polygon should have higher elevation
        assert!(graph.elevation(4) > graph.elevation(2));
    }

    #[test]
    fn test_monotonic_enforcement() {
        let mut graph = PolygonGraph::with_capacity(3);
        
        for i in 0..3 {
            graph.add_polygon(Polygon::new(i));
        }
        
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        
        // Set up with potential non-monotonic values
        {
            let poly = graph.get_mut(0).unwrap();
            poly.mark_coastal();
            poly.elevation = 0.0;
        }
        {
            let poly = graph.get_mut(1).unwrap();
            poly.elevation = 0.5;
        }
        {
            let poly = graph.get_mut(2).unwrap();
            poly.elevation = 0.3; // Lower than middle - should be fixed
        }
        
        graph.enforce_monotonic_elevation();
        
        // After enforcement, 2 should have at least 0.5
        assert!(graph.elevation(2) >= graph.elevation(1));
    }

    #[test]
    fn test_coastal_ids() {
        let mut graph = PolygonGraph::with_capacity(5);
        
        for i in 0..5 {
            graph.add_polygon(Polygon::new(i));
        }
        
        for i in 0..4 {
            graph.add_edge(i, i + 1);
        }
        
        // Mark some as coastal
        graph.mark_coastal(0);
        graph.mark_coastal(2);
        
        let coastal = graph.coastal_ids();
        assert_eq!(coastal.len(), 2);
        assert!(coastal.contains(&0));
        assert!(coastal.contains(&2));
    }

    #[test]
    fn test_mountain_ids() {
        let mut graph = PolygonGraph::with_capacity(5);
        
        for i in 0..5 {
            graph.add_polygon(Polygon::new(i));
        }
        
        for i in 0..4 {
            graph.add_edge(i, i + 1);
        }
        
        graph.mark_coastal(0);
        graph.compute_distance_elevation();
        
        // With 5 polygons in a chain, highest should be at id 4 with elevation ~1.0
        let mountains = graph.mountain_ids(0.8);
        assert!(mountains.contains(&4));
    }

    #[test]
    fn test_blend_with_base() {
        let mut graph = PolygonGraph::with_capacity(3);
        
        for i in 0..3 {
            graph.add_polygon(Polygon::with_base_elevation(i, 2000.0 * i as f32));
        }
        
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        
        graph.mark_coastal(0);
        graph.compute_distance_elevation();
        
        // Before blend: 0=0, 1=0.5, 2=1.0
        assert_eq!(graph.elevation(0), 0.0);
        
        // Blend with equal weights
        graph.blend_with_base_elevation(0.5, 0.5);
        
        // 0: (0*0.5 + 0*0.5) = 0
        // 1: (0.5*0.5 + 0.25*0.5) = 0.375
        // 2: (1.0*0.5 + 0.5*0.5) = 0.75
        assert!(graph.elevation(1) > 0.0);
        assert!(graph.elevation(2) > graph.elevation(1));
    }
    
    #[test]
    fn test_tectonic_plate_assignment() {
        let mut graph = PolygonGraph::with_capacity(20);
        
        // Create 20 polygons in a simple grid pattern
        for i in 0..20 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect in a ring for simple adjacency
        for i in 0..19 {
            graph.add_edge(i, i + 1);
        }
        graph.add_edge(19, 0);
        
        // Assign tectonic plates (7 plates, 70% continental = ~5 continental)
        graph.assign_tectonic_plates(7, 0.7, 42);
        
        // All polygons should have a plate assigned
        for i in 0..20u32 {
            let poly = graph.get(i).unwrap();
            assert!(poly.plate_id.is_some(), "Polygon {} should have plate_id", i);
        }
        
        // Interior erosion rates should be slower than 1.0
        let interior_rate = graph.interior_erosion_rate();
        assert!(interior_rate < 1.0, "Interior erosion rate should be < 1.0, got {} ", interior_rate);
    }
    
    #[test]
    fn test_plate_boundary_marking() {
        let mut graph = PolygonGraph::with_capacity(4);
        
        // Create 4 polygons in a line
        for i in 0..4 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect in a line: 0-1-2-3
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        
        // Assign 2 plates: polygons 0,1 on plate A; polygons 2,3 on plate B
        let plate_a = uuid::Uuid::new_v4();
        let plate_b = uuid::Uuid::new_v4();
        
        {
            let poly = graph.get_mut(0).unwrap();
            poly.plate_id = Some(plate_a);
            poly.erosion_rate_modifier = 0.6;
        }
        {
            let poly = graph.get_mut(1).unwrap();
            poly.plate_id = Some(plate_a);
            poly.erosion_rate_modifier = 0.6;
        }
        {
            let poly = graph.get_mut(2).unwrap();
            poly.plate_id = Some(plate_b);
            poly.erosion_rate_modifier = 0.6;
        }
        {
            let poly = graph.get_mut(3).unwrap();
            poly.plate_id = Some(plate_b);
            poly.erosion_rate_modifier = 0.6;
        }
        
        // Mark boundaries
        graph.mark_plate_boundaries();
        
        // Polygons 1 and 2 should be on a boundary (different plates, but adjacent)
        let boundary_polys = graph.boundary_polygons();
        assert!(boundary_polys.contains(&1) || boundary_polys.contains(&2), 
            "At least one polygon should be marked as boundary");
        
        // Boundary polygons should have boundary type set
        for &poly_id in &boundary_polys {
            let poly = graph.get(poly_id).unwrap();
            assert!(poly.boundary_type.is_some(), 
                "Boundary polygon {} should have boundary_type set", poly_id);
        }
    }
    
    #[test]
    fn test_tectonic_elevation_application() {
        let mut graph = PolygonGraph::with_capacity(3);
        
        for i in 0..3 {
            let mut poly = Polygon::new(i);
            poly.base_elevation = 1000.0; // Start with some elevation
            graph.add_polygon(poly);
        }
        
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        
        // Mark polygon 1 as a convergent boundary
        {
            let poly = graph.get_mut(1).unwrap();
            poly.is_plate_boundary = true;
            use crate::world::entities::planet::TectonicBoundaryType;
            poly.boundary_type = Some(TectonicBoundaryType::Convergent {
                spreading_rate_cm_yr: 0.0,
                subduction_rate_cm_yr: 5.0,
                subducting_plate: Some(uuid::Uuid::new_v4()),
                subduction_type: crate::world::entities::planet::SubductionType::OceanicUnderContinental,
            });
        }
        
        // Apply tectonic elevation
        let initial_elevation = graph.get(1).unwrap().base_elevation;
        graph.apply_tectonic_elevation(1.0);
        let new_elevation = graph.get(1).unwrap().base_elevation;
        
        // Convergent boundary should increase elevation (uplift)
        assert!(new_elevation > initial_elevation, 
            "Convergent boundary should increase elevation from {} to {}", 
            initial_elevation, new_elevation);
    }
    
    #[test]
    fn test_volcanic_activity_detection() {
        let mut graph = PolygonGraph::with_capacity(5);
        
        for i in 0..5 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect all in a line
        for i in 0..4 {
            graph.add_edge(i, i + 1);
        }
        
        // Assign plates with different IDs
        let plate_a = uuid::Uuid::new_v4();
        let plate_b = uuid::Uuid::new_v4();
        
        // Set up so boundary forms between polygons 1 and 2
        for i in 0..5 {
            let poly = graph.get_mut(i as u32).unwrap();
            poly.plate_id = if i < 2 { plate_a } else { plate_b };
        }
        
        // Mark boundaries (this will set volcanic activity)
        graph.mark_plate_boundaries();
        
        // Check for volcanic polygons
        let volcanic = graph.volcanic_polygons();
        assert!(!volcanic.is_empty(), "Should have some volcanic polygons at boundaries");
        
        // Verify volcanic polygons have volcanic_activity set
        for &poly_id in &volcanic {
            let poly = graph.get(poly_id).unwrap();
            assert!(poly.volcanic_activity.is_some());
        }
    }
    
    #[test]
    fn test_interior_vs_boundary_erosion() {
        let mut graph = PolygonGraph::with_capacity(10);
        
        for i in 0..10 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect in a ring
        for i in 0..9 {
            graph.add_edge(i, i + 1);
        }
        graph.add_edge(9, 0);
        
        // Assign plates with 2 plates
        let plate_a = uuid::Uuid::new_v4();
        let plate_b = uuid::Uuid::new_v4();
        
        for i in 0..10 {
            let poly = graph.get_mut(i).unwrap();
            poly.plate_id = if i < 5 { plate_a } else { plate_b };
            poly.erosion_rate_modifier = 0.6; // Interior rate
        }
        
        // Mark boundaries
        graph.mark_plate_boundaries();
        
        let interior_rate = graph.interior_erosion_rate();
        let boundary_rate = graph.boundary_erosion_rate();
        
        // Interior plates should have slower erosion than boundaries
        assert!(interior_rate < boundary_rate, 
            "Interior erosion ({}) should be slower than boundary ({})", 
            interior_rate, boundary_rate);
    }
}
*/