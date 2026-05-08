//! API v1 routes

pub mod worlds;

pub use worlds::{
    self, simulate_world, create_world, get_world, get_generation_status,
    list_worlds, register_world, update_generation_status,
    WorldState, WorldMetadata, WorldPhase, CreateWorldRequest, CreateWorldResponse,
    GenerationTask, GenerationStatus, SpeciesTemplate,
    // Dashboard endpoint types
    Disaster, ResourcesSummaryResponse, Resource, Figure, FiguresResponse, WorldStats,
    FiguresQueryParams, get_world_disasters, get_world_resources_summary,
    get_world_figures, get_world_stats, get_aggregate_stats,
    // Figure detail
    FigureDetail, register_figure, get_figure, list_figures,
    // History/events endpoint types
    HistoryEvent, HistoryQueryParams, get_world_history,
};

/// API error response type (for use by HTTP handlers)
#[derive(Debug)]
pub struct ApiAppError {
    pub status: u16,
    pub message: String,
}