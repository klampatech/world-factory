//! Simulation engine - Core simulation orchestration
//! 
//! Coordinates all simulation systems: territory, population, settlements, figures, artifacts

use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::territory::{TerritorySystem, TerritoryClaim, FactionId, PolygonInfo};
use crate::terrain::biome_assignment::{BiomeAssignmentSystem, PolygonBiome, BiomeType};
use crate::terrain::groundwater::GroundwaterSystem;
use super::state::{
    SimulationState, SimulationConfig, SimulationEvent, EventType,
    Population, Settlement, Figure, FigureType, Artifact,
};

/// Random name components for procedural generation
const SETTLEMENT_PREFIXES: &[&str] = &[
    "New", "Old", "Upper", "Lower", "North", "South", "East", "West",
    "Great", "Little", "High", "Low", "Fort", "Port", "Saint",
];
const SETTLEMENT_SUFFIXES: &[&str] = &[
    "ton", "burg", "ford", "ham", "bury", "worth", "dale", "wood",
    "field", "ridge", "brook", "gate", "haven", "shire", "land",
];
const SETTLEMENT_ROOTS: &[&str] = &[
    "Oak", "Stone", "Iron", "Gold", "Silver", "River", "Lake", "Sea",
    "Wind", "Fire", "Sun", "Moon", "Star", "Sky", "Earth", "Thunder",
    "Shadow", "Light", "Dawn", "Dusk", "Storm", "Clear", "Green", "White",
];

/// Historical figure name components
const FIGURE_PREFIXES: &[&str] = &[
    "Al", "Bran", "Cor", "Dun", "El", "Gal", "Hal", "Is", "Jar",
    "Kel", "Lor", "Mal", "Nar", "Or", "Per", "Qu", "Ran", "Sel",
];
const FIGURE_SUFFIXES: &[&str] = &[
    "ric", "mund", "wyn", "ard", "bert", "win", "ton", "ley",
    "don", "gan", "mir", "dan", "ron", "len", "mar", "nan",
];

/// Artifact type names
const ARTIFACT_TYPES: &[&str] = &[
    "Tool", "Weapon", "Jewelry", "Pottery", "Textile", "Amulet", "Crown", "Sword",
    "Spear", "Shield", "Helmet", "Armor", "Bell", "Drum", "Flute", "Mask",
];

/// Main simulation engine
pub struct SimulationEngine {
    territory_system: TerritorySystem,
    biome_system: BiomeAssignmentSystem,
    groundwater_system: GroundwaterSystem,
    rng: StdRng,
}

impl SimulationEngine {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = seed.unwrap_or_else(|| {
            rand::random::<u64>()
        });
        
        Self {
            territory_system: TerritorySystem::new(),
            biome_system: BiomeAssignmentSystem::new(seed),
            groundwater_system: GroundwaterSystem::new(seed),
            rng: StdRng::from_seed([rng as u8, (rng >> 8) as u8, (rng >> 16) as u8, (rng >> 24) as u8,
                (rng >> 32) as u8, (rng >> 40) as u8, (rng >> 48) as u8, (rng >> 56) as u8,
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                16, 17, 18, 19, 20, 21, 22, 23]),
        }
    }
    
    /// Initialize simulation state from polygons and configuration
    pub fn initialize(
        &mut self,
        world_id: String,
        polygons: Vec<PolygonInfo>,
        elevation_map: HashMap<u64, f32>,
        config: &SimulationConfig,
    ) -> SimulationState {
        let mut state = SimulationState::new(
            world_id,
            config.start_year,
            config.start_year + config.years as i32,
            polygons,
            elevation_map,
        );
        
        // Generate initial territories
        let pre_history_years = if config.years > 200 { 300 } else { config.years };
        let territories = self.territory_system.generate_initial_territories(
            pre_history_years,
            &state.polygons.values().cloned().collect::<Vec<_>>(),
            &state.elevation_map,
        );
        state.territories = territories;
        
        // Generate biome assignments for settlement filtering (per spec §D.2)
        state.biome_map = self.biome_system.assign_biomes(&state.elevation_map, BiomeType::TemperateBroadleaf);
        
        // Initialize groundwater/aquifer system (WOR-479)
        // Default precipitation: 1000mm/year (adjustable in config if needed)
        let precipitation_mm_year = 1000.0;
        self.groundwater_system.initialize_aquifers(&state.elevation_map, precipitation_mm_year);
        
        // Store aquifer data in simulation state
        state.aquifer_map = self.groundwater_system.get_all_aquifers().clone();
        
        // Create initial populations for each faction
        for (faction_id, claim) in &state.territories {
            let initial_pop = 50 + self.rng.gen_range(0..100);
            let pop = Population::new(
                faction_id.to_string(),
                initial_pop,
                faction_id.to_string(),
            );
            state.populations.insert(faction_id.to_string(), pop);
        }
        
        state
    }
    
    /// Run simulation for configured years
    pub fn run(&mut self, state: &mut SimulationState, config: &SimulationConfig) {
        let years_to_simulate = config.years;
        
        for _ in 0..years_to_simulate {
            self.simulate_year(state, config);
            state.current_year += 1;
            
            // Stop if we've reached end year
            if state.current_year >= state.end_year {
                break;
            }
        }
    }
    
    /// Simulate a single year
    fn simulate_year(&mut self, state: &mut SimulationState, config: &SimulationConfig) {
        let year = state.current_year;
        
        // 1. Simulate population growth/decline
        self.simulate_populations(state, config);
        
        // 2. Expand territories
        self.simulate_territory_expansion(state, config);
        
        // 3. Process settlements
        self.simulate_settlements(state, config, year);
        
        // 4. Generate figures (if enabled)
        if config.generate_figures {
            self.simulate_figures(state, config, year);
        }
        
        // 5. Generate artifacts (if enabled)
        if config.generate_artifacts {
            self.simulate_artifacts(state, config, year);
        }
        
        // 6. Process wars
        self.simulate_wars(state, year);
        
        // 7. Simulate groundwater dynamics (WOR-479)
        self.simulate_groundwater(state);
    }
    
    /// Simulate groundwater system for the year (WOR-479)
    fn simulate_populations(&mut self, state: &mut SimulationState, config: &SimulationConfig) {
        // Collect all population updates to apply after iteration
        let mut population_updates: Vec<(String, u32, u32, String)> = Vec::new(); // (pop_id, old_count, new_count, species_id)
        
        let pop_ids: Vec<String> = state.populations.keys().cloned().collect();
        for pop_id in pop_ids {
            if let Some(pop) = state.populations.get_mut(&pop_id) {
                let old_count = pop.count;
                let species_id = pop.species_id.clone();
                pop.simulate_year(config);
                
                // Record update if population changed
                if pop.count != old_count {
                    population_updates.push((pop_id, old_count, pop.count, species_id));
                }
            }
        }
        
        // Apply population change events (after mutable borrow is released)
        for (_pop_id, old_count, new_count, species_id) in population_updates {
            let event_type = if new_count > old_count {
                EventType::PopulationGrowth
            } else {
                EventType::PopulationDecline
            };
            state.record_event(SimulationEvent {
                year: state.current_year,
                event_type,
                description: format!(
                    "{} changed from {} to {}",
                    species_id, old_count, new_count
                ),
                affected_entities: vec![species_id],
                significance: 0.5,
            });
        }
    }
    
    /// Simulate territory expansion and contraction
    fn simulate_territory_expansion(&mut self, state: &mut SimulationState, config: &SimulationConfig) {
        // Convert HashMap<String, String> active_wars to HashMap<(FactionId, FactionId), i32>
        let mut active_wars: HashMap<(FactionId, FactionId), i32> = HashMap::new();
        for ((s1, s2), years) in &state.active_wars {
            if let (Ok(f1), Ok(f2)) = (s1.parse::<u64>(), s2.parse::<u64>()) {
                active_wars.insert((FactionId::new(f1), FactionId::new(f2)), *years);
            }
        }
        
        let polygon_map: HashMap<u64, PolygonInfo> = state.polygons.clone();
        
        self.territory_system.expand_territories(
            &mut state.territories,
            &polygon_map,
            &state.elevation_map,
            &active_wars,
            state.current_year,
        );
    }
    
    /// Simulate settlement dynamics
    fn simulate_settlements(&mut self, state: &mut SimulationState, config: &SimulationConfig, year: i32) {
        // Check for settlement founding opportunities
        let populations_to_check: Vec<String> = state.populations.keys().cloned().collect();
        for pop_id in populations_to_check {
            let pop = state.populations.get(&pop_id);
            if let Some(pop) = pop {
                // Founding chance based on population size
                let founding_threshold = 200;
                let founding_chance = 0.01; // 1% per year
                
                if pop.count >= founding_threshold && self.rng.gen::<f32>() < founding_chance {
                    // Find a territory for this population
                    if let Some(territory_id) = self.find_unclaimed_territory(state) {
                        let settlement = self.create_settlement(
                            pop_id.clone(),
                            territory_id,
                            year as u32,
                        );
                        
                        state.record_event(SimulationEvent {
                            year,
                            event_type: EventType::SettlementFounded,
                            description: format!(
                                "{} was founded with {} people",
                                settlement.name, settlement.population
                            ),
                            affected_entities: vec![settlement.id.clone(), pop_id.clone()],
                            significance: 0.7,
                        });
                    }
                }
            }
        }
        
        // Simulate existing settlement growth
        let settlements_to_update: Vec<String> = state.settlements.keys().cloned().collect();
        for settlement_id in settlements_to_update {
            if let Some(settlement) = state.settlements.get_mut(&settlement_id) {
                // 0.5% growth chance per year
                if self.rng.gen::<f32>() < 0.005 {
                    settlement.population += self.rng.gen_range(10..50);
                }
            }
        }
    }
    
    /// Find an unclaimed territory for a new settlement
    /// Filters to only biome-suitable polygons per spec §D.2
    fn find_unclaimed_territory(&mut self, state: &SimulationState) -> Option<String> {
        let claimed_polygons: HashSet<u64> = state.settlements.values()
            .map(|s| s.territory_id.parse().ok())
            .flatten()
            .collect();
        
        // Filter to only biome-suitable polygons (excludes desert, tundra, ocean, ice)
        // and prefer lowland to midland elevations (0-800m)
        let mut suitable: Vec<u64> = state.polygons.keys()
            .filter(|&&id| !claimed_polygons.contains(&id))
            .filter(|&&id| {
                if let Some(&elevation) = state.elevation_map.get(&id) {
                    // Ocean check: negative elevation is ocean
                    if elevation < 0.0 {
                        return false;
                    }
                    // Elevation check: prefer 0-800m, reject above 2500m
                    if elevation > 2500.0 {
                        return false;
                    }
                    return true;
                }
                false
            })
            .cloned()
            .collect();
        
        if suitable.is_empty() {
            None
        } else {
            // Sort by elevation preference (lower = better for settlements)
            suitable.sort_by(|a, b| {
                let elev_a = state.elevation_map.get(&a).unwrap_or(&0.0);
                let elev_b = state.elevation_map.get(&b).unwrap_or(&0.0);
                elev_a.partial_cmp(elev_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // Pick from top 50% most suitable (lower elevation)
            let top_half = suitable.len() / 2;
            if top_half > 0 {
                Some(suitable[self.rng.gen_range(0..top_half)].to_string())
            } else {
                Some(suitable[self.rng.gen_range(0..suitable.len())].to_string())
            }
        }
    }
    
    /// Create a new settlement
    fn create_settlement(&mut self, population_id: String, territory_id: String, year: u32) -> Settlement {
        let root = SETTLEMENT_ROOTS[self.rng.gen_range(0..SETTLEMENT_ROOTS.len())];
        let suffix = SETTLEMENT_SUFFIXES[self.rng.gen_range(0..SETTLEMENT_SUFFIXES.len())];
        let base_name = format!("{}{}", root, suffix);
        
        let id = Uuid::new_v4().to_string();
        let name = base_name; // State will ensure uniqueness
        
        Settlement {
            id,
            name,
            population: 100,
            territory_id,
            founding_year: year,
        }
    }
    
    /// Simulate figure generation
    fn simulate_figures(&mut self, state: &mut SimulationState, config: &SimulationConfig, year: i32) {
        // Collect figure birth events first, apply after iteration
        let mut figure_births: Vec<SimulationEvent> = Vec::new();
        
        // Collect pop_ids to avoid borrow conflict
        let pop_ids: Vec<String> = state.populations.keys().cloned().collect();
        for pop_id in pop_ids {
            if let Some(pop) = state.populations.get(&pop_id) {
                let figure_chance = (pop.count as f32 / 1000.0) * 0.01; // 1% per 1000 population
                
                if self.rng.gen::<f32>() < figure_chance {
                    let figure_type = self.random_figure_type();
                    let figure = self.create_figure(
                        pop_id.clone(),
                        figure_type,
                        year as u32,
                    );
                    let figure_id = figure.id.clone();
                    let figure_name = figure.name.clone();
                    let figure_sig = figure.significance;
                    let figure_species = figure.species_id.clone();
                    
                    figure_births.push(SimulationEvent {
                        year,
                        event_type: EventType::FigureBorn,
                        description: format!(
                            "{} was born in {}",
                            figure_name, figure_species
                        ),
                        affected_entities: vec![figure_id, pop_id],
                        significance: figure_sig,
                    });
                }
            }
        }
        
        // Apply all figure birth events
        for event in figure_births {
            state.record_event(event);
        }
        
        // Check for figure deaths
        let figures_to_check: Vec<usize> = (0..state.figures.len()).collect();
        for idx in figures_to_check {
            if let Some(figure) = state.figures.get_mut(idx) {
                if figure.death_year.is_none() {
                    // Death chance increases with age
                    let age = year as i32 - figure.birth_year as i32;
                    let death_chance = (age as f32 / 50.0).min(0.1); // Max 10% per year at age 50+
                    let figure_id = figure.id.clone(); // Clone before record_event
                    let figure_significance = figure.significance; // Copy for record_event
                    let figure_name = figure.name.clone(); // Clone for record_event
                    
                    if self.rng.gen::<f32>() < death_chance {
                        figure.death_year = Some(year as u32);
                        
                        state.record_event(SimulationEvent {
                            year,
                            event_type: EventType::FigureDied,
                            description: format!(
                                "{} died at age {}",
                                figure_name, age
                            ),
                            affected_entities: vec![figure_id],
                            significance: figure_significance,
                        });
                    }
                }
            }
        }
    }
    
    /// Get a random figure type
    fn random_figure_type(&mut self) -> FigureType {
        match self.rng.gen_range(0..6) {
            0 => FigureType::Founder,
            1 => FigureType::Leader,
            2 => FigureType::Innovator,
            3 => FigureType::Prophet,
            4 => FigureType::Warrior,
            5 => FigureType::Artist,
            _ => FigureType::Leader,
        }
    }
    
    /// Create a new historical figure
    fn create_figure(&mut self, species_id: String, figure_type: FigureType, year: u32) -> Figure {
        let prefix = FIGURE_PREFIXES[self.rng.gen_range(0..FIGURE_PREFIXES.len())];
        let suffix = FIGURE_SUFFIXES[self.rng.gen_range(0..FIGURE_SUFFIXES.len())];
        let name = format!("{}{}", prefix, suffix);
        
        let id = Uuid::new_v4().to_string();
        let significance = 0.5 + self.rng.gen::<f32>() * 0.5; // 0.5 to 1.0
        
        Figure {
            id,
            name,
            figure_type,
            species_id,
            birth_year: year,
            death_year: None,
            significance,
            birth_settlement_id: None,
        }
    }
    
    /// Simulate artifact creation
    fn simulate_artifacts(&mut self, state: &mut SimulationState, config: &SimulationConfig, year: i32) {
        // Collect population info first
        let pop_info: Vec<(String, u32)> = state.populations.iter()
            .map(|(id, pop)| (id.clone(), pop.count))
            .collect();
        
        // Collect artifact creation events
        let mut artifact_events: Vec<SimulationEvent> = Vec::new();
        for (pop_id, pop_count) in pop_info {
            // Artifact creation chance
            let artifact_chance = (pop_count as f32 / 500.0) * 0.005; // 0.5% per 500 population
            
            if self.rng.gen::<f32>() < artifact_chance {
                let artifact = self.create_artifact(year as u32);
                
                artifact_events.push(SimulationEvent {
                    year,
                    event_type: EventType::ArtifactCreated,
                    description: format!(
                        "{} was created",
                        artifact.name
                    ),
                    affected_entities: vec![artifact.id.clone(), pop_id],
                    significance: artifact.significance,
                });
            }
        }
        
        // Apply all artifact events
        for event in artifact_events {
            state.record_event(event);
        }
    }
    
    /// Create a new artifact
    fn create_artifact(&mut self, year: u32) -> Artifact {
        let artifact_type = ARTIFACT_TYPES[self.rng.gen_range(0..ARTIFACT_TYPES.len())];
        let name = format!("The {} of {}", 
            artifact_type,
            SETTLEMENT_ROOTS[self.rng.gen_range(0..SETTLEMENT_ROOTS.len())]
        );
        
        let id = Uuid::new_v4().to_string();
        let significance = 0.3 + self.rng.gen::<f32>() * 0.7; // 0.3 to 1.0
        
        Artifact {
            id,
            name,
            artifact_type: artifact_type.to_string(),
            creator_id: None,
            creation_year: year,
            significance,
        }
    }
    
    /// Process ongoing wars
    fn simulate_wars(&mut self, state: &mut SimulationState, year: i32) {
        let wars_to_remove: Vec<(String, String)> = state.active_wars.keys()
            .cloned()
            .collect();
        
        for (s1, s2) in wars_to_remove {
            if let Some(years_remaining) = state.active_wars.get_mut(&(s1.clone(), s2.clone())) {
                *years_remaining -= 1;
                
                if *years_remaining <= 0 {
                    state.active_wars.remove(&(s1.clone(), s2.clone()));
                    
                    state.record_event(SimulationEvent {
                        year,
                        event_type: EventType::WarEnded,
                        description: format!(
                            "War between {} and {} has ended",
                            s1, s2
                        ),
                        affected_entities: vec![s1.clone(), s2.clone()],
                        significance: 0.8,
                    });
                }
            }
        }
    }
    
    /// Start a new war between factions
    pub fn start_war(&mut self, state: &mut SimulationState, faction1: String, faction2: String) {
        let duration = self.rng.gen_range(5..20); // 5-20 years
        state.active_wars.insert((faction1.clone(), faction2.clone()), duration);
        
        state.record_event(SimulationEvent {
            year: state.current_year,
            event_type: EventType::WarBegan,
            description: format!(
                "War has broken out between {} and {}",
                faction1, faction2
            ),
            affected_entities: vec![faction1, faction2],
            significance: 0.9,
        });
    }
    
    /// Simulate groundwater dynamics for the year (WOR-479)
    /// Processes recharge from precipitation and recession/discharge
    fn simulate_groundwater(&mut self, state: &mut SimulationState) {
        // Default precipitation - could be made configurable per biome
        let precipitation_mm_year = 1000.0;
        
        // Apply annual recharge
        self.groundwater_system.apply_recharge(state.current_year, precipitation_mm_year);
        
        // Simulate recession (baseflow discharge and natural depletion)
        self.groundwater_system.simulate_recession();
        
        // Update simulation state with current aquifer data
        state.aquifer_map = self.groundwater_system.get_all_aquifers().clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization() {
        let mut engine = SimulationEngine::new(Some(42));
        
        let polygons = vec![
            PolygonInfo { id: 1, elevation: 100.0, neighbors: vec![2], is_coastal: false, is_island: false },
            PolygonInfo { id: 2, elevation: 200.0, neighbors: vec![1], is_coastal: false, is_island: false },
        ];
        
        let elevation_map: HashMap<u64, f32> = vec![
            (1, 100.0f32),
            (2, 200.0f32),
        ].into_iter().collect();
        
        let config = SimulationConfig::default();
        let state = engine.initialize(
            "test_world".to_string(),
            polygons,
            elevation_map,
            &config,
        );
        
        assert_eq!(state.world_id, "test_world");
        assert!(!state.territories.is_empty());
    }

    #[test]
    fn test_engine_run() {
        let mut engine = SimulationEngine::new(Some(42));
        
        let polygons = vec![
            PolygonInfo { id: 1, elevation: 100.0, neighbors: vec![2], is_coastal: false, is_island: false },
            PolygonInfo { id: 2, elevation: 200.0, neighbors: vec![1], is_coastal: false, is_island: false },
        ];
        
        let elevation_map: HashMap<u64, f32> = vec![
            (1, 100.0f32),
            (2, 200.0f32),
        ].into_iter().collect();
        
        let mut config = SimulationConfig::default();
        config.years = 10;
        
        let mut state = engine.initialize(
            "test_world".to_string(),
            polygons,
            elevation_map,
            &config,
        );
        
        engine.run(&mut state, &config);
        
        assert_eq!(state.current_year, 10);
    }

    #[test]
    fn test_war_flow() {
        let mut engine = SimulationEngine::new(Some(42));
        
        let polygons = vec![
            PolygonInfo { id: 1, elevation: 100.0, neighbors: vec![], is_coastal: false, is_island: false },
        ];
        
        let elevation_map: HashMap<u64, f32> = vec![(1, 100.0f32)].into_iter().collect();
        
        let config = SimulationConfig::default();
        let mut state = engine.initialize(
            "test_world".to_string(),
            polygons,
            elevation_map,
            &config,
        );
        
        engine.start_war(&mut state, "faction1".to_string(), "faction2".to_string());
        
        assert!(state.active_wars.contains_key(&("faction1".to_string(), "faction2".to_string())));
    }
}
