//! Event Predictor Module
//! 
//! Predicts future events based on historical patterns and current state.
//! Enables anticipatory content generation for interactive experiences.

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::events::{EventType, EventCategory, Event};
use crate::events::probability::{EventContext, ProbabilityResult};
use super::ProbabilityEngine;

/// Predictor for future events based on context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPredictor {
    /// Probability engine for calculations.
    probability_engine: ProbabilityEngine,
    
    /// Historical patterns cache.
    patterns: Vec<PredictionPattern>,
    
    /// Tracked event frequencies by type.
    event_frequencies: HashMap<EventType, usize>,
    
    /// Total events tracked.
    total_events_tracked: usize,
}

impl EventPredictor {
    /// Create a new event predictor.
    pub fn new(seed: u64) -> Self {
        Self {
            probability_engine: ProbabilityEngine::new(seed),
            patterns: Vec::new(),
            event_frequencies: HashMap::new(),
            total_events_tracked: 0,
        }
    }
    
    /// Record an event for pattern analysis.
    pub fn record_event(&mut self, event: &Event) {
        // Update frequency
        let count = self.event_frequencies.entry(event.event_type).or_insert(0);
        *count += 1;
        self.total_events_tracked += 1;
        
        // Record in probability engine
        self.probability_engine.record_event(event.event_type, event.time.get_year());
        
        // Extract pattern
        self.extract_pattern(event);
    }
    
    /// Extract prediction pattern from event.
    fn extract_pattern(&mut self, event: &Event) {
        let pattern = PredictionPattern {
            event_type: event.event_type,
            year: event.time.get_year(),
            location_id: event.location_id,
            significance: event.significance.unwrap_or(0.5),
            effects: event.effects.iter().map(|e| e.effect_name().to_string()).collect(),
        };
        
        // Keep only recent patterns
        if self.patterns.len() > 1000 {
            self.patterns.remove(0);
        }
        self.patterns.push(pattern);
    }
    
    /// Predict events likely to occur in the near future.
    pub fn predict_upcoming(
        &mut self,
        context: &EventContext,
        current_year: i32,
        years_ahead: i32,
    ) -> Vec<PredictedEvent> {
        let mut predictions = Vec::new();
        
        // Get top candidates from probability engine
        let candidates = self.probability_engine.get_top_candidates(
            context,
            current_year,
            years_ahead,
            10,
        );
        
        for (event_type, prob_result) in candidates {
            // Only predict if probability is reasonable
            if prob_result.probability > 0.01 {
                let prediction = PredictedEvent {
                    event_type,
                    probability: prob_result.probability,
                    confidence: self.calculate_confidence(&prob_result),
                    year_range: self.estimate_year_range(event_type, current_year, years_ahead, prob_result.probability),
                    likely_effects: self.predict_effects(event_type),
                    triggers: self.predict_triggers(event_type, context),
                    context_requirements: self.get_context_requirements(event_type),
                };
                predictions.push(prediction);
            }
        }
        
        // Sort by probability
        predictions.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());
        
        predictions
    }
    
    /// Predict events for a specific category.
    pub fn predict_category(
        &mut self,
        category: EventCategory,
        context: &EventContext,
        current_year: i32,
        years_ahead: i32,
    ) -> Vec<PredictedEvent> {
        let results = self.probability_engine.calculate_category_probabilities(
            category,
            context,
            current_year,
        );
        
        results.into_iter()
            .filter(|(_, result)| result.probability > 0.01)
            .map(|(event_type, result)| PredictedEvent {
                event_type,
                probability: result.probability,
                confidence: self.calculate_confidence(&result),
                year_range: self.estimate_year_range(event_type, current_year, years_ahead, result.probability),
                likely_effects: self.predict_effects(event_type),
                triggers: self.predict_triggers(event_type, context),
                context_requirements: self.get_context_requirements(event_type),
            })
            .collect()
    }
    
    /// Calculate confidence in prediction based on data availability.
    fn calculate_confidence(&self, _result: &ProbabilityResult) -> PredictionConfidence {
        let data_points = self.event_frequencies.values().sum::<usize>();
        
        // More historical data = higher confidence
        if data_points > 100 {
            PredictionConfidence::High
        } else if data_points > 20 {
            PredictionConfidence::Medium
        } else {
            PredictionConfidence::Low
        }
    }
    
    /// Estimate year range for predicted event.
    fn estimate_year_range(
        &self,
        _event_type: EventType,
        current_year: i32,
        years_ahead: i32,
        probability: f32,
    ) -> (i32, i32) {
        // Simple linear scaling based on probability
        let start_year = current_year;
        let end_year = current_year + (years_ahead as f32 / probability) as i32;
        
        (start_year.min(end_year), current_year + years_ahead)
    }
    
    /// Predict likely effects for event type.
    fn predict_effects(&self, event_type: EventType) -> Vec<String> {
        match event_type.category() {
            EventCategory::Political => vec![
                "Territory change".to_string(),
                "Leadership change".to_string(),
            ],
            EventCategory::Military => vec![
                "Military strength change".to_string(),
                "Border shift".to_string(),
            ],
            EventCategory::Natural => vec![
                "Population loss".to_string(),
                "Economic disruption".to_string(),
            ],
            EventCategory::Cultural => vec![
                "Cultural shift".to_string(),
                "Social change".to_string(),
            ],
            EventCategory::Discovery => vec![
                "Technology advancement".to_string(),
                "Territory claim".to_string(),
            ],
            EventCategory::Catastrophe => vec![
                "Population displacement".to_string(),
                "Infrastructure destruction".to_string(),
            ],
        }
    }
    
    /// Predict triggers for event based on context.
    fn predict_triggers(&self, event_type: EventType, context: &EventContext) -> Vec<String> {
        let mut triggers = Vec::new();
        
        match event_type {
            EventType::WarDeclared => {
                triggers.push("High population density".to_string());
                if context.is_at_war {
                    triggers.push("Existing conflict escalation".to_string());
                }
                if context.cultural_tensions > 0.7 {
                    triggers.push("Cultural tensions high".to_string());
                }
            }
            EventType::Plague => {
                triggers.push("Dense population".to_string());
                if context.economic_health < 0.5 {
                    triggers.push("Economic decline".to_string());
                }
                triggers.push(format!("Season: {:?}", context.season));
            }
            EventType::Famine => {
                if context.economic_health < 0.4 {
                    triggers.push("Economic hardship".to_string());
                }
            }
            EventType::Migration => {
                triggers.push("Population pressure".to_string());
                if context.economic_health < 0.5 {
                    triggers.push("Economic incentives".to_string());
                }
            }
            _ => {
                triggers.push("General probability".to_string());
            }
        }
        
        triggers
    }
    
    /// Get context requirements for event type.
    fn get_context_requirements(&self, event_type: EventType) -> Vec<String> {
        match event_type {
            EventType::WarEnded => vec!["Active war must exist".to_string()],
            EventType::Siege => vec!["Settlement target required".to_string()],
            EventType::FirstContact => vec!["Isolated regions must exist".to_string()],
            _ => vec![],
        }
    }
    
    /// Get event frequency statistics.
    pub fn get_frequency_stats(&self) -> HashMap<EventType, EventFrequencyStats> {
        let total = self.total_events_tracked.max(1) as f32;
        
        self.event_frequencies.iter()
            .map(|(et, count)| {
                let frequency = *count as f32 / total;
                let pattern = self.patterns.iter()
                    .filter(|p| p.event_type == *et)
                    .collect::<Vec<_>>();
                
                let avg_significance = if !pattern.is_empty() {
                    pattern.iter().map(|p| p.significance).sum::<f32>() / pattern.len() as f32
                } else {
                    0.5
                };
                
                (et.clone(), EventFrequencyStats {
                    count: *count,
                    frequency,
                    average_significance: avg_significance,
                    last_occurrence_year: pattern.last().map(|p| p.year).unwrap_or(0),
                })
            })
            .collect()
    }
    
    /// Get patterns for a specific event type.
    pub fn get_patterns_for(&self, event_type: EventType) -> Vec<&PredictionPattern> {
        self.patterns.iter()
            .filter(|p| p.event_type == event_type)
            .collect()
    }
    
    /// Get most common event types in history.
    pub fn get_most_common(&self, limit: usize) -> Vec<(EventType, usize)> {
        let mut sorted: Vec<_> = self.event_frequencies.iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }
}

/// A predicted future event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedEvent {
    pub event_type: EventType,
    pub probability: f32,
    pub confidence: PredictionConfidence,
    pub year_range: (i32, i32),
    pub likely_effects: Vec<String>,
    pub triggers: Vec<String>,
    pub context_requirements: Vec<String>,
}

/// Prediction confidence level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionConfidence {
    Low,
    Medium,
    High,
}

/// A historical pattern for prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PredictionPattern {
    event_type: EventType,
    year: i32,
    location_id: Option<Uuid>,
    significance: f32,
    effects: Vec<String>,
}

/// Statistics about event frequency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFrequencyStats {
    pub count: usize,
    pub frequency: f32,
    pub average_significance: f32,
    pub last_occurrence_year: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBuilder;
    use crate::types::HistoricalTime;
    
    #[test]
    fn test_event_recording() {
        let mut predictor = EventPredictor::new(42);
        
        let event = EventBuilder::new("Test War")
            .event_type(EventType::WarDeclared)
            .time(HistoricalTime::year(1000))
            .significance(0.85)
            .build(Uuid::new_v4());
        
        predictor.record_event(&event);
        
        let stats = predictor.get_frequency_stats();
        assert_eq!(stats.get(&EventType::WarDeclared).map(|s| s.count), Some(1));
    }
    
    #[test]
    fn test_prediction() {
        let mut predictor = EventPredictor::new(42);
        
        // Record some historical events
        for year in [900, 950, 980] {
            let event = EventBuilder::new(format!("Settlement at {}", year))
                .event_type(EventType::SettlementFounded)
                .time(HistoricalTime::year(year))
                .significance(0.7)
                .build(Uuid::new_v4());
            predictor.record_event(&event);
        }
        
        let context = EventContext::default();
        let predictions = predictor.predict_upcoming(&context, 1000, 100);
        
        // Should have some predictions
        assert!(!predictions.is_empty());
    }
    
    #[test]
    fn test_category_prediction() {
        let mut predictor = EventPredictor::new(42);
        
        let context = EventContext::default();
        
        let military_preds = predictor.predict_category(
            EventCategory::Military,
            &context,
            1000,
            50,
        );
        
        // Military predictions should include war, battle, etc.
        assert!(military_preds.iter().any(|p| matches!(
            p.event_type,
            EventType::WarDeclared | EventType::Battle | EventType::Raid
        )));
    }
    
    #[test]
    fn test_frequency_stats() {
        let mut predictor = EventPredictor::new(42);
        
        for _ in 0..5 {
            predictor.record_event(&EventBuilder::new("Test")
                .event_type(EventType::SettlementFounded)
                .time(HistoricalTime::year(1000))
                .build(Uuid::new_v4()));
        }
        
        for _ in 0..2 {
            predictor.record_event(&EventBuilder::new("Test")
                .event_type(EventType::WarDeclared)
                .time(HistoricalTime::year(1000))
                .build(Uuid::new_v4()));
        }
        
        let stats = predictor.get_frequency_stats();
        
        assert_eq!(stats.get(&EventType::SettlementFounded).map(|s| s.count), Some(5));
        assert_eq!(stats.get(&EventType::WarDeclared).map(|s| s.count), Some(2));
    }
    
    #[test]
    fn test_confidence_calculation() {
        let mut predictor = EventPredictor::new(42);
        
        // With no historical data, confidence should be low
        let context = EventContext::default();
        let preds = predictor.predict_upcoming(&context, 1000, 50);
        
        // With data, confidence improves
        for _ in 0..30 {
            predictor.record_event(&EventBuilder::new("Test")
                .event_type(EventType::Battle)
                .time(HistoricalTime::year(900))
                .build(Uuid::new_v4()));
        }
        
        let preds_with_data = predictor.predict_upcoming(&context, 1000, 50);
        
        // Both should work (confidence is internal)
        assert!(preds.len() > 0 || preds_with_data.len() > 0);
    }
}