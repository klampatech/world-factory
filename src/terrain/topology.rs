//! Topology types for World Factory.
//!
//! Provides explicit polygon adjacency and edge relationships for the terrain
//! system. Topology captures the "shape" of the Voronoi partition independent
//! of the mesh geometry.
//!
//! # Key Concepts
//!
//! - **Edge**: A boundary between two adjacent polygons (or a polygon and ocean/boundary)
//! - **Neighbor**: Another polygon sharing an edge with this polygon
//! - **Centroid**: The geometric center of the polygon (computed from cell points)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a topology map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopologyId(pub u32);

impl TopologyId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Default for TopologyId {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for TopologyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "topology:{}", self.0)
    }
}

/// Type of border/edge between polygons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderType {
    /// Edge between two land polygons.
    Interior,
    /// Polygon touches the ocean.
    Coast,
    /// Polygon is on the world boundary.
    Boundary,
    /// Edge contains a river (future extension).
    River,
    /// Edge contains a road (future extension).
    Road,
    /// Border between different political entities.
    Political,
}

impl BorderType {
    /// Check if this border is coastal.
    pub fn is_coastal(&self) -> bool {
        matches!(self, Self::Coast)
    }

    /// Check if this border is interior (land-to-land).
    pub fn is_interior(&self) -> bool {
        matches!(self, Self::Interior)
    }

    /// Check if this border is at the world edge.
    pub fn is_boundary(&self) -> bool {
        matches!(self, Self::Boundary)
    }

    /// Check if this border represents a significant feature.
    pub fn is_feature(&self) -> bool {
        matches!(self, Self::River | Self::Road | Self::Political)
    }
}

/// An edge between two polygons in the Voronoi partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonEdge {
    /// The first polygon ID.
    pub polygon_a: u32,
    /// The second polygon ID (None if border is with void/boundary).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polygon_b: Option<u32>,
    /// Midpoint coordinates of the edge.
    pub midpoint: [f32; 2],
    /// Edge length in world units.
    pub length: f32,
    /// Type of border this edge represents.
    pub border_type: BorderType,
    /// Angle of the edge (radians, for direction detection).
    pub angle: f32,
    /// Perpendicular distance to coastline if coastal (for river placement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coast_distance: Option<f32>,
}

impl PolygonEdge {
    /// Create a new edge between two polygons.
    pub fn new(polygon_a: u32, polygon_b: u32, midpoint: [f32; 2], length: f32) -> Self {
        Self {
            polygon_a,
            polygon_b: Some(polygon_b),
            midpoint,
            length,
            border_type: BorderType::Interior,
            angle: 0.0,
            coast_distance: None,
        }
    }

    /// Create a boundary edge (to void/boundary).
    pub fn boundary(polygon_a: u32, midpoint: [f32; 2], length: f32) -> Self {
        Self {
            polygon_a,
            polygon_b: None,
            midpoint,
            length,
            border_type: BorderType::Boundary,
            angle: 0.0,
            coast_distance: None,
        }
    }

    /// Create a coastal edge.
    pub fn coast(polygon_a: u32, midpoint: [f32; 2], length: f32, coast_distance: f32) -> Self {
        Self {
            polygon_a,
            polygon_b: None,
            midpoint,
            length,
            border_type: BorderType::Coast,
            angle: 0.0,
            coast_distance: Some(coast_distance),
        }
    }

    /// Get the neighboring polygon ID, if any.
    pub fn neighbor(&self) -> Option<u32> {
        self.polygon_b
    }

    /// Check if this edge is a boundary edge.
    pub fn is_boundary(&self) -> bool {
        self.polygon_b.is_none()
    }

    /// Set the edge angle.
    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    /// Set the border type.
    pub fn with_border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }
}

/// Explicit polygon adjacency data for a single polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonTopology {
    /// The polygon this topology describes.
    pub polygon_id: u32,
    /// All edges of this polygon.
    pub edges: Vec<PolygonEdge>,
    /// IDs of neighboring polygons (computed from edges).
    pub neighbors: Vec<u32>,
    /// The geometric centroid of the polygon.
    pub centroid: [f32; 2],
    /// Approximate area of the polygon (in Voronoi cell units squared).
    pub area_hint: f32,
    /// Perimeter length.
    pub perimeter: f32,
    /// Number of edges/sides of the polygon.
    pub edge_count: usize,
    /// Polygon shape descriptor (compactness measure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_factor: Option<f32>,
    /// Elevation of the centroid (useful for biome assignment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f32>,
}

impl PolygonTopology {
    /// Create a new polygon topology.
    pub fn new(polygon_id: u32, centroid: [f32; 2]) -> Self {
        Self {
            polygon_id,
            edges: Vec::new(),
            neighbors: Vec::new(),
            centroid,
            area_hint: 0.0,
            perimeter: 0.0,
            edge_count: 0,
            shape_factor: None,
            elevation: None,
        }
    }

    /// Add an edge to this polygon.
    pub fn add_edge(&mut self, edge: PolygonEdge) {
        // Track neighbor and length before moving edge
        let neighbor_id = edge.neighbor();
        let edge_length = edge.length;
        
        self.edges.push(edge);
        self.perimeter += edge_length;
        self.edge_count += 1;
        
        // Track neighbors
        if let Some(neighbor_id) = neighbor_id {
            if !self.neighbors.contains(&neighbor_id) {
                self.neighbors.push(neighbor_id);
            }
        }
    }

    /// Compute the shape factor (circularity = 4π * area / perimeter²).
    /// A perfect circle has shape factor of 1.0, irregular shapes are lower.
    pub fn compute_shape_factor(&mut self) {
        if self.perimeter > 0.0 && self.area_hint > 0.0 {
            self.shape_factor = Some(
                (4.0 * std::f32::consts::PI * self.area_hint) / (self.perimeter * self.perimeter)
            );
        }
    }

    /// Get all coastal edge indices.
    pub fn coastal_edge_indices(&self) -> Vec<usize> {
        self.edges
            .iter()
            .enumerate()
            .filter(|(_, e)| e.border_type.is_coastal())
            .map(|(i, _)| i)
            .collect()
    }

    /// Get the total coastline length.
    pub fn coastline_length(&self) -> f32 {
        self.edges
            .iter()
            .filter(|e| e.border_type.is_coastal())
            .map(|e| e.length)
            .sum()
    }

    /// Check if this polygon has any coastline.
    pub fn is_coastal(&self) -> bool {
        self.edges.iter().any(|e| e.border_type.is_coastal())
    }

    /// Get edge by neighbor polygon ID.
    pub fn edge_to(&self, neighbor_id: u32) -> Option<&PolygonEdge> {
        self.edges.iter().find(|e| e.neighbor() == Some(neighbor_id))
    }

    /// Set elevation value.
    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = Some(elevation);
        self
    }

    /// Set area hint.
    pub fn with_area(mut self, area: f32) -> Self {
        self.area_hint = area;
        self
    }

    /// Set shape factor directly.
    pub fn with_shape_factor(mut self, factor: f32) -> Self {
        self.shape_factor = Some(factor);
        self
    }
}

/// A complete topology map for all polygons in a Voronoi partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonTopologyMap {
    /// Unique identifier for this topology.
    pub id: TopologyId,
    /// Width of the world in Voronoi cells.
    pub world_width: u32,
    /// Height of the world in Voronoi cells.
    pub world_height: u32,
    /// Topology data for each polygon (indexed by polygon ID).
    pub polygons: Vec<PolygonTopology>,
    /// Metadata about the topology.
    pub metadata: TopologyMetadata,
}

impl PolygonTopologyMap {
    /// Create a new empty topology map.
    pub fn new(id: TopologyId, world_width: u32, world_height: u32) -> Self {
        Self {
            id,
            world_width,
            world_height,
            polygons: Vec::new(),
            metadata: TopologyMetadata::default(),
        }
    }

    /// Create with pre-allocated capacity.
    pub fn with_capacity(id: TopologyId, world_width: u32, world_height: u32, n_polygons: usize) -> Self {
        let mut map = Self::new(id, world_width, world_height);
        map.polygons.reserve(n_polygons);
        map
    }

    /// Add a polygon topology.
    pub fn add_polygon(&mut self, topology: PolygonTopology) {
        self.polygons.push(topology);
    }

    /// Get topology for a polygon.
    pub fn get(&self, polygon_id: u32) -> Option<&PolygonTopology> {
        self.polygons.get(polygon_id as usize)
    }

    /// Get mutable topology for a polygon.
    pub fn get_mut(&mut self, polygon_id: u32) -> Option<&mut PolygonTopology> {
        self.polygons.get_mut(polygon_id as usize)
    }

    /// Get the number of polygons.
    pub fn len(&self) -> usize {
        self.polygons.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.polygons.is_empty()
    }

    /// Get an iterator over all polygon topologies.
    pub fn iter(&self) -> impl Iterator<Item = &PolygonTopology> + '_ {
        self.polygons.iter()
    }

    /// Get a mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PolygonTopology> + '_ {
        self.polygons.iter_mut()
    }

    /// Find all coastal polygon IDs.
    pub fn coastal_polygon_ids(&self) -> Vec<u32> {
        self.polygons
            .iter()
            .filter(|p| p.is_coastal())
            .map(|p| p.polygon_id)
            .collect()
    }

    /// Find all polygons adjacent to a given polygon.
    pub fn neighbors_of(&self, polygon_id: u32) -> Vec<u32> {
        self.get(polygon_id)
            .map(|p| p.neighbors.clone())
            .unwrap_or_default()
    }

    /// Find the shared edge between two adjacent polygons.
    pub fn shared_edge(&self, polygon_a: u32, polygon_b: u32) -> Option<&PolygonEdge> {
        self.get(polygon_a)?.edge_to(polygon_b)
    }

    /// Get total coastline length across all polygons.
    pub fn total_coastline(&self) -> f32 {
        self.polygons.iter().map(|p| p.coastline_length()).sum::<f32>() * 0.5 // Each coast counted twice
    }

    /// Get total perimeter length.
    pub fn total_perimeter(&self) -> f32 {
        self.polygons.iter().map(|p| p.perimeter).sum::<f32>() * 0.5 // Interior edges counted twice
    }

    /// Compute average shape factor.
    pub fn average_shape_factor(&self) -> f32 {
        let factors: Vec<f32> = self.polygons
            .iter()
            .filter_map(|p| p.shape_factor)
            .collect();
        
        if factors.is_empty() {
            return 0.0;
        }
        factors.iter().sum::<f32>() / factors.len() as f32
    }

    /// Find polygons with coastline.
    pub fn coastal_polygons(&self) -> impl Iterator<Item = &PolygonTopology> + '_ {
        self.polygons.iter().filter(|p| p.is_coastal())
    }

    /// Find polygons with specific border type.
    pub fn polygons_with_border(&self, border_type: BorderType) -> Vec<&PolygonTopology> {
        self.polygons.iter()
            .filter(|p| p.edges.iter().any(|e| e.border_type == border_type))
            .collect()
    }

    /// Compute adjacency graph as a sparse representation.
    pub fn adjacency_map(&self) -> HashMap<u32, Vec<u32>> {
        self.polygons.iter()
            .map(|p| (p.polygon_id, p.neighbors.clone()))
            .collect()
    }
}

/// Metadata about a topology map.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologyMetadata {
    /// Source identifier (e.g., "voronoi:seed_12345").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Number of Voronoi cells/seeds.
    pub polygon_count: usize,
    /// Total number of edges.
    pub total_edges: usize,
    /// Number of coastal edges.
    pub coastal_edges: usize,
    /// Number of boundary edges.
    pub boundary_edges: usize,
    /// Whether Lloyd relaxation was applied.
    pub relaxed: bool,
    /// Number of Lloyd iterations if applied.
    pub lloyd_iterations: u32,
    /// Custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

impl TopologyMetadata {
    pub fn new(polygon_count: usize) -> Self {
        Self {
            source: None,
            polygon_count,
            total_edges: 0,
            coastal_edges: 0,
            boundary_edges: 0,
            relaxed: false,
            lloyd_iterations: 0,
            custom: None,
        }
    }

    /// Set the source identifier.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Mark as having Lloyd relaxation applied.
    pub fn with_lloyd(mut self, iterations: u32) -> Self {
        self.relaxed = true;
        self.lloyd_iterations = iterations;
        self
    }

    /// Calculate coast percentage.
    pub fn coast_percentage(&self) -> f32 {
        if self.total_edges > 0 {
            self.coastal_edges as f32 / self.total_edges as f32
        } else {
            0.0
        }
    }
}

/// Builder for constructing topology maps from Voronoi results.
#[derive(Debug, Clone)]
pub struct TopologyBuilder {
    world_width: u32,
    world_height: u32,
    polygon_count: usize,
}

impl TopologyBuilder {
    /// Create a new topology builder.
    pub fn new(world_width: u32, world_height: u32, polygon_count: usize) -> Self {
        Self {
            world_width,
            world_height,
            polygon_count,
        }
    }

    /// Build the complete topology map.
    /// This method should be called with the VoronoiResult to extract edge data.
    pub fn build(
        &self,
        result: &crate::generation::voronoi::VoronoiResult,
        polygon_data: &crate::terrain::PolygonGraph,
    ) -> PolygonTopologyMap {
        let mut topology_map = PolygonTopologyMap::with_capacity(
            TopologyId::default(),
            self.world_width,
            self.world_height,
            self.polygon_count,
        );

        // Step 1: Create polygon topologies with centroids
        self.create_polygon_topologies(result, polygon_data, &mut topology_map);

        // Step 2: Detect edges between polygons
        self.detect_edges(result, &mut topology_map);

        // Step 3: Classify edges (interior, coast, boundary)
        self.classify_edges(polygon_data, &mut topology_map);

        // Step 4: Update metadata
        self.update_metadata(&mut topology_map);

        topology_map
    }

    /// Create basic polygon topologies.
    fn create_polygon_topologies(
        &self,
        result: &crate::generation::voronoi::VoronoiResult,
        _polygon_data: &crate::terrain::PolygonGraph,
        topology_map: &mut PolygonTopologyMap,
    ) {
        for seed in &result.seeds {
            // Compute centroid from cell points
            let (sum_x, sum_y, count) = self.compute_cell_centroid(result, seed.id);
            
            let centroid = if count > 0 {
                [sum_x / count as f32, sum_y / count as f32]
            } else {
                [seed.x, seed.y] // Fallback to seed position
            };

            let area_hint = count as f32; // Each cell is approximately 1 unit²
            let topology = PolygonTopology::new(seed.id, centroid)
                .with_area(area_hint);
            
            topology_map.add_polygon(topology);
        }
    }

    /// Compute the centroid of a Voronoi cell.
    fn compute_cell_centroid(
        &self,
        result: &crate::generation::voronoi::VoronoiResult,
        seed_id: u32,
    ) -> (f32, f32, usize) {
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut count = 0usize;

        for y in 0..result.height {
            for x in 0..result.width {
                if result.cell_at(x, y) == Some(seed_id) {
                    sum_x += x as f32;
                    sum_y += y as f32;
                    count += 1;
                }
            }
        }

        (sum_x, sum_y, count)
    }

    /// Detect edges between polygons by scanning cell boundaries.
    fn detect_edges(
        &self,
        result: &crate::generation::voronoi::VoronoiResult,
        topology_map: &mut PolygonTopologyMap,
    ) {
        let width = result.width;
        let height = result.height;

        // Horizontal edges (right boundary of each cell)
        for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                let cell_a = result.cell_at(x, y);
                let cell_b = result.cell_at(x + 1, y);
                
                if cell_a != cell_b {
                    let midpoint = [(x as f32 + 0.5), y as f32];
                    let length = 1.0;
                    
                    if let (Some(id_a), Some(id_b)) = (cell_a, cell_b) {
                        // Add edge to both polygons
                        if let Some(poly_a) = topology_map.get_mut(id_a) {
                            poly_a.add_edge(PolygonEdge::new(id_a, id_b, midpoint, length));
                        }
                        if let Some(poly_b) = topology_map.get_mut(id_b) {
                            poly_b.add_edge(PolygonEdge::new(id_b, id_a, midpoint, length));
                        }
                    }
                }
            }
        }

        // Vertical edges (bottom boundary of each cell)
        for y in 0..height.saturating_sub(1) {
            for x in 0..width {
                let cell_a = result.cell_at(x, y);
                let cell_b = result.cell_at(x, y + 1);
                
                if cell_a != cell_b {
                    let midpoint = [x as f32, y as f32 + 0.5];
                    let length = 1.0;
                    
                    if let (Some(id_a), Some(id_b)) = (cell_a, cell_b) {
                        if let Some(poly_a) = topology_map.get_mut(id_a) {
                            poly_a.add_edge(PolygonEdge::new(id_a, id_b, midpoint, length));
                        }
                        if let Some(poly_b) = topology_map.get_mut(id_b) {
                            poly_b.add_edge(PolygonEdge::new(id_b, id_a, midpoint, length));
                        }
                    }
                }
            }
        }
    }

    /// Classify edges based on polygon properties.
    fn classify_edges(
        &self,
        polygon_data: &crate::terrain::PolygonGraph,
        topology_map: &mut PolygonTopologyMap,
    ) {
        for topology in &mut topology_map.polygons {
            for edge in &mut topology.edges {
                // Check if polygon is coastal
                let is_coastal = polygon_data
                    .get(edge.polygon_a)
                    .map(|p| p.is_coastal)
                    .unwrap_or(false);

                if is_coastal {
                    edge.border_type = BorderType::Coast;
                }
            }
        }
    }

    /// Update topology metadata.
    fn update_metadata(&self, topology_map: &mut PolygonTopologyMap) {
        let total_edges: usize = topology_map.polygons.iter().map(|p| p.edge_count).sum();
        let coastal_edges: usize = topology_map.polygons
            .iter()
            .filter(|p| p.is_coastal())
            .map(|p| p.coastline_length() as usize)
            .sum();

        topology_map.metadata.total_edges = total_edges;
        topology_map.metadata.coastal_edges = coastal_edges;
        topology_map.metadata.polygon_count = topology_map.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_type_checks() {
        assert!(BorderType::Coast.is_coastal());
        assert!(!BorderType::Interior.is_coastal());
        assert!(BorderType::Interior.is_interior());
        assert!(BorderType::Boundary.is_boundary());
        assert!(!BorderType::River.is_boundary());
        assert!(BorderType::River.is_feature());
        assert!(!BorderType::Interior.is_feature());
    }

    #[test]
    fn test_polygon_edge_creation() {
        let edge = PolygonEdge::new(0, 1, [1.5, 2.5], 1.0);
        assert_eq!(edge.polygon_a, 0);
        assert_eq!(edge.neighbor(), Some(1));
        assert!(!edge.is_boundary());
    }

    #[test]
    fn test_boundary_edge() {
        let edge = PolygonEdge::boundary(0, [0.5, 1.5], 1.0);
        assert_eq!(edge.border_type, BorderType::Boundary);
        assert!(edge.is_boundary());
        assert_eq!(edge.neighbor(), None);
    }

    #[test]
    fn test_coastal_edge() {
        let edge = PolygonEdge::coast(0, [2.5, 0.5], 1.0, 5.0);
        assert_eq!(edge.border_type, BorderType::Coast);
        assert!(edge.is_boundary());
        assert_eq!(edge.coast_distance, Some(5.0));
    }

    #[test]
    fn test_polygon_topology_basic() {
        let mut topology = PolygonTopology::new(0, [5.0, 5.0]);
        
        topology.add_edge(PolygonEdge::new(0, 1, [4.5, 5.0], 1.0));
        topology.add_edge(PolygonEdge::new(0, 2, [5.5, 5.0], 1.0));
        
        assert_eq!(topology.polygon_id, 0);
        assert_eq!(topology.neighbors.len(), 2);
        assert!(topology.neighbors.contains(&1));
        assert!(topology.neighbors.contains(&2));
        assert_eq!(topology.perimeter, 2.0);
        assert_eq!(topology.edge_count, 2);
    }

    #[test]
    fn test_polygon_topology_coastal() {
        let mut topology = PolygonTopology::new(0, [5.0, 5.0]);
        topology.add_edge(PolygonEdge::coast(0, [4.5, 5.0], 1.0, 0.0));
        topology.add_edge(PolygonEdge::new(0, 1, [5.5, 5.0], 1.0));
        
        assert!(topology.is_coastal());
        assert_eq!(topology.coastline_length(), 1.0);
    }

    #[test]
    fn test_polygon_topology_shape_factor() {
        let mut topology = PolygonTopology::new(0, [0.0, 0.0]);
        topology.add_edge(PolygonEdge::new(0, 1, [0.5, 0.0], 1.0));
        topology.add_edge(PolygonEdge::new(0, 1, [1.0, 0.5], 1.0));
        topology.add_edge(PolygonEdge::new(0, 1, [0.5, 1.0], 1.0));
        topology.add_edge(PolygonEdge::new(0, 1, [0.0, 0.5], 1.0));
        topology.perimeter = 4.0;
        topology.area_hint = 1.0;
        
        topology.compute_shape_factor();
        
        // A perfect square (1x1) should have shape factor around 0.785
        // 4π * 1 / 16 ≈ 0.785
        assert!(topology.shape_factor.is_some());
        let factor = topology.shape_factor.unwrap();
        assert!(factor > 0.7 && factor < 0.9);
    }

    #[test]
    fn test_topology_map_basic() {
        let mut map = PolygonTopologyMap::with_capacity(TopologyId::default(), 10, 10, 3);
        
        map.add_polygon(PolygonTopology::new(0, [5.0, 5.0]));
        map.add_polygon(PolygonTopology::new(1, [15.0, 5.0]));
        map.add_polygon(PolygonTopology::new(2, [5.0, 15.0]));
        
        assert_eq!(map.len(), 3);
        assert!(map.get(0).is_some());
        assert!(map.get(3).is_none());
    }

    #[test]
    fn test_topology_map_neighbors() {
        let mut map = PolygonTopologyMap::new(TopologyId::default(), 10, 10);
        
        let mut poly0 = PolygonTopology::new(0, [5.0, 5.0]);
        poly0.add_edge(PolygonEdge::new(0, 1, [10.0, 5.0], 1.0));
        map.add_polygon(poly0);
        
        let mut poly1 = PolygonTopology::new(1, [15.0, 5.0]);
        poly1.add_edge(PolygonEdge::new(1, 0, [10.0, 5.0], 1.0));
        map.add_polygon(poly1);
        
        let neighbors = map.neighbors_of(0);
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&1));
    }

    #[test]
    fn test_adjacency_map() {
        let mut map = PolygonTopologyMap::new(TopologyId::default(), 10, 10);
        
        let mut poly0 = PolygonTopology::new(0, [0.0, 0.0]);
        poly0.add_edge(PolygonEdge::new(0, 1, [0.5, 0.0], 1.0));
        map.add_polygon(poly0);
        
        let mut poly1 = PolygonTopology::new(1, [1.0, 0.0]);
        poly1.add_edge(PolygonEdge::new(1, 0, [0.5, 0.0], 1.0));
        map.add_polygon(poly1);
        
        let adj = map.adjacency_map();
        assert_eq!(adj.len(), 2);
        assert_eq!(adj[&0], vec![1]);
        assert_eq!(adj[&1], vec![0]);
    }

    #[test]
    fn test_topology_builder() {
        use crate::generation::voronoi::{VoronoiConfig, VoronoiGenerator};
        
        let config = VoronoiConfig {
            width: 8,
            height: 8,
            num_seeds: 4,
            blue_noise: false,
            ..Default::default()
        };
        
        let mut gen = VoronoiGenerator::new(config, 123);
        let result = gen.generate();
        
        // Create empty polygon graph
        let mut polygon_data = crate::terrain::PolygonGraph::with_capacity(4);
        for i in 0..4 {
            polygon_data.add_polygon(crate::terrain::Polygon::new(i as u32));
        }
        
        let builder = TopologyBuilder::new(8, 8, 4);
        let topology = builder.build(&result, &polygon_data);
        
        assert_eq!(topology.len(), 4);
        
        // Check that edges exist
        for i in 0..4 {
            let poly = topology.get(i).unwrap();
            // Some polygons should have edges
            let has_edges = poly.edge_count > 0;
            if !has_edges {
                // This is possible for isolated polygons
                assert_eq!(poly.neighbors.len(), 0);
            }
        }
    }

    #[test]
    fn test_serialization() {
        let mut topology = PolygonTopology::new(0, [5.0, 5.0]);
        topology.add_edge(PolygonEdge::new(0, 1, [4.5, 5.0], 1.0));
        topology.elevation = Some(0.5);
        
        let json = serde_json::to_string(&topology).unwrap();
        let restored: PolygonTopology = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.polygon_id, 0);
        assert_eq!(restored.centroid, [5.0, 5.0]);
        assert_eq!(restored.edge_count, 1);
        assert_eq!(restored.elevation, Some(0.5));
    }
}
