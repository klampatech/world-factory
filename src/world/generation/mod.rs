//! Generation module — procedural world generation algorithms.
//!
//! This module contains algorithms for procedural generation of world geometry,
//! terrain, and related structures.
//!
//! ## Modules
//!
//! - `lloyd_relaxation` — Centroidal Voronoi diagram relaxation

pub mod geography_generator;
pub mod lloyd_relaxation;

pub use geography_generator::GeographyGenerator;
