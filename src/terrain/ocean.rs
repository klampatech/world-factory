//! Ocean Detection Module
//! 
//! Deterministic algorithms for detecting and classifying ocean regions.
//! All detection is based on elevation data and Voronoi polygon adjacency.
//! 
//! Key features:
//! - Ocean cell identification based on elevation threshold
//! - Coastal cell detection (ocean adjacent to land)
//! - Depth zone classification for ocean rendering/gameplay
//! - Coastal metrics calculation (shoreline length, bay/peninsula detection)

use super::{PolygonGraph, Polygon, TerrainGrid};

/// Configuration for ocean detection algorithms.
#[derive(Debug, Clone)]
pub struct OceanDetectionConfig {
    /// Elevation threshold below which a cell is considered ocean.
    /// Default: 0.0 (cells at sea level or below)
    pub ocean_elevation_threshold: f32,
    /// Elevation threshold for shallow ocean (coastal waters).
    /// Default: 0.1 (10% of max elevation)
    pub shallow_ocean_threshold: f32,
    /// Elevation threshold for deep ocean.
    /// Default: 0.5 (50% of max elevation)
    pub deep_ocean_threshold: f32,
    /// Minimum number of ocean neighbors to be considered a valid ocean region.
    /// Prevents single-cell noise from being classified.
    pub min_ocean_neighbors: u32,
    /// Enable bay detection (concave coastline sections).
    pub enable_bay_detection: bool,
    /// Enable peninsula detection (convex land projections into ocean).
    pub enable_peninsula_detection: bool,
}

impl Default for OceanDetectionConfig {
    fn default() -> Self {
        Self {
            ocean_elevation_threshold: 0.0,
            shallow_ocean_threshold: 0.1,
            deep_ocean_threshold: 0.5,
            min_ocean_neighbors: 1,
            enable_bay_detection: true,
            enable_peninsula_detection: true,
        }
    }
}

/// Ocean depth zone classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OceanZone {
    /// Cell is above water (land).
    Land,
    /// Shallow ocean - coastal waters, typically < 200m depth.
    ShallowOcean,
    /// Medium depth ocean - continental shelf waters.
    MediumOcean,
    /// Deep ocean - oceanic waters.
    DeepOcean,
}

impl OceanZone {
    /// Get display name for this zone.
    pub fn name(&self) -> &'static str {
        match self {
            OceanZone::Land => "Land",
            OceanZone::ShallowOcean => "Shallow Ocean",
            OceanZone::MediumOcean => "Medium Ocean",
            OceanZone::DeepOcean => "Deep Ocean",
        }
    }
    
    /// Get a numeric depth level for rendering.
    /// Returns 0 for land, 1-3 for ocean depths (higher = deeper).
    pub fn depth_level(&self) -> u8 {
        match self {
            OceanZone::Land => 0,
            OceanZone::ShallowOcean => 1,
            OceanZone::MediumOcean => 2,
            OceanZone::DeepOcean => 3,
        }
    }
}

/// Coastal metrics for a specific polygon.
#[derive(Debug, Clone)]
pub struct CoastalMetrics {
    /// Polygon ID these metrics apply to.
    pub polygon_id: u32,
    /// Whether this polygon is coastal (adjacent to ocean).
    pub is_coastal: bool,
    /// Number of ocean neighbors.
    pub ocean_neighbor_count: u32,
    /// Number of land neighbors.
    pub land_neighbor_count: u32,
    /// Estimated coastline length in arbitrary units.
    pub coastline_length: f32,
    /// Whether this coastal section is part of a bay.
    pub is_bay: bool,
    /// Whether this coastal section is part of a peninsula.
    pub is_peninsula: bool,
    /// Whether this is a headland (prominent coastal point).
    pub is_headland: bool,
    /// Coastal curvature: negative = concave (bay), positive = convex (headland).
    pub curvature: f32,
}

impl CoastalMetrics {
    /// Check if this polygon is on a bay.
    pub fn is_bay_coast(&self) -> bool {
        self.is_bay
    }
    
    /// Check if this polygon is on a peninsula.
    pub fn is_peninsula_coast(&self) -> bool {
        self.is_peninsula
    }
}

/// Ocean detector for analyzing elevation grids.
#[derive(Debug, Clone)]
pub struct OceanDetector {
    config: OceanDetectionConfig,
}

impl OceanDetector {
    /// Create a new ocean detector with default configuration.
    pub fn new() -> Self {
        Self::with_config(OceanDetectionConfig::default())
    }
    
    /// Detect ocean zones for a TerrainGrid.
    /// Returns a Vec of (x, y, OceanZone) tuples for all cells.
    pub fn detect_ocean(&self, grid: &TerrainGrid) -> Vec<(u32, u32, OceanZone)> {
        let mut zones = Vec::new();
        let (width, height) = grid.dimensions();
        
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = grid.get(x, y) {
                    let elevation = cell.height() / 1023.0; // Normalize back to 0-1 range
                    let threshold = self.config.ocean_elevation_threshold;
                    let zone = if elevation > threshold {
                        OceanZone::Land
                    } else if elevation <= self.config.deep_ocean_threshold {
                        OceanZone::DeepOcean
                    } else if elevation <= self.config.shallow_ocean_threshold {
                        OceanZone::MediumOcean
                    } else {
                        OceanZone::ShallowOcean
                    };
                    zones.push((x, y, zone));
                }
            }
        }
        
        zones
    }
    
    /// Create a new ocean detector with custom configuration.
    pub fn with_config(config: OceanDetectionConfig) -> Self {
        Self { config }
    }
    
    /// Detect the ocean zone for a polygon based on elevation.
    ///
    /// # Arguments
    /// * `polygon` - The polygon to classify
    ///
    /// # Returns
    /// The OceanZone classification for this polygon.
    pub fn detect_zone(&self, polygon: &Polygon) -> OceanZone {
        let elevation = polygon.elevation;
        
        if elevation > self.config.ocean_elevation_threshold {
            // Above water
            OceanZone::Land
        } else if elevation <= self.config.deep_ocean_threshold {
            // Below shallow threshold = deep ocean
            OceanZone::DeepOcean
        } else if elevation <= self.config.shallow_ocean_threshold {
            // Below ocean threshold but above deep = medium
            OceanZone::MediumOcean
        } else {
            // Above shallow threshold but at/below sea level = shallow
            OceanZone::ShallowOcean
        }
    }
    
    /// Detect all ocean zones for an entire polygon graph.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph to analyze
    ///
    /// # Returns
    /// A vector of (polygon_id, OceanZone) pairs.
    pub fn detect_all_zones(&self, graph: &PolygonGraph) -> Vec<(u32, OceanZone)> {
        graph.polygon_ids()
            .filter_map(|id| {
                graph.get(id).map(|p| (id, self.detect_zone(p)))
            })
            .collect()
    }
    
    /// Identify all coastal polygons (ocean cells adjacent to land).
    ///
    /// # Arguments
    /// * `graph` - The polygon graph to analyze
    ///
    /// # Returns
    /// Vector of coastal polygon IDs.
    pub fn detect_coastal_polygons(&self, graph: &PolygonGraph) -> Vec<u32> {
        graph.polygon_ids()
            .filter(|&id| {
                if let Some(polygon) = graph.get(id) {
                    // Must be ocean (low elevation)
                    let zone = self.detect_zone(polygon);
                    if zone == OceanZone::Land {
                        return false;
                    }
                    
                    // Must have at least one land neighbor
                    polygon.neighbors.iter().any(|&neighbor_id| {
                        graph.get(neighbor_id)
                            .map(|n| self.detect_zone(n) == OceanZone::Land)
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// Calculate comprehensive coastal metrics for a polygon.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph
    /// * `polygon_id` - ID of the polygon to analyze
    /// * `coastal_polygons` - Pre-computed list of coastal polygon IDs for efficiency
    ///
    /// # Returns
    /// CoastalMetrics for the specified polygon.
    pub fn calculate_coastal_metrics(
        &self,
        graph: &PolygonGraph,
        polygon_id: u32,
        coastal_polygons: &[u32],
    ) -> Option<CoastalMetrics> {
        let polygon = graph.get(polygon_id)?;
        let zone = self.detect_zone(polygon);
        
        // Count neighbors by type
        let mut ocean_neighbors = 0u32;
        let mut land_neighbors = 0u32;
        
        for &neighbor_id in &polygon.neighbors {
            if let Some(neighbor) = graph.get(neighbor_id) {
                let neighbor_zone = self.detect_zone(neighbor);
                if neighbor_zone == OceanZone::Land {
                    land_neighbors += 1;
                } else {
                    ocean_neighbors += 1;
                }
            }
        }
        
        let is_coastal = zone != OceanZone::Land && land_neighbors > 0;
        
        // Estimate coastline length based on ocean-land transitions
        let coastline_length = land_neighbors as f32 * 0.5;
        
        // Calculate curvature and detect bay/peninsula
        let (is_bay, is_peninsula, is_headland, curvature) = 
            self.detect_coastal_features(polygon, graph, coastal_polygons);
        
        Some(CoastalMetrics {
            polygon_id,
            is_coastal,
            ocean_neighbor_count: ocean_neighbors,
            land_neighbor_count: land_neighbors,
            coastline_length,
            is_bay,
            is_peninsula,
            is_headland,
            curvature,
        })
    }
    
    /// Detect coastal features (bay, peninsula, headland).
    fn detect_coastal_features(
        &self,
        polygon: &Polygon,
        graph: &PolygonGraph,
        _coastal_polygons: &[u32],
    ) -> (bool, bool, bool, f32) {
        if !self.config.enable_bay_detection && !self.config.enable_peninsula_detection {
            return (false, false, false, 0.0);
        }
        
        let mut curvature_sum = 0.0f32;
        let mut neighbor_count = 0u32;
        
        // For each land neighbor, check if it's surrounded by ocean on the other sides
        // This indicates a peninsula (convex coast)
        for &neighbor_id in &polygon.neighbors {
            if let Some(neighbor) = graph.get(neighbor_id) {
                let neighbor_zone = self.detect_zone(neighbor);
                if neighbor_zone == OceanZone::Land {
                    // This is a land neighbor - count ocean neighbors of this neighbor
                    let neighbor_ocean_count = neighbor.neighbors.iter()
                        .filter(|&&nid| {
                            graph.get(nid)
                                .map(|n| self.detect_zone(n) != OceanZone::Land)
                                .unwrap_or(false)
                        })
                        .count();
                    
                    // If the land neighbor has mostly ocean neighbors, it's a peninsula
                    if neighbor_ocean_count > 2 {
                        curvature_sum += 1.0;
                    }
                    neighbor_count += 1;
                }
            }
        }
        
        // Calculate normalized curvature
        let curvature = if neighbor_count > 0 {
            curvature_sum / neighbor_count as f32
        } else {
            0.0
        };
        
        let is_peninsula = self.config.enable_peninsula_detection && curvature > 0.5;
        
        // Check for bay (concave coastline)
        // A bay is detected when ocean neighbors surround a land area
        let _ocean_surrounding_land = if polygon.neighbors.len() > 0 {
            let _ocean_count = polygon.neighbors.iter()
                .filter(|&&nid| {
                    graph.get(nid)
                        .map(|n| self.detect_zone(n) != OceanZone::Land)
                        .unwrap_or(false)
                })
                .count();
            
            // If this is a land cell surrounded by ocean, it's not a bay
            // But if we're already in ocean zone and most neighbors are also ocean
            // with land on one side, that could be a bay
            false // Simplified bay detection
        } else {
            false
        };
        
        let is_bay = false; // Disabled for now
        let is_headland = is_peninsula && curvature > 0.8;
        
        (is_bay, is_peninsula, is_headland, curvature)
    }
    
    /// Detect all ocean regions (connected ocean areas).
    ///
    /// # Arguments
    /// * `graph` - The polygon graph
    ///
    /// # Returns
    /// A vector of ocean regions, each containing polygon IDs.
    pub fn detect_ocean_regions(&self, graph: &PolygonGraph) -> Vec<Vec<u32>> {
        let zones = self.detect_all_zones(graph);
        let mut visited = vec![false; graph.len()];
        let mut regions: Vec<Vec<u32>> = Vec::new();
        
        for (id, zone) in &zones {
            if *zone != OceanZone::Land && !visited[*id as usize] {
                // Start flood fill from this ocean cell
                let mut region = Vec::new();
                self.flood_fill_ocean(*id, graph, &zones, &mut visited, &mut region);
                
                if !region.is_empty() {
                    regions.push(region);
                }
            }
        }
        
        regions
    }
    
    /// Flood fill to find connected ocean cells.
    fn flood_fill_ocean(
        &self,
        start_id: u32,
        graph: &PolygonGraph,
        zones: &[(u32, OceanZone)],
        visited: &mut [bool],
        region: &mut Vec<u32>,
    ) {
        let mut stack = vec![start_id];
        
        while let Some(id) = stack.pop() {
            if visited[id as usize] {
                continue;
            }
            
            visited[id as usize] = true;
            region.push(id);
            
            // Add unvisited ocean neighbors to stack
            if let Some(polygon) = graph.get(id) {
                for &neighbor_id in &polygon.neighbors {
                    if !visited[neighbor_id as usize] {
                        if let Some((_, zone)) = zones.iter().find(|(zid, _)| *zid == neighbor_id) {
                            if *zone != OceanZone::Land {
                                stack.push(neighbor_id);
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Calculate statistics for all coastal areas.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph
    ///
    /// # Returns
    /// CoastalStatistics with aggregated metrics.
    pub fn calculate_coastal_statistics(&self, graph: &PolygonGraph) -> CoastalStatistics {
        let coastal_ids = self.detect_coastal_polygons(graph);
        let ocean_regions = self.detect_ocean_regions(graph);
        
        let total_coastline: f32 = coastal_ids.iter()
            .filter_map(|&id| {
                self.calculate_coastal_metrics(graph, id, &coastal_ids)
                    .map(|m| m.coastline_length)
            })
            .sum();
        
        let bays = coastal_ids.iter()
            .filter_map(|&id| {
                self.calculate_coastal_metrics(graph, id, &coastal_ids)
                    .map(|m| m.is_bay)
            })
            .filter(|&b| b)
            .count();
        
        let peninsulas = coastal_ids.iter()
            .filter_map(|&id| {
                self.calculate_coastal_metrics(graph, id, &coastal_ids)
                    .map(|m| m.is_peninsula)
            })
            .filter(|&p| p)
            .count();
        
        CoastalStatistics {
            total_coastal_polygons: coastal_ids.len(),
            total_ocean_regions: ocean_regions.len(),
            total_coastline_length: total_coastline,
            bay_count: bays,
            peninsula_count: peninsulas,
            coastal_polygon_ids: coastal_ids,
        }
    }
    
    /// Classify a specific location by its ocean zone.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph
    /// * `polygon_id` - ID of the polygon to classify
    ///
    /// # Returns
    /// (OceanZone, CoastalMetrics) for the location.
    pub fn classify_location(&self, graph: &PolygonGraph, polygon_id: u32) -> Option<(OceanZone, CoastalMetrics)> {
        let polygon = graph.get(polygon_id)?;
        let zone = self.detect_zone(polygon);
        let coastal_ids = self.detect_coastal_polygons(graph);
        let metrics = self.calculate_coastal_metrics(graph, polygon_id, &coastal_ids)?;
        Some((zone, metrics))
    }
    
    /// Check if a polygon is submerged (below sea level).
    ///
    /// # Arguments
    /// * `polygon` - The polygon to check
    ///
    /// # Returns
    /// True if the polygon is ocean.
    pub fn is_ocean(&self, polygon: &Polygon) -> bool {
        self.detect_zone(polygon) != OceanZone::Land
    }
    
    /// Check if a polygon is coastal (adjacent to both ocean and land).
    ///
    /// # Arguments
    /// * `graph` - The polygon graph
    /// * `polygon_id` - ID of the polygon to check
    ///
    /// # Returns
    /// True if the polygon is coastal.
    pub fn is_coastal(&self, graph: &PolygonGraph, polygon_id: u32) -> bool {
        let Some(polygon) = graph.get(polygon_id) else {
            return false;
        };
        let zone = self.detect_zone(polygon);
        
        if zone == OceanZone::Land {
            // Land cell is coastal if it has ocean neighbors
            polygon.neighbors.iter().any(|&nid| {
                graph.get(nid)
                    .map(|n| self.detect_zone(n) != OceanZone::Land)
                    .unwrap_or(false)
            })
        } else {
            // Ocean cell is coastal if it has land neighbors
            polygon.neighbors.iter().any(|&nid| {
                graph.get(nid)
                    .map(|n| self.detect_zone(n) == OceanZone::Land)
                    .unwrap_or(false)
            })
        }
    }
}

impl Default for OceanDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated coastal statistics for a generated world.
#[derive(Debug, Clone)]
pub struct CoastalStatistics {
    /// Total number of coastal polygons.
    pub total_coastal_polygons: usize,
    /// Number of distinct ocean regions.
    pub total_ocean_regions: usize,
    /// Total estimated coastline length.
    pub total_coastline_length: f32,
    /// Number of detected bays.
    pub bay_count: usize,
    /// Number of detected peninsulas.
    pub peninsula_count: usize,
    /// List of coastal polygon IDs.
    pub coastal_polygon_ids: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Seed;
    use crate::terrain::Polygon;

    fn create_test_graph() -> PolygonGraph {
        let mut graph = PolygonGraph::with_capacity(9);
        
        // Create 3x3 grid:
        // Ocean Ocean Ocean
        // Ocean Land  Ocean
        // Ocean Ocean Ocean
        for i in 0..9 {
            graph.add_polygon(Polygon::new(i));
        }
        
        // Connect in grid pattern
        // Row 0: 0-1-2
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        // Row 1: 3-4-5
        graph.add_edge(3, 4);
        graph.add_edge(4, 5);
        // Row 2: 6-7-8
        graph.add_edge(6, 7);
        graph.add_edge(7, 8);
        // Vertical connections
        graph.add_edge(0, 3);
        graph.add_edge(1, 4);
        graph.add_edge(2, 5);
        graph.add_edge(3, 6);
        graph.add_edge(4, 7);
        graph.add_edge(5, 8);
        
        // Set elevations: center (4) is land, edges are ocean
        // Ocean polygons: 0, 1, 2, 3, 5, 6, 7, 8
        for i in 0..9 {
            let elevation = if i == 4 { 0.5 } else { -0.1 };
            graph.get_mut(i as u32).unwrap().set_elevation(elevation);
        }
        
        graph
    }

    #[test]
    fn test_ocean_zone_detection() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        
        // Center polygon (4) should be land
        let center = graph.get(4).unwrap();
        assert_eq!(detector.detect_zone(center), OceanZone::Land);
        
        // Edge polygons should be ocean
        let edge = graph.get(0).unwrap();
        assert_ne!(detector.detect_zone(edge), OceanZone::Land);
    }

    #[test]
    fn test_coastal_polygon_detection() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        
        let coastal = detector.detect_coastal_polygons(&graph);
        
        // Coastal should include ocean cells adjacent to land (0, 1, 3, 5, 7, 8)
        assert!(coastal.contains(&0));
        assert!(coastal.contains(&1));
        assert!(coastal.contains(&3));
        assert!(coastal.contains(&5));
        // Center land cell not coastal
        assert!(!coastal.contains(&4));
    }

    #[test]
    fn test_ocean_region_detection() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        
        let regions = detector.detect_ocean_regions(&graph);
        
        // All ocean cells should be one connected region
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].len(), 8); // 8 ocean cells
    }

    #[test]
    fn test_coastal_metrics() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        let coastal = detector.detect_coastal_polygons(&graph);
        
        // Check metrics for a coastal polygon
        if let Some(metrics) = detector.calculate_coastal_metrics(&graph, 4, &coastal) {
            // Polygon 4 is land with ocean neighbors
            assert!(!metrics.is_coastal); // Land cells aren't "coastal" in our definition
        }
        
        // Check metrics for an ocean cell
        if let Some(metrics) = detector.calculate_coastal_metrics(&graph, 1, &coastal) {
            assert!(metrics.is_coastal);
            assert!(metrics.land_neighbor_count > 0);
        }
    }

    #[test]
    fn test_depth_classification() {
        let mut graph = PolygonGraph::with_capacity(3);
        
        graph.add_polygon(Polygon::new(0));
        graph.add_polygon(Polygon::new(1));
        graph.add_polygon(Polygon::new(2));
        
        // Set different depths
        graph.get_mut(0).unwrap().set_elevation(-0.05); // Shallow
        graph.get_mut(1).unwrap().set_elevation(-0.3); // Medium
        graph.get_mut(2).unwrap().set_elevation(-0.7);  // Deep
        
        let detector = OceanDetector::new();
        
        assert_eq!(detector.detect_zone(graph.get(0).unwrap()), OceanZone::ShallowOcean);
        assert_eq!(detector.detect_zone(graph.get(1).unwrap()), OceanZone::MediumOcean);
        assert_eq!(detector.detect_zone(graph.get(2).unwrap()), OceanZone::DeepOcean);
    }

    #[test]
    fn test_deterministic_detection() {
        let graph1 = create_test_graph();
        let graph2 = create_test_graph();
        
        let detector = OceanDetector::new();
        
        let coastal1 = detector.detect_coastal_polygons(&graph1);
        let coastal2 = detector.detect_coastal_polygons(&graph2);
        
        assert_eq!(coastal1, coastal2);
    }

    #[test]
    fn test_is_ocean() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        
        assert!(!detector.is_ocean(graph.get(4).unwrap())); // Land
        assert!(detector.is_ocean(graph.get(0).unwrap()));  // Ocean
    }

    #[test]
    fn test_coastal_statistics() {
        let detector = OceanDetector::new();
        let graph = create_test_graph();
        
        let stats = detector.calculate_coastal_statistics(&graph);
        
        assert!(stats.total_coastal_polygons > 0);
        assert_eq!(stats.total_ocean_regions, 1);
    }
}