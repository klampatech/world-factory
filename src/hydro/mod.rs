//! Hydrology Module
//!
//! Handles all water-related generation including rivers, lakes, and ocean placement.
//! Integrates with terrain/elevation system for deterministic procedural generation.
//!
//! Two representations:
//! - Grid-based rivers (`rivers.rs`) - for pixel-based terrain
//! - Polygon-based rivers (`polygon_rivers.rs`) - for Voronoi grid worlds

pub mod drainage_basin;
pub mod polygon_rivers;
pub mod rivers;

pub use drainage_basin::{
    DrainageBasinCalculator, DrainageConfig, OutletType, PolygonDrainageBasin,
};
pub use polygon_rivers::{Confluence, PolygonRiver, PolygonRiverGenerator};
pub use rivers::{DrainTarget, River, RiverConfig, RiverGenerator, RiverId};
