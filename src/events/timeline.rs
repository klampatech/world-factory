//! Event Timeline Module
//! 
//! Provides timeline management for historical events.
//! Events are ordered chronologically and can be queried by time range,
//! type, participants, or location.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{Event, EventType, EventCategory, EventStore};

/// A timeline of events with ordering and querying capabilities.
/// 
/// The EventTimeline provides a view over an EventStore with additional
/// timeline-specific operations like range queries, filtering, and iteration.
/// 
/// # Example
/// 
/// ```rust
/// use world_factory::events::timeline::EventTimeline;
///
/// let mut timeline = EventTimeline::new();
/// timeline.add_event(event1);
/// timeline.add_event(event2);
/// timeline.sort();
///
/// // Iterate over events in chronological order
/// for event in timeline.iter() {
///     println!("{}: {}", event.time, event.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimeline {
    /// All events in this timeline.
    events: Vec<Event>,
    
    /// Year range of the timeline (computed on sort).
    #[serde(skip)]
    year_range: Option<(i32, i32)>,
    
    /// Whether the timeline is sorted.
    #[serde(skip)]
    sorted: bool,
}

impl Default for EventTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl EventTimeline {
    /// Create a new empty timeline.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            year_range: None,
            sorted: false,
        }
    }
    
    /// Create a timeline from an existing event store.
    pub fn from_store(store: &EventStore) -> Self {
        let mut timeline = Self::new();
        timeline.events.extend(store.events().iter().cloned());
        timeline.sort();
        timeline
    }
    
    /// Add an event to the timeline.
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.sorted = false;
        self.year_range = None;
    }
    
    /// Add multiple events to the timeline.
    pub fn add_events(&mut self, events: impl IntoIterator<Item = Event>) {
        self.events.extend(events);
        self.sorted = false;
        self.year_range = None;
    }
    
    /// Number of events in the timeline.
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Check if timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    /// Get all events.
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    
    /// Get events mutable.
    pub fn events_mut(&mut self) -> &mut Vec<Event> {
        self.sorted = false;
        self.year_range = None;
        &mut self.events
    }
    
    /// Sort events chronologically.
    /// 
    /// Events are sorted by:
    /// 1. Start year (ascending)
    /// 2. End time if present (for duration events)
    /// 3. Significance (descending) for same-year events
    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| {
            let ay = a.time.get_year();
            let by = b.time.get_year();
            
            if ay != by {
                return ay.cmp(&by);
            }
            
            // Same year: sort by significance (major events first)
            let sig_a = a.significance.unwrap_or(0.5);
            let sig_b = b.significance.unwrap_or(0.5);
            sig_b.partial_cmp(&sig_a).unwrap()
        });
        
        self.sorted = true;
        self.compute_year_range();
    }
    
    /// Compute the year range of this timeline.
    fn compute_year_range(&mut self) {
        let mut min_year = i32::MAX;
        let mut max_year = i32::MIN;
        
        for event in &self.events {
            let start_year = event.time.get_year();
            let end_year = event.end_time.as_ref()
                .map(|t| t.get_year())
                .unwrap_or(start_year);
            
            if start_year > 0 && start_year < min_year {
                min_year = start_year;
            }
            if end_year > 0 && end_year > max_year {
                max_year = end_year;
            }
        }
        
        self.year_range = if min_year != i32::MAX && max_year != i32::MIN {
            Some((min_year, max_year))
        } else {
            None
        };
    }
    
    /// Get the year range of this timeline.
    pub fn year_range(&self) -> Option<(i32, i32)> {
        self.year_range
    }
    
    /// Iterate over events in chronological order.
    /// 
    /// Requires timeline to be sorted first.
    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }
    
    /// Iterate over events mutable.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Event> {
        self.events.iter_mut()
    }
    
    /// Get events within a year range (inclusive).
    /// 
    /// # Arguments
    /// 
    /// * `start_year` - Start of range (inclusive)
    /// * `end_year` - End of range (inclusive)
    /// 
    /// # Returns
    /// 
    /// Events whose time falls within [start_year, end_year].
    pub fn in_range(&self, start_year: i32, end_year: i32) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| {
                let year = e.time.get_year();
                year >= start_year && year <= end_year
            })
            .collect()
    }
    
    /// Get events by type.
    pub fn by_type(&self, event_type: EventType) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }
    
    /// Get events by category.
    pub fn by_category(&self, category: EventCategory) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.event_type.category() == category)
            .collect()
    }
    
    /// Get events at a specific location.
    pub fn at_location(&self, location_id: Uuid) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.location_id == Some(location_id))
            .collect()
    }
    
    /// Get events involving a participant.
    pub fn with_participant(&self, participant_id: Uuid) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.participants.as_ref()
                .map(|p| p.contains(&participant_id))
                .unwrap_or(false))
            .collect()
    }
    
    /// Get events with a specific effect type.
    pub fn with_effect(&self, effect_name: &str) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.effects.iter().any(|eff| eff.effect_name() == effect_name))
            .collect()
    }
    
    /// Get events with significance >= threshold.
    pub fn significant(&self, threshold: f32) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.significance.unwrap_or(0.0) >= threshold)
            .collect()
    }
    
    /// Get the first event (earliest).
    pub fn first(&self) -> Option<&Event> {
        self.events.first()
    }
    
    /// Get the last event (latest).
    pub fn last(&self) -> Option<&Event> {
        self.events.last()
    }
    
    /// Get events that occurred during a specific year.
    /// 
    /// This includes:
    /// - Events that started in the year
    /// - Events that ended in the year (duration events)
    /// - Events ongoing during the year
    pub fn during_year(&self, year: i32) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| {
                let start = e.time.get_year();
                let end = e.end_time.as_ref()
                    .map(|t| t.get_year())
                    .unwrap_or(start);
                
                start <= year && end >= year
            })
            .collect()
    }
    
    /// Get timeline statistics.
    pub fn stats(&self) -> TimelineStats {
        let mut by_type: std::collections::HashMap<EventType, usize> = std::collections::HashMap::new();
        let mut by_category: std::collections::HashMap<EventCategory, usize> = std::collections::HashMap::new();
        let mut total_significance = 0.0f32;
        let mut significance_count = 0;
        
        for event in &self.events {
            *by_type.entry(event.event_type).or_insert(0) += 1;
            *by_category.entry(event.event_type.category()).or_insert(0) += 1;
            
            if let Some(sig) = event.significance {
                total_significance += sig;
                significance_count += 1;
            }
        }
        
        TimelineStats {
            total_events: self.events.len(),
            year_range: self.year_range,
            by_type,
            by_category,
            average_significance: if significance_count > 0 {
                total_significance / significance_count as f32
            } else {
                0.0
            },
        }
    }
    
    /// Create a slice of the timeline for a specific era.
    pub fn era(&self, start_year: i32, end_year: i32) -> TimelineSlice {
        let events = self.in_range(start_year, end_year);
        TimelineSlice {
            events: events.into_iter().cloned().collect(),
            start_year,
            end_year,
        }
    }
    
    /// Find events by name (partial match).
    pub fn find_by_name(&self, query: &str) -> Vec<&Event> {
        let query_lower = query.to_lowercase();
        self.events.iter()
            .filter(|e| e.name.to_lowercase().contains(&query_lower))
            .collect()
    }
    
    /// Get the most significant events (top N).
    pub fn top_events(&self, n: usize) -> Vec<&Event> {
        let mut sorted: Vec<_> = self.events.iter()
            .filter(|e| e.significance.is_some())
            .collect();
        
        sorted.sort_by(|a, b| {
            b.significance.unwrap().partial_cmp(&a.significance.unwrap()).unwrap()
        });
        
        sorted.into_iter().take(n).collect()
    }
    
    /// Get events affecting a specific entity (as location or participant).
    pub fn affecting(&self, entity_id: Uuid) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| {
                e.location_id == Some(entity_id) ||
                e.participants.as_ref()
                    .map(|p| p.contains(&entity_id))
                    .unwrap_or(false)
            })
            .collect()
    }
    
    /// Export events as a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }
    
    /// Import events from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let events: Vec<Event> = serde_json::from_str(json)?;
        let mut timeline = Self::new();
        timeline.add_events(events);
        timeline.sort();
        Ok(timeline)
    }
}

/// A slice of a timeline for a specific era.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSlice {
    events: Vec<Event>,
    start_year: i32,
    end_year: i32,
}

impl TimelineSlice {
    /// Get events in this slice.
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    
    /// Year range of this slice.
    pub fn year_range(&self) -> (i32, i32) {
        (self.start_year, self.end_year)
    }
    
    /// Number of events in this slice.
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Statistics about a timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStats {
    pub total_events: usize,
    pub year_range: Option<(i32, i32)>,
    pub by_type: std::collections::HashMap<EventType, usize>,
    pub by_category: std::collections::HashMap<EventCategory, usize>,
    pub average_significance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBuilder;
    use crate::HistoricalTime;
    use crate::Uuid;
    
    fn create_test_event(year: i32, event_type: EventType) -> Event {
        EventBuilder::new(format!("Event {}", year))
            .event_type(event_type)
            .time(HistoricalTime::year(year))
            .build(Uuid::new_v4())
    }
    
    #[test]
    fn test_timeline_creation() {
        let mut timeline = EventTimeline::new();
        timeline.add_event(create_test_event(1000, EventType::SettlementFounded));
        timeline.add_event(create_test_event(1200, EventType::WarDeclared));
        timeline.add_event(create_test_event(1100, EventType::Discovery));
        
        assert_eq!(timeline.len(), 3);
    }
    
    #[test]
    fn test_timeline_sorting() {
        let mut timeline = EventTimeline::new();
        timeline.add_event(create_test_event(1200, EventType::WarDeclared));
        timeline.add_event(create_test_event(1000, EventType::SettlementFounded));
        timeline.add_event(create_test_event(1100, EventType::Discovery));
        
        timeline.sort();
        
        let events: Vec<_> = timeline.iter().collect();
        assert_eq!(events[0].time.get_year(), 1000);
        assert_eq!(events[1].time.get_year(), 1100);
        assert_eq!(events[2].time.get_year(), 1200);
    }
    
    #[test]
    fn test_timeline_year_range() {
        let mut timeline = EventTimeline::new();
        timeline.add_event(create_test_event(1000, EventType::SettlementFounded));
        timeline.add_event(create_test_event(1500, EventType::WarDeclared));
        
        timeline.sort();
        
        assert_eq!(timeline.year_range(), Some((1000, 1500)));
    }
    
    #[test]
    fn test_timeline_range_query() {
        let mut timeline = EventTimeline::new();
        timeline.add_event(create_test_event(1000, EventType::SettlementFounded));
        timeline.add_event(create_test_event(1200, EventType::WarDeclared));
        timeline.add_event(create_test_event(1400, EventType::Discovery));
        
        timeline.sort();
        
        let events = timeline.in_range(1100, 1300);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].time.get_year(), 1200);
    }
    
    #[test]
    fn test_timeline_type_filter() {
        let mut timeline = EventTimeline::new();
        timeline.add_event(create_test_event(1000, EventType::SettlementFounded));
        timeline.add_event(create_test_event(1100, EventType::WarDeclared));
        timeline.add_event(create_test_event(1200, EventType::SettlementFounded));
        
        let settlements = timeline.by_type(EventType::SettlementFounded);
        assert_eq!(settlements.len(), 2);
    }
    
    #[test]
    fn test_timeline_stats() {
        let mut timeline = EventTimeline::new();
        let mut e1 = create_test_event(1000, EventType::SettlementFounded);
        e1.significance = Some(0.8);
        
        let mut e2 = create_test_event(1100, EventType::WarDeclared);
        e2.significance = Some(0.9);
        
        timeline.add_event(e1);
        timeline.add_event(e2);
        timeline.sort();
        
        let stats = timeline.stats();
        assert_eq!(stats.total_events, 2);
        assert!((stats.average_significance - 0.85).abs() < 0.01);
    }
}