//! Figures resource routes
//!
//! Handles historical figure retrieval and detail queries.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};

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
    Query(params): Query<FigureQueryParams>,
) -> Result<Json<ApiResponse<FigureDetailResponse>>, ApiError> {
    // Validate UUID format
    let _ = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid figure ID format".to_string()))?;

    // Search all worlds for the figure
    let worlds = state
        .storage
        .list_worlds()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    for world_info in &worlds {
        let figures_path = state.storage.figures_path(&world_info.world_id);
        if figures_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&figures_path) {
                // Try to parse as array of figures
                if let Ok(figures) = serde_json::from_str::<Vec<HistoricalFigure>>(&content) {
                    if let Some(figure) = figures.iter().find(|f| f.id == id) {
                        // Build response with optional details
                        let mut response = FigureDetailResponse {
                            figure: figure.clone(),
                            world_id: world_info.world_id.clone(),
                            relationships: None,
                            related_events: None,
                        };

                        // Optionally load relationships
                        if params.include_relationships.unwrap_or(false) {
                            // TODO: Load relationships from relationship store
                            response.relationships = Some(Vec::new());
                        }

                        // Optionally load related events
                        if params.include_events.unwrap_or(false) {
                            // TODO: Load events from events store
                            response.related_events = Some(Vec::new());
                        }

                        return Ok(Json(ApiResponse::new(response)));
                    }
                }
                // Try to parse as single figure object
                else if let Ok(figure) = serde_json::from_str::<HistoricalFigure>(&content) {
                    if figure.id == id {
                        let response = FigureDetailResponse {
                            figure,
                            world_id: world_info.world_id.clone(),
                            relationships: None,
                            related_events: None,
                        };
                        return Ok(Json(ApiResponse::new(response)));
                    }
                }
            }
        }
    }

    // Figure not found in any world
    Err(ApiError::NotFound(format!("Figure '{}' not found", id)))
}
