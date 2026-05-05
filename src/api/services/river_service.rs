//! River Service for API Layer
//!
//! Transforms internal river data from the hydro module into API response types.
//! Handles coordinate conversion between grid and geographic systems.

use crate::api::models::RiverView;
use crate::hydro::{River, RiverId, PolygonRiver};
use std::path::PathBuf;

/// Cell size in meters for length calculations
const CELL_SIZE_M: f64 = 1000.0;

/// River service for transforming and serving river data
#[derive(Debug, Clone, Default)]
pub struct RiverService {
    /// World storage path (for loading packages)
    world_storage_path: Option<PathBuf>,
}

impl RiverService {
    /// Create a new river service
    pub fn new() -> Self {
        Self {
            world_storage_path: None,
        }
    }

    /// Create a river service with a storage path
    pub fn with_storage(storage_path: PathBuf) -> Self {
        Self {
            world_storage_path: Some(storage_path),
        }
    }

    /// Get all rivers for a world as API views
    /// 
    /// Currently returns empty - integration with world storage pending.
    /// Once world storage is wired, this will load rivers from the world package.
    pub fn get_rivers_for_world(&self, _world_id: &str) -> Vec<RiverView> {
        // TODO: Load from world storage
        // For now, return empty - actual data requires world package integration
        Vec::new()
    }

    /// Transform grid-based rivers into API views
    /// 
    /// Use this method when you have direct access to generated river data.
    /// This is the primary method for Phase 2 integration with world generation.
    pub fn transform_grid_rivers(&self, rivers: &[River], width: usize, height: usize) -> Vec<RiverView> {
        rivers
            .iter()
            .map(|river| self.transform_grid_river(river, width, height))
            .collect()
    }

    /// Transform a grid-based River into API RiverView
    pub fn transform_grid_river(&self, river: &River, width: usize, height: usize) -> RiverView {
        let source = river.path.first();
        let mouth = river.path.last();
        
        let (source_lat, source_lon) = source
            .map(|p| (Self::grid_to_lat(p.y, height), Self::grid_to_lon(p.x, width)))
            .unwrap_or((0.0, 0.0));
        
        let (mouth_lat, mouth_lon) = mouth
            .map(|p| (Self::grid_to_lat(p.y, height), Self::grid_to_lon(p.x, width)))
            .unwrap_or((0.0, 0.0));
        
        let length_km = (river.length as f64) * CELL_SIZE_M / 1000.0;
        
        RiverView {
            id: format!("river-{}", river.id.0),
            name: Self::generate_river_name(river.id),
            length_km: Some(length_km),
            source_lat,
            source_lon,
            mouth_lat,
            mouth_lon,
            drainage_basin_id: None, // TODO: wire DrainageBasinCalculator
        }
    }

    /// Transform a polygon-based PolygonRiver into API RiverView
    /// 
    /// Note: This requires polygon center lookup which needs access to the PolygonGraph.
    /// For now, we use the polygon ID as a proxy for position.
    /// 
    /// # Arguments
    /// * `river` - The polygon river to transform
    /// * `polygon_centers` - Pre-computed (lat, lon) centers for each polygon ID
    /// * `basin_id` - Optional basin ID to assign to this river's drainage basin
    pub fn transform_polygon_river(
        &self, 
        river: &PolygonRiver, 
        polygon_centers: &[(f64, f64)],
        basin_id: Option<u32>,
    ) -> RiverView {
        let length_km = (river.length as f64) * CELL_SIZE_M / 1000.0;
        
        // Get source polygon center (if available)
        let (source_lat, source_lon) = river.source()
            .and_then(|id| {
                let idx = id as usize;
                if idx < polygon_centers.len() {
                    Some(polygon_centers[idx])
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0.0));
        
        // Get mouth polygon center (if available)
        let (mouth_lat, mouth_lon) = river.mouth()
            .and_then(|id| {
                let idx = id as usize;
                if idx < polygon_centers.len() {
                    Some(polygon_centers[idx])
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0.0));
        
        RiverView {
            id: format!("polygon-river-{}", river.id),
            name: river.name.clone().unwrap_or_else(|| Self::generate_polygon_river_name(river.id)),
            length_km: Some(length_km),
            source_lat,
            source_lon,
            mouth_lat,
            mouth_lon,
            drainage_basin_id: basin_id.map(|id| format!("basin-{}", id)),
        }
    }

    /// Convert grid Y coordinate to latitude
    /// 
    /// Grid: (0, 0) is NW corner, (height-1, height-1) is SE corner
    /// Geo: lat is 90° at north pole, -90° at south pole
    pub fn grid_to_lat(y: i32, height: usize) -> f64 {
        90.0 - ((y as f64 / height as f64) * 180.0)
    }

    /// Convert grid X coordinate to longitude  
    /// 
    /// Grid: (0, 0) is left edge, (width-1, width-1) is right edge
    /// Geo: lon is -180° at left edge, +180° at right edge
    /// Uses pixel center (x + 0.5) for proper coordinate mapping
    pub fn grid_to_lon(x: i32, width: usize) -> f64 {
        ((x as f64 + 0.5) / width as f64) * 360.0 - 180.0
    }

    /// Generate a procedural river name based on river ID
    /// 
    /// Uses a deterministic pattern based on ID to ensure consistent naming.
    /// Real names would come from world generation or naming API.
    fn generate_river_name(id: RiverId) -> String {
        // Simple ordinal naming
        let ordinal = match id.0 % 10 {
            1 if id.0 != 11 => "I",
            2 if id.0 != 12 => "II", 
            3 => "III",
            4 => "IV",
            5 => "V",
            6 => "VI",
            7 => "VII",
            8 => "VIII",
            9 => "IX",
            _ => "X",
        };
        
        let prefixes = ["River", "Stream", "Brook", "Creek", "Rivulet"];
        let prefix_idx = (id.0 as usize) % prefixes.len();
        
        format!("{} {}", prefixes[prefix_idx], ordinal)
    }

    /// Generate a name for polygon-based rivers
    fn generate_polygon_river_name(id: u32) -> String {
        let ordinal = (id % 100) + 1;
        format!("River #{}", ordinal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{Vec2, Seed};
    use crate::hydro::{RiverGenerator, RiverConfig};

    #[test]
    fn test_coordinate_conversion() {
        let service = RiverService::new();
        
        // Center of a 256x256 grid
        // Note: pixel 128 spans [128, 129), center at 127.5 → 0°
        // Pixel 128's center is at (128.5) → ~0.56°E
        assert!((RiverService::grid_to_lat(128, 256) - 0.0).abs() < 0.1);
        assert!((RiverService::grid_to_lon(128, 256) - 0.0).abs() < 1.0); // Allow 1° tolerance for center offset
        
        // NW corner - pixel 0 spans [0, 1), center 0.5 → (-179.5°, 89.9°)
        assert!((RiverService::grid_to_lat(0, 256) - 89.9).abs() < 1.0);
        assert!((RiverService::grid_to_lon(0, 256) - (-179.5)).abs() < 0.5);
        
        // SE corner - pixel 255 spans [255, 256), center 255.5 maps to ~179.5°
        // North is at y=0, so y=255 → near south pole
        assert!((RiverService::grid_to_lat(255, 256) - (-90.0)).abs() < 1.0); // Allow 1° tolerance for pixel center
        assert!((RiverService::grid_to_lon(255, 256) - 179.5).abs() < 0.5);
    }

    #[test]
    fn test_river_transformation() {
        let service = RiverService::new();
        
        // Create a simple river
        let path = vec![
            Vec2::new(100, 50),
            Vec2::new(101, 51),
            Vec2::new(102, 52),
        ];
        
        let river = River {
            id: RiverId(0),
            path,
            length: 3,
            flow_rate: 0.5,
            drains_into: crate::hydro::DrainTarget::Ocean,
            cells: vec![Vec2::new(100, 50), Vec2::new(101, 51), Vec2::new(102, 52)],
        };
        
        let view = service.transform_grid_river(&river, 256, 256);
        
        // Check ID
        assert_eq!(view.id, "river-0");
        
        // Check length (3 cells * 1000m / 1000 = 3km)
        assert!((view.length_km.unwrap() - 3.0).abs() < 0.1);
        
        // Check coordinates are in valid range
        assert!(view.source_lat >= -90.0 && view.source_lat <= 90.0);
        assert!(view.mouth_lat >= -90.0 && view.mouth_lat <= 90.0);
        assert!(view.source_lon >= -180.0 && view.source_lon <= 180.0);
        assert!(view.mouth_lon >= -180.0 && view.mouth_lon <= 180.0);
    }

    #[test]
    fn test_river_name_generation() {
        let service = RiverService::new();
        
        // Different rivers should get different names
        let river1 = RiverId(0);
        let river2 = RiverId(5);
        let river3 = RiverId(10);
        
        let name1 = RiverService::generate_river_name(river1);
        let name2 = RiverService::generate_river_name(river2);
        let name3 = RiverService::generate_river_name(river3);
        
        assert_ne!(name1, name2);
        assert_ne!(name2, name3);
    }

    #[test]
    fn test_polygon_river_transformation() {
        let service = RiverService::new();
        
        // Create polygon centers (simplified - just ID-based)
        let centers: Vec<(f64, f64)> = (0..10)
            .map(|i| (50.0 + i as f64, 10.0 + i as f64))
            .collect();
        
        let river = PolygonRiver {
            id: 5,
            name: Some("Great River".to_string()),
            path: vec![1, 2, 3, 4, 5],
            length: 5,
            volume: 0.5,
            elevation_change: 0.5,
            drains_to_ocean: true,
            confluences: vec![],
        };
        
        let view = service.transform_polygon_river(&river, &centers, Some(3));
        
        // Check name from river data
        assert_eq!(view.name, "Great River");
        
        // Check basin ID is correctly formatted
        assert_eq!(view.drainage_basin_id, Some("basin-3".to_string()));
        
        // Check coordinates from polygon centers
        assert_eq!(view.source_lat, 51.0);
        assert_eq!(view.source_lon, 11.0);
        assert_eq!(view.mouth_lat, 55.0);
        assert_eq!(view.mouth_lon, 15.0);
    }
}