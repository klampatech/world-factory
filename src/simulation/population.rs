//! Population Growth Model
//! 
//! Simulates population dynamics for settlements and regions over time.
//! 
//! # Growth Rate Calculation
//! 
//! Base growth rate is 1.5% per decade, modified by:
//! - **Species traits**: Adaptable +10%, Industrious +15%, etc.
//! - **Environment**: Biome carrying capacity, natural wonder bonuses
//! - **Settlement type**: Larger settlements have different growth dynamics
//! 
//! # Carrying Capacity
//! 
//! Each region has a carrying capacity based on:
//! - Biome agricultural potential
//! - Resource availability
//! - Geographic constraints (elevation, water access)
//! 
//! Growth slows as population approaches carrying capacity.
//! 
//! # Society Type Transitions
//! 
//! Settlements can transition between society types as population grows:
//! - Tribe: 50-500 (default)
//! - Chiefdom: 500-5000
//! - Nation: 5000+
//! 
//! Each transition can trigger events (founding, cultural shifts, etc.)
//! 
//! # Usage
//! 
//! ```rust
//! use world_factory::simulation::PopulationModel;
//! 
//! let mut model = PopulationModel::new(42);
//! 
//! // Add a settlement
//! let settlement_id = Uuid::new_v4();
//! model.add_settlement(settlement_id, 100, SpeciesId::Human, BiomeType::TemperateGrassland);
//! 
//! // Advance 10 years
//! model.advance_years(10);
//! 
//! // Get current population
//! let pop = model.get_population(settlement_id).unwrap();
//! ```

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::species::SpeciesId;
use crate::terrain::biome::BiomeType;
use crate::types::Settlement;
use crate::events::effect::EventEffect;
use crate::terrain::natural_wonders::NaturalWonder;

/// Disease event affecting a settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseOutbreak {
    /// Settlement affected.
    pub settlement_id: Uuid,
    /// Type of disease.
    pub disease_type: DiseaseType,
    /// Mortality rate (0.0 - 1.0).
    pub mortality_rate: f64,
    /// Duration in years.
    pub duration_years: i32,
    /// Year the outbreak started.
    pub start_year: i32,
}

impl DiseaseOutbreak {
    /// Calculate population loss from this outbreak.
    pub fn calculate_loss(&self, current_pop: u64) -> u64 {
        ((current_pop as f64) * self.mortality_rate) as u64
    }
}

/// Types of disease that can affect settlements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiseaseType {
    /// Common illness, low mortality.
    CommonCold,
    /// Seasonal flu-like illness.
    SeasonalFlu,
    /// Water-borne disease (cholera, dysentery).
    Waterborne,
    /// Food-borne illness.
    FoodPoisoning,
    /// Local plague, moderate mortality.
    LocalPlague,
    /// Regional pandemic, high mortality.
    Pandemic,
    /// Fungal infection (ergotism, etc.).
    Fungal,
    /// Magical disease (settings with magic).
    MagicalPlague,
}

impl DiseaseType {
    /// Get base mortality rate and duration for this disease type.
    pub fn base_severity(&self) -> (f64, i32) {
        match self {
            DiseaseType::CommonCold => (0.001, 1),
            DiseaseType::SeasonalFlu => (0.01, 1),
            DiseaseType::Waterborne => (0.05, 2),
            DiseaseType::FoodPoisoning => (0.02, 1),
            DiseaseType::LocalPlague => (0.15, 3),
            DiseaseType::Pandemic => (0.30, 5),
            DiseaseType::Fungal => (0.03, 2),
            DiseaseType::MagicalPlague => (0.50, 4),
        }
    }
    
    /// Check if this disease can spread to neighboring settlements.
    pub fn is_contagious(&self) -> bool {
        matches!(self, DiseaseType::SeasonalFlu | DiseaseType::LocalPlague | DiseaseType::Pandemic)
    }
    
    /// Get biome suitability for this disease.
    pub fn biome_suitability(&self, biome: BiomeType) -> f64 {
        match self {
            DiseaseType::CommonCold => 1.0,
            DiseaseType::SeasonalFlu => 1.0,
            DiseaseType::Waterborne => {
                match biome {
                    BiomeType::CoastalWetland | BiomeType::Mangrove | BiomeType::ToxicSwamp => 2.0,
                    BiomeType::TropicalRainforest | BiomeType::TropicalSeasonalForest => 1.5,
                    _ => 1.0,
                }
            },
            DiseaseType::FoodPoisoning => 1.0,
            DiseaseType::LocalPlague => {
                match biome {
                    BiomeType::TemperateDeciduousForest | BiomeType::TemperateGrassland => 1.5,
                    BiomeType::TemperateSteppe => 1.2,
                    _ => 1.0,
                }
            },
            DiseaseType::Pandemic => {
                match biome {
                    BiomeType::TemperateGrassland | BiomeType::TemperateDeciduousForest => 1.8,
                    BiomeType::TropicalSavanna => 1.5,
                    _ => 1.0,
                }
            },
            DiseaseType::Fungal => {
                match biome {
                    BiomeType::TropicalRainforest | BiomeType::BioluminescentOcean => 2.0,
                    BiomeType::TemperateRainforest | BiomeType::BorealForest => 1.5,
                    BiomeType::MagicalForest => 1.3,
                    _ => 1.0,
                }
            },
            DiseaseType::MagicalPlague => 1.0, // Magic can strike anywhere
        }
    }
}

/// Natural disaster types affecting settlements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisasterType {
    /// Earthquake.
    Earthquake,
    /// Volcanic eruption.
    VolcanicEruption,
    /// Flood.
    Flood,
    /// Drought.
    Drought,
    /// Hurricane/typhoon.
    Hurricane,
    /// Wildfire.
    Wildfire,
    /// Famine (food shortage).
    Famine,
    /// War/conflict.
    War,
    /// Locust swarm.
    LocustSwarm,
    /// Tsunami.
    Tsunami,
    /// Blizzard/harsh winter.
    Blizzard,
    /// Magical disaster.
    MagicalCatastrophe,
}

impl DisasterType {
    /// Get base population loss percentage and duration for this disaster type.
    pub fn base_severity(&self) -> (f64, i32) {
        match self {
            DisasterType::Earthquake => (0.10, 1),
            DisasterType::VolcanicEruption => (0.25, 2),
            DisasterType::Flood => (0.15, 1),
            DisasterType::Drought => (0.20, 5),
            DisasterType::Hurricane => (0.10, 1),
            DisasterType::Wildfire => (0.08, 1),
            DisasterType::Famine => (0.30, 3),
            DisasterType::War => (0.25, 5),
            DisasterType::LocustSwarm => (0.05, 2),
            DisasterType::Tsunami => (0.20, 1),
            DisasterType::Blizzard => (0.05, 1),
            DisasterType::MagicalCatastrophe => (0.50, 3),
        }
    }
    
    /// Get the biome types where this disaster is most likely to occur.
    pub fn common_biomes(&self) -> Vec<BiomeType> {
        match self {
            DisasterType::Earthquake => vec![BiomeType::VolcanicLandscape, BiomeType::BorealTaiga],
            DisasterType::VolcanicEruption => vec![BiomeType::VolcanicLandscape],
            DisasterType::Flood => vec![BiomeType::CoastalWetland, BiomeType::TemperateDeciduousForest],
            DisasterType::Drought => vec![BiomeType::HotDesert, BiomeType::TemperateDesert, BiomeType::SemiAridSteppe],
            DisasterType::Hurricane => vec![BiomeType::TropicalSavanna, BiomeType::CoastalWetland],
            DisasterType::Wildfire => vec![BiomeType::BorealForest, BiomeType::TemperateDeciduousForest],
            DisasterType::Famine => vec![], // Affects all biomes
            DisasterType::War => vec![], // Human-caused, any biome
            DisasterType::LocustSwarm => vec![BiomeType::TemperateSteppe, BiomeType::SemiAridSteppe],
            DisasterType::Tsunami => vec![], // Coastal, triggered by events
            DisasterType::Blizzard => vec![BiomeType::Arctic, BiomeType::Tundra, BiomeType::BorealTaiga],
            DisasterType::MagicalCatastrophe => vec![], // Any biome
        }
    }
    
    /// Check if this disaster affects food production.
    pub fn affects_food(&self) -> bool {
        matches!(self, DisasterType::Drought | DisasterType::Famine | DisasterType::LocustSwarm | 
                      DisasterType::Flood | DisasterType::Blizzard | DisasterType::Wildfire)
    }
}

/// Configuration for population simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    /// Base annual growth rate (decimal, e.g., 0.0015 = 0.15% per year).
    /// Default: 0.0015 (1.5% per decade).
    pub base_growth_rate: f64,
    
    /// Minimum population for a viable settlement.
    pub min_population: u64,
    
    /// Maximum population multiplier from natural wonders.
    pub max_wonder_bonus: f64,
    
    /// Carrying capacity multiplier (applies to biome base capacity).
    pub carrying_capacity_multiplier: f64,
    
    /// Enable society type transitions.
    pub enable_society_transitions: bool,
    
    /// Enable resource depletion effects.
    pub enable_resource_depletion: bool,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            base_growth_rate: 0.0015,
            min_population: 10,
            max_wonder_bonus: 2.0,
            carrying_capacity_multiplier: 1.0,
            enable_society_transitions: true,
            enable_resource_depletion: true,
        }
    }
}

/// Settlement population state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementPopulation {
    pub settlement_id: Uuid,
    pub population: u64,
    pub species_id: SpeciesId,
    pub biome: BiomeType,
    /// Current society type based on population.
    pub society_type: SocietyType,
    /// Carrying capacity for this location.
    pub carrying_capacity: u64,
    /// Growth rate modifier (from species traits, wonders, etc.).
    pub growth_rate_modifier: f64,
    /// Years at current population.
    pub years_at_current_pop: u32,
    /// Historical population samples for trend analysis.
    pub population_history: Vec<PopulationSample>,
}

impl SettlementPopulation {
    pub fn new(settlement_id: Uuid, population: u64, species_id: SpeciesId, biome: BiomeType) -> Self {
        let society_type = SocietyType::from_population(population);
        let carrying_capacity = calculate_carrying_capacity(biome, population);
        let growth_rate_modifier = 1.0; // Base modifier
        
        Self {
            settlement_id,
            population,
            species_id,
            biome,
            society_type,
            carrying_capacity,
            growth_rate_modifier,
            years_at_current_pop: 0,
            population_history: vec![PopulationSample {
                year: 0,
                population,
            }],
        }
    }
}

// Re-export from history module for consistency
pub use crate::history::society::{SocietyType, PopulationSample};

/// Population growth model simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationModel {
    config: PopulationConfig,
    /// Settlement populations keyed by ID.
    settlements: HashMap<Uuid, SettlementPopulation>,
    /// Natural wonder bonuses by location.
    wonder_bonuses: HashMap<Uuid, f64>,
    /// Active diseases by settlement.
    active_diseases: HashMap<Uuid, Vec<ActiveDisease>>,
    /// Food availability by settlement.
    food_availability: HashMap<Uuid, FoodAvailability>,
    /// Current simulation year.
    current_year: i32,
    /// RNG for stochastic elements.
    seed: u64,
    /// Disease outbreak probability per decade (0.0 - 1.0).
    disease_probability: f64,
    /// Disaster probability per decade (0.0 - 1.0).
    disaster_probability: f64,
}

impl PopulationModel {
    /// Create a new population model.
    pub fn new(seed: u64) -> Self {
        Self {
            config: PopulationConfig::default(),
            settlements: HashMap::new(),
            wonder_bonuses: HashMap::new(),
            active_diseases: HashMap::new(),
            food_availability: HashMap::new(),
            current_year: 0,
            seed,
            disease_probability: 0.02,  // 2% chance per decade
            disaster_probability: 0.05, // 5% chance per decade
        }
    }
    
    /// Create with custom configuration.
    pub fn with_config(config: PopulationConfig, seed: u64) -> Self {
        Self {
            config,
            settlements: HashMap::new(),
            wonder_bonuses: HashMap::new(),
            active_diseases: HashMap::new(),
            food_availability: HashMap::new(),
            current_year: 0,
            seed,
            disease_probability: 0.02,
            disaster_probability: 0.05,
        }
    }
    
    /// Set disease outbreak probability (per decade).
    pub fn set_disease_probability(&mut self, prob: f64) {
        self.disease_probability = prob.clamp(0.0, 1.0);
    }
    
    /// Set disaster probability (per decade).
    pub fn set_disaster_probability(&mut self, prob: f64) {
        self.disaster_probability = prob.clamp(0.0, 1.0);
    }
    
    /// Add a settlement to track.
    pub fn add_settlement(&mut self, settlement: &Settlement) {
        let biome = BiomeType::TemperateGrassland; // Would come from terrain data
        let population = settlement.population.unwrap_or(100);
        let settlement_uuid = settlement.id.id;
        
        let mut state = SettlementPopulation::new(
            settlement_uuid,
            population,
            settlement.species_id.unwrap_or(SpeciesId::Human),
            biome,
        );
        
        // Apply any wonder bonuses
        if let Some(&bonus) = self.wonder_bonuses.get(&settlement_uuid) {
            state.growth_rate_modifier *= bonus;
        }
        
        self.settlements.insert(settlement_uuid, state);
        
        // Initialize food availability
        self.food_availability.insert(settlement_uuid, FoodAvailability {
            settlement_id: settlement_uuid,
            production_per_capita: 1.2,
            consumption_per_capita: 1.0,
            surplus_ratio: 1.2,
            security_level: FoodSecurity::Abundant,
        });
    }
    
    /// Add multiple settlements.
    pub fn add_settlements(&mut self, settlements: &[Settlement]) {
        for settlement in settlements {
            self.add_settlement(settlement);
        }
    }
    
    /// Register a natural wonder bonus.
    pub fn add_wonder_bonus(&mut self, settlement_id: Uuid, bonus: f64) {
        let clamped_bonus = bonus.min(self.config.max_wonder_bonus);
        self.wonder_bonuses.insert(settlement_id, clamped_bonus);
        
        // Apply to existing settlement if present
        if let Some(state) = self.settlements.get_mut(&settlement_id) {
            state.growth_rate_modifier = clamped_bonus;
        }
    }
    
    /// Advance simulation by N years.
    pub fn advance_years(&mut self, years: i32) -> Vec<PopulationChange> {
        let mut changes = Vec::new();
        let start_year = self.current_year;
        let end_year = self.current_year + years;
        
        // Process each settlement
        let settlement_ids: Vec<Uuid> = self.settlements.keys().cloned().collect();
        for id in settlement_ids {
            let change = self.simulate_settlement(&id, start_year, end_year);
            if let Some(c) = change {
                changes.push(c);
            }
        }
        
        self.current_year = end_year;
        changes
    }
    
    /// Simulate a single settlement for a time period.
    fn simulate_settlement(&mut self, id: &Uuid, start_year: i32, end_year: i32) -> Option<PopulationChange> {
        let years_elapsed = end_year - start_year;
        
        if years_elapsed <= 0 {
            return None;
        }
        
        // Get values needed from state BEFORE any mutable borrow
        // This avoids nested borrow issues with self.settlements
        let initial_pop = self.settlements.get(id).map(|s| s.population).unwrap_or(0);
        let species_id = self.settlements.get(id).map(|s| s.species_id).unwrap_or(SpeciesId::Undefined);
        let biome = self.settlements.get(id).map(|s| s.biome.clone()).unwrap_or(BiomeType::OpenOcean);
        let growth_modifier = self.settlements.get(id).map(|s| s.growth_rate_modifier).unwrap_or(1.0);
        let carrying_capacity = self.settlements.get(id).map(|s| s.carrying_capacity).unwrap_or(10000);
        let current_population = initial_pop;
        let current_biome = biome.clone();
        let current_capacity = carrying_capacity;
        
        // Simulate disease outbreaks BEFORE mutable borrow
        let disease_outbreaks = self.simulate_disease_outbreaks(id, current_population, current_capacity, current_biome.clone(), start_year, years_elapsed);
        let disease_losses = disease_outbreaks.iter()
            .map(|d| d.calculate_loss(current_population) as f64)
            .sum::<f64>();
        
        // Simulate disasters BEFORE mutable borrow
        let disasters = self.simulate_disasters(id, current_population, current_biome.clone(), start_year, years_elapsed);
        let disaster_losses = disasters.iter()
            .map(|d| d.calculate_loss(current_population) as f64)
            .sum::<f64>();
        
        // Get random noise BEFORE mutable borrow
        let noise = self.seeded_random(id, start_year) * 0.1 - 0.05;
        
        // Get mutable reference AFTER extracting values we need
        let state = self.settlements.get_mut(id)?;
        
        // Calculate base growth rate
        let mut effective_rate = {
            let mut rate = self.config.base_growth_rate;
            let species_mod = match species_id {
                SpeciesId::Human => 1.0,
                SpeciesId::Elf => 0.8,
                SpeciesId::Dwarf => 1.1,
                SpeciesId::Orc => 1.3,
                SpeciesId::Halfling => 1.2,
                SpeciesId::Undefined => 1.0,
                _ => 1.0,
            };
            rate *= species_mod;
            rate *= growth_modifier;
            let biome_mod = get_biome_growth_modifier(biome);
            rate *= biome_mod;
            rate
        };
        
        // Get food availability modifier
        let food_avail = self.food_availability.get(id)
            .cloned()
            .unwrap_or_else(|| FoodAvailability {
                settlement_id: *id,
                production_per_capita: 1.0,
                consumption_per_capita: 1.0,
                surplus_ratio: 1.0,
                security_level: FoodSecurity::Adequate,
            });
        let food_mod = food_avail.growth_modifier();
        effective_rate *= food_mod;
        
        // Apply logistic growth (growth slows near carrying capacity)
        let capacity_ratio = initial_pop as f64 / carrying_capacity as f64;
        let growth_suppression = 1.0 - (capacity_ratio.powi(2)); // Quadratic suppression
        
        // Calculate natural growth
        let natural_growth = initial_pop as f64 * effective_rate * growth_suppression * years_elapsed as f64;
        
        // Apply active diseases (reduce population growth)
        if let Some(active) = self.active_diseases.get_mut(id) {
            for disease in active.iter_mut() {
                if disease.remaining_years > 0 {
                    // Disease reduces effective growth rate
                    effective_rate *= 1.0 - disease.mortality_rate * 0.5;
                    disease.remaining_years -= years_elapsed;
                }
            }
            // Remove expired diseases
            active.retain(|d| d.remaining_years > 0);
        }
        
        // Calculate final population change
        let total_growth = natural_growth - disease_losses - disaster_losses;
        let growth_with_noise = (total_growth * (1.0 + noise)).max(-(initial_pop as f64));
        
        let new_population = (initial_pop as f64 + growth_with_noise) as u64;
        let clamped_pop = new_population.max(self.config.min_population);
        
        // Track society type transitions
        let old_society = state.society_type;
        state.society_type = SocietyType::from_population(clamped_pop);
        
        let society_changed = old_society != state.society_type;
        
        // Update state
        state.population = clamped_pop;
        state.years_at_current_pop += years_elapsed as u32;
        
        // Record history sample (every 50 years)
        if years_elapsed >= 50 {
            state.population_history.push(PopulationSample {
                year: end_year,
                population: clamped_pop,
            });
        }
        
        // Recalculate carrying capacity periodically
        if state.years_at_current_pop > 100 {
            state.carrying_capacity = calculate_carrying_capacity(state.biome, clamped_pop);
        }
        
        Some(PopulationChange {
            settlement_id: *id,
            old_population: initial_pop,
            new_population: clamped_pop,
            change_amount: clamped_pop as i64 - initial_pop as i64,
            growth_rate: effective_rate,
            society_transition: if society_changed { Some(state.society_type) } else { None },
            years_elapsed,
            disease_outbreaks,
            disasters,
            food_availability: food_avail.surplus_ratio,
        })
    }
    
    /// Simulate disease outbreaks for a settlement.
    fn simulate_disease_outbreaks(&mut self, id: &Uuid, population: u64, carrying_capacity: u64, biome: BiomeType, start_year: i32, years: i32) -> Vec<DiseaseOutbreak> {
        let mut outbreaks = Vec::new();
        let _decades = (years / 10).max(1) as f64;
        
        // Probability of outbreak increases with population density
        let density_factor = (population as f64 / carrying_capacity as f64).min(2.0);
        let effective_prob = self.disease_probability * density_factor;
        
        // Each decade has a chance of outbreak
        let mut year = start_year;
        while year < start_year + years {
            let roll = self.seeded_random_with_range(id, year, 1000) as f64 / 1000.0;
            if roll < effective_prob {
                // Determine disease type based on biome and population
                let disease_type = self.select_disease_type(biome);
                let (mortality, duration) = disease_type.base_severity();
                
                // Apply biome suitability modifier
                let biome_mod = disease_type.biome_suitability(biome);
                let adjusted_mortality = (mortality * biome_mod).min(0.8);
                
                outbreaks.push(DiseaseOutbreak {
                    settlement_id: *id,
                    disease_type,
                    mortality_rate: adjusted_mortality,
                    duration_years: duration,
                    start_year: year,
                });
                
                // Register active disease
                self.active_diseases.entry(*id).or_default().push(ActiveDisease {
                    disease_type,
                    mortality_rate: adjusted_mortality,
                    remaining_years: duration,
                    start_year: year,
                });
            }
            year += 10;
        }
        
        outbreaks
    }
    
    /// Select an appropriate disease type based on settlement characteristics.
    fn select_disease_type(&self, _biome: BiomeType) -> DiseaseType {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.current_year.hash(&mut hasher);
        let roll = hasher.finish() % 100;
        
        // Random selection based on biome-influenced probability
        if roll < 10 {
            DiseaseType::Pandemic
        } else if roll < 30 {
            DiseaseType::LocalPlague
        } else if roll < 50 {
            DiseaseType::Waterborne
        } else {
            DiseaseType::SeasonalFlu
        }
    }
    
    /// Simulate disasters for a settlement.
    fn simulate_disasters(&mut self, id: &Uuid, _population: u64, biome: BiomeType, start_year: i32, years: i32) -> Vec<Disaster> {
        let mut disasters = Vec::new();
        
        // Disaster probability varies by biome
        let biome_base_prob = match biome {
            BiomeType::VolcanicLandscape => 0.15,
            BiomeType::HotDesert | BiomeType::TemperateDesert => 0.08,
            BiomeType::TropicalRainforest | BiomeType::TropicalSeasonalForest => 0.06,
            BiomeType::CoastalWetland => 0.07,
            BiomeType::Tundra | BiomeType::Arctic => 0.04,
            _ => 0.03,
        };
        
        let mut year = start_year;
        while year < start_year + years {
            let roll = self.seeded_random_with_range(id, year + 1000, 1000) as f64 / 1000.0;
            if roll < self.disaster_probability * biome_base_prob * 10.0 {
                let disaster_type = self.select_disaster_type(biome);
                let (loss_rate, duration) = disaster_type.base_severity();
                
                // Apply biome-specific adjustments
                let biome_loss_mod = if disaster_type.common_biomes().contains(&biome) {
                    1.5
                } else {
                    0.5
                };
                let adjusted_loss = (loss_rate * biome_loss_mod).min(0.6);
                
                disasters.push(Disaster {
                    settlement_id: *id,
                    disaster_type,
                    population_loss_rate: adjusted_loss,
                    duration_years: duration,
                    year,
                    affects_food: disaster_type.affects_food(),
                });
                
                // Update food availability if disaster affects food
                if disaster_type.affects_food() {
                    if let Some(food) = self.food_availability.get_mut(id) {
                        food.surplus_ratio *= 0.7; // Reduce food security
                        food.security_level = FoodSecurity::from_surplus(food.surplus_ratio);
                    }
                }
            }
            year += 10;
        }
        
        disasters
    }
    
    /// Select an appropriate disaster type based on settlement characteristics.
    fn select_disaster_type(&self, _biome: BiomeType) -> DisasterType {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.current_year.hash(&mut hasher);
        let roll = hasher.finish() % 100;
        
        // More common disasters have higher probability
        if roll < 15 {
            DisasterType::Famine
        } else if roll < 25 {
            DisasterType::Drought
        } else if roll < 35 {
            DisasterType::Flood
        } else if roll < 45 {
            DisasterType::Wildfire
        } else if roll < 55 {
            DisasterType::War
        } else if roll < 65 {
            DisasterType::Earthquake
        } else if roll < 75 {
            DisasterType::Blizzard
        } else if roll < 85 {
            DisasterType::Hurricane
        } else if roll < 92 {
            DisasterType::VolcanicEruption
        } else if roll < 97 {
            DisasterType::LocustSwarm
        } else {
            DisasterType::MagicalCatastrophe
        }
    }
    
    /// Generate seeded pseudo-random value with range.
    fn seeded_random_with_range(&self, id: &Uuid, seed: i32, range: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        seed.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        
        let hash = hasher.finish();
        hash % range
    }
    
    /// Calculate food availability for a settlement.
    pub fn calculate_food(&mut self, id: &Uuid) -> Option<FoodAvailability> {
        let state = self.settlements.get(id)?;
        
        // Base production varies by biome
        let base_production = match state.biome {
            BiomeType::TemperateGrassland => 1.5,
            BiomeType::TropicalSavanna => 1.4,
            BiomeType::TemperateDeciduousForest => 1.2,
            BiomeType::CoastalWetland => 1.1,
            BiomeType::TropicalRainforest => 1.0,
            BiomeType::BorealForest => 0.8,
            BiomeType::HotDesert => 0.2,
            _ => 0.8,
        };
        
        // Population affects consumption
        let population_factor = 1.0 + (state.population as f64 / 10000.0).min(0.5);
        
        let production = base_production * state.growth_rate_modifier;
        let consumption = 1.0 * population_factor;
        let surplus = production / consumption;
        
        let food = FoodAvailability {
            settlement_id: *id,
            production_per_capita: production,
            consumption_per_capita: consumption,
            surplus_ratio: surplus,
            security_level: FoodSecurity::from_surplus(surplus),
        };
        
        self.food_availability.insert(*id, food.clone());
        Some(food)
    }
    
    /// Calculate the effective growth rate for a settlement.
    fn calculate_growth_rate(&self, state: &SettlementPopulation) -> f64 {
        // Base rate from config
        let mut rate = self.config.base_growth_rate;
        
        // Apply species trait modifiers
        let species_mod = self.get_species_growth_modifier(state.species_id);
        rate *= species_mod;
        
        // Apply wonder bonuses
        rate *= state.growth_rate_modifier;
        
        // Biome-specific modifiers
        let biome_mod = get_biome_growth_modifier(state.biome);
        rate *= biome_mod;
        
        rate
    }
    
    /// Get growth rate modifier from species traits.
    fn get_species_growth_modifier(&self, species_id: SpeciesId) -> f64 {
        match species_id {
            SpeciesId::Human => 1.0,      // Base - adaptable
            SpeciesId::Elf => 0.8,         // Slower reproduction
            SpeciesId::Dwarf => 1.1,       // Industrious
            SpeciesId::Orc => 1.3,         // High birth rate
            SpeciesId::Halfling => 1.2,    // Family-oriented
            SpeciesId::Undefined => 1.0,
            _ => 1.0,                       // Unknown species get base rate
        }
    }
    
    /// Generate seeded pseudo-random value.
    fn seeded_random(&self, id: &Uuid, year: i32) -> f64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        year.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        
        let hash = hasher.finish();
        ((hash as f64) % 1000.0) / 1000.0
    }
    
    /// Get current population for a settlement.
    pub fn get_population(&self, id: &Uuid) -> Option<u64> {
        self.settlements.get(id).map(|s| s.population)
    }
    
    /// Get all settlements with their populations.
    pub fn get_all_populations(&self) -> &HashMap<Uuid, SettlementPopulation> {
        &self.settlements
    }
    
    /// Get total population across all settlements.
    pub fn total_population(&self) -> u64 {
        self.settlements.values().map(|s| s.population).sum()
    }
    
    /// Get population by society type.
    pub fn population_by_society(&self) -> HashMap<SocietyType, u64> {
        let mut result = HashMap::new();
        for state in self.settlements.values() {
            *result.entry(state.society_type).or_insert(0) += state.population;
        }
        result
    }
    
    /// Get population by species.
    pub fn population_by_species(&self) -> HashMap<SpeciesId, u64> {
        let mut result = HashMap::new();
        for state in self.settlements.values() {
            *result.entry(state.species_id).or_insert(0) += state.population;
        }
        result
    }
    
    /// Get current simulation year.
    pub fn current_year(&self) -> i32 {
        self.current_year
    }
    
    /// Reset to initial state.
    pub fn reset(&mut self, year: i32) {
        for state in self.settlements.values_mut() {
            state.years_at_current_pop = 0;
            state.population_history.clear();
            if let Some(first) = state.population_history.first() {
                state.population = first.population;
            }
        }
        self.current_year = year;
    }
}

/// Result of a population change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationChange {
    pub settlement_id: Uuid,
    pub old_population: u64,
    pub new_population: u64,
    pub change_amount: i64,
    pub growth_rate: f64,
    pub society_transition: Option<SocietyType>,
    pub years_elapsed: i32,
    /// Disease outbreaks that occurred during this period.
    pub disease_outbreaks: Vec<DiseaseOutbreak>,
    /// Disasters that occurred during this period.
    pub disasters: Vec<Disaster>,
    /// Food availability factor (0.0 = famine, 1.0 = abundant).
    pub food_availability: f64,
}

impl PopulationChange {
    /// Get the growth percentage.
    pub fn growth_percentage(&self) -> f64 {
        if self.old_population == 0 {
            return 0.0;
        }
        (self.change_amount as f64 / self.old_population as f64) * 100.0
    }
    
    /// Check if this represents growth.
    pub fn is_growth(&self) -> bool {
        self.change_amount > 0
    }
    
    /// Check if this represents decline.
    pub fn is_decline(&self) -> bool {
        self.change_amount < 0
    }
    
    /// Check if there were any negative events.
    pub fn had_adverse_events(&self) -> bool {
        !self.disease_outbreaks.is_empty() || !self.disasters.is_empty() || self.food_availability < 0.5
    }
    
    /// Get total population loss from disease.
    pub fn total_disease_loss(&self) -> u64 {
        self.disease_outbreaks.iter()
            .map(|d| (self.old_population as f64 * d.mortality_rate) as u64)
            .sum()
    }
    
    /// Get total population loss from disasters.
    pub fn total_disaster_loss(&self) -> u64 {
        self.disasters.iter()
            .map(|d| (self.old_population as f64 * d.population_loss_rate) as u64)
            .sum()
    }
    
    /// Convert to event effects.
    pub fn to_event_effects(&self) -> Vec<EventEffect> {
        let mut effects = Vec::new();
        
        if self.change_amount > 0 {
            effects.push(EventEffect::PopulationGrowth {
                target: self.settlement_id,
                amount: self.change_amount as u64,
                duration_years: Some(self.years_elapsed),
                cause: Some("Natural growth".to_string()),
            });
        } else if self.change_amount < 0 {
            effects.push(EventEffect::PopulationLoss {
                target: self.settlement_id,
                amount: (-self.change_amount) as u64,
                duration_years: Some(self.years_elapsed),
                cause: Some("Decline".to_string()),
            });
        }
        
        effects
    }
}

/// A natural disaster event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disaster {
    /// Settlement affected.
    pub settlement_id: Uuid,
    /// Type of disaster.
    pub disaster_type: DisasterType,
    /// Population loss rate (0.0 - 1.0).
    pub population_loss_rate: f64,
    /// Duration in years.
    pub duration_years: i32,
    /// Year the disaster occurred.
    pub year: i32,
    /// Whether it affects food production.
    pub affects_food: bool,
}

impl Disaster {
    /// Calculate population loss from this disaster.
    pub fn calculate_loss(&self, current_pop: u64) -> u64 {
        ((current_pop as f64) * self.population_loss_rate) as u64
    }
    
    /// Get recovery time in years.
    pub fn recovery_years(&self) -> i32 {
        match self.disaster_type {
            DisasterType::Earthquake => 2,
            DisasterType::VolcanicEruption => 5,
            DisasterType::Flood => 2,
            DisasterType::Drought => 5,
            DisasterType::Hurricane => 1,
            DisasterType::Wildfire => 3,
            DisasterType::Famine => 7,
            DisasterType::War => 10,
            DisasterType::LocustSwarm => 2,
            DisasterType::Tsunami => 3,
            DisasterType::Blizzard => 1,
            DisasterType::MagicalCatastrophe => 8,
        }
    }
}

/// Food availability data for a settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodAvailability {
    /// Settlement ID.
    pub settlement_id: Uuid,
    /// Food production per capita per year.
    pub production_per_capita: f64,
    /// Food consumption per capita per year.
    pub consumption_per_capita: f64,
    /// Surplus ratio (production/consumption).
    pub surplus_ratio: f64,
    /// Food security level.
    pub security_level: FoodSecurity,
}

impl FoodAvailability {
    /// Calculate growth modifier from food availability.
    pub fn growth_modifier(&self) -> f64 {
        match self.security_level {
            FoodSecurity::Starvation => 0.0,
            FoodSecurity::Scarcity => 0.3,
            FoodSecurity::Adequate => 0.8,
            FoodSecurity::Abundant => 1.2,
            FoodSecurity::Surplus => 1.5,
        }
    }
}

/// Food security levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodSecurity {
    /// No food, population will decline rapidly.
    Starvation,
    /// Insufficient food, growth suppressed.
    Scarcity,
    /// Basic food needs met, slow growth.
    Adequate,
    /// Good food supply, normal growth.
    Abundant,
    /// Surplus food, bonus growth.
    Surplus,
}

impl FoodSecurity {
    /// Determine security level from surplus ratio.
    pub fn from_surplus(surplus: f64) -> Self {
        if surplus <= 0.0 {
            FoodSecurity::Starvation
        } else if surplus < 0.5 {
            FoodSecurity::Scarcity
        } else if surplus < 1.0 {
            FoodSecurity::Adequate
        } else if surplus < 1.5 {
            FoodSecurity::Abundant
        } else {
            FoodSecurity::Surplus
        }
    }
}

/// Active disease tracking for a settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDisease {
    pub disease_type: DiseaseType,
    pub mortality_rate: f64,
    pub remaining_years: i32,
    pub start_year: i32,
}

/// Calculate carrying capacity based on biome.
fn calculate_carrying_capacity(biome: BiomeType, current_pop: u64) -> u64 {
    // Base capacity per km² (assuming 100x100 grid cells = ~100km² per "region")
    let base_capacity = match biome {
        // High capacity
        BiomeType::TemperateGrassland => 50_000,
        BiomeType::TropicalSavanna => 45_000,
        BiomeType::TemperateDeciduousForest => 40_000,
        BiomeType::TemperateMixedForest => 40_000,
        BiomeType::CoastalWetland => 35_000,
        
        // Medium capacity
        BiomeType::SubtropicalSeasonalForest => 30_000,
        BiomeType::TropicalSeasonalForest => 30_000,
        BiomeType::TemperateSteppe => 25_000,
        BiomeType::BorealForest => 20_000,
        BiomeType::BorealTaiga => 18_000,
        BiomeType::MontaneForest => 15_000,
        BiomeType::MontaneGrassland => 15_000,
        
        // Low capacity
        BiomeType::Mangrove => 10_000,
        BiomeType::SubtropicalSteppe => 10_000,
        BiomeType::SemiAridSteppe => 8_000,
        BiomeType::TropicalDryForest => 8_000,
        BiomeType::SubtropicalDesert => 3_000,
        BiomeType::TropicalRainforest => 40_000,
        BiomeType::TropicalSeasonalForest => 35_000,
        BiomeType::SubtropicalRainforest => 35_000,
        BiomeType::TemperateRainforest => 35_000,
        
        // Very low/none
        BiomeType::HotDesert => 1_000,
        BiomeType::ColdDesert => 1_000,
        BiomeType::TemperateDesert => 1_000,
        BiomeType::Tundra => 500,
        BiomeType::Arctic => 100,
        BiomeType::PolarDesert => 50,
        BiomeType::SnowGlacier => 0,
        BiomeType::AlpineTundra => 500,
        
        // Aquatic - no carrying capacity for settlements
        BiomeType::OpenOcean => 0,
        BiomeType::CoralReef => 0,
        BiomeType::KelpForest => 0,
        BiomeType::BioluminescentOcean => 0,
        
        // Fantasy biomes
        BiomeType::MagicalForest => 25_000,
        BiomeType::CrystallineDesert => 500,
        BiomeType::VolcanicLandscape => 2_000,
        BiomeType::ToxicSwamp => 5_000,
        BiomeType::FloatingIslands => 10_000,
    };
    
    // Scale with current population (settlements grow to fill capacity)
    let min_capacity = current_pop.max(100);
    base_capacity.max(min_capacity)
}

/// Get growth rate modifier for biome.
fn get_biome_growth_modifier(biome: BiomeType) -> f64 {
    match biome {
        // Optimal biomes
        BiomeType::TemperateGrassland => 1.2,
        BiomeType::TropicalSavanna => 1.15,
        BiomeType::TemperateDeciduousForest => 1.1,
        BiomeType::TemperateMixedForest => 1.1,
        BiomeType::TropicalRainforest => 1.1,
        BiomeType::TemperateRainforest => 1.05,
        
        // Good biomes
        BiomeType::CoastalWetland => 1.0,
        BiomeType::TemperateSteppe => 1.0,
        BiomeType::SubtropicalSeasonalForest => 0.95,
        BiomeType::TropicalSeasonalForest => 0.95,
        BiomeType::BorealForest => 0.9,
        BiomeType::SubtropicalRainforest => 0.95,
        
        // Challenging biomes
        BiomeType::BorealTaiga => 0.8,
        BiomeType::MontaneForest => 0.8,
        BiomeType::MontaneGrassland => 0.8,
        BiomeType::SubtropicalSteppe => 0.75,
        BiomeType::SemiAridSteppe => 0.7,
        BiomeType::TropicalDryForest => 0.7,
        BiomeType::Mangrove => 0.7,
        
        // Hostile biomes
        BiomeType::SubtropicalDesert => 0.4,
        BiomeType::HotDesert => 0.3,
        BiomeType::ColdDesert => 0.3,
        BiomeType::TemperateDesert => 0.3,
        BiomeType::Tundra => 0.3,
        BiomeType::Arctic => 0.1,
        BiomeType::PolarDesert => 0.05,
        BiomeType::SnowGlacier => 0.0,
        BiomeType::AlpineTundra => 0.3,
        
        // Aquatic - no settlement growth
        BiomeType::OpenOcean => 0.0,
        BiomeType::CoralReef => 0.0,
        BiomeType::KelpForest => 0.0,
        BiomeType::BioluminescentOcean => 0.0,
        
        // Fantasy biomes
        BiomeType::MagicalForest => 1.15,
        BiomeType::CrystallineDesert => 0.2,
        BiomeType::VolcanicLandscape => 0.5,
        BiomeType::ToxicSwamp => 0.4,
        BiomeType::FloatingIslands => 1.0,
    }
}

/// Convert natural wonder bonus to population growth modifier.
pub fn wonder_bonus_to_growth_modifier(_bonus: &NaturalWonder) -> f64 {
    // Natural wonders provide population growth bonuses
    // This would be read from the wonder's effects
    1.0 // Placeholder - actual implementation would check wonder effects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_society_type_from_population() {
        assert_eq!(SocietyType::from_population(50), SocietyType::Tribe);
        assert_eq!(SocietyType::from_population(500), SocietyType::Chiefdom);
        assert_eq!(SocietyType::from_population(5000), SocietyType::Nation);
        assert_eq!(SocietyType::from_population(100_000), SocietyType::Nation);
    }

    #[test]
    fn test_population_growth() {
        let mut model = PopulationModel::new(42);
        
        let id = Uuid::new_v4();
        model.add_settlement_raw(id, 1000, SpeciesId::Human, BiomeType::TemperateGrassland);
        
        // Simulate 1000 years - with base rate 0.0015/year and species modifier 1.0,
        // population should grow significantly over a millennium
        let changes = model.advance_years(1000);
        
        assert!(!changes.is_empty());
        let change = &changes[0];
        assert!(change.new_population > change.old_population, "Population should grow over 1000 years");
    }

    #[test]
    fn test_carrying_capacity() {
        let capacity_grass = calculate_carrying_capacity(BiomeType::TemperateGrassland, 100);
        let capacity_desert = calculate_carrying_capacity(BiomeType::HotDesert, 100);
        
        assert!(capacity_grass > capacity_desert);
    }

    #[test]
    fn test_total_population() {
        let mut model = PopulationModel::new(42);
        
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        model.add_settlement_raw(id1, 1000, SpeciesId::Human, BiomeType::TemperateGrassland);
        model.add_settlement_raw(id2, 2000, SpeciesId::Human, BiomeType::TemperateGrassland);
        
        assert_eq!(model.total_population(), 3000);
    }

    #[test]
    fn test_population_by_society() {
        let mut model = PopulationModel::new(42);
        
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        model.add_settlement_raw(id1, 100, SpeciesId::Human, BiomeType::TemperateGrassland);    // Tribe
        model.add_settlement_raw(id2, 2000, SpeciesId::Human, BiomeType::TemperateGrassland);   // Chiefdom
        model.add_settlement_raw(id3, 10000, SpeciesId::Human, BiomeType::TemperateGrassland);   // Nation
        
        let by_society = model.population_by_society();
        assert_eq!(by_society.get(&SocietyType::Tribe), Some(&100));
        assert_eq!(by_society.get(&SocietyType::Chiefdom), Some(&2000));
        assert_eq!(by_society.get(&SocietyType::Nation), Some(&10000));
    }

    #[test]
    fn test_population_change_effects() {
        let change = PopulationChange {
            settlement_id: Uuid::new_v4(),
            old_population: 1000,
            new_population: 1200,
            change_amount: 200,
            growth_rate: 0.015,
            society_transition: None,
            years_elapsed: 10,
            disease_outbreaks: vec![],
            disasters: vec![],
            food_availability: 1.2,
        };
        
        let effects = change.to_event_effects();
        assert_eq!(effects.len(), 1);
        match &effects[0] {
            EventEffect::PopulationGrowth { amount, .. } => {
                assert_eq!(*amount, 200);
            }
            _ => panic!("Expected PopulationGrowth effect"),
        }
    }
    
    #[test]
    fn test_disease_outbreak() {
        let outbreak = DiseaseOutbreak {
            settlement_id: Uuid::new_v4(),
            disease_type: DiseaseType::Pandemic,
            mortality_rate: 0.30,
            duration_years: 5,
            start_year: 100,
        };
        
        let loss = outbreak.calculate_loss(1000);
        assert_eq!(loss, 300); // 30% of 1000
    }
    
    #[test]
    fn test_disaster_loss() {
        let disaster = Disaster {
            settlement_id: Uuid::new_v4(),
            disaster_type: DisasterType::Drought,
            population_loss_rate: 0.20,
            duration_years: 5,
            year: 100,
            affects_food: true,
        };
        
        let loss = disaster.calculate_loss(2000);
        assert_eq!(loss, 400); // 20% of 2000
        assert_eq!(disaster.recovery_years(), 5);
    }
    
    #[test]
    fn test_food_security() {
        assert_eq!(FoodSecurity::from_surplus(0.3), FoodSecurity::Scarcity);
        assert_eq!(FoodSecurity::from_surplus(0.8), FoodSecurity::Adequate);
        assert_eq!(FoodSecurity::from_surplus(1.2), FoodSecurity::Abundant);
        assert_eq!(FoodSecurity::from_surplus(1.8), FoodSecurity::Surplus);
        
        let food = FoodAvailability {
            settlement_id: Uuid::new_v4(),
            production_per_capita: 1.5,
            consumption_per_capita: 1.0,
            surplus_ratio: 1.5,
            security_level: FoodSecurity::Abundant,
        };
        
        assert_eq!(food.growth_modifier(), 1.2);
    }
    
    #[test]
    fn test_disease_type_severity() {
        let (mortality, duration) = DiseaseType::Pandemic.base_severity();
        assert_eq!(mortality, 0.30);
        assert_eq!(duration, 5);
        
        let (mortality, duration) = DiseaseType::CommonCold.base_severity();
        assert_eq!(mortality, 0.001);
        assert_eq!(duration, 1);
    }
    
    #[test]
    fn test_disaster_type_severity() {
        let (loss, duration) = DisasterType::Famine.base_severity();
        assert_eq!(loss, 0.30);
        assert_eq!(duration, 3);
        
        assert!(DisasterType::Famine.affects_food());
        assert!(DisasterType::Drought.affects_food());
        assert!(!DisasterType::Earthquake.affects_food());
    }
}

impl PopulationModel {
    /// Add a settlement with raw parameters (internal use).
    fn add_settlement_raw(&mut self, id: Uuid, population: u64, species_id: SpeciesId, biome: BiomeType) {
        let state = SettlementPopulation::new(id, population, species_id, biome);
        self.settlements.insert(id, state);
    }
}