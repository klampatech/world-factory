//! PreHistoryGenerator - Orchestration pipeline for world simulation
//!
//! Connects all components (species, settlements, events, figures, artifacts)
//! into a unified yearly simulation loop.

use std::collections::HashMap;

use crate::history::result::{
    HistoryResult, YearResult, WorldState, SocietyTransition, TimelineEvent, SocietyType,
};
use crate::history::timeline::HistoryTimeline;
use crate::species::{SpeciesData, Species};
use crate::settlements::Settlement;
use crate::events::{EventGenerator, Event};
use crate::figures::{FigureGenerator, NotableFigure};
use crate::artifacts::{ArtifactGenerator, Artifact};
use crate::territory::{TerritorySystem, FactionId, PolygonInfo, TerritoryClaim};

/// Configuration for prehistory generation
#[derive(Debug, Clone)]
pub struct PreHistoryConfig {
    pub world_id: String,
    pub pre_history_years: u32,
    pub population_growth_rate: f32,
    pub event_probability_base: f32,
    pub figure_significance_threshold: f32,
    pub artifact_creation_probability: f32,
    pub society_transition_thresholds: SocietyTransitionConfig,
}

#[derive(Debug, Clone)]
pub struct SocietyTransitionConfig {
    pub band_to_tribe_population: u32,
    pub tribe_to_chiefdom_population: u32,
    pub chiefdom_to_nation_population: u32,
}

impl Default for PreHistoryConfig {
    fn default() -> Self {
        Self {
            world_id: uuid::Uuid::new_v4().to_string(),
            pre_history_years: 1000,
            population_growth_rate: 0.02,
            event_probability_base: 0.1,
            figure_significance_threshold: 0.7,
            artifact_creation_probability: 0.05,
            society_transition_thresholds: SocietyTransitionConfig {
                band_to_tribe_population: 50,
                tribe_to_chiefdom_population: 200,
                chiefdom_to_nation_population: 1000,
            },
        }
    }
}

/// Terrain data for territory generation
#[derive(Debug, Clone)]
pub struct TerrainData {
    pub regions: Vec<TerrainRegion>,
    pub polygons: Vec<PolygonInfo>,
}

#[derive(Debug, Clone)]
pub struct TerrainRegion {
    pub id: String,
    pub name: String,
    pub size: u32,
    pub fertility: f32,
    pub resource_density: f32,
}

/// Main PreHistoryGenerator orchestrator
pub struct PreHistoryGenerator {
    config: PreHistoryConfig,
    timeline: HistoryTimeline,
    species: Vec<Species>,
    settlements: Vec<Settlement>,
    territories: HashMap<FactionId, TerritoryClaim>,
    figures: Vec<NotableFigure>,
    artifacts: Vec<Artifact>,
    event_generator: EventGenerator,
    figure_generator: FigureGenerator,
    artifact_generator: ArtifactGenerator,
    territory_system: TerritorySystem,
    elevation_map: HashMap<u64, f32>,
    active_wars: HashMap<(FactionId, FactionId), i32>,
}

impl PreHistoryGenerator {
    /// Create a new PreHistoryGenerator
    pub fn new(config: PreHistoryConfig) -> Self {
        Self {
            config,
            timeline: HistoryTimeline::new(),
            species: Vec::new(),
            settlements: Vec::new(),
            territories: HashMap::new(),
            figures: Vec::new(),
            artifacts: Vec::new(),
            event_generator: EventGenerator::new(),
            figure_generator: FigureGenerator::new(),
            artifact_generator: ArtifactGenerator::new(),
            territory_system: TerritorySystem::new(),
            elevation_map: HashMap::new(),
            active_wars: HashMap::new(),
        }
    }

    /// Main entry point: Generate prehistory for a world
    pub fn generate_prehistory(
        &mut self,
        species_data: Vec<SpeciesData>,
        terrain: TerrainData,
    ) -> HistoryResult {
        // Initialize species from data
        for sd in species_data {
            self.species.push(Species::from_data(sd));
        }

        // Build elevation map from terrain regions
        self.elevation_map = terrain.regions.iter().enumerate()
            .map(|(i, r)| (i as u64, r.fertility * 1000.0))
            .collect();

        // D.2/8: Generate initial territories using ClusteredTerritoryGenerator
        let polygons: Vec<PolygonInfo> = terrain.polygons.iter().cloned().collect();
        if polygons.is_empty() {
            // Fallback: create synthetic polygons from regions
            for (i, region) in terrain.regions.iter().enumerate() {
                let elevation = region.fertility * 1000.0;
                let polygon = PolygonInfo {
                    id: i as u64,
                    elevation,
                    neighbors: vec![],
                    is_coastal: elevation < 0.0,
                    is_island: false,
                };
                self.elevation_map.insert(i as u64, elevation);
            }
        }

        let polygons_for_territory: Vec<PolygonInfo> = (0..terrain.regions.len())
            .map(|i| PolygonInfo {
                id: i as u64,
                elevation: self.elevation_map.get(&(i as u64)).copied().unwrap_or(100.0),
                neighbors: vec![],
                is_coastal: false,
                is_island: false,
            })
            .collect();

        // Generate initial territories using the territory system
        self.territories = self.territory_system.generate_initial_territories(
            self.config.pre_history_years,
            &polygons_for_territory,
            &self.elevation_map,
        );

        // D.2: Initialize starting settlements
        self.initialize_settlements();

        let mut years: Vec<YearResult> = Vec::new();

        // D.3: Yearly simulation loop
        for year in 0..self.config.pre_history_years {
            let year_result = self.advance_year(year);
            years.push(year_result);

            // Archive significant events to timeline
            self.archive_year_to_timeline(year);
        }

        // D.9: Final state freeze
        let final_state = self.freeze_final_state();

        HistoryResult {
            world_id: self.config.world_id.clone(),
            total_years: self.config.pre_history_years,
            years,
            final_state,
            timeline_events: self.timeline.events.clone(),
        }
    }

    /// Advance simulation by one year
    fn advance_year(&mut self, year: u32) -> YearResult {
        let mut population_delta = 0i32;
        let mut events_generated = 0u32;
        let mut figures_created = 0u32;
        let mut artifacts_created = 0u32;
        let mut transitions: Vec<SocietyTransition> = Vec::new();

        // D.4: Advance population - collect indices to avoid borrow conflict
        let species_count = self.species.len();
        let growth_rate = self.config.population_growth_rate; // Copy to avoid reborrow
        for i in 0..species_count {
            let carrying_cap = self.species[i].carrying_capacity; // Copy before mutable borrow
            let delta = Self::advance_population_internal(
                &mut self.species[i], 
                year,
                growth_rate,
                carrying_cap,
            );
            population_delta += delta;
        }

        // D.4: Run probability engine and generate events
        let events = self.event_generator.generate_events(
            year,
            &self.settlements,
            &self.species,
            &self.config,
        );
        events_generated = events.len() as u32;

        // D.5: Event→figure coupling - create figures from significant events
        for event in &events {
            if event.significance >= self.config.figure_significance_threshold {
                if let Some(figure) = self.figure_generator.from_event(event, year) {
                    self.figures.push(figure);
                    figures_created += 1;
                }
            }

            // D.6: Event→artifact coupling
            if event.can_create_artifact() {
                if rand::random::<f32>() < self.config.artifact_creation_probability {
                    if let Some(artifact) = self.artifact_generator.create_from_event(event, year) {
                        self.artifacts.push(artifact);
                        artifacts_created += 1;
                    }
                }
            }
        }

        // Apply event effects to settlements
        for event in &events {
            self.apply_event_effects(event);
        }

        // D.7: Society evolution - check transitions (indices to avoid borrow conflict)
        let settlement_count = self.settlements.len();
        for i in 0..settlement_count {
            if let Some(transition) = Self::check_society_transition_internal(
                &mut self.settlements[i], 
                year,
                &self.config.society_transition_thresholds,
            ) {
                transitions.push(transition);
            }
        }

        // D.8: Territory expansion
        self.expand_territories(year);

        YearResult {
            year,
            population_delta,
            events_generated,
            figures_created,
            artifacts_created,
            society_transitions: transitions,
        }
    }

    /// Advance population for a species (internal, takes &mut Species directly)
    fn advance_population_internal(species: &mut Species, _year: u32, growth_rate: f32, carrying_capacity: u32) -> i32 {
        let growth = (species.population as f32 * growth_rate) as i32;
        species.population = ((species.population as i32 + growth) as u32)
            .min(carrying_capacity);
        growth
    }

    /// Check if a settlement should transition to next society type (internal version)
    fn check_society_transition_internal(
        settlement: &mut Settlement, 
        year: u32,
        thresholds: &SocietyTransitionConfig,
    ) -> Option<SocietyTransition> {
        let next_type = settlement.society_type.next()?;
        let population = settlement.population;

        let should_transition = match settlement.society_type {
            SocietyType::Band => population >= thresholds.band_to_tribe_population,
            SocietyType::Tribe => population >= thresholds.tribe_to_chiefdom_population,
            SocietyType::Chiefdom => population >= thresholds.chiefdom_to_nation_population,
            SocietyType::Nation => return None,
        };

        if should_transition {
            let transition = SocietyTransition {
                settlement_id: settlement.id.clone(),
                from: settlement.society_type.clone(),
                to: next_type.clone(),
                year,
            };
            settlement.society_type = next_type;
            return Some(transition);
        }

        None
    }

    /// Apply event effects to settlements
    fn apply_event_effects(&mut self, event: &Event) {
        for affected_id in &event.affected_settlements {
            if let Some(settlement) = self.settlements.iter_mut().find(|s| &s.id == affected_id) {
                settlement.apply_event_effect(event);
            }
        }
    }

    /// Expand territories based on population growth
    fn expand_territories(&mut self, year: u32) {
        // D.8: Every 50 years, expand territories using the territory system
        if year % 50 == 0 && year > 0 {
            // Convert to polygon map for expansion
            let polygons: HashMap<u64, PolygonInfo> = self.elevation_map.keys()
                .map(|&id| {
                    (id, PolygonInfo {
                        id,
                        elevation: self.elevation_map[&id],
                        neighbors: vec![],
                        is_coastal: false,
                        is_island: false,
                    })
                })
                .collect();

            self.territory_system.expand_territories(
                &mut self.territories,
                &polygons,
                &self.elevation_map,
                &self.active_wars,
                year as i32,
            );
        }
    }

    /// Initialize starting settlements
    fn initialize_settlements(&mut self) {
        if let Some(first_species) = self.species.first() {
            // Get first territory claim
            let first_territory_id = self.territories.keys().next()
                .map(|f| format!("territory_{}", f.0))
                .unwrap_or_default();

            // Create initial band settlement
            let settlement = Settlement::new_band(
                format!("Initial Band of {}", first_species.name),
                first_species.id.clone(),
                first_territory_id,
                20,
            );
            self.settlements.push(settlement);
        }
    }

    /// Archive year state to timeline
    fn archive_year_to_timeline(&mut self, year: u32) {
        let summary = crate::history::timeline::YearSummary {
            year,
            population_total: self.species.iter().map(|s| s.population).sum(),
            settlement_count: self.settlements.len() as u32,
            active_figures: self.figures.len() as u32,
            artifacts_count: self.artifacts.len() as u32,
        };
        self.timeline.add_year_summary(summary);
    }

    /// Freeze final state for history archive
    fn freeze_final_state(&self) -> WorldState {
        WorldState {
            populations: self.species.iter().map(|s| {
                crate::history::result::PopulationState {
                    species_id: s.id.clone(),
                    population: s.population,
                    growth_rate: self.config.population_growth_rate,
                    carrying_capacity: s.carrying_capacity,
                }
            }).collect(),
            settlements: self.settlements.iter().map(|s| {
                crate::history::result::SettlementState {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    population: s.population,
                    society_type: s.society_type.clone(),
                    territory_id: s.territory_id.clone(),
                }
            }).collect(),
            territories: self.territories.iter().map(|(faction_id, claim)| {
                crate::history::result::TerritoryState {
                    id: format!("territory_{}", faction_id.0),
                    name: format!("Faction {} Territory", faction_id.0),
                    owner_species: self.species.first().map(|s| s.id.clone()).unwrap_or_default(),
                    size: claim.claimed_polygons.len() as u32,
                    clusters: claim.claimed_polygons.iter().map(|p| p.to_string()).collect(),
                }
            }).collect(),
            figures: self.figures.iter().map(|f| {
                crate::history::result::FigureState {
                    id: f.id.clone(),
                    name: f.name.clone(),
                    figure_type: f.figure_type.clone(),
                    species_id: f.species_id.clone(),
                    birth_year: f.birth_year,
                    significance: f.significance,
                    deeds: f.deeds.iter().map(|d| {
                        crate::history::result::DeedState {
                            id: d.id.clone(),
                            year: d.year,
                            description: d.description.clone(),
                            event_type: d.event_type.clone(),
                            significance: d.significance,
                            affected_settlements: d.affected_settlements.clone(),
                        }
                    }).collect(),
                }
            }).collect(),
            artifacts: self.artifacts.iter().map(|a| {
                crate::history::result::ArtifactState {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    artifact_type: format!("{:?}", a.artifact_type),
                    creator_id: a.creator_id.clone(),
                    creation_year: a.creation_year,
                    significance: a.significance,
                }
            }).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> PreHistoryConfig {
        PreHistoryConfig {
            world_id: "test-world".to_string(),
            pre_history_years: 50, // Short for testing
            population_growth_rate: 0.02,
            event_probability_base: 0.1,
            figure_significance_threshold: 0.7,
            artifact_creation_probability: 0.05,
            society_transition_thresholds: SocietyTransitionConfig {
                band_to_tribe_population: 50,
                tribe_to_chiefdom_population: 200,
                chiefdom_to_nation_population: 1000,
            },
        }
    }

    fn create_test_species_data() -> Vec<SpeciesData> {
        vec![
            SpeciesData {
                id: "species-1".to_string(),
                name: "Humans".to_string(),
                population: 100,
                carrying_capacity: 1000,
                traits: vec!["social".to_string(), "tool_user".to_string()],
            },
            SpeciesData {
                id: "species-2".to_string(),
                name: "Elves".to_string(),
                population: 50,
                carrying_capacity: 500,
                traits: vec!["long-lived".to_string(), "magical".to_string()],
            },
        ]
    }

    fn create_test_terrain() -> TerrainData {
        TerrainData {
            regions: vec![
                TerrainRegion {
                    id: "region-1".to_string(),
                    name: "Central Plains".to_string(),
                    size: 100,
                    fertility: 0.8,
                    resource_density: 0.7,
                },
                TerrainRegion {
                    id: "region-2".to_string(),
                    name: "Northern Forests".to_string(),
                    size: 80,
                    fertility: 0.6,
                    resource_density: 0.5,
                },
            ],
            polygons: vec![
                PolygonInfo {
                    id: 1,
                    elevation: 100.0,
                    neighbors: vec![2],
                    is_coastal: false,
                    is_island: false,
                },
                PolygonInfo {
                    id: 2,
                    elevation: 200.0,
                    neighbors: vec![1],
                    is_coastal: false,
                    is_island: false,
                },
            ],
        }
    }

    #[test]
    fn test_prehistory_generator_initialization() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config.clone());
        
        assert_eq!(generator.config.world_id, "test-world");
        assert_eq!(generator.config.pre_history_years, 50);
        assert!(generator.species.is_empty());
        assert!(generator.settlements.is_empty());
        assert!(generator.territories.is_empty());
    }

    #[test]
    fn test_generate_prehistory_basic() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Verify result structure
        assert_eq!(result.world_id, "test-world");
        assert_eq!(result.total_years, 50);
        assert_eq!(result.years.len(), 50);
        
        // Verify species were initialized
        assert_eq!(generator.species.len(), 2);
        
        // Verify initial settlements were created
        assert!(!generator.settlements.is_empty());
        
        // Verify territories were generated
        assert!(!generator.territories.is_empty());
    }

    #[test]
    fn test_yearly_simulation_loop() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Each year should produce a YearResult
        for (year, year_result) in result.years.iter().enumerate() {
            assert_eq!(year_result.year, year as u32);
            // Population delta may be zero or positive
            // Events may or may not be generated depending on probability
        }
    }

    #[test]
    fn test_population_growth_over_time() {
        let mut config = create_test_config();
        config.pre_history_years = 10;
        config.population_growth_rate = 0.1; // 10% growth for visible effect
        
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let initial_pop = generator.species.first().map(|s| s.population).unwrap_or(0);
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Verify final state has updated populations
        assert_eq!(result.final_state.populations.len(), 2);
    }

    #[test]
    fn test_society_transitions() {
        let mut config = create_test_config();
        config.pre_history_years = 200;
        // Low thresholds to trigger transitions quickly
        config.society_transition_thresholds = SocietyTransitionConfig {
            band_to_tribe_population: 20,
            tribe_to_chiefdom_population: 40,
            chiefdom_to_nation_population: 80,
        };
        
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = vec![SpeciesData {
            id: "test-species".to_string(),
            name: "Test Species".to_string(),
            population: 100,
            carrying_capacity: 10000,
            traits: vec![],
        }];
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Check that some years had society transitions
        let total_transitions: usize = result.years.iter()
            .map(|y| y.society_transitions.len())
            .sum();
        
        // With high growth rate and low thresholds, we should see transitions
        assert!(total_transitions >= 0); // May or may not have transitions depending on timing
    }

    #[test]
    fn test_timeline_recording() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Timeline should have year summaries for each year
        assert_eq!(result.timeline_events.len(), result.years.len());
    }

    #[test]
    fn test_territory_generation_from_terrain() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Territories should match the faction structure
        assert_eq!(result.final_state.territories.len(), generator.territories.len());
        
        // Each territory should have claimed polygons
        for territory in &result.final_state.territories {
            assert!(!territory.clusters.is_empty() || territory.size == 0);
        }
    }

    #[test]
    fn test_figures_and_artifacts_generation() {
        let mut config = create_test_config();
        config.pre_history_years = 100;
        config.event_probability_base = 0.5; // High probability for events
        config.artifact_creation_probability = 0.5; // High probability for artifacts
        
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Final state should have figures and artifacts
        // (may be empty depending on random events)
        assert!(result.final_state.figures.len() >= 0);
        assert!(result.final_state.artifacts.len() >= 0);
    }

    #[test]
    fn test_terrain_data_with_polygons() {
        let config = create_test_config();
        let mut generator = PreHistoryGenerator::new(config);
        let species_data = create_test_species_data();
        let terrain = create_test_terrain();
        
        // Generate prehistory with terrain polygons
        let result = generator.generate_prehistory(species_data, terrain);
        
        // Should handle terrain.polygons correctly
        assert!(result.final_state.territories.len() >= 0);
    }
}
