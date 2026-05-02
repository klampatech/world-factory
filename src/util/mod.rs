//! Utility modules for World Factory

pub mod noise;
pub mod geometry;

pub use noise::{SimplexNoise, SimplexNoise3D, Rng};
pub use geometry::{Vec2, Direction, Seed};
