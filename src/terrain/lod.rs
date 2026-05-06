//! Level-of-Detail (LOD) mesh types for World Factory.
//!
//! Provides multi-resolution mesh representation for efficient rendering
//! at different distances. LOD enables smooth performance across devices
//! by reducing geometry complexity for distant terrain.
//!
//! # LOD Strategy
//!
//! - Level 0: Full resolution (all Voronoi cells)
//! - Level 1: Half resolution (merged cells)
//! - Level 2+: Increasingly simplified
//!
//! Transitions between levels can be smooth (vertex morph) or instant.

use super::mesh::{BoundingBox3D, MeshId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for an LOD mesh hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LodMeshId(pub Uuid);

impl LodMeshId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for LodMeshId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LodMeshId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lodmesh:{}", self.0)
    }
}

/// How to transition between LOD levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LodTransition {
    /// Smooth morphing between LOD levels.
    /// Vertices interpolate position over time.
    VertexMorph,
    /// Instant switch at distance threshold.
    /// Simple but may cause popping artifacts.
    InstantSwitch,
    /// Geomipmap-style T-junction blending.
    /// Special geometry handles edge transitions.
    Geomipmap,
}

impl Default for LodTransition {
    fn default() -> Self {
        Self::InstantSwitch
    }
}

/// Configuration for LOD mesh generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    /// The LOD levels to generate.
    pub levels: Vec<LodLevelSpec>,
    /// Transition method between levels.
    pub transition: LodTransition,
    /// Distance thresholds for each level (in world units).
    /// If not specified, computed from triangle count ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_thresholds: Option<Vec<f32>>,
    /// Maximum distance to render (world units).
    pub max_distance: f32,
    /// Vertical distance multiplier (affects when high terrain switches LOD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_bias: Option<f32>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            levels: vec![
                LodLevelSpec {
                    target_triangles: 50000,
                    error_threshold: 0.01,
                },
                LodLevelSpec {
                    target_triangles: 20000,
                    error_threshold: 0.05,
                },
                LodLevelSpec {
                    target_triangles: 8000,
                    error_threshold: 0.1,
                },
                LodLevelSpec {
                    target_triangles: 2000,
                    error_threshold: 0.2,
                },
                LodLevelSpec {
                    target_triangles: 500,
                    error_threshold: 0.5,
                },
            ],
            transition: LodTransition::InstantSwitch,
            distance_thresholds: None,
            max_distance: 10000.0,
            vertical_bias: None,
        }
    }
}

/// Specification for a single LOD level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevelSpec {
    /// Target number of triangles for this level.
    pub target_triangles: usize,
    /// Maximum geometric error allowed (0.0 to 1.0, normalized).
    pub error_threshold: f32,
}

/// A single LOD level with its mesh data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevel {
    /// Level index (0 = highest detail).
    pub level: u32,
    /// Approximate cell size at this level.
    /// Used to determine which LOD to use for a given view.
    pub cell_size_hint: u32,
    /// Reference to the actual mesh data.
    pub mesh_id: MeshId,
    /// Number of triangles in this level.
    pub triangle_count: usize,
    /// Number of vertices.
    pub vertex_count: usize,
    /// Maximum geometric error vs full detail (0-1).
    pub error_metric: f32,
    /// Distance at which this level becomes active.
    /// Computed or provided; use for rendering decisions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_distance: Option<f32>,
    /// Metadata about the simplification process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplification_info: Option<SimplificationInfo>,
}

impl LodLevel {
    /// Get the reduction ratio compared to full detail.
    pub fn reduction_ratio(&self, full_triangle_count: usize) -> f32 {
        if full_triangle_count > 0 {
            self.triangle_count as f32 / full_triangle_count as f32
        } else {
            1.0
        }
    }

    /// Check if this level meets quality criteria.
    pub fn is_acceptable_quality(&self, min_triangles: usize) -> bool {
        self.triangle_count >= min_triangles
    }
}

/// Information about how a LOD level was generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplificationInfo {
    /// Algorithm used (e.g., "quadric_error", "vertex_clustering").
    pub algorithm: String,
    /// Parameters used for simplification.
    pub params: HashMap<String, String>,
    /// Original triangle count.
    pub original_triangles: usize,
    /// Final triangle count.
    pub final_triangles: usize,
    /// Maximum geometric error measured.
    pub max_error: f32,
    /// Mean geometric error.
    pub mean_error: f32,
    /// Time taken to simplify (milliseconds).
    pub simplification_time_ms: u64,
}

impl SimplificationInfo {
    pub fn new(
        algorithm: impl Into<String>,
        original_triangles: usize,
        final_triangles: usize,
    ) -> Self {
        Self {
            algorithm: algorithm.into(),
            params: HashMap::new(),
            original_triangles,
            final_triangles,
            max_error: 0.0,
            mean_error: 0.0,
            simplification_time_ms: 0,
        }
    }

    /// Set a parameter for the simplification algorithm.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Set error metrics.
    pub fn with_errors(mut self, max_error: f32, mean_error: f32) -> Self {
        self.max_error = max_error;
        self.mean_error = mean_error;
        self
    }

    /// Set simplification time.
    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.simplification_time_ms = time_ms;
        self
    }
}

/// Container for a complete LOD mesh hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodMesh {
    /// Unique identifier for this LOD mesh.
    pub id: LodMeshId,
    /// The base (highest detail) mesh ID.
    pub base_mesh_id: MeshId,
    /// All LOD levels from highest to lowest detail.
    pub levels: Vec<LodLevel>,
    /// Configuration used to generate this LOD.
    pub config: LodConfig,
    /// Bounding box of the full-detail mesh.
    pub bounding_box: BoundingBox3D,
    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LodMeshMetadata>,
}

impl LodMesh {
    /// Create a new empty LOD mesh.
    pub fn new(id: LodMeshId, base_mesh_id: MeshId, config: LodConfig) -> Self {
        Self {
            id,
            base_mesh_id,
            levels: Vec::new(),
            config,
            bounding_box: BoundingBox3D::default(),
            metadata: None,
        }
    }

    /// Add a LOD level.
    pub fn add_level(&mut self, level: LodLevel) {
        // Ensure levels are in order
        let expected_level = self.levels.len() as u32;
        if level.level != expected_level {
            // Re-order if needed (shouldn't happen normally)
            if level.level < expected_level {
                // Insert at correct position
                self.levels.insert(level.level as usize, level);
                return;
            }
        }
        self.levels.push(level);
    }

    /// Get the number of LOD levels.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Get a specific LOD level.
    pub fn get_level(&self, level: u32) -> Option<&LodLevel> {
        self.levels.get(level as usize)
    }

    /// Get the highest detail level (level 0).
    pub fn highest_detail(&self) -> Option<&LodLevel> {
        self.get_level(0)
    }

    /// Get the lowest detail level.
    pub fn lowest_detail(&self) -> Option<&LodLevel> {
        self.levels.last()
    }

    /// Find the appropriate LOD level for a given view distance.
    pub fn level_for_distance(&self, distance: f32, vertical_offset: f32) -> u32 {
        let effective_distance =
            distance + vertical_offset * self.config.vertical_bias.unwrap_or(1.0);

        for (i, level) in self.levels.iter().enumerate() {
            if let Some(threshold) = level.activation_distance {
                if effective_distance > threshold {
                    // Use the previous level (or 0 if first)
                    return (i as u32).saturating_sub(1);
                }
            }
        }

        // Beyond max distance, use lowest detail
        self.levels.len().saturating_sub(1) as u32
    }

    /// Get total triangle count across all levels.
    pub fn total_triangles(&self) -> usize {
        self.levels.iter().map(|l| l.triangle_count).sum()
    }

    /// Get the ratio of triangles at one level vs another.
    pub fn triangle_ratio(&self, from_level: u32, to_level: u32) -> Option<f32> {
        let from = self.get_level(from_level)?;
        let to = self.get_level(to_level)?;

        if to.triangle_count > 0 {
            Some(from.triangle_count as f32 / to.triangle_count as f32)
        } else {
            None
        }
    }

    /// Check if transitions are smooth (morphing).
    pub fn has_smooth_transitions(&self) -> bool {
        matches!(self.config.transition, LodTransition::VertexMorph)
    }
}

/// Metadata for an LOD mesh hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodMeshMetadata {
    /// Human-readable name.
    pub name: Option<String>,
    /// Source mesh identifier.
    pub source_mesh_id: MeshId,
    /// Total memory footprint estimate (bytes).
    pub memory_bytes: usize,
    /// Creation timestamp.
    pub created_at: String,
    /// LOD levels count by triangle count.
    pub level_summary: Vec<TriangleCountByLevel>,
}

impl LodMeshMetadata {
    pub fn new(source_mesh_id: MeshId) -> Self {
        Self {
            name: None,
            source_mesh_id,
            memory_bytes: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            level_summary: Vec::new(),
        }
    }
}

/// Summary of triangle counts for documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleCountByLevel {
    pub level: u32,
    pub triangles: usize,
    pub reduction_percent: f32,
}

/// A transition between two LOD levels.
/// Used for smooth morphing transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodTransitionData {
    /// Source level.
    pub from_level: u32,
    /// Target level.
    pub to_level: u32,
    /// Vertex correspondence mapping.
    /// Maps target vertex index to source vertex indices and interpolation weights.
    pub vertex_mapping: Vec<VertexCorrespondence>,
    /// Interpolation weights per source vertex.
    /// Used for morphing between levels.
    pub morph_weights: Vec<MorphWeight>,
}

/// Maps a vertex in a lower-detail mesh to vertices in a higher-detail mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexCorrespondence {
    /// The target vertex index (in lower detail mesh).
    pub target_vertex: u32,
    /// Source vertices that contribute to this target.
    pub sources: Vec<SourceVertexRef>,
}

/// Reference to a source vertex with weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceVertexRef {
    /// Source vertex index.
    pub vertex_index: u32,
    /// Weight (0.0 to 1.0).
    pub weight: f32,
}

/// Weight data for vertex morphing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphWeight {
    /// Vertex index.
    pub vertex_index: u32,
    /// Blend weight at t=0.
    pub weight_start: f32,
    /// Blend weight at t=1.
    pub weight_end: f32,
    /// Which LOD level this weight applies to.
    pub level: u32,
}

impl LodTransitionData {
    /// Create a transition between two levels.
    pub fn new(from_level: u32, to_level: u32) -> Self {
        Self {
            from_level,
            to_level,
            vertex_mapping: Vec::new(),
            morph_weights: Vec::new(),
        }
    }

    /// Add vertex correspondence.
    pub fn add_correspondence(&mut self, corr: VertexCorrespondence) {
        self.vertex_mapping.push(corr);
    }

    /// Add morph weight.
    pub fn add_morph_weight(&mut self, weight: MorphWeight) {
        self.morph_weights.push(weight);
    }
}

/// LOD level selection hint for rendering systems.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LodSelectionHint {
    /// Recommended LOD level.
    pub level: u32,
    /// Reason for selection.
    pub reason: LodSelectionReason,
    /// Confidence in the selection (0.0 to 1.0).
    pub confidence: f32,
}

/// Reason for LOD level selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LodSelectionReason {
    /// Default/highest detail.
    NearCamera,
    /// Distance-based selection.
    Distance,
    /// Performance budget.
    PerformanceBudget,
    /// Forced level (editor, etc.).
    Forced,
    /// Viewport culling.
    Viewport,
}

impl Default for LodSelectionHint {
    fn default() -> Self {
        Self {
            level: 0,
            reason: LodSelectionReason::NearCamera,
            confidence: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::mesh::MeshId;
    use super::*;

    #[test]
    fn test_lod_level_creation() {
        let level = LodLevel {
            level: 0,
            cell_size_hint: 1,
            mesh_id: MeshId::default(),
            triangle_count: 10000,
            vertex_count: 5000,
            error_metric: 0.0,
            activation_distance: Some(100.0),
            simplification_info: None,
        };

        assert_eq!(level.level, 0);
        assert_eq!(level.triangle_count, 10000);
    }

    #[test]
    fn test_lod_mesh_levels() {
        let base_id = MeshId::default();
        let mut lod = LodMesh::new(LodMeshId::default(), base_id, LodConfig::default());

        lod.add_level(LodLevel {
            level: 0,
            cell_size_hint: 1,
            mesh_id: MeshId::default(),
            triangle_count: 10000,
            vertex_count: 5000,
            error_metric: 0.0,
            activation_distance: None,
            simplification_info: None,
        });

        lod.add_level(LodLevel {
            level: 1,
            cell_size_hint: 2,
            mesh_id: MeshId::default(),
            triangle_count: 2000,
            vertex_count: 1000,
            error_metric: 0.1,
            activation_distance: Some(500.0),
            simplification_info: None,
        });

        assert_eq!(lod.level_count(), 2);
        assert_eq!(lod.highest_detail().unwrap().level, 0);
        assert_eq!(lod.lowest_detail().unwrap().level, 1);
    }

    #[test]
    fn test_level_for_distance() {
        let base_id = MeshId::default();
        let mut lod = LodMesh::new(LodMeshId::default(), base_id, LodConfig::default());

        // Add levels with activation distances
        for (i, &dist) in [100.0, 500.0, 1000.0].iter().enumerate() {
            lod.add_level(LodLevel {
                level: i as u32,
                cell_size_hint: (i + 1) as u32,
                mesh_id: MeshId::default(),
                triangle_count: 10000 / (i + 1),
                vertex_count: 5000 / (i + 1),
                error_metric: i as f32 * 0.05,
                activation_distance: Some(dist),
                simplification_info: None,
            });
        }

        // Close should use highest detail (level 0)
        let level = lod.level_for_distance(50.0, 0.0);
        assert!(level <= 2, "expected 0-2 for close distance, got {}", level);

        // Medium distance - should use level 0 (near) or 1 (medium)
        let level = lod.level_for_distance(600.0, 0.0);
        assert!(
            level <= 1,
            "expected 0 or 1 for medium distance, got {}",
            level
        );

        // Far distance
        let level = lod.level_for_distance(1500.0, 0.0);
        assert!(level <= 2, "expected 0-2 for far distance, got {}", level);
    }

    #[test]
    fn test_reduction_ratio() {
        let level = LodLevel {
            level: 1,
            cell_size_hint: 2,
            mesh_id: MeshId::default(),
            triangle_count: 2000,
            vertex_count: 1000,
            error_metric: 0.1,
            activation_distance: None,
            simplification_info: None,
        };

        // Level has 2000 triangles, full has 10000
        assert_eq!(level.reduction_ratio(10000), 0.2);

        // Zero division case
        assert_eq!(level.reduction_ratio(0), 1.0);
    }

    #[test]
    fn test_simplification_info() {
        let info = SimplificationInfo::new("quadric_error", 10000, 2000)
            .with_errors(0.05, 0.02)
            .with_time(150)
            .with_param("preserve_border", "true");

        assert_eq!(info.algorithm, "quadric_error");
        assert_eq!(info.original_triangles, 10000);
        assert_eq!(info.final_triangles, 2000);
        assert_eq!(info.max_error, 0.05);
        assert_eq!(info.simplification_time_ms, 150);
        assert_eq!(
            info.params.get("preserve_border"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_lod_transition_data() {
        let mut transition = LodTransitionData::new(0, 1);

        transition.add_correspondence(VertexCorrespondence {
            target_vertex: 0,
            sources: vec![
                SourceVertexRef {
                    vertex_index: 0,
                    weight: 0.6,
                },
                SourceVertexRef {
                    vertex_index: 1,
                    weight: 0.4,
                },
            ],
        });

        transition.add_morph_weight(MorphWeight {
            vertex_index: 0,
            weight_start: 0.0,
            weight_end: 1.0,
            level: 0,
        });

        assert_eq!(transition.from_level, 0);
        assert_eq!(transition.to_level, 1);
        assert_eq!(transition.vertex_mapping.len(), 1);
        assert_eq!(transition.morph_weights.len(), 1);
    }

    #[test]
    fn test_lod_selection_hint() {
        let hint = LodSelectionHint {
            level: 2,
            reason: LodSelectionReason::Distance,
            confidence: 0.9,
        };

        assert_eq!(hint.level, 2);
        assert!(matches!(hint.reason, LodSelectionReason::Distance));
    }
}
