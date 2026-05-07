//! Event resource routes
//!
//! Handles event detail retrieval.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};

use crate::api::error::ApiError;
use crate::api::models::*;

/// Registers event routes under /api/v1/events
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/{id}", get(get_event))
        .with_state(state)
}

/// GET /api/v1/events/{id} - Get event details
async fn get_event(
    State(_state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Query(_params): Query<EventQueryParams>,
) -> Result<Json<ApiResponse<EventResponse>>, ApiError> {
    uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid event ID format".to_string()))?;

    // TODO: Fetch event from EventStore
    // TODO: Include related events if params.include_related is set

    Err(ApiError::NotFound(format!("Event '{}' not found", id)))
}
