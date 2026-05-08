//! Terrain module - Topographic and environmental data
//! 
//! This module provides terrain-related systems including:
//! - Biome assignment
//! - Groundwater/aquifer systems

pub mod biome_assignment;
pub mod groundwater;

pub use biome_assignment::{BiomeType, BiomeAssignmentSystem, PolygonBiome};
pub use groundwater::{GroundwaterSystem, AquiferData, AquiferType, AquiferRecharge};
