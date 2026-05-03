//! Probability Engine
//! 
//! Core probability calculation for event triggering.
//! Implements deterministic probability calculation using world seed.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{ProbabilityConfig, EventContext, ProbabilityResult, ProbabilityFactor, Season};
use crate::events::{EventType, EventCategory};
use crate::terrain::biome::BiomeType;

/// Probability engine for calculating event triggering chances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityEngine {
    /// World seed for deterministic randomness.
    seed: u64,
    
    /// Configuration.
    config: ProbabilityConfig,
    
    /// Cache of base probabilities by event type.
    base_probabilities: HashMap<EventType, f32>,
    
    /// Event type cooldowns (time since last occurrence).
    event_cooldowns: HashMap<EventType, i32>,
    
    /// Default cooldown period for event types.
    default_cooldown_years: i32,
}

impl ProbabilityEngine {
    /// Create a new probability engine.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, ProbabilityConfig::default())
    }
    
    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: ProbabilityConfig) -> Self {
        let mut engine = Self {
            seed,
            config,
            base_probabilities: HashMap::new(),
            event_cooldowns: HashMap::new(),
            default_cooldown_years: 10,
        };
        engine.initialize_base_probabilities();
        engine
    }
    
    /// Initialize base probabilities for all event types.
    fn initialize_base_probabilities(&mut self) {
        // Initialize with default base probabilities per year
        // These are annual probabilities - actual check happens at epoch boundaries
        
        // High base probability events (common occurrences)
        self.base_probabilities.insert(EventType::SettlementFounded, 0.0002);  // ~1 per 5000 years per region
        self.base_probabilities.insert(EventType::PopulationGrowth, 0.01);     // Continuous process
        self.base_probabilities.insert(EventType::Migration, 0.001);           // ~1 per 1000 years
        self.base_probabilities.insert(EventType::Immigration, 0.0008);
        self.base_probabilities.insert(EventType::Festival, 0.002);            // ~1 per 500 years
        self.base_probabilities.insert(EventType::TradeRouteEstablished, 0.0003);
        self.base_probabilities.insert(EventType::Discovery, 0.0005);
        
        // Medium base probability events
        self.base_probabilities.insert(EventType::WarDeclared, 0.0001);        // ~1 per 10000 years (rare but impactful)
        self.base_probabilities.insert(EventType::WarEnded, 0.0001);
        self.base_probabilities.insert(EventType::Battle, 0.0008);           // ~1 per 1250 years during conflicts
        self.base_probabilities.insert(EventType::Siege, 0.0005);
        self.base_probabilities.insert(EventType::Conquest, 0.0001);
        self.base_probabilities.insert(EventType::Raid, 0.0006);
        self.base_probabilities.insert(EventType::Victory, 0.0004);
        self.base_probabilities.insert(EventType::Defeat, 0.0004);
        self.base_probabilities.insert(EventType::Plague, 0.00005);           // ~1 per 20000 years (rare but devastating)
        self.base_probabilities.insert(EventType::Famine, 0.0002);
        self.base_probabilities.insert(EventType::Earthquake, 0.0003);
        self.base_probabilities.insert(EventType::Flood, 0.0004);
        self.base_probabilities.insert(EventType::Drought, 0.0003);
        self.base_probabilities.insert(EventType::Volcano, 0.0001);
        self.base_probabilities.insert(EventType::Storm, 0.0005);
        self.base_probabilities.insert(EventType::Exploration, 0.0004);
        self.base_probabilities.insert(EventType::Treaty, 0.0002);
        self.base_probabilities.insert(EventType::AllianceFormed, 0.0002);
        self.base_probabilities.insert(EventType::AllianceBroken, 0.0002);
        self.base_probabilities.insert(EventType::Succession, 0.0008);
        self.base_probabilities.insert(EventType::GovernmentReform, 0.0001);
        self.base_probabilities.insert(EventType::CulturalAchievement, 0.0003);
        self.base_probabilities.insert(EventType::ReligiousEvent, 0.0004);
        self.base_probabilities.insert(EventType::Invention, 0.0002);
        self.base_probabilities.insert(EventType::LawEnacted, 0.0003);
        
        // Phase 2 Event Types
        self.base_probabilities.insert(EventType::SocietyFormed, 0.00005);     // ~1 per 20000 years
        self.base_probabilities.insert(EventType::FigureRises, 0.001);         // ~1 per 1000 years
        self.base_probabilities.insert(EventType::FigureDies, 0.001);          // ~1 per 1000 years
        self.base_probabilities.insert(EventType::ArtifactCreated, 0.0002);    // ~1 per 5000 years
        self.base_probabilities.insert(EventType::ArtifactActivated, 0.00001); // ~1 per 100000 years (rare & dangerous)
        
        // Low base probability events (major occurrences)
        self.base_probabilities.insert(EventType::NationFounded, 0.00002);    // ~1 per 50000 years
        self.base_probabilities.insert(EventType::Coup, 0.00005);
        self.base_probabilities.insert(EventType::GoldenAge, 0.00003);
        self.base_probabilities.insert(EventType::ReligiousReformation, 0.00002);
        self.base_probabilities.insert(EventType::FirstContact, 0.00001);
        self.base_probabilities.insert(EventType::ResourceDiscovery, 0.0003);
        self.base_probabilities.insert(EventType::Wildfire, 0.0004);
        self.base_probabilities.insert(EventType::Tsunami, 0.0001);
        self.base_probabilities.insert(EventType::Avalanche, 0.0002);
        self.base_probabilities.insert(EventType::MonumentCompleted, 0.0001);
        self.base_probabilities.insert(EventType::Collapse, 0.00001);
        self.base_probabilities.insert(EventType::Destruction, 0.0002);
        self.base_probabilities.insert(EventType::MeteorStrike, 0.000001);    // Extremely rare
        self.base_probabilities.insert(EventType::Extinction, 0.000002);
        self.base_probabilities.insert(EventType::MagicalCatastrophe, 0.00001);
    }
    
    /// Calculate probability for an event type given context.
    /// 
    /// # Arguments
    /// 
    /// * `event_type` - The type of event to calculate probability for
    /// * `context` - Current world state context
    /// * `current_year` - Current simulation year
    /// 
    /// # Returns
    /// 
    /// Probability result with breakdown of factors.
    pub fn calculate_event_probability(
        &mut self,
        event_type: EventType,
        context: &EventContext,
        current_year: i32,
    ) -> ProbabilityResult {
        // Start with base probability
        let base_prob = self.get_base_probability(event_type);
        let mut factors: Vec<ProbabilityFactor> = Vec::new();
        
        // 1. Environmental modifiers
        let env_mod = self.calculate_environmental_modifier(event_type, context);
        factors.push(ProbabilityFactor::new(
            "environmental",
            1.0,
            env_mod,
            &format!("Biome/location modifiers for {:?}", event_type),
        ));
        
        // 2. Population modifiers
        let pop_mod = self.calculate_population_modifier(event_type, context);
        factors.push(ProbabilityFactor::new(
            "population",
            context.population.unwrap_or(1000) as f32,
            pop_mod,
            "Population-based scaling",
        ));
        
        // 3. Historical context modifiers
        let hist_mod = self.calculate_historical_modifier(event_type, context, current_year);
        factors.push(ProbabilityFactor::new(
            "historical",
            hist_mod,
            hist_mod,
            "Recent event frequency and cooldowns",
        ));
        
        // 4. Calculate combined probability
        let combined = base_prob * env_mod * pop_mod * hist_mod * self.config.base_multiplier;
        
        // 5. Add deterministic random modifier
        let random_seed = self.hash_values(self.seed, event_type.name().len() as u64, current_year as u64);
        let random_mod = self.deterministic_random(random_seed);
        factors.push(ProbabilityFactor::new(
            "random_seed",
            random_seed as f32,
            1.0 + (random_mod - 0.5) * self.config.random_variance,
            "Deterministic variation from seed",
        ));
        
        // Apply random modifier
        let final_prob = (combined * (1.0 + (random_mod - 0.5) * self.config.random_variance))
            .clamp(self.config.min_probability, self.config.max_probability);
        
        ProbabilityResult {
            probability: final_prob,
            base_probability: base_prob,
            environmental_modifier: env_mod,
            population_modifier: pop_mod,
            historical_modifier: hist_mod,
            random_modifier: random_mod,
            factors,
        }
    }
    
    /// Get base probability for event type.
    pub fn get_base_probability(&self, event_type: EventType) -> f32 {
        self.base_probabilities.get(&event_type).copied().unwrap_or(0.0001)
    }
    
    /// Set custom base probability for event type.
    pub fn set_base_probability(&mut self, event_type: EventType, probability: f32) {
        self.base_probabilities.insert(event_type, probability);
    }
    
    /// Calculate environmental modifiers based on biome and location.
    fn calculate_environmental_modifier(&self, event_type: EventType, context: &EventContext) -> f32 {
        let biome = context.biome.unwrap_or(BiomeType::TemperateGrassland);
        let mut modifier = 1.0;
        
        match event_type {
            // Natural disasters are biome-dependent
            EventType::Flood => {
                modifier = match biome {
                    BiomeType::CoastalWetland => 8.0,
                    BiomeType::TropicalRainforest => 4.0,
                    BiomeType::Mangrove => 6.0,
                    BiomeType::TemperateRainforest => 3.0,
                    BiomeType::TemperateGrassland => 1.5,
                    _ => 1.0,
                };
            }
            EventType::Drought => {
                modifier = match biome {
                    BiomeType::HotDesert => 10.0,
                    BiomeType::ColdDesert => 8.0,
                    BiomeType::TemperateDesert => 6.0,
                    BiomeType::SemiAridSteppe => 5.0,
                    BiomeType::SubtropicalDesert => 4.0,
                    BiomeType::TemperateSteppe => 2.0,
                    _ => 1.0,
                };
            }
            EventType::Earthquake => {
                // Tectonic activity zones
                modifier = match biome {
                    BiomeType::VolcanicLandscape => 8.0,
                    BiomeType::MontaneForest => 3.0,
                    BiomeType::MontaneGrassland => 3.0,
                    _ => 1.0,
                };
            }
            EventType::Volcano => {
                modifier = match biome {
                    BiomeType::VolcanicLandscape => 15.0,
                    _ => 1.0,
                };
            }
            EventType::Storm => {
                modifier = match biome {
                    BiomeType::OpenOcean => 10.0,
                    BiomeType::CoastalWetland => 5.0,
                    BiomeType::CoralReef => 4.0,
                    BiomeType::TemperateGrassland => 2.0,
                    _ => 1.0,
                };
            }
            EventType::Tsunami => {
                modifier = match biome {
                    BiomeType::Mangrove => 5.0,
                    BiomeType::CoastalWetland => 3.0,
                    BiomeType::OpenOcean => 8.0, // Ocean is where they form
                    _ => 1.0,
                };
            }
            EventType::Wildfire => {
                modifier = match biome {
                    BiomeType::BorealForest => 6.0,
                    BiomeType::BorealTaiga => 5.0,
                    BiomeType::TemperateDeciduousForest => 3.0,
                    BiomeType::TemperateMixedForest => 3.0,
                    BiomeType::TropicalSeasonalForest => 4.0,
                    BiomeType::TropicalDryForest => 5.0,
                    BiomeType::MagicalForest => 0.5, // Magical forests resist fire
                    _ => 1.0,
                };
            }
            
            // Cultural events are affected by population density
            EventType::SettlementFounded => {
                modifier = match biome {
                    BiomeType::TemperateGrassland => 3.0,
                    BiomeType::TemperateSteppe => 2.5,
                    BiomeType::TropicalSavanna => 2.0,
                    BiomeType::CoastalWetland => 1.8,
                    BiomeType::HotDesert => 0.3,
                    BiomeType::SnowGlacier => 0.0,
                    BiomeType::OpenOcean => 0.0,
                    _ => 1.0,
                };
            }
            EventType::Festival => {
                modifier = match biome {
                    BiomeType::TemperateGrassland => 2.0,
                    BiomeType::TropicalSavanna => 1.8,
                    _ => 1.0,
                };
            }
            
            // Military events affected by terrain
            EventType::Battle => {
                modifier = match biome {
                    BiomeType::TemperateGrassland => 2.5, // Open plains favor battles
                    BiomeType::TemperateSteppe => 3.0,
                    BiomeType::TropicalSavanna => 2.0,
                    BiomeType::BorealForest => 0.5,
                    BiomeType::MontaneForest => 0.4,
                    BiomeType::BorealTaiga => 0.3,
                    BiomeType::HotDesert => 1.5,
                    _ => 1.0,
                };
            }
            
            // Disease is affected by population density and environment
            EventType::Plague => {
                modifier = match biome {
                    BiomeType::ToxicSwamp => 5.0,
                    BiomeType::Mangrove => 3.0,
                    BiomeType::CoastalWetland => 2.5,
                    BiomeType::TemperateDeciduousForest => 1.5,
                    BiomeType::OpenOcean => 0.1,
                    BiomeType::HotDesert => 0.2,
                    _ => 1.0,
                };
            }
            
            _ => {}
        }
        
        // Latitude modifiers for temperature-dependent events
        if let Some(lat) = context.latitude {
            let lat_abs = lat.abs();
            
            match event_type {
                EventType::Earthquake | EventType::Volcano => {
                    // Tectonic activity more common at certain latitudes
                    if lat_abs > 30.0 && lat_abs < 60.0 {
                        modifier *= 1.5; // "Ring of Fire" latitude band
                    }
                }
                EventType::Storm => {
                    // Hurricanes/tropical storms near equator
                    if lat_abs < 30.0 {
                        modifier *= 3.0;
                    }
                }
                EventType::Avalanche => {
                    // Mountain events at high latitudes
                    if lat_abs > 45.0 {
                        modifier *= 4.0;
                    }
                }
                _ => {}
            }
        }
        
        modifier
    }
    
    /// Calculate population-based modifiers.
    fn calculate_population_modifier(&self, event_type: EventType, context: &EventContext) -> f32 {
        let population = context.population.unwrap_or(0);
        let world_pop = context.world_population.unwrap_or(1).max(1);
        
        let mut modifier = 1.0;
        
        // Population density affects many events
        let _density = population as f32 / world_pop as f32 * 1000.0;
        
        match event_type {
            // High population increases conflict
            EventType::WarDeclared | EventType::Battle | EventType::Siege | EventType::Raid => {
                if population > 10000 {
                    modifier *= 2.0;
                } else if population > 5000 {
                    modifier *= 1.5;
                } else if population < 500 {
                    modifier *= 0.3;
                }
            }
            
            // More people = more potential for cultural achievements
            EventType::Festival | EventType::CulturalAchievement | EventType::Invention => {
                if population > 10000 {
                    modifier *= 3.0;
                } else if population > 5000 {
                    modifier *= 2.0;
                } else if population < 500 {
                    modifier *= 0.2;
                }
            }
            
            // Disease spreads more in dense populations
            EventType::Plague => {
                if population > 20000 {
                    modifier *= 5.0;
                } else if population > 10000 {
                    modifier *= 3.0;
                } else if population < 1000 {
                    modifier *= 0.2;
                }
            }
            
            // Population growth more likely with good conditions
            EventType::PopulationGrowth => {
                if population > 5000 {
                    modifier *= 1.5;
                }
            }
            
            // Settlement founding more likely with population
            EventType::SettlementFounded => {
                if population > 1000 {
                    modifier *= 2.5;
                } else if population > 500 {
                    modifier *= 1.5;
                }
            }
            
            // Trade routes need population centers
            EventType::TradeRouteEstablished => {
                if population > 5000 {
                    modifier *= 2.0;
                } else if population > 1000 {
                    modifier *= 1.3;
                }
            }
            
            // Society formation needs population threshold
            EventType::SocietyFormed => {
                if population > 5000 {
                    modifier *= 3.0;
                } else if population > 1000 {
                    modifier *= 1.5;
                } else {
                    modifier *= 0.2;
                }
            }
            
            // Figure events depend on society size
            EventType::FigureRises | EventType::FigureDies => {
                if population > 10000 {
                    modifier *= 2.5;
                } else if population > 5000 {
                    modifier *= 1.5;
                } else if population < 500 {
                    modifier *= 0.3;
                }
            }
            
            // Artifact creation more likely with advanced societies
            EventType::ArtifactCreated => {
                if population > 10000 {
                    modifier *= 3.0;
                } else if population > 5000 {
                    modifier *= 1.5;
                }
            }
            
            // Artifact activation rare - depends on existing artifacts
            EventType::ArtifactActivated => {
                modifier *= 0.1; // Very rare event
            }
            
            _ => {}
        }
        
        // War state affects military event probabilities
        if context.is_at_war {
            match event_type {
                EventType::Battle | EventType::Siege | EventType::Raid => {
                    modifier *= 5.0; // War dramatically increases conflict events
                }
                EventType::Festival | EventType::CulturalAchievement => {
                    modifier *= 0.1; // War suppresses cultural events
                }
                _ => {}
            }
        }
        
        modifier
    }
    
    /// Calculate historical context modifiers (cooldowns, dependencies).
    fn calculate_historical_modifier(&self, event_type: EventType, context: &EventContext, current_year: i32) -> f32 {
        let mut modifier = 1.0;
        
        // Check cooldown period
        if let Some(last_year) = self.event_cooldowns.get(&event_type) {
            let years_since = current_year - last_year;
            let cooldown = self.get_cooldown_years(event_type);
            
            if years_since < cooldown {
                // Apply strong suppression for events that just occurred
                let suppression = 1.0 - (years_since as f32 / cooldown as f32);
                modifier *= suppression * suppression; // Square for stronger effect
            }
        }
        
        // Check active events (conflicts prevent similar events)
        if context.is_at_war && matches!(event_type, EventType::WarDeclared) {
            modifier *= 0.1; // Can't start a new war while at war
        }
        
        // Recent event frequency affects probability
        let recent_count = context.recent_events.iter()
            .filter(|e| e.event_type == event_type)
            .count();
        
        if recent_count > 0 {
            // If this event happened recently, reduce probability
            modifier *= 0.5_f32.powi(recent_count.min(5) as i32);
        }
        
        // Category-based dependencies
        match event_type {
            EventType::WarEnded => {
                // War ending requires active war
                if !context.is_at_war {
                    modifier *= 0.0; // Can't end war if not at war
                }
            }
            EventType::Treaty => {
                // Treaties often follow wars
                if !context.is_at_war {
                    modifier *= 0.2; // Less likely without active conflict
                } else {
                    modifier *= 2.0; // More likely during conflict
                }
            }
            EventType::AllianceFormed => {
                // More likely during conflict
                if context.is_at_war {
                    modifier *= 2.0;
                }
            }
            EventType::PopulationGrowth => {
                // Check if there are recent population losses
                let has_recent_loss = context.recent_events.iter()
                    .any(|e| matches!(e.event_type, EventType::Plague | EventType::Famine | EventType::WarDeclared) && e.year > current_year - 50);
                
                if has_recent_loss {
                    modifier *= 0.3; // Hard to grow after disaster
                }
            }
            _ => {}
        }
        
        modifier.max(0.0)
    }
    
    /// Get cooldown period for event type.
    fn get_cooldown_years(&self, event_type: EventType) -> i32 {
        // Different events have different natural frequencies
        match event_type {
            // Frequent events have short cooldowns
            EventType::Festival => 5,
            EventType::Battle => 10,
            EventType::Raid => 15,
            EventType::Immigration => 20,
            EventType::Migration => 30,
            EventType::Discovery => 25,
            EventType::Invention => 40,
            
            // Major events have longer cooldowns
            EventType::WarDeclared | EventType::WarEnded => 50,
            EventType::Plague => 100,
            EventType::NationFounded => 200,
            EventType::Coup => 75,
            EventType::Succession => 30,
            EventType::GovernmentReform => 100,
            EventType::ReligiousReformation => 200,
            EventType::GoldenAge => 150,
            EventType::SocietyFormed => 200,
            EventType::FigureRises => 50,
            EventType::FigureDies => 50,
            EventType::ArtifactCreated => 100,
            EventType::ArtifactActivated => 500,
            
            // Rare catastrophic events
            EventType::MeteorStrike => 10000,
            EventType::Extinction => 5000,
            EventType::Collapse => 300,
            
            _ => self.default_cooldown_years,
        }
    }
    
    /// Update cooldown tracking after event occurrence.
    pub fn record_event(&mut self, event_type: EventType, year: i32) {
        self.event_cooldowns.insert(event_type, year);
    }
    
    /// Generate deterministic pseudo-random value from seed.
    fn deterministic_random(&self, seed: u64) -> f32 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        
        let hash = hasher.finish();
        ((hash as f64) % 1000.0 / 1000.0) as f32
    }
    
    /// Hash multiple values together.
    fn hash_values(&self, a: u64, b: u64, c: u64) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        a.hash(&mut hasher);
        b.hash(&mut hasher);
        c.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        
        hasher.finish()
    }
    
    /// Calculate probability for all event types in a category.
    pub fn calculate_category_probabilities(
        &mut self,
        category: EventCategory,
        context: &EventContext,
        current_year: i32,
    ) -> HashMap<EventType, ProbabilityResult> {
        let mut results = HashMap::new();
        
        // Get all event types in category
        for event_type in category.event_types() {
            let result = self.calculate_event_probability(event_type, context, current_year);
            results.insert(event_type, result);
        }
        
        results
    }
    
    /// Batch calculate probabilities for multiple event types.
    pub fn calculate_probabilities(
        &mut self,
        event_types: &[EventType],
        context: &EventContext,
        current_year: i32,
    ) -> Vec<ProbabilityResult> {
        event_types.iter()
            .map(|et| self.calculate_event_probability(*et, context, current_year))
            .collect()
    }
    
    /// Determine which events should trigger in a given year/epoch.
    /// 
    /// Returns a list of (event_type, probability_result) pairs that should be checked.
    /// The caller should use the probability to roll dice and decide which actually trigger.
    pub fn get_events_for_epoch(
        &mut self,
        context: &EventContext,
        current_year: i32,
        epoch_years: i32,
    ) -> Vec<(EventType, ProbabilityResult)> {
        let mut results = Vec::new();
        
        // Collect event types first to avoid borrow conflict
        let event_types: Vec<_> = self.base_probabilities.keys().cloned().collect();
        
        // Check all event types
        for event_type in &event_types {
            let prob = self.calculate_event_probability(*event_type, context, current_year);
            
            // Scale by epoch length
            let epoch_prob = 1.0 - (1.0 - prob.probability).powi(epoch_years);
            
            results.push((*event_type, ProbabilityResult {
                probability: epoch_prob,
                ..prob
            }));
        }
        
        // Sort by probability (highest first)
        results.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());
        
        results
    }
    
    /// Get events most likely to occur in next epoch.
    pub fn get_top_candidates(
        &mut self,
        context: &EventContext,
        current_year: i32,
        epoch_years: i32,
        top_n: usize,
    ) -> Vec<(EventType, ProbabilityResult)> {
        let all = self.get_events_for_epoch(context, current_year, epoch_years);
        all.into_iter().take(top_n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uuid;
    
    #[test]
    fn test_base_probability_lookup() {
        let engine = ProbabilityEngine::new(42);
        
        assert!(engine.get_base_probability(EventType::WarDeclared) > 0.0);
        assert!(engine.get_base_probability(EventType::SettlementFounded) > 0.0);
        assert!(engine.get_base_probability(EventType::MeteorStrike) < engine.get_base_probability(EventType::Festival));
    }
    
    #[test]
    fn test_probability_calculation() {
        let mut engine = ProbabilityEngine::new(42);
        
        let context = EventContext {
            location_id: Some(Uuid::new_v4()),
            biome: Some(BiomeType::TemperateGrassland),
            population: Some(5000),
            world_population: Some(100000),
            latitude: Some(45.0),
            season: Some(Season::Spring),
            active_events: Vec::new(),
            recent_events: Vec::new(),
            neighboring_entities: Vec::new(),
            is_at_war: false,
            trade_connections: Vec::new(),
            cultural_tensions: 0.3,
            economic_health: 0.7,
        };
        
        let result = engine.calculate_event_probability(
            EventType::SettlementFounded,
            &context,
            1000,
        );
        
        assert!(result.probability > 0.0);
        assert!(result.probability <= 1.0);
    }
    
    #[test]
    fn test_environmental_modifier() {
        let mut engine = ProbabilityEngine::new(42);
        
        let desert_context = EventContext {
            biome: Some(BiomeType::HotDesert),
            population: Some(1000),
            ..Default::default()
        };
        
        let forest_context = EventContext {
            biome: Some(BiomeType::TemperateDeciduousForest),
            population: Some(1000),
            ..Default::default()
        };
        
        let drought_prob_desert = engine.calculate_event_probability(
            EventType::Drought,
            &desert_context,
            1000,
        );
        
        let drought_prob_forest = engine.calculate_event_probability(
            EventType::Drought,
            &forest_context,
            1000,
        );
        
        assert!(drought_prob_desert.environmental_modifier > drought_prob_forest.environmental_modifier);
    }
    
    #[test]
    fn test_war_state_modifier() {
        let mut engine = ProbabilityEngine::new(42);
        
        let peaceful = EventContext {
            is_at_war: false,
            population: Some(10000),
            ..Default::default()
        };
        
        let at_war = EventContext {
            is_at_war: true,
            population: Some(10000),
            ..Default::default()
        };
        
        let battle_peaceful = engine.calculate_event_probability(
            EventType::Battle,
            &peaceful,
            1000,
        );
        
        let battle_war = engine.calculate_event_probability(
            EventType::Battle,
            &at_war,
            1000,
        );
        
        assert!(battle_war.population_modifier > battle_peaceful.population_modifier);
    }
    
    #[test]
    fn test_cooldown_tracking() {
        let mut engine = ProbabilityEngine::new(42);
        
        engine.record_event(EventType::WarDeclared, 1000);
        
        let context = EventContext::default();
        
        // Immediately after war declared
        let prob_immediate = engine.calculate_event_probability(
            EventType::WarDeclared,
            &context,
            1000,
        );
        assert!(prob_immediate.historical_modifier < 0.5);
        
        // 50 years later
        let prob_later = engine.calculate_event_probability(
            EventType::WarDeclared,
            &context,
            1050,
        );
        assert!(prob_later.historical_modifier > prob_immediate.historical_modifier);
    }
    
    #[test]
    fn test_determinism() {
        let mut engine1 = ProbabilityEngine::new(42);
        let mut engine2 = ProbabilityEngine::new(42);
        
        let context = EventContext::default();
        
        let result1 = engine1.calculate_event_probability(
            EventType::Battle,
            &context,
            1500,
        );
        
        let result2 = engine2.calculate_event_probability(
            EventType::Battle,
            &context,
            1500,
        );
        
        assert_eq!(result1.probability, result2.probability);
        assert_eq!(result1.random_modifier, result2.random_modifier);
    }
    
    #[test]
    fn test_different_seeds_different_results() {
        let mut engine1 = ProbabilityEngine::new(42);
        let mut engine2 = ProbabilityEngine::new(123);
        
        let context = EventContext::default();
        
        let result1 = engine1.calculate_event_probability(
            EventType::Discovery,
            &context,
            1200,
        );
        
        let result2 = engine2.calculate_event_probability(
            EventType::Discovery,
            &context,
            1200,
        );
        
        // With different seeds, random modifiers should differ
        assert_ne!(result1.random_modifier, result2.random_modifier);
    }
    
    #[test]
    fn test_epoch_scaling() {
        let mut engine = ProbabilityEngine::new(42);
        let context = EventContext::default();
        
        // Single year check
        let single = engine.get_events_for_epoch(&context, 1000, 1);
        
        // 100 year epoch
        let century = engine.get_events_for_epoch(&context, 1000, 100);
        
        // Century probabilities should be higher
        for (et, result) in single.iter() {
            if let Some((_, century_result)) = century.iter().find(|(e, _)| *e == *et) {
                assert!(century_result.probability >= result.probability);
            }
        }
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self {
            location_id: None,
            biome: Some(BiomeType::TemperateGrassland),
            population: Some(1000),
            world_population: Some(50000),
            latitude: Some(45.0),
            season: Some(Season::Spring),
            active_events: Vec::new(),
            recent_events: Vec::new(),
            neighboring_entities: Vec::new(),
            is_at_war: false,
            trade_connections: Vec::new(),
            cultural_tensions: 0.5,
            economic_health: 0.5,
        }
    }
}