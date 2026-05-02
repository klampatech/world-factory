//! Population Growth Service
//! 
//! Implements population dynamics for settlements and societies.
//! 
//! # Growth Formula
//! 
//! ```
//! growth_rate = base_reproduction_rate × food_surplus_factor × disease_factor
//! food_surplus_factor = min(1.0, available_food / food_requirement)
//! disease_factor = 1.0 - (population_density / carrying_capacity × 0.3)
//! population += floor(population × growth_rate × (1 - population / carrying_capacity))
//! ```
//! 
//! # Food Calculation
//! 
//! Available food is computed from FertileSoil, Fish, and Game resources
//! in the settlement's polygon and neighboring cells.
//! 
//! # Usage
//! 
//! ```rust
//! use world_factory::history::population::{PopulationGrowthService, GrowthConfig};
//! 
//! let mut service = PopulationGrowthService::new(42);
//! service.add_settlement(settlement_id, population, species_id, carrying_capacity);
//! let result = service.advance_years(100);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::history::society::{Society, SocietyRegistry, SocietyType, PopulationSample};
use crate::species::{SpeciesId, SpeciesData};
use crate::terrain::resource_types::ResourceType;

/// Configuration for population growth simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthConfig {
    /// Base reproduction rate (per year).
    /// Default: 0.015 (1.5% per decade, ~0.15% per year).
    pub base_reproduction_rate: f32,
    
    /// Food requirement per person per year.
    pub food_requirement_per_capita: f32,
    
    /// Maximum population density factor (for disease calculation).
    pub max_density_factor: f32,
    
    /// Enable population capping at carrying capacity.
    pub enable_carrying_capacity: bool,
    
    /// Enable disease effects.
    pub enable_disease: bool,
    
    /// Enable food surplus effects.
    pub enable_food_surplus: bool,
}

impl Default for GrowthConfig {
    fn default() -> Self {
        Self {
            base_reproduction_rate: 0.015,
            food_requirement_per_capita: 1.0,
            max_density_factor: 0.3,
            enable_carrying_capacity: true,
            enable_disease: true,
            enable_food_surplus: true,
        }
    }
}

/// Result of a single population tick (year advance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationTickResult {
    /// Society ID.
    pub society_id: Uuid,
    /// Population before this tick.
    pub old_population: u64,
    /// Population after this tick.
    pub new_population: u64,
    /// Net change.
    pub change: i64,
    /// Growth rate applied.
    pub growth_rate: f32,
    /// Food surplus factor (0.0-1.0+).
    pub food_surplus_factor: f32,
    /// Disease factor (0.0-1.0).
    pub disease_factor: f32,
    /// Society type transition that occurred.
    pub society_transition: Option<SocietyTransition>,
    /// Year of this tick.
    pub year: i32,
}

/// Society type transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocietyTransition {
    pub from_type: SocietyType,
    pub to_type: SocietyType,
    pub trigger_population: u64,
}

/// Result of advancing simulation years.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Population changes for each society per tick.
    pub tick_results: Vec<PopulationTickResult>,
    /// Total population change across all societies.
    pub total_population_change: i64,
    /// Number of society transitions that occurred.
    pub transition_count: usize,
    /// Summary statistics.
    pub stats: SimulationStats,
}

/// Summary statistics for a simulation run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationStats {
    pub years_elapsed: i32,
    pub societies_count: usize,
    pub total_starting_population: u64,
    pub total_ending_population: u64,
    pub bands_remaining: usize,
    pub tribes_remaining: usize,
    pub chiefdoms_remaining: usize,
    pub nations_remaining: usize,
}

/// Resource availability for a settlement's territory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoodAvailability {
    /// FertileSoil quantity.
    pub fertile_soil: f32,
    /// Fish quantity.
    pub fish: f32,
    /// Game quantity.
    pub game: f32,
    /// Total available food.
    pub total: f32,
}

impl FoodAvailability {
    /// Calculate total available food from resources.
    pub fn from_resources(resources: &HashMap<ResourceType, f32>) -> Self {
        Self {
            fertile_soil: *resources.get(&ResourceType::FertileSoil).unwrap_or(&0.0),
            fish: *resources.get(&ResourceType::Fish).unwrap_or(&0.0),
            game: *resources.get(&ResourceType::Game).unwrap_or(&0.0),
            total: 0.0,
        }
    }
    
    /// Get total available food.
    pub fn total_available(&self) -> f32 {
        self.fertile_soil + self.fish + self.game
    }
}

/// Population growth service for settlements and societies.
#[derive(Debug, Clone)]
pub struct PopulationGrowthService {
    config: GrowthConfig,
    /// Settlement states keyed by settlement UUID.
    settlements: HashMap<Uuid, SettlementState>,
    /// Society registry.
    societies: SocietyRegistry,
    /// Resource availability by settlement (from terrain).
    resources: HashMap<Uuid, FoodAvailability>,
    /// Species data for trait lookups.
    species_data: Option<SpeciesData>,
    /// Current simulation year.
    current_year: i32,
    /// RNG seed.
    seed: u64,
}

impl PopulationGrowthService {
    /// Create a new population growth service.
    pub fn new(seed: u64) -> Self {
        Self {
            config: GrowthConfig::default(),
            settlements: HashMap::new(),
            societies: SocietyRegistry::new(),
            resources: HashMap::new(),
            species_data: None,
            current_year: 0,
            seed,
        }
    }
    
    /// Create with custom configuration.
    pub fn with_config(config: GrowthConfig, seed: u64) -> Self {
        Self {
            config,
            settlements: HashMap::new(),
            societies: SocietyRegistry::new(),
            resources: HashMap::new(),
            species_data: None,
            current_year: 0,
            seed,
        }
    }
    
    /// Set species data for trait lookups.
    pub fn with_species_data(mut self, data: SpeciesData) -> Self {
        self.species_data = Some(data);
        self
    }
    
    /// Add a settlement to track.
    pub fn add_settlement(
        &mut self,
        settlement_id: Uuid,
        population: u64,
        species_id: SpeciesId,
        carrying_capacity: u64,
    ) {
        let state = SettlementState {
            settlement_id,
            population,
            species_id,
            carrying_capacity,
            society_id: None,
            food_consumed: 0.0,
            disease_rate: 0.0,
            pop_change: 0i64,
        };
        
        self.settlements.insert(settlement_id, state);
    }
    
    /// Register a society for population tracking.
    pub fn register_society(&mut self, society: Society) {
        self.societies.register(society);
    }
    
    /// Set resource availability for a settlement.
    pub fn set_resources(&mut self, settlement_id: Uuid, availability: FoodAvailability) {
        self.resources.insert(settlement_id, availability);
    }
    
    /// Set resources from a hash map (convenience method).
    pub fn set_resources_from_map(&mut self, settlement_id: Uuid, resources: HashMap<ResourceType, f32>) {
        let availability = FoodAvailability::from_resources(&resources);
        self.resources.insert(settlement_id, availability);
    }
    
    /// Get current population for a settlement.
    pub fn get_population(&self, settlement_id: Uuid) -> Option<u64> {
        self.settlements.get(&settlement_id).map(|s| s.population)
    }
    
    /// Get all settlement populations.
    pub fn get_all_populations(&self) -> &HashMap<Uuid, SettlementState> {
        &self.settlements
    }
    
    /// Get the society registry.
    pub fn societies(&self) -> &SocietyRegistry {
        &self.societies
    }
    
    /// Get mutable society registry.
    pub fn societies_mut(&mut self) -> &mut SocietyRegistry {
        &mut self.societies
    }
    
    /// Get total population across all settlements.
    pub fn total_population(&self) -> u64 {
        self.settlements.values().map(|s| s.population).sum()
    }
    
    /// Get current simulation year.
    pub fn current_year(&self) -> i32 {
        self.current_year
    }
    
    /// Advance simulation by N years.
    /// Returns results for each tick.
    pub fn advance_years(&mut self, years: i32) -> SimulationResult {
        let mut tick_results = Vec::new();
        let start_pop = self.total_population();
        let start_year = self.current_year;
        let mut transition_count = 0;
        
        // Process each settlement
        let settlement_ids: Vec<Uuid> = self.settlements.keys().cloned().collect();
        
        for id in settlement_ids {
            // Get pre-computed values to avoid nested borrows
            let mut population = {
                if let Some(s) = self.settlements.get(&id) {
                    s.population
                } else {
                    continue;
                }
            };
            let (carrying_capacity, species_id) = {
                if let Some(s) = self.settlements.get(&id) {
                    (s.carrying_capacity, s.species_id)
                } else {
                    continue;
                }
            };
            
            let (food_surplus, disease_factor) = self.calculate_factors(id, population, carrying_capacity);
            
            // Run growth simulation for all years
            for year in 0..years {
                let current_tick_year = start_year + year + 1;
                let result = self.simulate_year(
                    id,
                    population,
                    carrying_capacity,
                    species_id,
                    food_surplus,
                    disease_factor,
                    current_tick_year,
                );
                
                if let Some(tick) = result {
                    if tick.society_transition.is_some() {
                        transition_count += 1;
                    }
                    let new_pop = tick.new_population; // Extract before moving
                    tick_results.push(tick);
                    population = new_pop;
                }
            }
        }
        
        self.current_year += years;
        
        let end_pop = self.total_population();
        
        // Compute final stats
        let stats = SimulationStats {
            years_elapsed: years,
            societies_count: self.societies.societies.len(),
            total_starting_population: start_pop,
            total_ending_population: end_pop,
            bands_remaining: self.societies.by_type(SocietyType::Band).len(),
            tribes_remaining: self.societies.by_type(SocietyType::Tribe).len(),
            chiefdoms_remaining: self.societies.by_type(SocietyType::Chiefdom).len(),
            nations_remaining: self.societies.by_type(SocietyType::Nation).len(),
        };
        
        SimulationResult {
            tick_results,
            total_population_change: end_pop as i64 - start_pop as i64,
            transition_count,
            stats,
        }
    }
    
    /// Calculate food surplus and disease factors for a settlement.
    fn calculate_factors(
        &self,
        settlement_id: Uuid,
        population: u64,
        carrying_capacity: u64,
    ) -> (f32, f32) {
        // Food surplus factor
        let food_surplus = if self.config.enable_food_surplus {
            if let Some(resources) = self.resources.get(&settlement_id) {
                let available = resources.total_available();
                let required = population as f32 * self.config.food_requirement_per_capita;
                (available / required).min(2.0) // Cap at 2.0
            } else {
                1.0 // Default if no resource data
            }
        } else {
            1.0
        };
        
        // Disease factor
        let disease_factor = if self.config.enable_disease {
            let density_ratio = if carrying_capacity > 0 {
                (population as f32 / carrying_capacity as f32).min(1.0)
            } else {
                0.0
            };
            1.0 - (density_ratio * self.config.max_density_factor)
        } else {
            1.0
        };
        
        (food_surplus, disease_factor)
    }
    
    /// Simulate one year of population change.
    fn simulate_year(
        &mut self,
        settlement_id: Uuid,
        current_pop: u64,
        carrying_capacity: u64,
        species_id: SpeciesId,
        food_surplus: f32,
        disease_factor: f32,
        year: i32,
    ) -> Option<PopulationTickResult> {
        // Get species modifier
        let species_modifier = self.get_species_growth_modifier(species_id);
        
        // Calculate effective growth rate
        // growth_rate = base × food_surplus × disease × species
        let growth_rate = self.config.base_reproduction_rate 
            * food_surplus 
            * disease_factor 
            * species_modifier;
        
        // Calculate logistic growth suppression
        // (1 - population / carrying_capacity)
        let capacity_factor = if self.config.enable_carrying_capacity && carrying_capacity > 0 {
            1.0 - (current_pop as f32 / carrying_capacity as f32)
        } else {
            1.0
        };
        
        // Calculate population change
        // population += floor(population × growth_rate × (1 - population / carrying_capacity))
        let effective_growth = growth_rate * capacity_factor;
        let pop_change = (current_pop as f32 * effective_growth).floor() as i64;
        
        let new_population = (current_pop as i64 + pop_change).max(1) as u64;
        
        // Check for society type transition
        let new_society_type = SocietyType::from_population(new_population);
        let old_society_type = SocietyType::from_population(current_pop);
        
        let society_transition = if new_society_type != old_society_type {
            Some(SocietyTransition {
                from_type: old_society_type,
                to_type: new_society_type,
                trigger_population: new_population,
            })
        } else {
            None
        };
        
        // Update settlement state
        if let Some(state) = self.settlements.get_mut(&settlement_id) {
            state.population = new_population;
            state.pop_change = pop_change;
            state.food_consumed = current_pop as f32 * self.config.food_requirement_per_capita;
            state.disease_rate = 1.0 - disease_factor;
        }
        
        // Update society if registered
        if let Some(society) = self.societies.get_mut(settlement_id) {
            if society_transition.is_some() {
                society.check_transition();
            }
            society.record_population(year, new_population);
        }
        
        Some(PopulationTickResult {
            society_id: settlement_id,
            old_population: current_pop,
            new_population,
            change: pop_change,
            growth_rate: effective_growth,
            food_surplus_factor: food_surplus,
            disease_factor,
            society_transition,
            year,
        })
    }
    
    /// Get species growth rate modifier.
    fn get_species_growth_modifier(&self, species_id: SpeciesId) -> f32 {
        self.species_data
            .as_ref()
            .and_then(|data| data.get(species_id))
            .map(|species| species.trait_growth_modifier())
            .unwrap_or(1.0)
    }
}

/// Internal state for a settlement in population simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettlementState {
    settlement_id: Uuid,
    population: u64,
    species_id: SpeciesId,
    carrying_capacity: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    society_id: Option<Uuid>,
    food_consumed: f32,
    disease_rate: f32,
    pop_change: i64,
}

/// Food availability helper for terrain integration.
#[derive(Debug, Clone, Default)]
pub struct SettlementFoodCalculator;

impl SettlementFoodCalculator {
    /// Calculate available food for a settlement from nearby terrain resources.
    /// 
    /// This should be called with the terrain grid and resource data
    /// during world generation to pre-compute food availability.
    pub fn calculate_for_region(
        resources: &HashMap<ResourceType, f32>,
        neighbor_resources: &[(Uuid, HashMap<ResourceType, f32>)],
    ) -> FoodAvailability {
        let mut total = FoodAvailability::default();
        
        // Add own resources
        let own = FoodAvailability::from_resources(resources);
        total.fertile_soil += own.fertile_soil;
        total.fish += own.fish;
        total.game += own.game;
        
        // Add neighbor resources (at 50% weight)
        for (_, neighbor_res) in neighbor_resources {
            let neighbor = FoodAvailability::from_resources(neighbor_res);
            total.fertile_soil += neighbor.fertile_soil * 0.5;
            total.fish += neighbor.fish * 0.5;
            total.game += neighbor.game * 0.5;
        }
        
        total.total = total.total_available();
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_society_type_from_population() {
        assert_eq!(SocietyType::from_population(10), SocietyType::Band);
        assert_eq!(SocietyType::from_population(50), SocietyType::Tribe);
        assert_eq!(SocietyType::from_population(500), SocietyType::Chiefdom);
        assert_eq!(SocietyType::from_population(5000), SocietyType::Nation);
    }
    
    #[test]
    fn test_food_availability() {
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 100.0);
        resources.insert(ResourceType::Fish, 50.0);
        resources.insert(ResourceType::Game, 25.0);
        
        let availability = FoodAvailability::from_resources(&resources);
        
        assert_eq!(availability.fertile_soil, 100.0);
        assert_eq!(availability.fish, 50.0);
        assert_eq!(availability.game, 25.0);
        assert_eq!(availability.total_available(), 175.0);
    }
    
    #[test]
    fn test_population_growth() {
        let mut service = PopulationGrowthService::new(42);
        
        let settlement_id = Uuid::new_v4();
        service.add_settlement(settlement_id, 100, SpeciesId::HUMAN, 10_000);
        
        // Set good food resources
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 500.0);
        resources.insert(ResourceType::Fish, 200.0);
        resources.insert(ResourceType::Game, 100.0);
        service.set_resources_from_map(settlement_id, resources);
        
        // Simulate 10 years
        let result = service.advance_years(10);
        
        assert_eq!(result.stats.years_elapsed, 10);
        assert!(result.total_population_change > 0, "Population should grow");
    }
    
    #[test]
    fn test_carrying_capacity() {
        let mut service = PopulationGrowthService::new(42);
        
        let settlement_id = Uuid::new_v4();
        // Start at 80% of carrying capacity
        service.add_settlement(settlement_id, 800, SpeciesId::HUMAN, 1000);
        
        // Set low food resources (simulate scarcity)
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 100.0);
        resources.insert(ResourceType::Fish, 50.0);
        resources.insert(ResourceType::Game, 10.0);
        service.set_resources_from_map(settlement_id, resources);
        
        // Simulate 10 years - growth should be suppressed
        let result = service.advance_years(10);
        
        // Population change should be smaller due to food scarcity
        let change_rate = result.total_population_change as f32 / 800.0;
        assert!(change_rate < 0.05, "Growth should be minimal with low food");
    }
    
    #[test]
    fn test_society_transition() {
        let mut service = PopulationGrowthService::new(42);
        
        let settlement_id = Uuid::new_v4();
        service.add_settlement(settlement_id, 45, SpeciesId::HUMAN, 10_000);
        
        // Set abundant food for fast growth
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 1000.0);
        resources.insert(ResourceType::Fish, 500.0);
        resources.insert(ResourceType::Game, 200.0);
        service.set_resources_from_map(settlement_id, resources);
        
        // Simulate enough years to reach Tribe threshold (50)
        let result = service.advance_years(20);
        
        assert!(result.transition_count >= 1, "Should transition from Band to Tribe");
    }
    
    #[test]
    fn test_logistic_growth_suppression() {
        let mut service = PopulationGrowthService::new(42);
        
        let settlement_id = Uuid::new_v4();
        // Start at 95% of carrying capacity
        service.add_settlement(settlement_id, 950, SpeciesId::HUMAN, 1000);
        
        // Set abundant food
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 5000.0);
        resources.insert(ResourceType::Fish, 2000.0);
        resources.insert(ResourceType::Game, 1000.0);
        service.set_resources_from_map(settlement_id, resources);
        
        let result = service.advance_years(100);
        
        // Growth should be heavily suppressed near capacity
        assert!(result.total_population_change < 100, "Growth should be suppressed");
    }
    
    #[test]
    fn test_disease_factor() {
        let mut service = PopulationGrowthService::new(42);
        
        let settlement_id = Uuid::new_v4();
        // High population density
        service.add_settlement(settlement_id, 900, SpeciesId::HUMAN, 1000);
        
        // Set abundant food
        let mut resources = HashMap::new();
        resources.insert(ResourceType::FertileSoil, 5000.0);
        resources.insert(ResourceType::Fish, 2000.0);
        resources.insert(ResourceType::Game, 1000.0);
        service.set_resources_from_map(settlement_id, resources);
        
        let result = service.advance_years(10);
        
        // With 90% density, disease factor should be ~0.73
        // Growth should be noticeably reduced
        assert!(result.total_population_change < 50, "Disease should reduce growth");
    }
    
    #[test]
    fn test_total_population() {
        let mut service = PopulationGrowthService::new(42);
        
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        
        service.add_settlement(id1, 100, SpeciesId::HUMAN, 10_000);
        service.add_settlement(id2, 200, SpeciesId::ELF, 10_000);
        
        assert_eq!(service.total_population(), 300);
    }
    
    #[test]
    fn test_settlement_food_calculator() {
        let mut own_resources = HashMap::new();
        own_resources.insert(ResourceType::FertileSoil, 100.0);
        own_resources.insert(ResourceType::Fish, 50.0);
        
        let neighbor_resources = vec![
            (Uuid::new_v4(), {
                let mut r = HashMap::new();
                r.insert(ResourceType::Game, 20.0);
                r
            }),
        ];
        
        let availability = SettlementFoodCalculator::calculate_for_region(
            &own_resources,
            &neighbor_resources,
        );
        
        // Own: 100 fertile + 50 fish = 150
        // Neighbor: 20 game × 0.5 = 10
        // Total: 160
        assert_eq!(availability.total_available(), 160.0);
    }
}
