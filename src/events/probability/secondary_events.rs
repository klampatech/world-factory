//! Secondary Event Triggering
//!
//! Handles cascading event generation from primary event effects.
//! Implements conditional rules for automatic secondary event creation.

use crate::events::{Event, EventBuilder, EventEffect, EventType};
use crate::types::HistoricalTime;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Queue of pending secondary events to be processed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecondaryEventQueue {
    /// Pending events in FIFO order.
    events: VecDeque<SecondaryEventCandidate>,

    /// Maximum queue size to prevent memory overflow.
    max_size: usize,
}

impl SecondaryEventQueue {
    /// Create a new queue with default max size.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            max_size: 1000,
        }
    }

    /// Create with custom max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_size,
        }
    }

    /// Add a secondary event candidate to the queue.
    pub fn enqueue(&mut self, candidate: SecondaryEventCandidate) -> bool {
        if self.events.len() < self.max_size {
            self.events.push_back(candidate);
            true
        } else {
            false
        }
    }

    /// Get the next event from the queue (does not remove).
    pub fn peek(&self) -> Option<&SecondaryEventCandidate> {
        self.events.front()
    }

    /// Pop the next event from the queue.
    pub fn dequeue(&mut self) -> Option<SecondaryEventCandidate> {
        self.events.pop_front()
    }

    /// Check if queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get queue length.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Clear the queue.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get all events matching a filter.
    pub fn get_matching<F>(&self, filter: F) -> Vec<&SecondaryEventCandidate>
    where
        F: Fn(&SecondaryEventCandidate) -> bool,
    {
        self.events.iter().filter(|e| filter(e)).collect()
    }
}

/// A candidate secondary event awaiting processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEventCandidate {
    /// The primary event that triggered this secondary event.
    pub source_event_id: Uuid,

    /// The triggered event type.
    pub event_type: EventType,

    /// Time offset from primary event.
    pub time_offset_years: i32,

    /// Location of secondary event (defaults to source location).
    pub location_id: Option<Uuid>,

    /// Probability of this secondary event triggering (0.0-1.0).
    pub probability: f32,

    /// Priority for processing (higher = earlier).
    pub priority: i32,

    /// Description of why this event was triggered.
    pub trigger_reason: String,

    /// Suggested effects for the secondary event.
    pub suggested_effects: Vec<EventEffect>,
}

impl SecondaryEventCandidate {
    /// Create a new secondary event candidate.
    pub fn new(source_event_id: Uuid, event_type: EventType, time_offset_years: i32) -> Self {
        Self {
            source_event_id,
            event_type,
            time_offset_years,
            location_id: None,
            probability: 1.0,
            priority: 0,
            trigger_reason: String::new(),
            suggested_effects: Vec::new(),
        }
    }

    /// Set location.
    pub fn with_location(mut self, location_id: Uuid) -> Self {
        self.location_id = Some(location_id);
        self
    }

    /// Set probability.
    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability.clamp(0.0, 1.0);
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set trigger reason.
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.trigger_reason = reason.to_string();
        self
    }

    /// Add a suggested effect.
    pub fn add_effect(mut self, effect: EventEffect) -> Self {
        self.suggested_effects.push(effect);
        self
    }

    /// Convert to actual Event.
    pub fn to_event(self, world_id: Uuid, base_year: i32) -> Event {
        let event_year = base_year + self.time_offset_years;

        let mut builder = EventBuilder::new(format!("Secondary: {:?}", self.event_type))
            .event_type(self.event_type)
            .time(HistoricalTime::year(event_year))
            .with_reason(&self.trigger_reason);

        if let Some(loc) = self.location_id {
            builder = builder.location(loc);
        }

        for effect in self.suggested_effects {
            builder = builder.effect(effect);
        }

        builder.significance(0.5).build(world_id)
    }
}

/// Rules for secondary event triggering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRule {
    /// The primary effect type that triggers this rule.
    pub trigger_effect: String,

    /// The secondary event type to generate.
    pub secondary_event_type: EventType,

    /// Base probability of secondary event occurring.
    pub base_probability: f32,

    /// Time offset in years from primary event.
    pub time_offset_years: i32,

    /// Priority (higher processed first).
    pub priority: i32,

    /// Whether to inherit location from primary event.
    pub inherit_location: bool,

    /// Minimum significance of primary event to trigger.
    pub min_significance: f32,
}

impl TriggerRule {
    /// Create a new trigger rule.
    pub fn new(
        trigger_effect: &str,
        secondary_event: EventType,
        base_probability: f32,
        time_offset: i32,
    ) -> Self {
        Self {
            trigger_effect: trigger_effect.to_string(),
            secondary_event_type: secondary_event,
            base_probability,
            time_offset_years: time_offset,
            priority: 0,
            inherit_location: true,
            min_significance: 0.0,
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set inherit location.
    pub fn with_no_location_inherit(mut self) -> Self {
        self.inherit_location = false;
        self
    }

    /// Set minimum significance.
    pub fn with_min_significance(mut self, significance: f32) -> Self {
        self.min_significance = significance;
        self
    }
}

/// Default trigger rules for common effect -> event mappings.
pub fn default_trigger_rules() -> Vec<TriggerRule> {
    vec![
        // War → aftermath events
        TriggerRule::new("population_loss", EventType::Plague, 0.3, 5)
            .with_priority(20)
            .with_min_significance(0.6),
        TriggerRule::new("population_loss", EventType::Famine, 0.4, 3)
            .with_priority(15)
            .with_min_significance(0.5),
        TriggerRule::new("border_shift", EventType::Migration, 0.5, 10)
            .with_priority(10)
            .with_min_significance(0.4),
        // Plague → social collapse
        TriggerRule::new("population_loss", EventType::Collapse, 0.2, 20)
            .with_priority(5)
            .with_min_significance(0.8),
        TriggerRule::new("population_loss", EventType::Migration, 0.6, 2)
            .with_priority(25)
            .with_min_significance(0.3),
        // Famine → population displacement
        TriggerRule::new("population_loss", EventType::Migration, 0.7, 1)
            .with_priority(30)
            .with_min_significance(0.3),
        TriggerRule::new("population_loss", EventType::Raid, 0.3, 5)
            .with_priority(15)
            .with_min_significance(0.4),
        // Battle → victory/defeat events
        TriggerRule::new("military_change", EventType::Victory, 0.5, 0)
            .with_priority(20)
            .with_min_significance(0.5),
        TriggerRule::new("military_change", EventType::Defeat, 0.4, 0)
            .with_priority(20)
            .with_min_significance(0.5),
        // Conquest → cultural changes
        TriggerRule::new("border_shift", EventType::GovernmentReform, 0.3, 15)
            .with_priority(10)
            .with_min_significance(0.6),
        TriggerRule::new("border_shift", EventType::CulturalAdoption, 0.4, 20)
            .with_priority(5)
            .with_min_significance(0.4),
        // Alliance → trade boost
        TriggerRule::new("alliance_formed", EventType::TradeRouteEstablished, 0.5, 5)
            .with_priority(15)
            .with_min_significance(0.4),
        TriggerRule::new("alliance_formed", EventType::CulturalAchievement, 0.3, 10)
            .with_priority(10)
            .with_min_significance(0.5),
        // Disaster → recovery events
        TriggerRule::new("destruction", EventType::Reconstruction, 0.6, 10)
            .with_priority(20)
            .with_min_significance(0.3),
        TriggerRule::new("destruction", EventType::Migration, 0.4, 5)
            .with_priority(15)
            .with_min_significance(0.4),
        // Settlement → population growth
        TriggerRule::new("territory_claim", EventType::SettlementFounded, 0.3, 20)
            .with_priority(10)
            .with_min_significance(0.5),
        // Discovery → technological advancement
        TriggerRule::new("discovery", EventType::Invention, 0.4, 10)
            .with_priority(15)
            .with_min_significance(0.4),
        TriggerRule::new("resource_discovery", EventType::EconomicChange, 0.5, 5)
            .with_priority(20)
            .with_min_significance(0.3),
        // Conflict → diplomatic events
        TriggerRule::new("hostility", EventType::AllianceBroken, 0.6, 0)
            .with_priority(30)
            .with_min_significance(0.5),
        TriggerRule::new("peace", EventType::Treaty, 0.7, 0)
            .with_priority(30)
            .with_min_significance(0.6),
        // Society formation rules
        TriggerRule::new("society_formation", EventType::CulturalAchievement, 0.4, 20)
            .with_priority(10)
            .with_min_significance(0.5),
        TriggerRule::new("society_formation", EventType::SettlementFounded, 0.3, 30)
            .with_priority(5)
            .with_min_significance(0.4),
        // Figure events
        TriggerRule::new("figure_rise", EventType::Succession, 0.3, 0)
            .with_priority(25)
            .with_min_significance(0.6),
        TriggerRule::new("figure_rise", EventType::GoldenAge, 0.2, 50)
            .with_priority(5)
            .with_min_significance(0.7),
        TriggerRule::new("figure_death", EventType::Succession, 0.5, 0)
            .with_priority(30)
            .with_min_significance(0.4),
        TriggerRule::new("figure_death", EventType::CivilUnrest, 0.3, 5)
            .with_priority(15)
            .with_min_significance(0.6),
        // Artifact events
        TriggerRule::new("artifact_creation", EventType::CulturalAchievement, 0.6, 0)
            .with_priority(20)
            .with_min_significance(0.5),
        TriggerRule::new("artifact_creation", EventType::Festival, 0.4, 10)
            .with_priority(15)
            .with_min_significance(0.4),
        TriggerRule::new(
            "artifact_activation",
            EventType::MagicalCatastrophe,
            0.15,
            0,
        )
        .with_priority(40)
        .with_min_significance(0.8),
        TriggerRule::new("artifact_activation", EventType::ReligiousEvent, 0.5, 0)
            .with_priority(25)
            .with_min_significance(0.6),
    ]
}

/// Secondary event processor that generates cascading events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEventProcessor {
    /// Rules for triggering secondary events.
    rules: Vec<TriggerRule>,

    /// Queue of pending secondary events.
    queue: SecondaryEventQueue,

    /// Seed for deterministic randomness.
    seed: u64,
}

impl SecondaryEventProcessor {
    /// Create a new processor with default rules.
    pub fn new(seed: u64) -> Self {
        Self::with_rules(seed, default_trigger_rules())
    }

    /// Create with custom rules.
    pub fn with_rules(seed: u64, rules: Vec<TriggerRule>) -> Self {
        Self {
            rules,
            queue: SecondaryEventQueue::new(),
            seed,
        }
    }

    /// Process a primary event and generate secondary candidates.
    pub fn process_primary_event(
        &mut self,
        event: &Event,
        _current_year: i32,
    ) -> Vec<SecondaryEventCandidate> {
        let mut candidates = Vec::new();
        let significance = event.significance.unwrap_or(0.5);

        for rule in &self.rules {
            // Check significance threshold
            if significance < rule.min_significance {
                continue;
            }

            // Check if any effect matches the trigger
            for effect in &event.effects {
                if effect.effect_name() == rule.trigger_effect {
                    // Calculate probability with randomness
                    let prob =
                        self.calculate_trigger_probability(rule.base_probability, &event.id.id);

                    if prob > 0.0 {
                        let candidate = SecondaryEventCandidate::new(
                            event.id.id,
                            rule.secondary_event_type,
                            rule.time_offset_years,
                        )
                        .with_probability(prob)
                        .with_priority(rule.priority)
                        .with_reason(&format!(
                            "Triggered by {:?} from {:?}",
                            rule.trigger_effect, event.name
                        ));

                        // Inherit location if configured
                        if rule.inherit_location {
                            if let Some(loc) = event.location_id {
                                candidates.push(candidate.with_location(loc));
                            } else {
                                candidates.push(candidate);
                            }
                        } else {
                            candidates.push(candidate);
                        }
                    }
                }
            }
        }

        // Sort by priority
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Add to queue
        for candidate in &candidates {
            self.queue.enqueue(candidate.clone());
        }

        candidates
    }

    /// Calculate probability with deterministic randomness.
    fn calculate_trigger_probability(&self, base_prob: f32, event_id: &Uuid) -> f32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        event_id.hash(&mut hasher);
        base_prob.to_bits().hash(&mut hasher);
        self.seed.hash(&mut hasher);

        let hash = hasher.finish();
        let random_factor = ((hash % 1000) as f32) / 1000.0;

        // If random factor is below base probability, trigger occurs
        if random_factor < base_prob {
            base_prob
        } else {
            0.0
        }
    }

    /// Get the next pending secondary event.
    pub fn get_next(&mut self) -> Option<SecondaryEventCandidate> {
        self.queue.dequeue()
    }

    /// Process all pending secondary events for a world.
    pub fn process_all(&mut self, world_id: Uuid, base_year: i32) -> Vec<Event> {
        let mut events = Vec::new();

        while let Some(candidate) = self.queue.dequeue() {
            // Apply probability check
            if self.should_trigger(&candidate) {
                events.push(candidate.to_event(world_id, base_year));
            }
        }

        events
    }

    /// Determine if a candidate should trigger.
    fn should_trigger(&self, candidate: &SecondaryEventCandidate) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        candidate.source_event_id.hash(&mut hasher);
        candidate.event_type.name().hash(&mut hasher);
        self.seed.hash(&mut hasher);

        let hash = hasher.finish();
        let random_factor = ((hash % 1000) as f32) / 1000.0;

        random_factor < candidate.probability
    }

    /// Get all rules.
    pub fn get_rules(&self) -> &[TriggerRule] {
        &self.rules
    }

    /// Add a custom rule.
    pub fn add_rule(&mut self, rule: TriggerRule) {
        self.rules.push(rule);
    }

    /// Clear the queue.
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Get queue length.
    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBuilder;
    use uuid::Uuid;

    #[test]
    fn test_secondary_event_queue() {
        let mut queue = SecondaryEventQueue::new();

        let candidate = SecondaryEventCandidate::new(Uuid::new_v4(), EventType::Plague, 5);

        assert!(queue.enqueue(candidate.clone()));
        assert_eq!(queue.len(), 1);

        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.event_type, EventType::Plague);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_trigger_rule_creation() {
        let rule = TriggerRule::new("population_loss", EventType::Famine, 0.5, 3);

        assert_eq!(rule.trigger_effect, "population_loss");
        assert_eq!(rule.secondary_event_type, EventType::Famine);
        assert_eq!(rule.base_probability, 0.5);
        assert_eq!(rule.time_offset_years, 3);
    }

    #[test]
    fn test_default_rules_exist() {
        let rules = default_trigger_rules();
        assert!(!rules.is_empty());

        // Check some expected rules exist
        let rule_names: Vec<_> = rules.iter().map(|r| r.trigger_effect.as_str()).collect();

        assert!(rule_names.contains(&"population_loss"));
        assert!(rule_names.contains(&"border_shift"));
    }

    #[test]
    fn test_process_primary_event() {
        let mut processor = SecondaryEventProcessor::new(42);

        let target_id = Uuid::new_v4();
        let event = EventBuilder::new("The Great Plague")
            .event_type(EventType::Plague)
            .time(HistoricalTime::year(1347))
            .location(target_id)
            .significance(0.8)
            .effect(EventEffect::PopulationLoss {
                target: target_id,
                amount: 1000000,
                duration_years: Some(50),
                cause: Some("The Great Plague".to_string()),
            })
            .build(Uuid::new_v4());

        let candidates = processor.process_primary_event(&event, 1347);

        // Should generate some secondary events
        assert!(!candidates.is_empty());

        // Check queue has events
        assert!(processor.queue_length() > 0);
    }

    #[test]
    fn test_candidate_to_event() {
        let world_id = Uuid::new_v4();
        let candidate = SecondaryEventCandidate::new(Uuid::new_v4(), EventType::Migration, 5)
            .with_probability(0.7)
            .with_priority(10)
            .with_reason("Plague aftermath")
            .with_location(Uuid::new_v4());

        let event = candidate.to_event(world_id, 1347);

        assert_eq!(event.event_type, EventType::Migration);
        assert!(event.time.get_year() > 1347);
    }

    #[test]
    fn test_determinism() {
        let mut processor1 = SecondaryEventProcessor::new(42);
        let mut processor2 = SecondaryEventProcessor::new(42);

        let event = EventBuilder::new("Test")
            .event_type(EventType::Plague)
            .time(HistoricalTime::year(1000))
            .significance(0.8)
            .effect(EventEffect::PopulationLoss {
                target: Uuid::new_v4(),
                amount: 1000,
                duration_years: None,
                cause: None,
            })
            .build(Uuid::new_v4());

        let candidates1 = processor1.process_primary_event(&event, 1000);
        let candidates2 = processor2.process_primary_event(&event, 1000);

        assert_eq!(candidates1.len(), candidates2.len());
    }
}
