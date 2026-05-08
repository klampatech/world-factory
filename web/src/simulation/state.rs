//! Simulation state management
//! 
//! Defines the state types for simulation input/output and configuration

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::territory::{TerritorySystem, TerritoryClaim, FactionId, PolygonInfo};
use crate::terrain::biome_assignment::PolygonBiome;
use crate::terrain::groundwater::AquiferData;
use crate::wonders::Wonder;

/// Configuration for simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of years to simulate
    pub years: u32,
    /// Starting year (default 0 = start of recorded history)
    pub start_year: i32,
    /// Simulation seed for reproducibility (None = random)
    pub seed: Option<u64>,
    /// Enable detailed event generation
    pub detailed_events: bool,
    /// Enable figure generation
    pub generate_figures: bool,
    /// Enable artifact generation
    pub generate_artifacts: bool,
    /// Maximum population growth rate per year (as multiplier)
    pub max_growth_rate: f32,
    /// Carrying capacity multiplier
    pub carrying_capacity_multiplier: f32,
    /// World width for wonder filtering (per WOR-481)
    pub world_width: u32,
    /// World height for wonder filtering (per WOR-481)
    pub world_height: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            years: 100,
            start_year: 0,
            seed: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
            carrying_capacity_multiplier: 1.0,
            world_width: 64,
            world_height: 64,
        }
    }
}

/// Population state for a single species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Population {
    pub species_id: String,
    pub count: u32,
    pub growth_rate: f32,
    pub carrying_capacity: u32,
    pub territory_id: String,
}

impl Population {
    pub fn new(species_id: String, initial_count: u32, territory_id: String) -> Self {
        Self {
            species_id,
            count: initial_count,
            growth_rate: 1.0,
            carrying_capacity: 1000,
            territory_id,
        }
    }

    /// Apply one year of population growth
    pub fn simulate_year(&mut self, config: &SimulationConfig) {
        // Apply growth rate capped by max
        let effective_rate = self.growth_rate.min(config.max_growth_rate);
        
        // Calculate new population with carrying capacity constraint
        let growth = (self.count as f32 * (effective_rate - 1.0)) as i32;
        let capacity_room = (self.carrying_capacity as i32 - self.count as i32).max(0);
        
        self.count = ((self.count as i32 + growth).max(0) as u32)
            .min(self.carrying_capacity);
        
        // Adjust growth rate based on proximity to carrying capacity
        if self.count >= self.carrying_capacity {
            self.growth_rate = 1.0;
        } else if self.count > self.carrying_capacity / 2 {
            self.growth_rate = 1.0 + (self.growth_rate - 1.0) * 0.5;
        }
    }
}

/// Settlement state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub name: String,
    pub population: u32,
    pub territory_id: String,
    pub founding_year: u32,
}

impl Settlement {
    pub fn new(id: String, name: String, territory_id: String, founding_year: u32) -> Self {
        Self {
            id,
            name,
            population: 100,
            territory_id,
            founding_year,
        }
    }
}

/// Figure (historical character) state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Figure {
    pub id: String,
    pub name: String,
    pub figure_type: FigureType,
    pub species_id: String,
    pub birth_year: u32,
    pub death_year: Option<u32>,
    pub significance: f32,
    pub birth_settlement_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FigureType {
    Founder,
    Leader,
    Innovator,
    Prophet,
    Warrior,
    Artist,
}

impl Figure {
    pub fn new(
        id: String,
        name: String,
        figure_type: FigureType,
        species_id: String,
        birth_year: u32,
    ) -> Self {
        Self {
            id,
            name,
            figure_type,
            species_id,
            birth_year,
            death_year: None,
            significance: 1.0,
            birth_settlement_id: None,
        }
    }
}

/// Artifact state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type: String,
    pub creator_id: Option<String>,
    pub creation_year: u32,
    pub significance: f32,
}

impl Artifact {
    pub fn new(
        id: String,
        name: String,
        artifact_type: String,
        creation_year: u32,
    ) -> Self {
        Self {
            id,
            name,
            artifact_type,
            creator_id: None,
            creation_year,
            significance: 1.0,
        }
    }
}

/// Main simulation state container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub world_id: String,
    pub current_year: i32,
    pub end_year: i32,
    
    // Geographic data
    pub polygons: HashMap<u64, PolygonInfo>,
    pub elevation_map: HashMap<u64, f32>,
    
    // Simulation state
    pub populations: HashMap<String, Population>,
    pub settlements: HashMap<String, Settlement>,
    pub territories: HashMap<FactionId, TerritoryClaim>,
    
    // Generated entities
    pub figures: Vec<Figure>,
    pub artifacts: Vec<Artifact>,
    
    // Event tracking
    pub events: Vec<SimulationEvent>,
    
    // Internal tracking
    pub active_wars: HashMap<(String, String), i32>, // faction pairs -> years remaining
    pub settlement_names: HashSet<String>, // for uniqueness
    
    // Biome data for settlement placement filtering (per spec §D.2)
    pub biome_map: HashMap<u64, PolygonBiome>,
    
    // Groundwater/aquifer data for hydrological simulation
    pub aquifer_map: HashMap<u64, AquiferData>,
    
    // Wonder data with size-based filtering per WOR-481
    pub wonders: Vec<Wonder>,
}

impl SimulationState {
    pub fn new(
        world_id: String,
        start_year: i32,
        end_year: i32,
        polygons: Vec<PolygonInfo>,
        elevation_map: HashMap<u64, f32>,
    ) -> Self {
        Self {
            world_id,
            current_year: start_year,
            end_year,
            polygons: polygons.into_iter().map(|p| (p.id, p)).collect(),
            elevation_map,
            populations: HashMap::new(),
            settlements: HashMap::new(),
            territories: HashMap::new(),
            figures: Vec::new(),
            artifacts: Vec::new(),
            events: Vec::new(),
            active_wars: HashMap::new(),
            settlement_names: HashSet::new(),
            biome_map: HashMap::new(),
            aquifer_map: HashMap::new(),
            wonders: Vec::new(),
        }
    }
    
    /// Record a simulation event
    pub fn record_event(&mut self, event: SimulationEvent) {
        self.events.push(event);
    }
    
    /// Generate a unique settlement name
    pub fn generate_settlement_name(&mut self, base: &str) -> String {
        let mut name = base.to_string();
        let mut suffix = 1;
        
        while self.settlement_names.contains(&name) {
            name = format!("{} {}", base, suffix);
            suffix += 1;
        }
        
        self.settlement_names.insert(name.clone());
        name
    }
}

/// Event generated during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationEvent {
    pub year: i32,
    pub event_type: EventType,
    pub description: String,
    pub affected_entities: Vec<String>,
    pub significance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    PopulationGrowth,
    PopulationDecline,
    SettlementFounded,
    SettlementAbandoned,
    TerritoryExpansion,
    TerritoryContraction,
    WarBegan,
    WarEnded,
    FigureBorn,
    FigureDied,
    ArtifactCreated,
    SocietyTransition,
    AquiferRecharge,
    AquiferDepletion,
    WaterTableRise,
    WaterTableFall,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::PopulationGrowth => "population_growth",
            EventType::PopulationDecline => "population_decline",
            EventType::SettlementFounded => "settlement_founded",
            EventType::SettlementAbandoned => "settlement_abandoned",
            EventType::TerritoryExpansion => "territory_expansion",
            EventType::TerritoryContraction => "territory_contraction",
            EventType::WarBegan => "war_began",
            EventType::WarEnded => "war_ended",
            EventType::FigureBorn => "figure_born",
            EventType::FigureDied => "figure_died",
            EventType::ArtifactCreated => "artifact_created",
            EventType::SocietyTransition => "society_transition",
            EventType::AquiferRecharge => "aquifer_recharge",
            EventType::AquiferDepletion => "aquifer_depletion",
            EventType::WaterTableRise => "water_table_rise",
            EventType::WaterTableFall => "water_table_fall",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_growth() {
        let mut pop = Population::new("test".to_string(), 100, "territory1".to_string());
        pop.carrying_capacity = 1000;
        pop.growth_rate = 1.1;
        
        let config = SimulationConfig::default();
        pop.simulate_year(&config);
        
        assert_eq!(pop.count, 110);
    }

    #[test]
    fn test_population_carrying_capacity() {
        let mut pop = Population::new("test".to_string(), 950, "territory1".to_string());
        pop.carrying_capacity = 1000;
        pop.growth_rate = 1.1;
        
        let config = SimulationConfig::default();
        pop.simulate_year(&config);
        
        assert!(pop.count <= 1000);
    }

    #[test]
    fn test_settlement_name_uniqueness() {
        let mut state = SimulationState::new(
            "test".to_string(),
            0,
            100,
            vec![],
            HashMap::new(),
        );
        
        let name1 = state.generate_settlement_name("Rome");
        let name2 = state.generate_settlement_name("Rome");
        
        assert_eq!(name1, "Rome");
        assert_eq!(name2, "Rome 1");
    }
}
