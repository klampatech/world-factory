//! Utility modules for World Factory

pub mod geometry;
pub mod noise;

pub use geometry::{Direction, Seed, Vec2};
pub use noise::{Rng, SimplexNoise, SimplexNoise3D};
