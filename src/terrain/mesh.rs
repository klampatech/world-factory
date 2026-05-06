//! Mesh types for World Factory.
//!
//! Provides explicit geometry representation for rendering and export.
//! Meshes are derived from Voronoi results with elevation data.
//!
//! # Mesh Structure
//!
//! A mesh consists of:
//! - **Vertices**: Position + optional normal/UV/attributes
//! - **Faces**: Triangle faces referencing three vertices each
//! - **Material Groups**: Logical groupings for rendering

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MeshId(pub Uuid);

impl MeshId {
    /// Create a new unique mesh ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for MeshId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MeshId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mesh:{}", self.0)
    }
}

/// A vertex in 3D mesh space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshVertex {
    /// Position in world space (x, y, z).
    /// For terrain: x and y are horizontal, z is elevation.
    pub position: [f32; 3],
    /// Surface normal for lighting (nx, ny, nz).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<[f32; 3]>,
    /// Texture coordinates (u, v).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv: Option<[f32; 2]>,
    /// Arbitrary vertex attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, AttributeValue>>,
}

impl MeshVertex {
    /// Create a new vertex at the given position.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: None,
            uv: None,
            attributes: None,
        }
    }

    /// Create a vertex with position and normal.
    pub fn with_normal(x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: Some([nx, ny, nz]),
            uv: None,
            attributes: None,
        }
    }

    /// Create a vertex with position and UV coordinates.
    pub fn with_uv(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: None,
            uv: Some([u, v]),
            attributes: None,
        }
    }

    /// Add a custom attribute to this vertex.
    pub fn with_attribute(mut self, key: impl Into<String>, value: AttributeValue) -> Self {
        self.attributes.get_or_insert_with(HashMap::new);
        self.attributes.as_mut().unwrap().insert(key.into(), value);
        self
    }

    /// Get the x coordinate.
    pub fn x(&self) -> f32 {
        self.position[0]
    }

    /// Get the y coordinate.
    pub fn y(&self) -> f32 {
        self.position[1]
    }

    /// Get the z (elevation) coordinate.
    pub fn z(&self) -> f32 {
        self.position[2]
    }
}

/// Attribute values that can be stored on vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Int(i32),
    Int2([i32; 2]),
    Int3([i32; 3]),
    Int4([i32; 4]),
}

impl AttributeValue {
    /// Get as f32 if possible.
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            AttributeValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as Float3 if possible.
    pub fn as_f32_vec3(&self) -> Option<[f32; 3]> {
        match self {
            AttributeValue::Float3(v) => Some(*v),
            _ => None,
        }
    }
}

/// A triangle face referencing three vertex indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshFace {
    /// Indices of the three vertices forming this triangle.
    /// Specified in counter-clockwise order for outward-facing normals.
    pub vertices: [u32; 3],
    /// Pre-computed face normal (nx, ny, nz).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<[f32; 3]>,
    /// Material group ID for rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_id: Option<u32>,
    /// Optional UV/attribute data per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertex_attributes: Option<[[AttributeValue; 3]; 4]>, // Up to 4 attributes per vertex
}

impl MeshFace {
    /// Create a new face with the given vertex indices.
    pub fn new(v0: u32, v1: u32, v2: u32) -> Self {
        Self {
            vertices: [v0, v1, v2],
            normal: None,
            material_id: None,
            vertex_attributes: None,
        }
    }

    /// Create a face with a material ID.
    pub fn with_material(v0: u32, v1: u32, v2: u32, material_id: u32) -> Self {
        Self {
            vertices: [v0, v1, v2],
            normal: None,
            material_id: Some(material_id),
            vertex_attributes: None,
        }
    }

    /// Get the vertex index at the given corner (0, 1, or 2).
    pub fn vertex(&self, corner: usize) -> u32 {
        self.vertices[corner.min(2)]
    }

    /// Check if this face contains a given vertex index.
    pub fn contains_vertex(&self, idx: u32) -> bool {
        self.vertices.contains(&idx)
    }

    /// Compute the face normal from vertex positions.
    pub fn compute_normal<V: VertexProvider>(&self, vertices: &[V]) -> Option<[f32; 3]> {
        let v0 = vertices.get(self.vertices[0] as usize)?;
        let v1 = vertices.get(self.vertices[1] as usize)?;
        let v2 = vertices.get(self.vertices[2] as usize)?;

        let p0 = v0.position();
        let p1 = v1.position();
        let p2 = v2.position();

        // Edge vectors
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        // Cross product
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];

        // Normalize
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 0.0 {
            Some([nx / len, ny / len, nz / len])
        } else {
            None
        }
    }
}

/// Trait for types that provide vertex position data.
pub trait VertexProvider {
    fn position(&self) -> [f32; 3];
}

// Implement VertexProvider for MeshVertex
impl VertexProvider for MeshVertex {
    fn position(&self) -> [f32; 3] {
        self.position
    }
}

/// Mesh metadata for tracking and management.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshMetadata {
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Source data origin (e.g., "voronoi:world_seed_12345").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Original world dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_size: Option<[u32; 2]>,
    /// Original polygon/seed count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_polygon_count: Option<usize>,
    /// File format version for compatibility.
    pub version: u32,
    /// Custom metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

impl MeshMetadata {
    pub fn new() -> Self {
        Self {
            name: None,
            source: None,
            world_size: None,
            source_polygon_count: None,
            version: 1,
            custom: None,
        }
    }

    /// Set the mesh name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the source identifier.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// A material group for organizing faces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialGroup {
    pub id: u32,
    pub name: String,
    pub face_range: Option<(usize, usize)>, // Start and end indices
    pub material_params: HashMap<String, AttributeValue>,
}

impl MaterialGroup {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            face_range: None,
            material_params: HashMap::new(),
        }
    }
}

/// Axis-aligned bounding box in 3D space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox3D {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox3D {
    /// Create a bounding box from min and max points.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Create a bounding box containing a set of points.
    pub fn from_points(points: &[[f32; 3]]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for p in points {
            min = [min[0].min(p[0]), min[1].min(p[1]), min[2].min(p[2])];
            max = [max[0].max(p[0]), max[1].max(p[1]), max[2].max(p[2])];
        }

        Some(Self { min, max })
    }

    /// Get the center point of the bounding box.
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Get the size (width, height, depth) of the bounding box.
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Get the volume of the bounding box.
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s[0] * s[1] * s[2]
    }

    /// Check if a point is inside the bounding box.
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0]
            && p[0] <= self.max[0]
            && p[1] >= self.min[1]
            && p[1] <= self.max[1]
            && p[2] >= self.min[2]
            && p[2] <= self.max[2]
    }

    /// Check if another bounding box is inside this one.
    pub fn contains_box(&self, other: &BoundingBox3D) -> bool {
        self.contains_point(other.min) && self.contains_point(other.max)
    }

    /// Expand the bounding box to include a point.
    pub fn expand_to_include(&mut self, p: [f32; 3]) {
        self.min = [
            self.min[0].min(p[0]),
            self.min[1].min(p[1]),
            self.min[2].min(p[2]),
        ];
        self.max = [
            self.max[0].max(p[0]),
            self.max[1].max(p[1]),
            self.max[2].max(p[2]),
        ];
    }

    /// Expand to include another bounding box.
    pub fn expand_to_include_box(&mut self, other: &BoundingBox3D) {
        self.expand_to_include(other.min);
        self.expand_to_include(other.max);
    }
}

impl Default for BoundingBox3D {
    fn default() -> Self {
        Self {
            min: [f32::MAX, f32::MAX, f32::MAX],
            max: [f32::MIN, f32::MIN, f32::MIN],
        }
    }
}

/// Configuration for mesh generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Vertical scale factor for elevation.
    pub elevation_scale: f32,
    /// Include normals in generated mesh.
    pub generate_normals: bool,
    /// Include UV coordinates.
    pub generate_uvs: bool,
    /// Include per-vertex biome/elevation attributes.
    pub include_attributes: bool,
    /// World width in Voronoi cells.
    pub world_width: u32,
    /// World height in Voronoi cells.
    pub world_height: u32,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            elevation_scale: 100.0, // 1.0 elevation = 100 units height
            generate_normals: true,
            generate_uvs: true,
            include_attributes: true,
            world_width: 256,
            world_height: 256,
        }
    }
}

/// A complete mesh with vertices and faces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub id: MeshId,
    pub vertices: Vec<MeshVertex>,
    pub faces: Vec<MeshFace>,
    pub material_groups: Vec<MaterialGroup>,
    pub bounding_box: BoundingBox3D,
    pub metadata: MeshMetadata,
}

impl Mesh {
    /// Create a new empty mesh.
    pub fn new(id: MeshId) -> Self {
        Self {
            id,
            vertices: Vec::new(),
            faces: Vec::new(),
            material_groups: Vec::new(),
            bounding_box: BoundingBox3D::default(),
            metadata: MeshMetadata::new(),
        }
    }

    /// Add a vertex and return its index.
    pub fn add_vertex(&mut self, vertex: MeshVertex) -> u32 {
        let idx = self.vertices.len() as u32;
        self.vertices.push(vertex);
        self.recompute_bounding_box();
        idx
    }

    /// Add a face and return its index.
    pub fn add_face(&mut self, face: MeshFace) -> u32 {
        let idx = self.faces.len() as u32;
        self.faces.push(face);
        idx
    }

    /// Add a material group.
    pub fn add_material_group(&mut self, group: MaterialGroup) {
        self.material_groups.push(group);
    }

    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of faces.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Get the total number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.faces.len()
    }

    /// Check if the mesh is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.faces.is_empty()
    }

    /// Recompute the bounding box from current vertices.
    pub fn recompute_bounding_box(&mut self) {
        self.bounding_box = BoundingBox3D::from_points(
            &self.vertices.iter().map(|v| v.position).collect::<Vec<_>>(),
        )
        .unwrap_or_default();
    }

    /// Generate face normals from vertex positions.
    pub fn generate_face_normals(&mut self) {
        for face in &mut self.faces {
            if face.normal.is_none() {
                face.normal = face.compute_normal(&self.vertices);
            }
        }
    }

    /// Generate smooth vertex normals by averaging face normals.
    pub fn generate_vertex_normals(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        // Initialize accumulators
        let mut normals: Vec<[f32; 3]> = vec![[0.0; 3]; self.vertices.len()];
        let mut counts: Vec<u32> = vec![0; self.vertices.len()];

        // Accumulate face normals
        for face in &self.faces {
            if let Some(normal) = face.compute_normal(&self.vertices) {
                for &vi in &face.vertices {
                    let idx = vi as usize;
                    normals[idx][0] += normal[0];
                    normals[idx][1] += normal[1];
                    normals[idx][2] += normal[2];
                    counts[idx] += 1;
                }
            }
        }

        // Normalize and assign
        for (i, normal) in normals.iter_mut().enumerate() {
            if counts[i] > 0 {
                let len =
                    (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
                if len > 0.0 {
                    normal[0] /= len;
                    normal[1] /= len;
                    normal[2] /= len;
                }
            }

            if self.vertices[i].normal.is_none() {
                self.vertices[i].normal = Some(*normal);
            }
        }
    }

    /// Get an iterator over all triangles as vertex position triples.
    pub fn triangles(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        self.faces.iter().filter_map(move |face| {
            let p0 = self.vertices.get(face.vertices[0] as usize)?.position();
            let p1 = self.vertices.get(face.vertices[1] as usize)?.position();
            let p2 = self.vertices.get(face.vertices[2] as usize)?.position();
            Some([p0, p1, p2])
        })
    }

    /// Calculate total surface area.
    pub fn surface_area(&self) -> f32 {
        self.faces
            .iter()
            .filter_map(|face| {
                let p0 = self.vertices.get(face.vertices[0] as usize)?.position();
                let p1 = self.vertices.get(face.vertices[1] as usize)?.position();
                let p2 = self.vertices.get(face.vertices[2] as usize)?.position();

                // Edge vectors
                let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

                // Cross product magnitude = triangle area * 2
                let cx = e1[1] * e2[2] - e1[2] * e2[1];
                let cy = e1[2] * e2[0] - e1[0] * e2[2];
                let cz = e1[0] * e2[1] - e1[1] * e2[0];
                let area = (cx * cx + cy * cy + cz * cz).sqrt() * 0.5;

                Some(area)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_vertex_creation() {
        let v = MeshVertex::new(1.0, 2.0, 3.0);
        assert_eq!(v.position, [1.0, 2.0, 3.0]);
        assert!(v.normal.is_none());
        assert!(v.uv.is_none());
    }

    #[test]
    fn test_mesh_vertex_with_normal() {
        let v = MeshVertex::with_normal(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(v.normal, Some([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_mesh_face_creation() {
        let f = MeshFace::new(0, 1, 2);
        assert_eq!(f.vertices, [0, 1, 2]);
        assert!(f.material_id.is_none());
    }

    #[test]
    fn test_mesh_face_with_material() {
        let f = MeshFace::with_material(0, 1, 2, 5);
        assert_eq!(f.material_id, Some(5));
    }

    #[test]
    fn test_bounding_box_from_points() {
        let points = [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [-1.0, 1.0, 2.0]];
        let bb = BoundingBox3D::from_points(&points).unwrap();
        assert_eq!(bb.min, [-1.0, 0.0, 0.0]);
        assert_eq!(bb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_bounding_box_center() {
        let bb = BoundingBox3D::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        assert_eq!(bb.center(), [5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_mesh_basic_operations() {
        let mut mesh = Mesh::new(MeshId::new());
        mesh.metadata.name = Some("Test Mesh".to_string());

        // Add vertices
        let v0 = mesh.add_vertex(MeshVertex::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(MeshVertex::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(MeshVertex::new(0.0, 1.0, 0.0));

        // Add face
        mesh.add_face(MeshFace::new(v0, v1, v2));

        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_mesh_serialization() {
        let mut mesh = Mesh::new(MeshId::default());
        mesh.add_vertex(MeshVertex::new(0.0, 0.0, 0.0));
        mesh.add_vertex(MeshVertex::new(1.0, 0.0, 0.0));
        mesh.add_vertex(MeshVertex::new(0.0, 1.0, 0.0));
        mesh.add_face(MeshFace::new(0, 1, 2));

        // Serialize and deserialize
        let json = serde_json::to_string(&mesh).unwrap();
        let restored: Mesh = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.vertex_count(), 3);
        assert_eq!(restored.face_count(), 1);
    }

    #[test]
    fn test_attribute_values() {
        let float_val = AttributeValue::Float(1.5);
        assert_eq!(float_val.as_f32(), Some(1.5));

        let float3_val = AttributeValue::Float3([1.0, 2.0, 3.0]);
        assert_eq!(float3_val.as_f32_vec3(), Some([1.0, 2.0, 3.0]));
        assert_eq!(float3_val.as_f32(), None); // Can't convert Float3 to Float
    }

    #[test]
    fn test_material_group() {
        let group = MaterialGroup::new(0, "grass");
        assert_eq!(group.name, "grass");
        assert_eq!(group.id, 0);
    }
}
