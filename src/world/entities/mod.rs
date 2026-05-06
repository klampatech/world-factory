//! Entities module — geographic and planetary entity types.
//!
//! This module contains the core domain types for planetary bodies and their
//! physical geography. It is organized around the `planet.rs` module which
//! defines the `Planet` aggregate and its constituent types.

pub mod planet;
pub mod polygon;

// Planet & Geography Types (WOR-8)
pub use planet::{
    ClimateClassification, ClimateZone, DrainageBasin, DrainageError, DrainageType, ElevationZone,
    Geography, Planet, PlanetDimensions, PlanetValidationError, Precipitation, PrecipitationZone,
    SoilType, SubductionType, TectonicBoundary, TectonicBoundaryType, TectonicError, TectonicPlate,
    TectonicPlateType, Temperature, TemperatureZone,
};

// Polygon & Mesh Types (WOR-7)
pub use polygon::{BoundingBox, Point2D, Polygon, PolygonMesh, Triangle};
