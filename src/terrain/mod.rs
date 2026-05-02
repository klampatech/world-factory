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

pub mod elevation;
pub mod elevation_assignment;
pub mod elevation_grid;
pub mod terrain_grid;
pub mod biome;
pub mod biome_assignment;
pub mod climate_calculator;
pub mod terrain_generator;
pub mod erosion;
pub mod ocean;
pub mod resource_types;
pub mod mesh;
pub mod lod;
pub mod topology;
pub mod resource_spawner;
pub mod tectonic;
pub mod natural_wonders;

// Re-export main types for convenience
pub use elevation::{Polygon, PolygonGraph, ElevationStats};
pub use elevation_assignment::{
    ElevationAssigner, ElevationConfig, ElevationAssignmentResult
};
pub use elevation_grid::ElevationGrid;
pub use terrain_grid::{TerrainGrid, TerrainCell, CHUNK_SIZE};
pub use biome::{BiomeType, VegetationType, ClimateZone, MoistureLevel, ElevationZone, ResourceCategory, BiomeColor, BiomeColorMapping, AlpineBiomeConfig};
pub use biome_assignment::{BiomeAssignmentMatrix, BiomeAssignment, AssignmentFactor, CoherenceConfig};
pub use climate_calculator::{ClimateCalculator, ClimateCalculatorConfig, PolygonClimate, WindDirection};
pub use terrain_generator::{TerrainGenerator, TerrainConfig, TerrainLayer};
pub use erosion::{ErosionSimulator, ErosionConfig, ErosionStats};
pub use ocean::{OceanDetector, OceanDetectionConfig, OceanZone, CoastalMetrics, CoastalStatistics};
pub use mesh::{MeshId, Mesh, MeshVertex, MeshFace, MeshMetadata, MeshConfig, BoundingBox3D};
pub use lod::{LodMeshId, LodConfig, LodLevel, LodMesh, LodTransition, LodLevelSpec};
pub use topology::{TopologyId, PolygonEdge, PolygonTopology, PolygonTopologyMap, BorderType};
pub use resource_spawner::{ResourceSpawner, ResourceSpawnConfig, RegionResourceSpawn, ResourceSpawnStats, TectonicBoundaryData, BoundaryEffectType};
pub use tectonic::{TectonicSimulator, TectonicSimConfig, TectonicResult, BoundaryEffect, ElevationModifier};
pub use natural_wonders::{
    NaturalWonder, WonderType, WonderCategory, WonderBonus, WonderBonusType,
    NaturalWonderSpawner, WonderSpawnConfig, WonderSpawnResult, WonderSpawnStats,
    WonderIconType, WonderVisualProperties,
};

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
    use crate::generation::voronoi::{self, VoronoiConfig, BoundaryMode};

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
    pub fn relax(polygons: &mut PolygonGraph, iterations: usize, width: u32, height: u32, seed: u64) {
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
        
        log::debug!("Lloyd relaxation complete: {} iterations on {}x{} grid", 
                   iterations, width, height);
    }

    /// Quick relaxation with sensible defaults.
    ///
    /// Performs 2 iterations of Lloyd relaxation, which typically produces
    /// good results with minimal computational cost.
    pub fn quick_relax(polygons: &mut PolygonGraph, width: u32, height: u32, seed: u64) {
        relax(polygons, 2, width, height, seed);
    }
}