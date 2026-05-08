//! History timeline for archiving simulation state

use super::result::{TimelineEvent, YearResult};
use serde::{Deserialize, Serialize};

/// History timeline - archives events and state changes during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryTimeline {
    pub events: Vec<TimelineEvent>,
    pub year_summaries: Vec<YearSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearSummary {
    pub year: u32,
    pub population_total: u32,
    pub settlement_count: u32,
    pub active_figures: u32,
    pub artifacts_count: u32,
}

impl HistoryTimeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            year_summaries: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
    }

    pub fn add_year_summary(&mut self, summary: YearSummary) {
        self.year_summaries.push(summary);
    }

    pub fn get_events_in_range(&self, start_year: u32, end_year: u32) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.year >= start_year && e.year <= end_year)
            .collect()
    }
}

impl Default for HistoryTimeline {
    fn default() -> Self {
        Self::new()
    }
}
