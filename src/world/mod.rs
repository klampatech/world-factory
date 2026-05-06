//! World module — planetary and geographic domain types.
//!
//! This module contains types for the physical/geographic layer of a world,
//! as distinct from the narrative/societal layer in `types.rs`.
//!
//! ## Design Principles
//!
//! 1. **Explicit units**: All geographic scalar values (temperature, precipitation)
//!    are wrapped in newtype structs that make units explicit (°C, mm/yr, etc.).
//! 2. **Validation at construction**: Type constructors validate invariants and
//!    return `Option` or `Result` rather than panicking on bad input.
//! 3. **Serializable**: All types derive `Serialize`/`Deserialize` for JSON persistence.
//! 4. **Testable invariants**: All non-trivial invariants are tested in the `#[cfg(test)]`
//!    module.
//!
//! ## Module Organization
//!
//! - `entities/` — Entity types (Planet, Geography, TectonicPlate, etc.)
//!
//! ## Relationship to `types.rs`
//!
//! The `types.rs` module defines the *narrative* layer: `World`, `Region`,
//! `Settlement`, `Person`, `HistoricalEvent`, `Timeline`. This module defines
//! the *physical* layer: `Planet`, `Geography`, `TectonicPlate`, etc.
//!
//! A `World` (narrative) references one `Planet` (physical) and zero or more
//! `Region` entities. `Geography` provides the geographic context for each `Region`.

pub mod entities;
pub mod generation;
pub use generation::geography_generator::{GeographyConfig, GeographyGenerator};

pub use entities::{
    BoundingBox, ClimateClassification, ClimateZone, DrainageBasin, DrainageError, DrainageType,
    ElevationZone, Geography, Planet, PlanetDimensions, PlanetValidationError, Point2D, Polygon,
    PolygonMesh, Precipitation, PrecipitationZone, SoilType, SubductionType, TectonicBoundary,
    TectonicBoundaryType, TectonicError, TectonicPlate, TectonicPlateType, Temperature,
    TemperatureZone, Triangle,
};
