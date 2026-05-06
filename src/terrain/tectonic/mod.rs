//! Tectonic Plate Simulation Module
//!
//! Procedural simulation of tectonic plate movement, boundary formation,
//! and orographic (mountain-building) effects on terrain elevation.
//!
//! ## Algorithm Overview
//!
//! 1. **Plate Generation**: Divide the world into N plates using Voronoi
//!    tessellation with continental/oceanic type assignment.
//!
//! 2. **Boundary Classification**: Identify boundary regions between plates
//!    and classify them as divergent, convergent, or transform.
//!
//! 3. **Orographic Effects**: Apply elevation changes based on:
//!    - Convergent boundaries: crustal shortening → mountains
//!    - Divergent boundaries: crustal thinning → rifts/valleys
//!    - Transform boundaries: minimal vertical displacement
//!
//! 4. **Output**: Generate `TectonicPlate` and `TectonicBoundary` entities
//!    that can be stored in the `Planet` aggregate.

mod elevation_effects;
mod plate_generator;
mod simulation;

pub use elevation_effects::{BoundaryEffect, ElevationModifier};
pub use plate_generator::{PlateBuilder, PlateCellAllocator};
pub use simulation::{TectonicResult, TectonicSimConfig, TectonicSimulator};
