//! Terrain generation module for World Factory.
//!
//! This module handles all terrain-related generation including:
//! - Elevation calculation (distance from coastline)
//! - Voronoi polygon management
//! - Terrain grid and cells
//! - Biome assignment
//! - Resource types and deposits
//! - Erosion simulation
//! - Ocean detection and coastal analysis

pub mod biome;
pub mod biome_assignment;
pub mod climate_calculator;
pub mod elevation;
pub mod elevation_assignment;
pub mod elevation_grid;
pub mod erosion;
pub mod lod;
pub mod mesh;
pub mod natural_wonders;
pub mod ocean;
pub mod resource_spawner;
pub mod resource_types;
pub mod tectonic;
pub mod terrain_generator;
pub mod terrain_grid;
pub mod topology;

// Re-export main types for convenience
pub use biome::{
    AlpineBiomeConfig, BiomeColor, BiomeColorMapping, BiomeType, ClimateZone, ElevationZone,
    MoistureLevel, ResourceCategory, VegetationType,
};
pub use biome_assignment::{
    AssignmentFactor, BiomeAssignment, BiomeAssignmentMatrix, CoherenceConfig,
};
pub use climate_calculator::{
    ClimateCalculator, ClimateCalculatorConfig, PolygonClimate, WindDirection,
};
pub use elevation::{ElevationStats, Polygon, PolygonGraph};
pub use elevation_assignment::{ElevationAssigner, ElevationAssignmentResult, ElevationConfig};
pub use elevation_grid::ElevationGrid;
pub use erosion::{ErosionConfig, ErosionSimulator, ErosionStats};
pub use lod::{LodConfig, LodLevel, LodLevelSpec, LodMesh, LodMeshId, LodTransition};
pub use mesh::{BoundingBox3D, Mesh, MeshConfig, MeshFace, MeshId, MeshMetadata, MeshVertex};
pub use natural_wonders::{
    NaturalWonder, NaturalWonderSpawner, WonderBonus, WonderBonusType, WonderCategory,
    WonderIconType, WonderSpawnConfig, WonderSpawnResult, WonderSpawnStats, WonderType,
    WonderVisualProperties,
};
pub use ocean::{
    CoastalMetrics, CoastalStatistics, OceanDetectionConfig, OceanDetector, OceanZone,
};
pub use resource_spawner::{
    BoundaryEffectType, RegionResourceSpawn, ResourceSpawnConfig, ResourceSpawnStats,
    ResourceSpawner, TectonicBoundaryData,
};
pub use tectonic::{
    BoundaryEffect, ElevationModifier, TectonicResult, TectonicSimConfig, TectonicSimulator,
};
pub use terrain_generator::{TerrainConfig, TerrainGenerator, TerrainLayer};
pub use terrain_grid::{TerrainCell, TerrainGrid, CHUNK_SIZE};
pub use topology::{BorderType, PolygonEdge, PolygonTopology, PolygonTopologyMap, TopologyId};

/// Lloyd relaxation for polygon graphs.
///
/// Implements centroidal Voronoi tessellation via Lloyd's algorithm:
/// 1. Compute Voronoi diagram (using the generation::VoronoiGenerator)
/// 2. Calculate centroids of each cell
/// 3. Move seeds toward centroids
/// 4. Rebuild Voronoi diagram
/// 5. Repeat for specified iterations
pub mod lloyd_relaxation {
    use super::*;
    use crate::generation::voronoi::{self, BoundaryMode, VoronoiConfig};

    /// Perform Lloyd relaxation on a polygon graph.
    ///
    /// This creates more regular, evenly-sized Voronoi cells by iteratively
    /// moving cell centers toward their geometric centroids.
    ///
    /// # Arguments
    /// * `polygons` - The polygon graph to relax (will be rebuilt)
    /// * `iterations` - Number of relaxation iterations (typically 1-5)
    /// * `width` - Grid width in cells
    /// * `height` - Grid height in cells
    /// * `seed` - Random seed for deterministic generation
    ///
    /// # Algorithm Complexity
    /// * O(k * n * m) where k=iterations, n=seeds, m=cells
    /// * Each iteration: centroid calculation (O(m)) + Voronoi reassignment (O(n*m))
    pub fn relax(
        polygons: &mut PolygonGraph,
        iterations: usize,
        width: u32,
        height: u32,
        seed: u64,
    ) {
        if iterations == 0 {
            return;
        }

        // Determine number of seeds from polygon count
        let num_seeds = polygons.len() as u32;
        if num_seeds == 0 {
            return;
        }

        // Configure Voronoi generation with Lloyd relaxation
        let config = VoronoiConfig {
            width,
            height,
            num_seeds,
            lloyd_iterations: iterations as u32,
            boundary_mode: BoundaryMode::Torus,
            jitter: 0.5,
            blue_noise: true,
        };

        // Generate relaxed Voronoi diagram
        let graph = voronoi::generate_voronoi_graph(config, seed);

        // Replace polygon graph content
        polygons.replace_polygons(graph.into_polygons());

        log::debug!(
            "Lloyd relaxation complete: {} iterations on {}x{} grid",
            iterations,
            width,
            height
        );
    }

    /// Quick relaxation with sensible defaults.
    ///
    /// Performs 2 iterations of Lloyd relaxation, which typically produces
    /// good results with minimal computational cost.
    pub fn quick_relax(polygons: &mut PolygonGraph, width: u32, height: u32, seed: u64) {
        relax(polygons, 2, width, height, seed);
    }
}
