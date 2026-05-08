//! Simulation module - Core simulation engine for world generation
//! 
//! This module provides the simulation orchestration that coordinates:
//! - TerritorySystem for territorial dynamics
//! - Population simulation
//! - Settlement development
//! - Figure generation
//! - Artifact creation

pub mod engine;
pub mod state;
pub mod handler;

pub use engine::SimulationEngine;
pub use state::{SimulationState, SimulationConfig};
pub use handler::{SimulateRequest, SimulateResponse, SimulationStats, SimulationError, handle_simulate};
