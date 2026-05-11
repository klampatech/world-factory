//! Figures resource routes
//!
//! Handles historical figure retrieval and detail queries.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::models::*;

/// Registers figure routes under /api/v1/figures
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        // GET /api/v1/figures/{id} - Get single figure by ID (cross-world lookup)
        .route("/{id}", get(get_figure))
        .with_state(state)
}

/// Query parameters for GET /api/v1/figures/{id}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FigureQueryParams {
    /// Include relationships with other figures
    #[serde(default)]
    pub include_relationships: Option<bool>,
    /// Include related events
    #[serde(default)]
    pub include_events: Option<bool>,
}

/// GET /api/v1/figures/{id} - Get figure details by ID
///
/// Searches across all worlds for a figure with the given ID.
///
/// Query params:
/// - include_relationships: Include related figure relationships (default: false)
/// - include_events: Include related historical events (default: false)
async fn get_figure(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Query(_params): Query<FigureQueryParams>,
) -> Result<Json<ApiResponse<HistoricalFigure>>, ApiError> {
    // Accept both UUID and legacy ID formats (e.g., 'fig-0')
    // Search for figure using both UUID and string representation
    let search_id = id.clone();

    // Search all worlds for the figure
    let worlds = state
        .storage
        .list_worlds()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    for world_info in &worlds {
        let figures_path = state.storage.figures_path(&world_info.world_id);
        if figures_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&figures_path) {
                // Try to parse as array of NotableFigure (domain type)
                if let Ok(figures) =
                    serde_json::from_str::<Vec<crate::figures::NotableFigure>>(&content)
                {
                    // Try UUID match first
                    if let Some(figure) = figures.iter().find(|f| f.id.to_uuid().to_string() == search_id) {
                        let response = HistoricalFigure::from(figure);
                        return Ok(Json(ApiResponse::new(response)));
                    }
                    // Try legacy ID match
                    if let Some(figure) = figures.iter().find(|f| f.id.to_string() == search_id) {
                        let response = HistoricalFigure::from(figure);
                        return Ok(Json(ApiResponse::new(response)));
                    }
                }
                // Try to parse as single NotableFigure object
                else if let Ok(figure) =
                    serde_json::from_str::<crate::figures::NotableFigure>(&content)
                {
                    if figure.id.to_uuid().to_string() == search_id || figure.id.to_string() == search_id {
                        let response = HistoricalFigure::from(&figure);
                        return Ok(Json(ApiResponse::new(response)));
                    }
                }
            }
        }
    }

    // Figure not found in any world - return 404
    Err(ApiError::NotFound(format!("Figure '{}' not found", id)))
}
