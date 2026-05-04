//! River Generation Module
//! 
//! Generates rivers and water systems based on terrain elevation.
//! Uses deterministic flow simulation with erosion effects.
//! 
//! Algorithm:
//! 1. Find water sources (lakes, springs, high points)
//! 2. Trace flow paths downhill using gradient descent
//! 3. Apply erosion effects along paths
//! 4. Merge tributaries into main rivers

use crate::terrain::ElevationGrid;
use crate::util::{Rng, Vec2, Direction};
use std::collections::HashMap;

/// Represents a river with its flow path and properties
#[derive(Debug, Clone)]
pub struct River {
    /// Unique identifier for this river
    pub id: RiverId,
    /// Ordered list of coordinates forming the river path (headwaters to mouth)
    pub path: Vec<Vec2<i32>>,
    /// Length in grid cells
    pub length: u32,
    /// Average flow rate (relative units 0.0 - 1.0)
    pub flow_rate: f32,
    /// Whether river drains into ocean/lake
    pub drains_into: DrainTarget,
    /// Grid cells this river occupies (for erosion)
    pub cells: Vec<Vec2<i32>>,
}

/// Target where river drains
#[derive(Debug, Clone, PartialEq)]
pub enum DrainTarget {
    Ocean,
    Lake(Vec2<i32>),
    Sinkhole,
    Border,
}

/// River generation configuration
#[derive(Debug, Clone)]
pub struct RiverConfig {
    /// Target number of rivers (approximate)
    pub river_density: f32,        // 0.0 - 1.0
    /// Minimum river length in cells
    pub min_length: u32,
    /// Maximum river length in cells
    pub max_length: u32,
    /// Erosion intensity (affects terrain modification)
    pub erosion_intensity: f32,   // 0.0 - 1.0
    /// Minimum elevation to spawn rivers
    pub source_elevation: f32,
    /// Flow accumulation threshold to start river
    pub accumulation_threshold: f32,
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            river_density: 0.3,
            min_length: 10,
            max_length: 500,
            erosion_intensity: 0.5,
            source_elevation: 0.6,
            accumulation_threshold: 0.1,
        }
    }
}

/// River generation state
#[derive(Debug)]
pub struct RiverGenerator {
    config: RiverConfig,
    flow_accumulation: HashMap<Vec2<i32>, f32>,
    rivers: Vec<River>,
    next_river_id: u32,
}

impl RiverGenerator {
    /// Create a new river generator with given configuration
    pub fn new(config: RiverConfig) -> Self {
        Self {
            config,
            flow_accumulation: HashMap::new(),
            rivers: Vec::new(),
            next_river_id: 0,
        }
    }

    /// Calculate flow accumulation map from elevation
    /// Higher values indicate areas where water would naturally collect
    pub fn calculate_flow_accumulation(&mut self, elevation: &ElevationGrid, _rng: &mut Rng) {
        let width = elevation.width as i32;
        let height = elevation.height as i32;
        
        self.flow_accumulation.clear();
        
        // For each cell, calculate how much water flows into it
        for y in 0..height {
            for x in 0..width {
                let pos = Vec2::new(x, y);
                let elevation_value = elevation.get_value_unchecked(x, y);
                
                // Trace all water sources that flow into this cell
                let mut accumulated = 0.0;
                
                // Self contribution
                accumulated += 1.0;
                
                // Contributions from higher neighbors
                for dir in Direction::cardinal() {
                    let neighbor = pos + dir.delta();
                    if elevation.is_valid(neighbor.x, neighbor.y) {
                        let neighbor_elev = elevation.get_value_unchecked(neighbor.x, neighbor.y);
                        // If neighbor is higher, water flows from neighbor to us
                        if neighbor_elev > elevation_value {
                            accumulated += self.flow_accumulation.get(&neighbor).copied().unwrap_or(0.1);
                        }
                    }
                }
                
                self.flow_accumulation.insert(pos, accumulated);
            }
        }
        
        // Normalize flow accumulation
        let max_flow = self.flow_accumulation.values().fold(0.0f32, |a, b| a.max(*b));
        if max_flow > 0.0 {
            for value in self.flow_accumulation.values_mut() {
                *value /= max_flow;
            }
        }
    }

    /// Generate rivers based on terrain and flow accumulation
    pub fn generate_rivers(
        &mut self, 
        elevation: &ElevationGrid, 
        sea_level: f32,
        rng: &mut Rng
    ) -> Vec<River> {
        self.rivers.clear();
        
        // First calculate flow accumulation
        self.calculate_flow_accumulation(elevation, rng);
        
        // Find potential source points (high elevation with high accumulation)
        let sources = self.find_source_points(elevation, rng);
        
        // Generate rivers from sources
        for source in sources {
            if let Some(river) = self.trace_river(elevation, sea_level, source, rng) {
                // Check if this is a duplicate or too short
                if river.length >= self.config.min_length {
                    // Check for overlap with existing rivers
                    let overlaps = self.rivers.iter().any(|r| {
                        river.path.iter().any(|p| r.cells.contains(p))
                    });
                    
                    if !overlaps {
                        self.rivers.push(river);
                    }
                }
            }
        }
        
        self.rivers.clone()
    }

    /// Find potential source points for rivers
    fn find_source_points(&mut self, elevation: &ElevationGrid, rng: &mut Rng) -> Vec<Vec2<i32>> {
        let width = elevation.width as i32;
        let height = elevation.height as i32;
        let _sources: Vec<Vec2<i32>> = Vec::new();
        
        // Target number based on density setting
        let target_count = ((width * height) as f32 * self.config.river_density * 0.01) as usize;
        
        // Collect potential sources (high elevation cells with good flow)
        let mut candidates: Vec<(Vec2<i32>, f32)> = Vec::new();
        
        for y in 0..height {
            for x in 0..width {
                let pos = Vec2::new(x, y);
                let elev = elevation.get_value_unchecked(x, y);
                let flow = self.flow_accumulation.get(&pos).copied().unwrap_or(0.0);
                
                // Source criteria: high elevation and reasonable flow
                let score = elev * self.config.source_elevation + flow * 0.5;
                if elev >= self.config.source_elevation && flow >= self.config.accumulation_threshold {
                    candidates.push((pos, score));
                }
            }
        }
        
        // Sort by score and take top candidates
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Randomly select from top candidates to add variety
        let selection_range = (target_count * 2).min(candidates.len());
        let mut selected: Vec<Vec2<i32>> = Vec::new();
        
        for i in 0..selection_range {
            if selected.len() >= target_count {
                break;
            }
            // Stochastic selection for variety
            if rng.random_float() < 0.7 || selected.len() < target_count / 2 {
                selected.push(candidates[i].0);
            }
        }
        
        selected
    }

    /// Trace a river from source to terminus
    fn trace_river(
        &mut self,
        elevation: &ElevationGrid,
        sea_level: f32,
        mut current: Vec2<i32>,
        rng: &mut Rng
    ) -> Option<River> {
        let mut path = vec![current];
        let _cells = vec![current];
        let mut visited = std::collections::HashSet::new();
        visited.insert(current);
        
        let start_elevation = elevation.get_value_unchecked(current.x, current.y);
        let mut min_elevation = start_elevation;
        let mut flow_rate = 0.5 + rng.random_float() * 0.5;
        
        // Trace downhill until we reach sea level, edge, or loop
        while path.len() < self.config.max_length as usize {
            // Find the steepest downhill neighbor
            let mut best_next: Option<(Vec2<i32>, f32)> = None;
            let current_elev = elevation.get_value_unchecked(current.x, current.y);
            
            for dir in Direction::cardinal() {
                let neighbor = current + dir.delta();
                
                if visited.contains(&neighbor) {
                    continue;
                }
                
                if !elevation.is_valid(neighbor.x, neighbor.y) {
                    // Edge of map - river drains here
                    path.push(neighbor);
                    return Some(self.create_river(path, flow_rate, DrainTarget::Border));
                }
                
                let neighbor_elev = elevation.get_value_unchecked(neighbor.x, neighbor.y);
                let slope = current_elev - neighbor_elev;
                
                if slope > 0.0 {
                    let is_better = match best_next {
                        None => true,
                        Some((_, best_slope)) => slope > best_slope,
                    };
                    if is_better {
                        best_next = Some((neighbor, slope));
                    }
                }
            }
            
            match best_next {
                Some((next, slope)) => {
                    path.push(next);
                    visited.insert(next);
                    min_elevation = min_elevation.min(elevation.get_value_unchecked(next.x, next.y));
                    
                    // Flow increases as we go downhill (adds tributaries implicitly)
                    flow_rate = (flow_rate + slope * 0.1).min(1.0);
                    
                    current = next;
                    
                    let next_elev = elevation.get_value_unchecked(next.x, next.y);
                    
                    // Check if we've reached sea level
                    if next_elev <= sea_level {
                        // River drains into ocean
                        path.push(next);
                        return Some(self.create_river(path, flow_rate, DrainTarget::Ocean));
                    }
                    
                    // Check if we've hit a local minimum (potential lake)
                    if slope < 0.01 {
                        // Small chance to form a lake
                        if rng.random_float() < 0.2 {
                            path.push(next);
                            return Some(self.create_river(path, flow_rate, DrainTarget::Lake(next)));
                        }
                    }
                }
                None => {
                    // Stuck in a depression - may be a sinkhole or end in lake
                    let current_elev = elevation.get_value_unchecked(current.x, current.y);
                    if current_elev <= sea_level + 0.05 {
                        return Some(self.create_river(path, flow_rate, DrainTarget::Sinkhole));
                    } else {
                        return Some(self.create_river(path, flow_rate, DrainTarget::Sinkhole));
                    }
                }
            }
        }
        
        // Exceeded max length
        if path.len() >= self.config.max_length as usize {
            return Some(self.create_river(path, flow_rate, DrainTarget::Sinkhole));
        }
        
        None
    }

    /// Create a River struct from path data
    fn create_river(&mut self, path: Vec<Vec2<i32>>, flow_rate: f32, drains_into: DrainTarget) -> River {
        let id = RiverId(self.next_river_id);
        self.next_river_id += 1;
        
        // Collect cells from path (with some width)
        let cells: Vec<Vec2<i32>> = path.iter().flat_map(|p| {
            // River occupies 1-3 cells wide
            let result = vec![*p];
            result
        }).collect();
        
        River {
            id,
            path: path.clone(),
            length: path.len() as u32,
            flow_rate,
            drains_into,
            cells,
        }
    }

    /// Apply erosion effects to elevation grid based on rivers
    pub fn apply_erosion(&self, elevation: &mut ElevationGrid) {
        let erosion_factor = self.config.erosion_intensity;
        
        for river in &self.rivers {
            // Apply erosion along river path
            for (i, pos) in river.path.iter().enumerate() {
                if elevation.is_valid(pos.x, pos.y) {
                    let current = elevation.get_value_unchecked(pos.x, pos.y);
                    
                    // Deeper erosion near the middle of the river
                    let depth_factor = if river.length > 10 {
                        let mid = river.length / 2;
                        let dist_from_mid = (i as u32).abs_diff(mid);
                        1.0 - (dist_from_mid as f32 / river.length as f32)
                    } else {
                        0.5
                    };
                    
                    let erosion = 0.02 * erosion_factor * depth_factor * river.flow_rate;
                    elevation.set_value_unchecked(pos.x, pos.y, current - erosion);
                }
            }
        }
    }

    /// Get all generated rivers
    pub fn get_rivers(&self) -> &[River] {
        &self.rivers
    }

    /// Get river by ID
    pub fn get_river(&self, id: RiverId) -> Option<&River> {
        self.rivers.iter().find(|r| r.id == id)
    }

    /// Calculate total river length across all rivers
    pub fn total_river_length(&self) -> u32 {
        self.rivers.iter().map(|r| r.length).sum()
    }

    /// Get rivers that drain into a specific target
    pub fn get_rivers_draining_into(&self, target: &DrainTarget) -> Vec<&River> {
        self.rivers.iter()
            .filter(|r| &r.drains_into == target)
            .collect()
    }
}

/// Unique identifier for a river
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RiverId(pub u32);

impl RiverId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Seed;

    fn create_test_elevation() -> ElevationGrid {
        let mut grid = ElevationGrid::new(50, 50, 0.0);
        
        // Create a simple slope: high in NW corner, low in SE corner
        for y in 0..50 {
            for x in 0..50 {
                let elev = 1.0 - ((x + y) as f32 / 100.0);
                grid.set_value_unchecked(x, y, elev);
            }
        }
        
        grid
    }

    #[test]
    fn test_river_generation() {
        let elevation = create_test_elevation();
        let mut rng = Rng::new(42);
        let config = RiverConfig::default();
        
        let mut generator = RiverGenerator::new(config);
        let rivers = generator.generate_rivers(&elevation, 0.3, &mut rng);
        
        // River generation should complete without panicking
        // Result may be empty depending on terrain configuration
        for river in &rivers {
            // Each river should have a valid path
            assert!(river.path.len() >= 2, "River path should have at least 2 points");
        }
    }

    #[test]
    fn test_flow_accumulation() {
        let elevation = create_test_elevation();
        let mut rng = Rng::new(42);
        let config = RiverConfig::default();
        
        let mut generator = RiverGenerator::new(config);
        generator.calculate_flow_accumulation(&elevation, &mut rng);
        
        // SE corner should have highest accumulation (water flows there)
        let se_corner = Vec2::new(49, 49);
        let se_flow = generator.flow_accumulation.get(&se_corner).unwrap();
        
        // NW corner should have lower accumulation
        let nw_corner = Vec2::new(0, 0);
        let nw_flow = generator.flow_accumulation.get(&nw_corner).unwrap();
        
        assert!(
            se_flow > nw_flow,
            "SE corner should have higher flow accumulation"
        );
    }

    #[test]
    fn test_deterministic_rivers() {
        let elevation = create_test_elevation();
        let mut rng1 = Rng::new(12345);
        let mut rng2 = Rng::new(12345);
        let config = RiverConfig::default();
        
        let mut gen1 = RiverGenerator::new(config.clone());
        let rivers1 = gen1.generate_rivers(&elevation, 0.3, &mut rng1);
        
        let mut gen2 = RiverGenerator::new(config);
        let rivers2 = gen2.generate_rivers(&elevation, 0.3, &mut rng2);
        
        // Same seed should produce identical rivers
        assert_eq!(rivers1.len(), rivers2.len(), "Same seed should produce same number of rivers");
        
        for (r1, r2) in rivers1.iter().zip(rivers2.iter()) {
            assert_eq!(r1.path.len(), r2.path.len(), "River paths should be same length");
        }
    }

    #[test]
    fn test_river_id() {
        let id1 = RiverId::new(0);
        let id2 = RiverId::new(1);
        
        assert_ne!(id1, id2);
        assert_eq!(id1, RiverId::new(0));
    }
}
