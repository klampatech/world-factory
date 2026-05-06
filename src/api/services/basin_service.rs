//! Basin Service for API Layer
//!
//! Transforms internal drainage basin data from the hydro module into API response types.

use crate::api::models::DrainageBasinView;
use crate::hydro::{OutletType, PolygonDrainageBasin};

/// Basin service for transforming and serving drainage basin data
#[derive(Debug, Clone, Default)]
pub struct BasinService;

impl BasinService {
    /// Create a new basin service
    pub fn new() -> Self {
        Self
    }

    /// Transform an internal PolygonDrainageBasin into an API DrainageBasinView
    pub fn transform_basin(basin: &PolygonDrainageBasin) -> DrainageBasinView {
        DrainageBasinView {
            id: format!("basin-{}", basin.id),
            area_polygons: basin.area,
            outlet_type: match basin.outlet_type {
                OutletType::Coastal => "coastal",
                OutletType::River => "river",
                OutletType::Lake => "lake",
                OutletType::Endorheic => "endorheic",
            }
            .to_string(),
            outlet_id: basin.outlet_id,
            avg_elevation: basin.avg_elevation,
            elevation_range: basin.elevation_range,
            river_polygon_count: basin.river_polygon_count,
            polygon_ids: basin.polygon_ids.clone(),
        }
    }

    /// Transform a collection of basins into API views
    pub fn transform_basins(basins: &[PolygonDrainageBasin]) -> Vec<DrainageBasinView> {
        basins.iter().map(Self::transform_basin).collect()
    }

    /// Get the basin ID for a river (based on its mouth polygon)
    pub fn get_basin_for_river(
        rivers: &[crate::hydro::PolygonRiver],
        basin_id: Option<u32>,
    ) -> Option<String> {
        basin_id.map(|id| format!("basin-{}", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hydro::PolygonDrainageBasin;
    use crate::terrain::{Polygon, PolygonGraph};

    #[test]
    fn test_transform_basin() {
        let mut basin = PolygonDrainageBasin::new(5);
        basin.area = 42;
        basin.outlet_id = 10;
        basin.outlet_type = OutletType::Coastal;
        basin.avg_elevation = 0.65;
        basin.elevation_range = 0.45;
        basin.river_polygon_count = 8;
        basin.polygon_ids = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let view = BasinService::transform_basin(&basin);

        assert_eq!(view.id, "basin-5");
        assert_eq!(view.area_polygons, 42);
        assert_eq!(view.outlet_type, "coastal");
        assert_eq!(view.outlet_id, 10);
        assert!(view.avg_elevation > 0.0);
        assert!(view.elevation_range > 0.0);
        assert_eq!(view.river_polygon_count, 8);
        assert_eq!(view.polygon_ids.len(), 8);
    }

    #[test]
    fn test_transform_basins() {
        let mut basin1 = PolygonDrainageBasin::new(0);
        basin1.area = 10;
        basin1.outlet_type = OutletType::Coastal;

        let mut basin2 = PolygonDrainageBasin::new(1);
        basin2.area = 15;
        basin2.outlet_type = OutletType::River;

        let basins = vec![basin1, basin2];
        let views = BasinService::transform_basins(&basins);

        assert_eq!(views.len(), 2);
        assert_eq!(views[0].id, "basin-0");
        assert_eq!(views[1].id, "basin-1");
        assert_eq!(views[0].outlet_type, "coastal");
        assert_eq!(views[1].outlet_type, "river");
    }

    #[test]
    fn test_basin_service_all_outlet_types() {
        for (outlet_type, expected) in [
            (OutletType::Coastal, "coastal"),
            (OutletType::River, "river"),
            (OutletType::Lake, "lake"),
            (OutletType::Endorheic, "endorheic"),
        ] {
            let mut basin = PolygonDrainageBasin::new(0);
            basin.outlet_type = outlet_type;
            basin.area = 1;

            let view = BasinService::transform_basin(&basin);
            assert_eq!(view.outlet_type, expected);
        }
    }
}
