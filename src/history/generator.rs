//! HistoryGenerator - Entry Point for World History Simulation
//!
//! This module provides the main orchestrator for generating world history.
//! It wires together all Phase 2 subsystems and drives the simulation forward.
//!
//! ## HistoryGenerator Overview
//!
//! The HistoryGenerator is the central coordinator that:
//! - Loads species templates from plugin files
//! - Finds suitable settlement locations on terrain
//! - Spawns initial settlements with species assignment
//! - Forms societies from settlements
//! - Runs the population growth simulation for configured years
//! - Generates notable figures from significant events
//! - Creates artifacts tied to events and figures
//! - Records the complete event timeline

use std::collections::HashMap;
use uuid::Uuid;

use crate::artifacts::{Artifact, ArtifactCategory, ArtifactStore};
use crate::events::{Event, EventBuilder, EventStore, EventType};
use crate::figures::{FigureGenerator, FigureGeneratorConfig, FigureStore};
use crate::history::population::PopulationGrowthService;
use crate::history::Society;
use crate::history::SocietyRegistry;
use crate::settlements::{SettlementConfig, SettlementGenerator};
use crate::species::loader::SpeciesLoader;
use crate::species::{SpeciesData, SpeciesId};
use crate::terrain::{BiomeType, ClimateZone};
use crate::types::{GeoLocation, HistoricalTime, Settlement, SettlementType};
use crate::util::Rng;
use crate::World;

/// Configuration for history generation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Number of pre-history years to simulate.
    /// Default: 500 years.
    pub pre_history_years: i32,

    /// Seed for deterministic generation.
    /// If None, uses world seed.
    pub seed: Option<u64>,

    /// Initial number of settlements to spawn.
    /// Default: computed from world size (0.5 per 1000 cells).
    pub initial_settlement_count: Option<usize>,

    /// Minimum population for initial settlements.
    pub min_initial_population: u64,

    /// Maximum population for initial settlements.
    pub max_initial_population: u64,

    /// Whether to generate artifacts.
    /// Default: true.
    pub generate_artifacts: bool,

    /// Whether to generate notable figures.
    /// Default: true.
    pub generate_figures: bool,

    /// Species template directory path.
    /// Default: "species_templates/".
    pub species_template_dir: Option<String>,

    /// Population simulation config.
    pub population_config: GrowthConfig,

    /// Figure generation config.
    pub figure_config: FigureGeneratorConfig,

    /// Cataclysm cap (no more than N cataclysms per world).
    /// Default: 3.
    pub cataclysm_cap: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            pre_history_years: 500,
            seed: None,
            initial_settlement_count: None,
            min_initial_population: 50,
            max_initial_population: 500,
            generate_artifacts: true,
            generate_figures: true,
            species_template_dir: None,
            population_config: GrowthConfig::default(),
            figure_config: FigureGeneratorConfig::default(),
            cataclysm_cap: 3,
        }
    }
}

/// Population growth configuration.
#[derive(Debug, Clone)]
pub struct GrowthConfig {
    pub base_reproduction_rate: f32,
    pub max_growth_rate: f32,
}

impl Default for GrowthConfig {
    fn default() -> Self {
        Self {
            base_reproduction_rate: 0.01, // 1% base rate
            max_growth_rate: 0.05,        // 5% max rate
        }
    }
}

/// Result of history generation.
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Generated settlements.
    pub settlements: Vec<Settlement>,

    /// Formed societies.
    pub societies: SocietyRegistry,

    /// Generated events in chronological order.
    pub events: EventStore,

    /// Generated notable figures.
    pub figures: FigureStore,

    /// Generated artifacts.
    pub artifacts: ArtifactStore,

    /// Statistics about the generation.
    pub stats: GenerationStats,
}

/// Statistics about the history generation.
#[derive(Debug, Clone)]
pub struct GenerationStats {
    /// Total years simulated.
    pub years_simulated: i32,

    /// Number of settlements generated.
    pub settlement_count: usize,

    /// Number of societies formed.
    pub society_count: usize,

    /// Number of events generated.
    pub event_count: usize,

    /// Number of figures generated.
    pub figure_count: usize,

    /// Number of artifacts generated.
    pub artifact_count: usize,

    /// Population at end of simulation.
    pub final_population: u64,

    /// Number of cataclysms triggered.
    pub cataclysm_count: usize,
}

impl GenerationStats {
    pub fn new() -> Self {
        Self {
            years_simulated: 0,
            settlement_count: 0,
            society_count: 0,
            event_count: 0,
            figure_count: 0,
            artifact_count: 0,
            final_population: 0,
            cataclysm_count: 0,
        }
    }
}

impl Default for GenerationStats {
    fn default() -> Self {
        Self::new()
    }
}

/// HistoryGenerator - Main orchestrator for world history simulation.
///
/// This struct holds references to all Phase 2 subsystems and orchestrates
/// the generation of world history from initial settlement through
/// centuries of population growth, societal evolution, and historical events.
#[derive(Debug, Clone)]
pub struct HistoryGenerator {
    /// Species template loader.
    species_loader: SpeciesLoader,

    /// Seed for deterministic RNG.
    seed: u64,

    /// Figure generator.
    figure_generator: FigureGenerator,

    /// Population simulator.
    population_simulator: PopulationGrowthService,
}

impl HistoryGenerator {
    /// Create a new HistoryGenerator with default configuration.
    pub fn new() -> Self {
        Self::with_config(GeneratorConfig::default(), None)
    }

    /// Create a HistoryGenerator with explicit configuration.
    pub fn with_config(config: GeneratorConfig, seed: Option<u64>) -> Self {
        let effective_seed = seed.unwrap_or(42);

        Self {
            species_loader: SpeciesLoader::new(),
            seed: effective_seed,
            figure_generator: FigureGenerator::new(config.figure_config.clone()),
            population_simulator: PopulationGrowthService::new(effective_seed),
        }
    }

    /// Generate world history from a world and configuration.
    ///
    /// # Steps
    ///
    /// 1. Load species templates from configured directory
    /// 2. Find suitable settlement locations based on terrain
    /// 3. Spawn initial settlements with species assignment
    /// 4. Form initial societies from settlements
    /// 5. Run population simulation for pre_history_years
    /// 6. Generate events, figures, and artifacts throughout
    /// 7. Return complete generation result with all entities
    #[allow(unused_variables)]
    pub fn generate(&mut self, world: &World, config: GeneratorConfig) -> GenerationResult {
        let start_year = 0;
        let end_year = config.pre_history_years;
        let world_id = world.id.to_uuid();

        // Create RNG for this generation run
        let mut rng = Rng::new(self.seed);

        // Step 1: Load species templates
        let species_data = self.load_species(&config);

        // Step 2: Find suitable settlement locations
        let terrain_data = self.extract_terrain_data();
        let settlement_sites = self.find_settlement_sites(&terrain_data, &config, &mut rng);

        // Step 3: Spawn initial settlements with species assignment
        let settlements =
            self.spawn_settlements(world_id, settlement_sites, &species_data, &config, &mut rng);

        // Step 4: Form initial societies
        let mut societies = self.form_initial_societies(&settlements, start_year, &species_data);

        // Initialize event store
        let mut event_store = EventStore::new();

        // Step 5: Run history simulation
        let simulation_result = self.run_simulation(
            world_id,
            &mut societies,
            &settlements,
            start_year,
            end_year,
            &config,
            &mut rng,
        );

        // Add simulation events
        let sim_events = simulation_result.events;
        for event in sim_events {
            event_store.add(event);
        }

        // Step 6: Generate figures and artifacts
        let figures = if config.generate_figures {
            self.generate_figures(world_id, &event_store, &settlements, &mut rng)
        } else {
            FigureStore::new()
        };

        let artifacts = if config.generate_artifacts {
            self.generate_artifacts(world_id, &event_store)
        } else {
            ArtifactStore::new()
        };

        // Build stats
        let mut stats = GenerationStats::new();
        stats.years_simulated = end_year - start_year;
        stats.settlement_count = settlements.len();
        stats.society_count = societies.societies.len();
        stats.event_count = event_store.len();
        stats.figure_count = figures.count(&world_id);
        stats.artifact_count = artifacts.len();
        stats.final_population = simulation_result.final_population;

        // Build result
        GenerationResult {
            settlements,
            societies,
            events: event_store,
            figures,
            artifacts,
            stats,
        }
    }

    /// Load species templates from configured directory.
    fn load_species(&self, _config: &GeneratorConfig) -> SpeciesData {
        // Use default species if no custom template directory
        SpeciesData::default_species()
    }

    /// Extract terrain data from world for settlement placement.
    fn extract_terrain_data(&self) -> TerrainData {
        // Extract elevation, biome, and climate grids from world
        // For now, use defaults based on world dimensions
        let width = 64;
        let height = 64;

        TerrainData {
            elevation_grid: vec![0.6; width * height],
            biome_grid: vec![BiomeType::TemperateGrassland; width * height],
            climate_grid: vec![ClimateZone::Temperate; width * height],
            sea_level: 0.5,
            width,
            height,
        }
    }

    /// Find suitable settlement sites on terrain.
    fn find_settlement_sites(
        &mut self,
        terrain: &TerrainData,
        _config: &GeneratorConfig,
        rng: &mut Rng,
    ) -> Vec<SettlementSite> {
        let mut generator =
            SettlementGenerator::new(SettlementConfig::default(), rng.next() as u64);

        let result = generator.generate_with_species(
            &terrain.elevation_grid,
            &terrain.biome_grid,
            &terrain.climate_grid,
            &SpeciesData::default_species(),
            terrain.sea_level,
            terrain.width,
            terrain.height,
            None,
        );

        result
            .settlements
            .into_iter()
            .map(|s| SettlementSite {
                id: s.id.to_uuid(),
                name: s.name,
                x: 0,
                y: 0,
                biome: BiomeType::TemperateGrassland,
                population: s.population.unwrap_or(100),
                species_id: s.species_id.unwrap_or(SpeciesId::Human),
            })
            .collect()
    }

    /// Spawn initial settlements from site selection.
    fn spawn_settlements(
        &mut self,
        _world_id: Uuid,
        sites: Vec<SettlementSite>,
        species_data: &SpeciesData,
        _config: &GeneratorConfig,
        rng: &mut Rng,
    ) -> Vec<Settlement> {
        let mut settlements = Vec::new();

        for (i, site) in sites.into_iter().enumerate() {
            // Determine settlement type based on population
            let settlement_type = if site.population < 100 {
                SettlementType::Hamlet
            } else if site.population < 1000 {
                SettlementType::Village
            } else if site.population < 10000 {
                SettlementType::Town
            } else {
                SettlementType::City
            };

            // Generate culturally-appropriate name
            let name = species_data.generate_name(site.species_id, rng);

            // Create location
            let lat = (i as f64 * 0.5) % 90.0;
            let lon = (i as f64 * 0.7) % 180.0;
            let location = GeoLocation {
                latitude: lat,
                longitude: lon,
                elevation_m: None,
            };

            // Create settlement
            let mut settlement = Settlement::with_details(
                site.id,
                Some(i as u32),
                name,
                settlement_type,
                site.population,
                location,
                Some(format!(
                    "Initial {} settlement",
                    species_data
                        .get(site.species_id)
                        .map(|s| s.name.as_ref())
                        .unwrap_or("Unknown")
                )),
            );

            settlement.species_id = Some(site.species_id);
            settlement.founded_year = Some(0);
            settlement.society_id = None;

            settlements.push(settlement);
        }

        settlements
    }

    /// Form initial societies from settlements.
    fn form_initial_societies(
        &mut self,
        settlements: &[Settlement],
        year: i32,
        species_data: &SpeciesData,
    ) -> SocietyRegistry {
        let mut registry = SocietyRegistry::new();

        for settlement in settlements {
            let species_id = settlement.species_id.unwrap_or(SpeciesId::Human);
            let population = settlement.population.unwrap_or(100);
            let name = format!(
                "{} of {}",
                species_data
                    .get(species_id)
                    .map(|s| s.name.as_ref())
                    .unwrap_or("Unknown"),
                settlement.name
            );

            let society = Society::from_settlement(
                settlement.id.to_uuid(),
                name,
                species_id,
                population,
                year,
            );

            registry.register(society);
        }

        registry
    }

    /// Run the population simulation for configured years.
    fn run_simulation(
        &mut self,
        world_id: Uuid,
        societies: &mut SocietyRegistry,
        settlements: &[Settlement],
        start_year: i32,
        end_year: i32,
        _config: &GeneratorConfig,
        rng: &mut Rng,
    ) -> SimulationOutput {
        let mut events = Vec::new();
        let mut current_year = start_year;
        let mut final_population = 0u64;

        // Initial settlement founding events
        for settlement in settlements {
            let event = Event::settlement_founded(
                world_id,
                settlement.id.to_uuid(),
                &settlement.name,
                HistoricalTime::year(current_year),
            );
            events.push(event);
        }

        // Simulate year by year
        // For performance, we aggregate simulation rather than per-year
        let year_step = 10; // Simulate in 10-year chunks

        // Get societies into a mutable hashmap
        let mut societies_map: HashMap<Uuid, Society> = societies.societies.drain().collect();

        while current_year < end_year {
            // Run population simulation for each society
            for (id, society) in societies_map.iter_mut() {
                let growth_rate_f64 = society.growth_rate(0.01) as f64; // 1% base rate
                let population_f = society.population as f64;
                let growth_amount =
                    (population_f * growth_rate_f64 * (year_step as f64)).round() as i64;

                let new_population = (society.population as i64 + growth_amount).max(10) as u64;
                society.update_population(new_population);

                // Record population history
                society.record_population(current_year + year_step, new_population);

                // Check for society type transitions
                if let Some(_old_type) = society.check_transition() {
                    let event = EventBuilder::new(format!("{} Evolved", society.name))
                        .event_type(EventType::CulturalAchievement)
                        .time(HistoricalTime::year(current_year + year_step))
                        .location(*id)
                        .significance(0.6)
                        .build(world_id);
                    events.push(event);
                }

                // Random chance of events based on population density
                let random_val = (rng.next() as f64) / (u32::MAX as f64);
                if random_val < 0.1 * (year_step as f64 / 100.0) {
                    let event_type = self.random_population_event(rng);
                    let event = EventBuilder::new(format!(
                        "{} - Year {}",
                        event_type.name(),
                        current_year + year_step
                    ))
                    .event_type(event_type)
                    .time(HistoricalTime::year(current_year + year_step))
                    .location(*id)
                    .significance(0.3 + random_val as f32 * 0.4)
                    .build(world_id);
                    events.push(event);
                }
            }

            current_year += year_step;
        }

        // Calculate final population
        final_population = societies_map.values().map(|s| s.population).sum();

        // Store back to registry
        for (id, society) in societies_map {
            societies.societies.insert(id, society);
        }

        SimulationOutput {
            events,
            final_population,
        }
    }

    /// Generate a random population-related event type.
    fn random_population_event(&mut self, rng: &mut Rng) -> EventType {
        let events = [
            EventType::Festival,
            EventType::Migration,
            EventType::Plague,
            EventType::Famine,
            EventType::Discovery,
            EventType::CulturalAchievement,
        ];
        let idx = (rng.next() as usize) % events.len();
        events[idx]
    }

    /// Generate notable figures from events.
    fn generate_figures(
        &mut self,
        world_id: Uuid,
        events: &EventStore,
        settlements: &[Settlement],
        rng: &mut Rng,
    ) -> FigureStore {
        let mut store = FigureStore::new();

        let settlement_ids: Vec<Uuid> = settlements.iter().map(|s| s.id.to_uuid()).collect();

        let cultures: Vec<String> = settlements.iter().map(|s| s.name.clone()).collect();

        let figures = self.figure_generator.generate_from_events(
            world_id,
            events.events(),
            &settlement_ids,
            &cultures,
            rng,
        );

        for figure in figures {
            store.add(figure);
        }

        store
    }

    /// Generate artifacts from significant events.
    fn generate_artifacts(&self, _world_id: Uuid, events: &EventStore) -> ArtifactStore {
        let mut store = ArtifactStore::new();

        // Filter significant events
        let significant_events: Vec<_> = events
            .events()
            .iter()
            .filter(|e| e.significance.unwrap_or(0.0) >= 0.6)
            .cloned()
            .collect();

        for event in significant_events {
            // Only create artifact for certain event types
            if let Some(category) = self.artifact_category_from_event(&event.event_type) {
                let artifact = Artifact::from_event(&event, category);
                store.add(artifact);
            }
        }

        store
    }

    /// Determine artifact category from event type.
    fn artifact_category_from_event(&self, event_type: &EventType) -> Option<ArtifactCategory> {
        match event_type {
            EventType::CulturalAchievement => Some(ArtifactCategory::Magical),
            EventType::MonumentCompleted => Some(ArtifactCategory::Monument),
            EventType::ReligiousEvent => Some(ArtifactCategory::Sacred),
            EventType::Battle | EventType::Victory => Some(ArtifactCategory::Trophy),
            EventType::Conquest => Some(ArtifactCategory::CrownJewel),
            EventType::ReligiousReformation => Some(ArtifactCategory::Relic),
            EventType::Invention => Some(ArtifactCategory::Magical),
            EventType::Discovery => Some(ArtifactCategory::Document),
            _ => None,
        }
    }
}

impl Default for HistoryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Terrain data extracted from world for settlement placement.
#[derive(Debug, Clone)]
struct TerrainData {
    elevation_grid: Vec<f32>,
    biome_grid: Vec<BiomeType>,
    climate_grid: Vec<ClimateZone>,
    sea_level: f32,
    width: usize,
    height: usize,
}

/// A candidate settlement site.
#[derive(Debug, Clone)]
struct SettlementSite {
    id: Uuid,
    name: String,
    x: usize,
    y: usize,
    biome: BiomeType,
    population: u64,
    species_id: SpeciesId,
}

/// Output from simulation run.
struct SimulationOutput {
    events: Vec<Event>,
    final_population: u64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_config_defaults() {
        let config = GeneratorConfig::default();
        assert_eq!(config.pre_history_years, 500);
        assert!(config.generate_artifacts);
        assert!(config.generate_figures);
        assert_eq!(config.cataclysm_cap, 3);
    }

    #[test]
    fn test_history_generator_creation() {
        let generator = HistoryGenerator::new();
        // Should not panic
        drop(generator);
    }

    #[test]
    fn test_generation_result_empty() {
        // Test that we can create empty result for testing
        let stats = GenerationStats::new();
        assert_eq!(stats.years_simulated, 0);
        assert_eq!(stats.settlement_count, 0);
    }

    #[test]
    fn test_artifact_category_from_event() {
        let generator = HistoryGenerator::new();

        assert_eq!(
            generator.artifact_category_from_event(&EventType::Battle),
            Some(ArtifactCategory::Trophy)
        );
        assert_eq!(
            generator.artifact_category_from_event(&EventType::Conquest),
            Some(ArtifactCategory::CrownJewel)
        );
        assert_eq!(
            generator.artifact_category_from_event(&EventType::SettlementFounded),
            None
        );
    }
}
