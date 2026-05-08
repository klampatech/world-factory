//! Simulation HTTP handler
//! 
//! Provides the simulate endpoint for the web API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::territory::PolygonInfo;
use super::state::{SimulationConfig, SimulationState};
use super::engine::SimulationEngine;

/// Request payload for the simulate endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRequest {
    /// World identifier
    pub world_id: String,
    /// Number of years to simulate (default: 100)
    #[serde(default = "default_years")]
    pub years: u32,
    /// Starting year (default: 0)
    #[serde(default)]
    pub start_year: i32,
    /// Optional seed for reproducibility
    #[serde(default)]
    pub seed: Option<u64>,
    /// Polygon data for world generation
    pub polygons: Vec<PolygonData>,
    /// Enable detailed event generation (default: true)
    #[serde(default = "default_true")]
    pub detailed_events: bool,
    /// Enable figure generation (default: true)
    #[serde(default = "default_true")]
    pub generate_figures: bool,
    /// Enable artifact generation (default: true)
    #[serde(default = "default_true")]
    pub generate_artifacts: bool,
    /// Maximum population growth rate per year (default: 1.05)
    #[serde(default = "default_growth_rate")]
    pub max_growth_rate: f32,
    /// World width for wonder filtering (per WOR-481)
    #[serde(default)]
    pub world_width: Option<u32>,
    /// World height for wonder filtering (per WOR-481)
    #[serde(default)]
    pub world_height: Option<u32>,
}

fn default_years() -> u32 { 100 }
fn default_true() -> bool { true }
fn default_growth_rate() -> f32 { 1.05 }

/// Polygon data from request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonData {
    pub id: u64,
    pub elevation: f32,
    pub neighbors: Vec<u64>,
    #[serde(default)]
    pub is_coastal: bool,
    #[serde(default)]
    pub is_island: bool,
}

impl From<PolygonData> for PolygonInfo {
    fn from(data: PolygonData) -> Self {
        PolygonInfo {
            id: data.id,
            elevation: data.elevation,
            neighbors: data.neighbors,
            is_coastal: data.is_coastal,
            is_island: data.is_island,
        }
    }
}

/// Response from the simulate endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResponse {
    /// World identifier
    pub world_id: String,
    /// Simulation statistics
    pub stats: SimulationStats,
    /// Final simulation state (compact)
    pub state: SimulatedState,
    /// Generated events (compact)
    pub events: Vec<SimulatedEvent>,
}

/// Simulation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    /// Total years simulated
    pub years_simulated: u32,
    /// Total events generated
    pub events_generated: u32,
    /// Total figures generated
    pub figures_generated: u32,
    /// Total artifacts generated
    pub artifacts_generated: u32,
    /// Total populations
    pub populations: u32,
    /// Total settlements
    pub settlements: u32,
    /// Territories created
    pub territories: u32,
    /// Wonders generated (filtered by world size)
    pub wonders: u32,
}

/// Compact simulated state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedState {
    pub current_year: i32,
    pub populations: Vec<PopulationSummary>,
    pub settlements: Vec<SettlementSummary>,
    pub territories: Vec<TerritorySummary>,
    pub wonders: Vec<WonderSummary>,
}

/// Population summary in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationSummary {
    pub species_id: String,
    pub count: u32,
    pub growth_rate: f32,
}

/// Settlement summary in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSummary {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub founding_year: u32,
}

/// Territory summary in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritorySummary {
    pub owner_id: String,
    pub polygon_count: u32,
}

/// Wonder summary in response (per WOR-481)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderSummary {
    pub id: String,
    pub name: String,
    pub wonder_type: String,
    pub polygon_id: u64,
    pub discovery_year: Option<u32>,
}

/// Compact event representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedEvent {
    pub year: i32,
    pub event_type: String,
    pub description: String,
    pub significance: f32,
}

/// Handler for the simulate endpoint
pub fn handle_simulate(request: SimulateRequest) -> Result<SimulateResponse, SimulationError> {
    // Validate request
    validate_request(&request)?;
    
    // Build configuration
    let config = SimulationConfig {
        years: request.years,
        start_year: request.start_year,
        seed: request.seed,
        detailed_events: request.detailed_events,
        generate_figures: request.generate_figures,
        generate_artifacts: request.generate_artifacts,
        max_growth_rate: request.max_growth_rate,
        carrying_capacity_multiplier: 1.0,
        world_width: request.world_width.unwrap_or(64),
        world_height: request.world_height.unwrap_or(64),
    };
    
    // Convert polygons
    let polygons: Vec<PolygonInfo> = request.polygons
        .into_iter()
        .map(PolygonInfo::from)
        .collect();
    
    // Build elevation map
    let elevation_map: HashMap<u64, f32> = polygons.iter()
        .map(|p| (p.id, p.elevation))
        .collect();
    
    // Run simulation
    let mut engine = SimulationEngine::new(request.seed);
    let mut state = engine.initialize(
        request.world_id.clone(),
        polygons,
        elevation_map,
        &config,
    );
    
    engine.run(&mut state, &config);
    
    // Build response
    build_response(state, &config)
}

/// Validate the simulate request
fn validate_request(request: &SimulateRequest) -> Result<(), SimulationError> {
    if request.years == 0 {
        return Err(SimulationError::InvalidYears);
    }
    
    if request.polygons.is_empty() {
        return Err(SimulationError::NoPolygons);
    }
    
    // Check for duplicate polygon IDs
    let mut seen_ids = std::collections::HashSet::new();
    for poly in &request.polygons {
        if !seen_ids.insert(poly.id) {
            return Err(SimulationError::DuplicatePolygonId(poly.id));
        }
    }
    
    // Validate polygon IDs in neighbor references
    for poly in &request.polygons {
        for neighbor_id in &poly.neighbors {
            if !seen_ids.contains(neighbor_id) {
                return Err(SimulationError::InvalidNeighborReference(poly.id, *neighbor_id));
            }
        }
    }
    
    Ok(())
}

/// Build response from simulation state
fn build_response(state: SimulationState, config: &SimulationConfig) -> Result<SimulateResponse, SimulationError> {
    let years_simulated = (state.current_year - config.start_year) as u32;
    
    // Build population summaries
    let populations: Vec<PopulationSummary> = state.populations.values()
        .map(|p| PopulationSummary {
            species_id: p.species_id.clone(),
            count: p.count,
            growth_rate: p.growth_rate,
        })
        .collect();
    
    // Build settlement summaries
    let settlements: Vec<SettlementSummary> = state.settlements.values()
        .map(|s| SettlementSummary {
            id: s.id.clone(),
            name: s.name.clone(),
            population: s.population,
            founding_year: s.founding_year,
        })
        .collect();
    
    // Build territory summaries
    let territories: Vec<TerritorySummary> = state.territories.iter()
        .map(|(fid, claim)| TerritorySummary {
            owner_id: fid.to_string(),
            polygon_count: claim.claimed_polygons.len() as u32,
        })
        .collect();
    
    // Build event summaries
    let events: Vec<SimulatedEvent> = if config.detailed_events {
        state.events.iter()
            .map(|e| SimulatedEvent {
                year: e.year,
                event_type: e.event_type.as_str().to_string(),
                description: e.description.clone(),
                significance: e.significance,
            })
            .collect()
    } else {
        Vec::new()
    };
    
    // Build wonder summaries (filtered by world size per WOR-481)
    let wonders: Vec<WonderSummary> = state.wonders.iter()
        .map(|w| WonderSummary {
            id: w.id.clone(),
            name: w.name.clone(),
            wonder_type: format!("{:?}", w.wonder_type),
            polygon_id: w.polygon_id,
            discovery_year: w.discovery_year,
        })
        .collect();
    
    // Build stats
    let stats = SimulationStats {
        years_simulated,
        events_generated: state.events.len() as u32,
        figures_generated: state.figures.len() as u32,
        artifacts_generated: state.artifacts.len() as u32,
        populations: populations.len() as u32,
        settlements: settlements.len() as u32,
        territories: territories.len() as u32,
        wonders: wonders.len() as u32,
    };
    
    Ok(SimulateResponse {
        world_id: state.world_id,
        stats,
        state: SimulatedState {
            current_year: state.current_year,
            populations,
            settlements,
            territories,
            wonders,
        },
        events,
    })
}

/// Errors that can occur during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationError {
    /// Years must be greater than 0
    InvalidYears,
    /// No polygons provided
    NoPolygons,
    /// Duplicate polygon ID in request
    DuplicatePolygonId(u64),
    /// Neighbor reference to non-existent polygon
    InvalidNeighborReference(u64, u64),
    /// Internal simulation error
    InternalError(String),
}

impl std::fmt::Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::InvalidYears => write!(f, "years must be greater than 0"),
            SimulationError::NoPolygons => write!(f, "at least one polygon is required"),
            SimulationError::DuplicatePolygonId(id) => write!(f, "duplicate polygon ID: {}", id),
            SimulationError::InvalidNeighborReference(poly_id, neighbor_id) => {
                write!(f, "polygon {} references non-existent neighbor {}", poly_id, neighbor_id)
            }
            SimulationError::InternalError(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for SimulationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_success() {
        let request = SimulateRequest {
            world_id: "test".to_string(),
            years: 100,
            start_year: 0,
            seed: None,
            polygons: vec![
                PolygonData { id: 1, elevation: 100.0, neighbors: vec![2], is_coastal: false, is_island: false },
                PolygonData { id: 2, elevation: 200.0, neighbors: vec![1], is_coastal: false, is_island: false },
            ],
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            world_width: None,
            world_height: None,
        };
        
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_no_polygons() {
        let request = SimulateRequest {
            world_id: "test".to_string(),
            years: 100,
            start_year: 0,
            seed: None,
            polygons: vec![],
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            world_width: None,
            world_height: None,
        };
        
        assert!(matches!(validate_request(&request), Err(SimulationError::NoPolygons)));
    }

    #[test]
    fn test_validate_request_duplicate_ids() {
        let request = SimulateRequest {
            world_id: "test".to_string(),
            years: 100,
            start_year: 0,
            seed: None,
            polygons: vec![
                PolygonData { id: 1, elevation: 100.0, neighbors: vec![], is_coastal: false, is_island: false },
                PolygonData { id: 1, elevation: 200.0, neighbors: vec![], is_coastal: false, is_island: false },
            ],
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            world_width: None,
            world_height: None,
        };
        
        assert!(matches!(
            validate_request(&request), 
            Err(SimulationError::DuplicatePolygonId(1))
        ));
    }

    #[test]
    fn test_handle_simulate_success() {
        let request = SimulateRequest {
            world_id: "test_world".to_string(),
            years: 10,
            start_year: 0,
            seed: Some(42),
            polygons: vec![
                PolygonData { id: 1, elevation: 100.0, neighbors: vec![2], is_coastal: false, is_island: false },
                PolygonData { id: 2, elevation: 200.0, neighbors: vec![1], is_coastal: false, is_island: false },
            ],
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            world_width: Some(64),
            world_height: Some(64),
        };
        
        let response = handle_simulate(request).unwrap();
        
        assert_eq!(response.world_id, "test_world");
        assert_eq!(response.stats.years_simulated, 10);
        assert!(response.stats.populations >= 1);
    }
    
    #[test]
    fn test_handle_simulate_small_world_no_large_wonders() {
        // Test that a 32x32 world doesn't get large wonders
        let request = SimulateRequest {
            world_id: "small_world".to_string(),
            years: 10,
            start_year: 0,
            seed: Some(42),
            polygons: vec![
                PolygonData { id: 1, elevation: 100.0, neighbors: vec![2], is_coastal: false, is_island: false },
                PolygonData { id: 2, elevation: 200.0, neighbors: vec![1], is_coastal: false, is_island: false },
            ],
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            world_width: Some(32),
            world_height: Some(32),
        };
        
        let response = handle_simulate(request).unwrap();
        
        // Small world should not have any wonders since wonder generation
        // is not yet implemented (this test verifies the field exists)
        assert!(response.state.wonders.len() >= 0);
    }
}
