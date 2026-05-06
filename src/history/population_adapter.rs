//! Population to History Events Adapter
//!
//! Converts PopulationGrowthService results into HistoricalEvent records
//! for the history chronicle system.
//!
//! # Usage
//!
//! ```rust
//! use world_factory::history::population_adapter::PopulationEventAdapter;
//!
//! let adapter = PopulationEventAdapter::new(world_id, 1000);
//! let events = adapter.convert_simulation_result(&simulation_result);
//! // Pass events to HistoryChronicleService
//! ```

use crate::history::population::{
    PopulationTickResult, SimulationResult, SocietyTransition as PopSocietyTransition,
};
use crate::history::society::Society;
use crate::types::{HistoricalEvent, HistoricalTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for population event generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationEventConfig {
    /// Minimum population change to generate an event.
    pub min_population_change: u64,

    /// Generate events for every population tick (expensive).
    pub verbose_events: bool,

    /// Generate society transition events.
    pub generate_transitions: bool,

    /// Generate food scarcity events.
    pub generate_food_events: bool,

    /// Generate disease outbreak events.
    pub generate_disease_events: bool,
}

impl Default for PopulationEventConfig {
    fn default() -> Self {
        Self {
            min_population_change: 10,
            verbose_events: false,
            generate_transitions: true,
            generate_food_events: true,
            generate_disease_events: true,
        }
    }
}

/// Adapter for converting population simulation to history events.
#[derive(Debug, Clone)]
pub struct PopulationEventAdapter {
    world_id: Uuid,
    start_year: i32,
    config: PopulationEventConfig,
}

impl PopulationEventAdapter {
    /// Create a new adapter.
    pub fn new(world_id: Uuid, start_year: i32) -> Self {
        Self {
            world_id,
            start_year,
            config: PopulationEventConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(world_id: Uuid, start_year: i32, config: PopulationEventConfig) -> Self {
        Self {
            world_id,
            start_year,
            config,
        }
    }

    /// Convert a full simulation result to historical events.
    pub fn convert_simulation_result(&self, result: &SimulationResult) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();

        for tick in &result.tick_results {
            events.extend(self.convert_tick_result(tick));
        }

        // Also add summary events for milestones
        if result.transition_count > 0 {
            events.push(self.create_transition_summary_event(result));
        }

        events
    }

    /// Convert a single tick result to events.
    pub fn convert_tick_result(&self, tick: &PopulationTickResult) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();
        let abs_change = tick.change.abs() as u64;

        // Skip minor changes unless verbose
        if !self.config.verbose_events && abs_change < self.config.min_population_change {
            return events;
        }

        // Population growth event
        if tick.change > 0 {
            events.push(self.create_growth_event(tick));
        } else if tick.change < 0 {
            events.push(self.create_decline_event(tick));
        }

        // Society transition event
        if self.config.generate_transitions {
            if let Some(ref transition) = tick.society_transition {
                events.push(self.create_society_transition_event(tick, transition));
            }
        }

        // Food scarcity warning
        if self.config.generate_food_events && tick.food_surplus_factor < 0.5 {
            events.push(self.create_food_scarcity_event(tick));
        }

        // Disease warning
        if self.config.generate_disease_events && tick.disease_factor < 0.7 {
            events.push(self.create_disease_warning_event(tick));
        }

        events
    }

    /// Create a population growth event.
    fn create_growth_event(&self, tick: &PopulationTickResult) -> HistoricalEvent {
        let growth_pct = if tick.old_population > 0 {
            (tick.change as f64 / tick.old_population as f64 * 100.0) as i32
        } else {
            0
        };

        let description = format!(
            "Population grew from {} to {} (+{} souls, +{}%) in year {}.",
            tick.old_population, tick.new_population, tick.change, growth_pct, tick.year
        );

        let mut event = HistoricalEvent::new(
            self.world_id,
            format!("Population Growth in Year {}", tick.year),
            HistoricalTime::year(tick.year),
            description,
        );

        event.event_type = Some(crate::events::event_type::EventType::PopulationGrowth);

        event
    }

    /// Create a population decline event.
    fn create_decline_event(&self, tick: &PopulationTickResult) -> HistoricalEvent {
        let description = format!(
            "Population declined from {} to {} ({} souls lost) in year {}.",
            tick.old_population,
            tick.new_population,
            tick.change.abs(),
            tick.year
        );

        let mut event = HistoricalEvent::new(
            self.world_id,
            format!("Population Decline in Year {}", tick.year),
            HistoricalTime::year(tick.year),
            description,
        );

        event.event_type = Some(crate::events::event_type::EventType::Plague);

        event
    }

    /// Create a society transition event.
    fn create_society_transition_event(
        &self,
        tick: &PopulationTickResult,
        transition: &PopSocietyTransition,
    ) -> HistoricalEvent {
        let from_name = format!("{:?}", transition.from_type);
        let to_name = format!("{:?}", transition.to_type);

        let description = format!(
            "The {} {} transitioned to a {} {} with a population of {} souls in year {}.",
            tick.society_id, from_name, to_name, tick.society_id, tick.new_population, tick.year
        );

        let mut event = HistoricalEvent::new(
            self.world_id,
            format!("{} → {} Transition", from_name, to_name),
            HistoricalTime::year(tick.year),
            description,
        );

        event.event_type = Some(crate::events::event_type::EventType::SocietyFormed);

        event
    }

    /// Create a food scarcity warning event.
    fn create_food_scarcity_event(&self, tick: &PopulationTickResult) -> HistoricalEvent {
        let description = format!(
            "Food shortage in {} {}. Food surplus factor: {:.2}. Population growth severely limited.",
            tick.society_id,
            tick.year,
            tick.food_surplus_factor
        );

        HistoricalEvent::new(
            self.world_id,
            format!("Food Scarcity Warning - Year {}", tick.year),
            HistoricalTime::year(tick.year),
            description,
        )
    }

    /// Create a disease warning event.
    fn create_disease_warning_event(&self, tick: &PopulationTickResult) -> HistoricalEvent {
        let description = format!(
            "High population density in {} {}. Disease factor: {:.2}. Health conditions deteriorating.",
            tick.society_id,
            tick.year,
            tick.disease_factor
        );

        let mut event = HistoricalEvent::new(
            self.world_id,
            format!("Disease Warning - Year {}", tick.year),
            HistoricalTime::year(tick.year),
            description,
        );

        event.event_type = Some(crate::events::event_type::EventType::Plague);
        event
    }
    fn create_transition_summary_event(&self, result: &SimulationResult) -> HistoricalEvent {
        let description = format!(
            "During this period, {} society type transitions occurred. Total population changed by {} souls.",
            result.transition_count,
            result.total_population_change
        );

        HistoricalEvent::new(
            self.world_id,
            "Society Evolution Report".to_string(),
            HistoricalTime::year(self.start_year + result.stats.years_elapsed),
            description,
        )
    }

    /// Convert society entity to history events.
    pub fn convert_society_to_events(&self, society: &Society) -> Vec<HistoricalEvent> {
        let mut events = Vec::new();

        // Society formation event
        let mut formation = HistoricalEvent::new(
            self.world_id,
            format!("Society {} Founded", society.name),
            HistoricalTime::year(society.formed_year),
            format!(
                "A new {} {} called '{}' was formed with {} souls.",
                society.species_id,
                format!("{:?}", society.society_type).to_lowercase(),
                society.name,
                society.population
            ),
        );

        formation.event_type = Some(crate::events::event_type::EventType::SocietyFormed);

        events.push(formation);

        events
    }

    /// Create a population milestone event.
    pub fn create_population_milestone(
        &self,
        society_id: Uuid,
        population: u64,
        year: i32,
    ) -> HistoricalEvent {
        let milestone_type = if population >= 5000 {
            "civilizational milestone"
        } else if population >= 500 {
            "tribal milestone"
        } else if population >= 50 {
            "settled community milestone"
        } else {
            "community milestone"
        };

        HistoricalEvent::new(
            self.world_id,
            format!("Population Milestone: {}", population),
            HistoricalTime::year(year),
            format!(
                "{} reached {} souls, a significant {}.",
                society_id, population, milestone_type
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::population::SocietyTransition;
    use crate::history::society::SocietyType;

    fn create_test_tick() -> PopulationTickResult {
        PopulationTickResult {
            society_id: Uuid::new_v4(),
            old_population: 100,
            new_population: 115,
            change: 15,
            growth_rate: 0.15,
            food_surplus_factor: 1.0,
            disease_factor: 1.0,
            society_transition: None,
            year: 100,
        }
    }

    #[test]
    fn test_adapter_growth_event() {
        let adapter = PopulationEventAdapter::new(Uuid::new_v4(), 100);
        let tick = create_test_tick();

        let events = adapter.convert_tick_result(&tick);
        assert!(!events.is_empty());
        assert!(events[0].description.contains("115"));
    }

    #[test]
    fn test_adapter_transition_event() {
        let adapter = PopulationEventAdapter::new(Uuid::new_v4(), 100);
        let mut tick = create_test_tick();
        tick.old_population = 45;
        tick.new_population = 55;
        tick.change = 10;
        tick.society_transition = Some(SocietyTransition {
            from_type: SocietyType::Band,
            to_type: SocietyType::Tribe,
            trigger_population: 55,
        });

        let events = adapter.convert_tick_result(&tick);
        assert!(events.len() >= 2); // Growth + transition
    }

    #[test]
    fn test_adapter_food_scarcity() {
        let adapter = PopulationEventAdapter::new(Uuid::new_v4(), 100);
        let mut tick = create_test_tick();
        tick.food_surplus_factor = 0.3;

        let events = adapter.convert_tick_result(&tick);
        assert!(events.iter().any(|e| e.name.contains("Food Scarcity")));
    }

    #[test]
    fn test_adapter_disease_warning() {
        let adapter = PopulationEventAdapter::new(Uuid::new_v4(), 100);
        let mut tick = create_test_tick();
        tick.disease_factor = 0.5;

        let events = adapter.convert_tick_result(&tick);
        assert!(events.iter().any(|e| e.name.contains("Disease")));
    }

    #[test]
    fn test_min_population_threshold() {
        let adapter = PopulationEventAdapter::new(Uuid::new_v4(), 100);
        let mut tick = create_test_tick();
        tick.change = 5; // Below default threshold of 10

        // With verbose off, should be empty
        let events = adapter.convert_tick_result(&tick);
        assert!(events.is_empty());

        // With verbose on, should have event
        let verbose_adapter = PopulationEventAdapter::with_config(
            Uuid::new_v4(),
            100,
            PopulationEventConfig {
                verbose_events: true,
                ..Default::default()
            },
        );
        let events = verbose_adapter.convert_tick_result(&tick);
        assert!(!events.is_empty());
    }
}
