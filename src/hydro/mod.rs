//! Hydrology Module
//! 
//! Handles all water-related generation including rivers, lakes, and ocean placement.
//! Integrates with terrain/elevation system for deterministic procedural generation.
//! 
//! Two representations:
//! - Grid-based rivers (`rivers.rs`) - for pixel-based terrain
//! - Polygon-based rivers (`polygon_rivers.rs`) - for Voronoi grid worlds

pub mod rivers;
pub mod polygon_rivers;
pub mod drainage_basin;

pub use rivers::{River, RiverId, RiverConfig, RiverGenerator, DrainTarget};
pub use polygon_rivers::{PolygonRiver, PolygonRiverGenerator, Confluence};
pub use drainage_basin::{PolygonDrainageBasin, DrainageBasinCalculator, DrainageConfig, OutletType};
