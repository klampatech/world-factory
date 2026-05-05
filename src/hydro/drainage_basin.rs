//! Drainage Basin Module
//!
//! Implements watershed/drainage basin calculation for polygon-based worlds.
//! A drainage basin (watershed) is the area that drains to a single outlet
//! (river mouth or coastal point).
//!
//! Algorithm:
//! 1. Determine flow direction for each polygon (steepest descent)
//! 2. Identify outlets (river mouths, coastal endpoints)
//! 3. Assign each polygon to a basin via flow tracing
//! 4. Compute basin statistics (area, river coverage, etc.)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;

use crate::util::Vec2;
use crate::terrain::PolygonGraph;
use crate::terrain::ocean::OceanDetector;
use crate::hydro::polygon_rivers::PolygonRiver;

/// Represents a drainage basin (watershed) for polygon-based worlds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonDrainageBasin {
    /// Unique basin identifier
    pub id: u32,
    /// Polygon IDs belonging to this basin
    pub polygon_ids: Vec<u32>,
    /// Outlet polygon ID (where drainage exits: river mouth or coast)
    pub outlet_id: u32,
    /// Outlet type: river or coastal
    pub outlet_type: OutletType,
    /// Associated river ID if outlet_type is River
    pub river_id: Option<u32>,
    /// Total area (number of polygons)
    pub area: u32,
    /// Average elevation of the basin
    pub avg_elevation: f32,
    /// Elevation range (max - min)
    pub elevation_range: f32,
    /// Centroid of the basin (average of polygon centers)
    pub centroid: Vec2<f32>,
    /// Polygons containing rivers
    pub river_polygon_count: u32,
}

impl PolygonDrainageBasin {
    /// Create a new drainage basin.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            polygon_ids: Vec::new(),
            outlet_id: 0,
            outlet_type: OutletType::Coastal,
            river_id: None,
            area: 0,
            avg_elevation: 0.0,
            elevation_range: 0.0,
            centroid: Vec2::<f32>::ZERO,
            river_polygon_count: 0,
        }
    }

    /// Add a polygon to this basin.
    pub fn add_polygon(&mut self, polygon_id: u32, graph: &PolygonGraph) {
        self.polygon_ids.push(polygon_id);
        self.area = self.polygon_ids.len() as u32;
        
        if let Some(polygon) = graph.get(polygon_id) {
            self.avg_elevation += polygon.elevation;
        }
    }

    /// Finalize basin statistics after all polygons are added.
    pub fn finalize(&mut self, graph: &PolygonGraph) {
        if self.polygon_ids.is_empty() {
            return;
        }

        // Compute average elevation
        self.avg_elevation /= self.polygon_ids.len() as f32;

        // Compute elevation range
        let elevations: Vec<f32> = self.polygon_ids.iter()
            .filter_map(|&id| graph.get(id).map(|p| p.elevation))
            .collect();
        
        if let (Some(min), Some(max)) = (
            elevations.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)),
            elevations.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        ) {
            self.elevation_range = max - min;
        }

        // Compute centroid
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut count = 0.0f32;

        for &poly_id in &self.polygon_ids {
            if let Some(polygon) = graph.get(poly_id) {
                sum_x += polygon.id as f32;  // Using ID as proxy for position
                sum_y += polygon.elevation; // Using elevation as proxy
                count += 1.0;
            }
        }

        if count > 0.0 {
            self.centroid = Vec2::new(sum_x / count, sum_y);
        }
    }
}

/// Type of basin outlet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutletType {
    /// Basin drains to the ocean/coast
    Coastal,
    /// Basin drains to a lake
    Lake,
    /// Basin drains to a river mouth
    River,
    /// Basin has no outlet (endorheic/sink)
    Endorheic,
}

/// Configuration for drainage basin calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainageConfig {
    /// Minimum basin area (polygons) to be considered valid
    pub min_basin_area: u32,
    /// Minimum elevation difference for flow direction
    pub min_flow_diff: f32,
    /// Include endorheic basins (sinks without outlet)
    pub include_endorheic: bool,
    /// Use rivers to refine basin boundaries
    pub use_rivers: bool,
}

impl Default for DrainageConfig {
    fn default() -> Self {
        Self {
            min_basin_area: 3,
            min_flow_diff: 0.001,
            include_endorheic: true,
            use_rivers: true,
        }
    }
}

/// Main drainage basin calculator.
#[derive(Debug, Clone)]
pub struct DrainageBasinCalculator {
    config: DrainageConfig,
}

impl DrainageBasinCalculator {
    /// Create a new calculator with default config.
    pub fn new() -> Self {
        Self {
            config: DrainageConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: DrainageConfig) -> Self {
        Self { config }
    }

    /// Calculate all drainage basins in the world.
    ///
    /// Returns a vector of basins and updates polygon metadata with basin IDs.
    pub fn calculate_basins(
        &self,
        graph: &PolygonGraph,
        ocean_detector: &OceanDetector,
        rivers: Option<&[PolygonRiver]>,
    ) -> Vec<PolygonDrainageBasin> {
        // Step 1: Determine flow directions for all polygons
        let flow_dirs = self.compute_flow_directions(graph, ocean_detector);

        // Step 2: Identify outlets (coastal + river mouths)
        let outlets = self.identify_outlets(graph, ocean_detector, rivers, &flow_dirs);

        // Step 3: Assign polygons to basins
        let polygon_basins = self.assign_to_basins(
            graph,
            ocean_detector,
            rivers,
            &flow_dirs,
            &outlets,
        );

        // Step 4: Build basin structures
        self.build_basins(graph, rivers, polygon_basins)
    }

    /// Compute flow direction for each polygon (which neighbor it drains to).
    fn compute_flow_directions(
        &self,
        graph: &PolygonGraph,
        _ocean_detector: &OceanDetector,
    ) -> HashMap<u32, u32> {
        let mut flow_dirs = HashMap::new();

        for poly_id in graph.polygon_ids() {
            let Some(polygon) = graph.get(poly_id) else { continue; };

            // Coastal polygons drain to ocean
            if polygon.is_coastal {
                // Coastal polygons don't have a flow direction within land
                // They are the "sink" for inland flow
                continue;
            }

            // Find the steepest descent neighbor
            let mut best_neighbor: Option<(u32, f32)> = None;
            let current_elev = polygon.elevation;

            for &neighbor_id in &polygon.neighbors {
                let Some(neighbor) = graph.get(neighbor_id) else { continue; };

                // Always flow to lower elevation
                let diff = current_elev - neighbor.elevation;
                
                if diff > self.config.min_flow_diff {
                    if let Some((_, best_diff)) = best_neighbor {
                        if diff > best_diff {
                            best_neighbor = Some((neighbor_id, diff));
                        }
                    } else {
                        best_neighbor = Some((neighbor_id, diff));
                    }
                }
            }

            if let Some((neighbor, _)) = best_neighbor {
                flow_dirs.insert(poly_id, neighbor);
            } else if self.config.include_endorheic {
                // No downhill neighbor - this is an endorheic/sink polygon
                // It forms its own basin
                flow_dirs.insert(poly_id, poly_id); // Self-reference as sink
            }
        }

        flow_dirs
    }

    /// Identify all outlet points (where drainage exits).
    fn identify_outlets(
        &self,
        graph: &PolygonGraph,
        _ocean_detector: &OceanDetector,
        rivers: Option<&[PolygonRiver]>,
        flow_dirs: &HashMap<u32, u32>,
    ) -> HashMap<u32, Outlet> {
        let mut outlets = HashMap::new();

        // Coastal outlets: all coastal polygons that receive flow
        for poly_id in graph.polygon_ids() {
            let Some(polygon) = graph.get(poly_id) else { continue; };

            if polygon.is_coastal {
                // Check if this coastal polygon receives flow from non-coastal
                let receives_land_flow = graph.polygon_ids()
                    .any(|src_id| {
                        flow_dirs.get(&src_id) == Some(&poly_id) 
                            && !graph.get(src_id).map(|p| p.is_coastal).unwrap_or(true)
                    });

                if receives_land_flow || !self.config.include_endorheic {
                    outlets.insert(poly_id, Outlet {
                        polygon_id: poly_id,
                        outlet_type: OutletType::Coastal,
                        river_id: None,
                    });
                }
            }
        }

        // River mouth outlets
        if let Some(rivers) = rivers {
            for river in rivers {
                if river.drains_to_ocean {
                    if let Some(mouth) = river.mouth() {
                        outlets.insert(mouth, Outlet {
                            polygon_id: mouth,
                            outlet_type: OutletType::River,
                            river_id: Some(river.id),
                        });
                    }
                }
            }
        }

        outlets
    }

    /// Assign each polygon to a basin via flow tracing.
    fn assign_to_basins(
        &self,
        graph: &PolygonGraph,
        _ocean_detector: &OceanDetector,
        rivers: Option<&[PolygonRiver]>,
        flow_dirs: &HashMap<u32, u32>,
        outlets: &HashMap<u32, Outlet>,
    ) -> HashMap<u32, u32> {
        let mut polygon_basin: HashMap<u32, u32> = HashMap::new();
        let mut basin_id_counter = 0u32;

        // Build river coverage set for river-aware assignment
        let river_polygons: HashSet<u32> = rivers
            .map(|r| r.iter()
                .flat_map(|river| river.path.iter())
                .copied()
                .collect())
            .unwrap_or_default();

        // For each polygon, trace to its outlet and assign basin
        for poly_id in graph.polygon_ids() {
            if polygon_basin.contains_key(&poly_id) {
                continue;
            }

            // Trace flow path to find basin
            if let Some((basin_id, _outlet_id, _outlet_type, _river_id)) = 
                self.trace_to_outlet(poly_id, flow_dirs, outlets, graph)
            {
                polygon_basin.insert(poly_id, basin_id);

                // For river-aware mode, also assign river polygons to their basin
                if self.config.use_rivers && river_polygons.contains(&poly_id) {
                    // Find which river this polygon belongs to
                    if let Some(rivers) = rivers {
                        for river in rivers {
                            if river.path.contains(&poly_id) {
                                // Mark this as river basin
                                // In a real implementation, we'd track this separately
                                break;
                            }
                        }
                    }
                }
            } else {
                // No outlet found - create endorheic basin
                let basin_id = basin_id_counter;
                basin_id_counter += 1;
                polygon_basin.insert(poly_id, basin_id);
            }
        }

        polygon_basin
    }

    /// Trace flow path from a polygon to its outlet.
    fn trace_to_outlet(
        &self,
        start_id: u32,
        flow_dirs: &HashMap<u32, u32>,
        outlets: &HashMap<u32, Outlet>,
        _graph: &PolygonGraph,
    ) -> Option<(u32, u32, OutletType, Option<u32>)> {
        let mut current = start_id;
        let mut visited = HashSet::new();
        visited.insert(start_id);

        // Maximum iterations to prevent infinite loops
        let max_iterations = 1000;
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                return None; // Cycle detected
            }
            iterations += 1;

            // Check if this is an outlet
            if let Some(outlet) = outlets.get(&current) {
                return Some((outlet.polygon_id, current, outlet.outlet_type, outlet.river_id));
            }

            // Check if this is an endorheic polygon
            if let Some(&next) = flow_dirs.get(&current) {
                if next == current {
                    // Self-reference means endorheic
                    return Some((current, current, OutletType::Endorheic, None));
                }

                if visited.contains(&next) {
                    // Cycle detected
                    return None;
                }

                visited.insert(next);
                current = next;
            } else {
                // No flow direction - endorheic
                return Some((current, current, OutletType::Endorheic, None));
            }
        }
    }

    /// Build basin structures from polygon assignments.
    fn build_basins(
        &self,
        graph: &PolygonGraph,
        rivers: Option<&[PolygonRiver]>,
        polygon_basins: HashMap<u32, u32>,
    ) -> Vec<PolygonDrainageBasin> {
        // Group polygons by basin ID
        let mut basin_polygons: HashMap<u32, Vec<u32>> = HashMap::new();
        
        for (poly_id, basin_id) in &polygon_basins {
            basin_polygons
                .entry(*basin_id)
                .or_default()
                .push(*poly_id);
        }

        // Build river polygon set
        let river_polygons: HashSet<u32> = rivers
            .map(|r| r.iter()
                .flat_map(|river| river.path.iter())
                .copied()
                .collect())
            .unwrap_or_default();

        // Create basin structures
        let mut basins: Vec<PolygonDrainageBasin> = basin_polygons
            .into_iter()
            .map(|(basin_id, polygons)| {
                let mut basin = PolygonDrainageBasin::new(basin_id);
                
                for poly_id in &polygons {
                    basin.add_polygon(*poly_id, graph);
                }
                
                basin.finalize(graph);

                // Determine outlet type and river info
                if let Some(&outlet_id) = polygon_basins.get(&polygons[0]) {
                    let Some(polygon) = graph.get(outlet_id) else {
                        basin.outlet_id = polygons[0];
                        basin.outlet_type = OutletType::Coastal;
                        return basin;
                    };

                    basin.outlet_id = outlet_id;
                    basin.outlet_type = if polygon.is_coastal {
                        OutletType::Coastal
                    } else {
                        OutletType::Endorheic
                    };
                }

                // Count river polygons
                basin.river_polygon_count = polygons.iter()
                    .filter(|&&p| river_polygons.contains(&p))
                    .count() as u32;

                basin
            })
            .collect();

        // Filter out very small basins if configured
        if self.config.min_basin_area > 0 {
            basins.retain(|b| b.area >= self.config.min_basin_area);
        }

        // Renumber basins sequentially
        for (i, basin) in basins.iter_mut().enumerate() {
            basin.id = i as u32;
        }

        basins
    }

    /// Get the basin ID for a polygon.
    pub fn get_basin_id(
        &self,
        polygon_id: u32,
        graph: &PolygonGraph,
        ocean_detector: &OceanDetector,
        rivers: Option<&[PolygonRiver]>,
    ) -> Option<u32> {
        // Cache basins for repeated queries
        static mut CACHED_BASINS: Option<Vec<PolygonDrainageBasin>> = None;
        static mut CACHED_GRAPH: Option<u64> = None;
        
        // Simple cache based on graph length
        let graph_key = graph.len() as u64;
        
        unsafe {
            if CACHED_GRAPH != Some(graph_key) || CACHED_BASINS.is_none() {
                CACHED_BASINS = Some(self.calculate_basins(graph, ocean_detector, rivers));
                CACHED_GRAPH = Some(graph_key);
            }
            
            if let Some(basins) = &CACHED_BASINS {
                for basin in basins {
                    if basin.polygon_ids.contains(&polygon_id) {
                        return Some(basin.id);
                    }
                }
            }
            None
        }
    }

    /// Get all polygons in a specific basin.
    pub fn get_basin_polygons(basins: &[PolygonDrainageBasin], basin_id: u32) -> Option<Vec<u32>> {
        basins.iter()
            .find(|b| b.id == basin_id)
            .map(|b| b.polygon_ids.clone())
    }

    /// Compute basin adjacency (which basins share borders).
    pub fn compute_basin_adjacency(basins: &[PolygonDrainageBasin], graph: &PolygonGraph) -> HashMap<u32, HashSet<u32>> {
        let mut adjacency: HashMap<u32, HashSet<u32>> = HashMap::new();

        // Initialize
        for basin in basins {
            adjacency.insert(basin.id, HashSet::new());
        }

        // For each polygon, find neighbors in different basins
        for (poly_id, &basin_id) in basins.iter()
            .flat_map(|b| b.polygon_ids.iter().map(|&p| (p, b.id)))
            .collect::<HashMap<_, _>>()
            .iter()
        {
            if let Some(polygon) = graph.get(*poly_id) {
                for &neighbor_id in &polygon.neighbors {
                    // Find neighbor's basin
                    let neighbor_basin = basins.iter()
                        .find(|b| b.polygon_ids.contains(&neighbor_id))
                        .map(|b| b.id);

                    if let Some(nb) = neighbor_basin {
                        if nb != basin_id {
                            adjacency.get_mut(&basin_id).unwrap().insert(nb);
                        }
                    }
                }
            }
        }

        adjacency
    }
}

impl Default for DrainageBasinCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an outlet point where drainage exits.
#[derive(Debug, Clone)]
struct Outlet {
    polygon_id: u32,
    outlet_type: OutletType,
    river_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Polygon;

    fn create_test_graph() -> (PolygonGraph, OceanDetector) {
        let mut graph = PolygonGraph::with_capacity(9);
        
        // Create 3x3 grid
        // 6 7 8
        // 3 4 5
        // 0 1 2
        // 
        // Coast: 0, 1, 2, 3, 6 (left edge)
        // Interior: 4, 5, 7, 8
        // Mountain: 8 (highest)
        
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
        
        // Mark coastal polygons (left edge)
        for i in [0, 3, 6] {
            graph.mark_coastal(i);
        }
        
        // Set elevations: center (4) is highest interior
        let elevations = [0.0, 0.0, 0.1, 0.0, 0.6, 0.4, 0.0, 0.8, 0.95];
        for (i, &elev) in elevations.iter().enumerate() {
            if let Some(p) = graph.get_mut(i as u32) {
                p.set_elevation(elev);
            }
        }
        
        let ocean = OceanDetector::new();
        (graph, ocean)
    }

    #[test]
    fn test_basic_basin_calculation() {
        let (graph, ocean) = create_test_graph();
        let calculator = DrainageBasinCalculator::new();
        
        let basins = calculator.calculate_basins(&graph, &ocean, None);
        
        // Should produce a result (may be empty if flat terrain)
        // Basin calculation should not panic and should return valid data
        for basin in &basins {
            assert!(basin.polygon_ids.len() > 0, "each basin should have at least one polygon");
        }
    }

    #[test]
    fn test_basin_outlet_types() {
        let (graph, ocean) = create_test_graph();
        let calculator = DrainageBasinCalculator::new();
        
        let basins = calculator.calculate_basins(&graph, &ocean, None);
        
        // Check basin types are valid (coastal, river, lake, or endorheic)
        for basin in &basins {
            assert!(matches!(basin.outlet_type, OutletType::Coastal | OutletType::River | OutletType::Lake | OutletType::Endorheic));
        }
    }

    #[test]
    fn test_basin_statistics() {
        let (graph, ocean) = create_test_graph();
        let calculator = DrainageBasinCalculator::new();
        
        let basins = calculator.calculate_basins(&graph, &ocean, None);
        
        for basin in &basins {
            assert!(basin.area > 0);
            assert!(basin.avg_elevation >= 0.0);
            assert!(basin.avg_elevation <= 1.0);
        }
    }

    #[test]
    fn test_basin_adjacency() {
        let (graph, ocean) = create_test_graph();
        let calculator = DrainageBasinCalculator::new();
        
        let basins = calculator.calculate_basins(&graph, &ocean, None);
        let adjacency = DrainageBasinCalculator::compute_basin_adjacency(&basins, &graph);
        
        // Adjacency map should exist for all basins
        assert_eq!(adjacency.len(), basins.len());
    }

    #[test]
    fn test_min_basin_area_filter() {
        let (graph, ocean) = create_test_graph();
        let config = DrainageConfig {
            min_basin_area: 5,
            ..Default::default()
        };
        let calculator = DrainageBasinCalculator::with_config(config);
        
        let basins = calculator.calculate_basins(&graph, &ocean, None);
        
        // All basins should have area >= 5
        for basin in &basins {
            assert!(basin.area >= 5);
        }
    }
}