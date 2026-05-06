//! Society Entity Module
//!
//! Implements the Society struct for representing civilizations and their
//! evolution over time. Societies form from settlements, grow, and can
//! transition between types (Band → Tribe → Chiefdom → Nation).
//!
//! # Population Growth Formula
//!
//! ```
//! growth_rate = base_reproduction_rate × food_surplus_factor × disease_factor
//! food_surplus_factor = min(1.0, available_food / food_requirement)
//! disease_factor = 1.0 - (population_density / carrying_capacity × 0.3)
//! population += floor(population × growth_rate × (1 - population / carrying_capacity))
//! ```
//!
//! # Society Transitions
//!
//! - **Band**: 10-50 population (initial for very small settlements)
//! - **Tribe**: 50-500 population  
//! - **Chiefdom**: 500-5000 population
//! - **Nation**: 5000+ population
//!
//! Transitions occur automatically when population crosses thresholds.

use crate::species::SpeciesId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Errors specific to society operations.
#[derive(Debug, thiserror::Error)]
pub enum SocietyError {
    #[error("Society {0} not found")]
    NotFound(Uuid),

    #[error("Invalid population: {0}")]
    InvalidPopulation(String),

    #[error("Cannot transition: population {pop} below threshold for {society_type:?}")]
    TransitionBelowThreshold { pop: u64, society_type: SocietyType },

    #[error("No settlements in society {0}")]
    NoSettlements(Uuid),
}

/// Types of societies based on organizational complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocietyType {
    /// Small family groups (10-50 people).
    /// Initial form for new settlements.
    Band,
    /// Larger family groups with shared identity (50-500 people).
    Tribe,
    /// Organized groups with leadership (500-5000 people).
    Chiefdom,
    /// Complex societies with institutions (5000+ people).
    Nation,
}

impl SocietyType {
    /// Get population range for this society type.
    pub fn population_range(&self) -> (u64, u64) {
        match self {
            SocietyType::Band => (10, 50),
            SocietyType::Tribe => (50, 500),
            SocietyType::Chiefdom => (500, 5_000),
            SocietyType::Nation => (5_000, u64::MAX),
        }
    }

    /// Get minimum population threshold for this type.
    pub fn min_population(&self) -> u64 {
        self.population_range().0
    }

    /// Get the next society type, if any.
    pub fn evolve_to(&self) -> Option<SocietyType> {
        match self {
            SocietyType::Band => Some(SocietyType::Tribe),
            SocietyType::Tribe => Some(SocietyType::Chiefdom),
            SocietyType::Chiefdom => Some(SocietyType::Nation),
            SocietyType::Nation => None,
        }
    }

    /// Determine society type from population.
    pub fn from_population(population: u64) -> Self {
        if population >= 5_000 {
            SocietyType::Nation
        } else if population >= 500 {
            SocietyType::Chiefdom
        } else if population >= 50 {
            SocietyType::Tribe
        } else {
            SocietyType::Band
        }
    }

    /// Check if population qualifies for this society type.
    pub fn accepts_population(&self, population: u64) -> bool {
        let (min, max) = self.population_range();
        population >= min && population < max
    }
}

/// Core society entity representing a civilization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Society {
    /// Unique identifier for this society.
    pub id: Uuid,

    /// Name of the society.
    pub name: String,

    /// Species inhabiting this society.
    pub species_id: SpeciesId,

    /// Current organizational type.
    pub society_type: SocietyType,

    /// Settlement IDs belonging to this society.
    pub settlement_ids: Vec<Uuid>,

    /// Territory polygon IDs (terrain cell indices).
    pub territory_ids: Vec<u32>,

    /// Total population across all settlements.
    pub population: u64,

    /// ID of the society's current leader (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader_id: Option<Uuid>,

    /// Parent society ID (for sub-societies or vassals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_society_id: Option<Uuid>,

    /// Year the society was formed.
    pub formed_year: i32,

    /// Historical population samples for timeline.
    #[serde(default)]
    pub population_history: Vec<PopulationSample>,

    /// Food availability factor (0.0 = famine, 1.0 = surplus).
    #[serde(default)]
    pub food_surplus_factor: f32,

    /// Disease factor (0.0-1.0, affects growth).
    #[serde(default)]
    pub disease_factor: f32,
}

impl Society {
    /// Create a new society from an initial settlement.
    pub fn from_settlement(
        settlement_id: Uuid,
        name: String,
        species_id: SpeciesId,
        initial_population: u64,
        formed_year: i32,
    ) -> Self {
        let society_type = SocietyType::from_population(initial_population);

        Self {
            id: Uuid::new_v4(),
            name,
            species_id,
            society_type,
            settlement_ids: vec![settlement_id],
            territory_ids: Vec::new(),
            population: initial_population,
            leader_id: None,
            parent_society_id: None,
            formed_year,
            population_history: vec![PopulationSample {
                year: formed_year,
                population: initial_population,
            }],
            food_surplus_factor: 1.0,
            disease_factor: 1.0,
        }
    }

    /// Get the carrying capacity based on territory and biome.
    pub fn carrying_capacity(&self) -> u64 {
        // Each territory cell supports ~1000 people on average
        // This should be enhanced with actual biome data
        (self.territory_ids.len() as u64) * 1000
    }

    /// Add a settlement to this society.
    pub fn add_settlement(&mut self, settlement_id: Uuid) {
        if !self.settlement_ids.contains(&settlement_id) {
            self.settlement_ids.push(settlement_id);
        }
    }

    /// Remove a settlement from this society.
    pub fn remove_settlement(&mut self, settlement_id: Uuid) -> bool {
        if let Some(pos) = self
            .settlement_ids
            .iter()
            .position(|&id| id == settlement_id)
        {
            self.settlement_ids.remove(pos);
            true
        } else {
            false
        }
    }

    /// Add territory to this society.
    pub fn add_territory(&mut self, territory_id: u32) {
        if !self.territory_ids.contains(&territory_id) {
            self.territory_ids.push(territory_id);
        }
    }

    /// Update the society's population total.
    pub fn update_population(&mut self, new_population: u64) {
        self.population = new_population;
    }

    /// Add to the population total.
    pub fn add_population(&mut self, amount: u64) {
        self.population += amount;
    }

    /// Check and perform a society type transition if thresholds are crossed.
    /// Returns Some(old_type) if transition occurred.
    pub fn check_transition(&mut self) -> Option<SocietyType> {
        let new_type = SocietyType::from_population(self.population);

        if new_type != self.society_type {
            let old_type = self.society_type;
            self.society_type = new_type;
            Some(old_type)
        } else {
            None
        }
    }

    /// Get the target expansion rate based on society type.
    pub fn target_expansion_rate(&self) -> f32 {
        match self.society_type {
            SocietyType::Band => 0.01,     // Very slow expansion
            SocietyType::Tribe => 0.03,    // Slow expansion
            SocietyType::Chiefdom => 0.08, // Moderate expansion
            SocietyType::Nation => 0.15,   // Aggressive expansion
        }
    }

    /// Calculate growth rate based on food and disease factors.
    ///
    /// Formula: `growth_rate = base × food_surplus_factor × disease_factor`
    pub fn growth_rate(&self, base_reproduction_rate: f32) -> f32 {
        base_reproduction_rate * self.food_surplus_factor * self.disease_factor
    }

    /// Update food surplus factor.
    pub fn set_food_surplus(&mut self, surplus: f32) {
        self.food_surplus_factor = surplus.clamp(0.0, 2.0);
    }

    /// Update disease factor.
    pub fn set_disease_factor(&mut self, disease: f32) {
        self.disease_factor = disease.clamp(0.0, 1.0);
    }

    /// Record a population sample for history.
    pub fn record_population(&mut self, year: i32, population: u64) {
        self.population_history
            .push(PopulationSample { year, population });
        self.population = population;
    }
}

/// A historical population sample for a society.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PopulationSample {
    pub year: i32,
    pub population: u64,
}

/// Collection of societies for efficient lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocietyRegistry {
    pub societies: HashMap<Uuid, Society>,
}

impl SocietyRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new society.
    pub fn register(&mut self, society: Society) {
        self.societies.insert(society.id, society);
    }

    /// Get a society by ID.
    pub fn get(&self, id: Uuid) -> Option<&Society> {
        self.societies.get(&id)
    }

    /// Get a mutable society by ID.
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Society> {
        self.societies.get_mut(&id)
    }

    /// Get all societies of a given type.
    pub fn by_type(&self, society_type: SocietyType) -> Vec<&Society> {
        self.societies
            .values()
            .filter(|s| s.society_type == society_type)
            .collect()
    }

    /// Get all societies for a given species.
    pub fn by_species(&self, species_id: SpeciesId) -> Vec<&Society> {
        self.societies
            .values()
            .filter(|s| s.species_id == species_id)
            .collect()
    }

    /// Get societies at or above a population threshold.
    pub fn above_population(&self, threshold: u64) -> Vec<&Society> {
        self.societies
            .values()
            .filter(|s| s.population >= threshold)
            .collect()
    }

    /// Get total population across all societies.
    pub fn total_population(&self) -> u64 {
        self.societies.values().map(|s| s.population).sum()
    }

    /// Get total settlement count across all societies.
    pub fn total_settlements(&self) -> usize {
        self.societies
            .values()
            .map(|s| s.settlement_ids.len())
            .sum()
    }

    /// Form initial societies from settlements.
    /// Each settlement starts as a Band, upgrades to Tribe at 50 pop.
    pub fn form_initial_societies(&mut self, settlements: &[(Uuid, String, SpeciesId, u64, i32)]) {
        for (id, name, species_id, population, year) in settlements {
            let society =
                Society::from_settlement(*id, name.clone(), *species_id, *population, *year);
            self.register(society);
        }
    }

    /// Get societies that could merge (same species, adjacent territory).
    /// Returns pairs of society IDs that could be merged.
    pub fn mergeable_societies(&self) -> Vec<(Uuid, Uuid)> {
        let mut pairs = Vec::new();

        for (id1, s1) in &self.societies {
            for (id2, s2) in &self.societies {
                if id1 >= id2 {
                    continue;
                }

                // Can merge if same species and adjacent or overlapping territory
                if s1.species_id == s2.species_id && !s1.territory_ids.is_empty() {
                    let has_adjacent = s1.territory_ids.iter().any(|t1| {
                        s2.territory_ids.iter().any(|t2| {
                            let diff = (*t1 as i32 - *t2 as i32).abs();
                            diff <= 1 || diff == (100) // Assuming 100-wide grid
                        })
                    });

                    if has_adjacent {
                        pairs.push((*id1, *id2)); // Push as tuple pair
                    }
                }
            }
        }

        pairs
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
    fn test_society_type_ranges() {
        assert!(SocietyType::Band.accepts_population(25));
        assert!(!SocietyType::Band.accepts_population(50));

        assert!(SocietyType::Tribe.accepts_population(50));
        assert!(SocietyType::Tribe.accepts_population(100));
        assert!(!SocietyType::Tribe.accepts_population(500));
    }

    #[test]
    fn test_society_creation() {
        let society = Society::from_settlement(
            Uuid::new_v4(),
            "Test Tribe's".to_string(),
            SpeciesId::Human,
            100,
            100,
        );

        assert_eq!(society.society_type, SocietyType::Tribe);
        assert_eq!(society.population, 100);
        assert_eq!(society.settlement_ids.len(), 1);
        assert_eq!(society.formed_year, 100);
    }

    #[test]
    fn test_society_transition() {
        let mut society = Society::from_settlement(
            Uuid::new_v4(),
            "Small Band".to_string(),
            SpeciesId::Human,
            30, // Band-level population
            100,
        );

        assert_eq!(society.society_type, SocietyType::Band);

        // Grow to Tribe level
        society.update_population(75);
        let old_type = society.check_transition();

        assert_eq!(old_type, Some(SocietyType::Band));
        assert_eq!(society.society_type, SocietyType::Tribe);
    }

    #[test]
    fn test_growth_rate_calculation() {
        let mut society = Society::from_settlement(
            Uuid::new_v4(),
            "Test".to_string(),
            SpeciesId::Human,
            100,
            100,
        );

        // Base rate with good conditions
        society.set_food_surplus(1.0);
        society.set_disease_factor(1.0);

        let rate = society.growth_rate(0.01);
        assert!((rate - 0.01).abs() < 0.001);

        // Famine conditions
        society.set_food_surplus(0.3);
        let rate = society.growth_rate(0.01);
        assert!((rate - 0.003).abs() < 0.001);
    }

    #[test]
    fn test_society_registry() {
        let mut registry = SocietyRegistry::new();

        let settlements = vec![
            (
                Uuid::new_v4(),
                "Settlement A".to_string(),
                SpeciesId::Human,
                100,
                100,
            ),
            (
                Uuid::new_v4(),
                "Settlement B".to_string(),
                SpeciesId::Human,
                150,
                100,
            ),
            (
                Uuid::new_v4(),
                "Settlement C".to_string(),
                SpeciesId::Elf,
                80,
                100,
            ),
        ];

        registry.form_initial_societies(&settlements);

        assert_eq!(registry.societies.len(), 3);
        assert_eq!(registry.total_population(), 330);

        // Check by species
        let human_societies = registry.by_species(SpeciesId::Human);
        assert_eq!(human_societies.len(), 2);

        let elf_societies = registry.by_species(SpeciesId::Elf);
        assert_eq!(elf_societies.len(), 1);
    }

    #[test]
    fn test_population_sample_recording() {
        let mut society = Society::from_settlement(
            Uuid::new_v4(),
            "Test".to_string(),
            SpeciesId::Human,
            100,
            100,
        );

        society.record_population(110, 120);
        society.record_population(120, 145);

        assert_eq!(society.population_history.len(), 3); // Initial + 2 recordings
        assert_eq!(society.population, 145);
    }
}
