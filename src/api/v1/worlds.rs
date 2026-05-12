//! World resource routes
//!
//! Handles world CRUD, generation triggering, and map retrieval.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "api")]
use tracing;

// Type alias for delete response
type DeleteResponse = crate::api::models::DeleteResponse;
type WorldDeleteResponse = crate::api::ApiResponse<DeleteResponse>;

use crate::api::error::ApiError;
use crate::api::models::*;
use crate::api::services::RiverService;

// Re-export WondersQueryParams for route handler
use crate::api::models::WondersQueryParams;

/// Registers world routes under /api/v1/worlds
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_worlds).post(create_world))
        .route("/{id}", get(get_world).delete(delete_world))
        .route("/{id}/generate", post(trigger_generation))
        .route("/{id}/map", get(get_world_map))
        .route("/{id}/timeline", get(get_world_timeline))
        .route("/{id}/events", get(get_world_events))
        .route("/{id}/history", get(get_world_history))
        .route("/{id}/history/events", get(get_history_events))
        .route("/{id}/figures", get(get_world_figures))
        .route("/{id}/figures/{figure_id}", get(get_world_figure))
        .route("/{id}/societies", get(get_world_societies))
        .route("/{id}/planet", get(get_world_planet))
        .route("/{id}/tectonics", get(get_world_tectonics))
        .route("/{id}/artifacts", get(get_world_artifacts))
        .route("/{id}/cataclysms", get(get_world_cataclysms))
        .route("/{id}/wonders", get(get_world_wonders))
        .route("/{id}/resources", get(get_world_resources))
        .route("/{id}/resources/summary", get(get_world_resources_summary))
        .route("/{id}/settlements", get(get_world_settlements))
        .route("/{id}/settlements/map", get(get_world_settlements_map))
        .route("/{id}/export", get(get_world_export))
        .route("/{id}/export.json", get(get_world_export_json))
        .route("/{id}/disasters", get(get_world_disasters))
        .route("/{id}/stats", get(get_world_stats))
        .route("/{id}/turn", get(get_world_turn).post(execute_turn_action))
        .route("/{id}/turn/action", post(execute_turn_action))
        .with_state(state)
}

// =============================================================================
// Handlers
// =============================================================================

// =============================================================================
// World Retrieval Handlers
// =============================================================================

/// Extended query parameters for GET /api/v1/worlds
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedListWorldsParams {
    /// Maximum number of results (default: 20, max: 100)
    #[serde(default = "default_worlds_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by genre (case-insensitive)
    #[serde(default)]
    pub genre: Option<String>,
    /// Filter by era (case-insensitive)
    #[serde(default)]
    pub era: Option<String>,
    /// Search by name (case-insensitive partial match)
    #[serde(default)]
    pub search: Option<String>,
    /// Sort field: created_at, updated_at, name (default: created_at)
    #[serde(default = "default_world_sort_field")]
    pub sort_by: String,
    /// Sort direction: asc, desc (default: desc)
    #[serde(default = "default_world_sort_dir")]
    pub sort_dir: String,
}

fn default_worlds_limit() -> usize {
    20
}
fn default_world_sort_field() -> String {
    "created_at".to_string()
}
fn default_world_sort_dir() -> String {
    "desc".to_string()
}

/// World summary for list view
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummary {
    pub id: String,
    pub name: String,
    pub genre: String,
    pub era: String,
    pub status: WorldStatus,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for world list endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldListResponse {
    pub total_worlds: usize,
    pub worlds: Vec<WorldSummary>,
    pub pagination: Pagination,
}

/// GET /api/v1/worlds - List all worlds with pagination and filtering
async fn list_worlds(
    State(state): State<crate::api::AppState>,
    Query(params): Query<ExtendedListWorldsParams>,
) -> Result<Json<ApiResponse<WorldListResponse>>, ApiError> {
    // Enforce pagination limits
    let limit = params.limit.min(100);
    let offset = params.offset.unwrap_or(0);

    // Validate sort field
    let valid_sort_fields = ["created_at", "updated_at", "name"];
    if !valid_sort_fields.contains(&params.sort_by.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid sort_by: '{}'. Valid fields: {:?}",
            params.sort_by, valid_sort_fields
        )));
    }

    // Validate sort direction
    if params.sort_dir != "asc" && params.sort_dir != "desc" {
        return Err(ApiError::BadRequest(
            "Invalid sort_dir: must be 'asc' or 'desc'".to_string(),
        ));
    }

    // Load worlds from storage with metadata
    let stored_worlds = state
        .storage
        .list_worlds()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Build world summaries with metadata loaded from storage
    let mut world_summaries: Vec<WorldSummary> = Vec::new();
    for stored in &stored_worlds {
        let metadata_path = state.storage.world_metadata_path(&stored.world_id);
        let (name, status, created_at) = if metadata_path.exists() {
            // Try to load from quick-access metadata JSON
            if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let name = json
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&stored.world_id)
                        .to_string();
                    let status = json
                        .get("status")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "Pending" => WorldStatus::Pending,
                            "Generating" => WorldStatus::Generating,
                            "Ready" => WorldStatus::Ready,
                            "Failed" => WorldStatus::Failed,
                            _ => WorldStatus::Ready,
                        })
                        .unwrap_or(WorldStatus::Ready);
                    let created_at = json
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    (name, status, created_at)
                } else {
                    (stored.world_id.clone(), WorldStatus::Ready, String::new())
                }
            } else {
                (stored.world_id.clone(), WorldStatus::Ready, String::new())
            }
        } else {
            (stored.world_id.clone(), WorldStatus::Ready, String::new())
        };

        let modified = stored
            .modified_at
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
            .unwrap_or_default();

        world_summaries.push(WorldSummary {
            id: stored.world_id.clone(),
            name,
            genre: "fantasy".to_string(),
            era: "medieval".to_string(),
            status,
            created_at,
            updated_at: modified,
            description: None,
        });
    }

    // Apply search filter if specified
    let filtered: Vec<WorldSummary> = if let Some(ref search) = params.search {
        let search_lower = search.to_lowercase();
        world_summaries
            .into_iter()
            .filter(|w| {
                w.name.to_lowercase().contains(&search_lower)
                    || w.id.to_lowercase().contains(&search_lower)
            })
            .collect()
    } else {
        world_summaries
    };

    // Apply sorting
    let mut worlds: Vec<WorldSummary> = filtered;
    match (params.sort_by.as_str(), params.sort_dir.as_str()) {
        ("name", "asc") => worlds.sort_by(|a, b| a.name.cmp(&b.name)),
        ("name", "desc") => worlds.sort_by(|a, b| b.name.cmp(&a.name)),
        ("updated_at", "asc") => worlds.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
        ("updated_at", "desc") => worlds.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
        _ => {} // Default: sorted by created_at desc (newest first)
    };

    let total = worlds.len();

    // Apply pagination
    worlds = worlds.into_iter().skip(offset).take(limit).collect();

    let response = WorldListResponse {
        total_worlds: total,
        worlds,
        pagination: Pagination {
            limit,
            offset,
            has_more: offset + limit < total,
        },
    };

    Ok(Json(ApiResponse::new(response)))
}

/// POST /api/v1/worlds - Create a new world for generation
async fn create_world(
    State(state): State<crate::api::AppState>,
    Json(req): Json<CreateWorldRequest>,
) -> Result<(StatusCode, Json<ApiResponse<World>>), ApiError> {
    // Validate name if provided
    let world_name = req.name.unwrap_or_else(|| "Untitled World".to_string());
    if world_name.len() > 100 {
        return Err(ApiError::BadRequest(
            "World name must be 100 characters or less".to_string(),
        ));
    }
    if world_name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "World name cannot be empty".to_string(),
        ));
    }

    // Generate or use provided seed
    let seed = req.parameters.as_ref().map(|p| p.seed).unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    });

    // Create the world entity using the domain World type from types.rs
    let domain_world = crate::types::World::new(world_name.clone(), seed);
    let world_id = domain_world.id.to_string(); // Format: "world:{uuid}"

    let world = World {
        id: world_id.clone(),
        name: world_name,
        status: WorldStatus::Generating, // Immediately mark as generating
        progress: Some(0.0),
        created_at: chrono::Utc::now().to_rfc3339(),
        parameters: req
            .parameters
            .clone()
            .unwrap_or_else(|| crate::api::models::WorldParameters {
                seed,
                size: crate::api::models::WorldSize::Medium,
            }),
    };

    // Save world package to storage directory
    let package = crate::packaging::WorldPackage {
        world: domain_world,
        regions: Vec::new(),
        settlements: Vec::new(),
        persons: Vec::new(),
        events: Vec::new(),
        timelines: Vec::new(),
        terrain: None,
    };

    let package_path = state.storage.world_package_path(&world.id);

    // Ensure the world directory exists
    if let Some(parent) = package_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ApiError::Internal(format!("Failed to create world directory: {}", e)))?;
    }

    crate::packaging::save_world_package(&package, &package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to save world package: {}", e)))?;

    // Save quick-access metadata JSON for list_worlds
    let metadata_path = state.storage.world_metadata_path(&world.id);
    let metadata = serde_json::json!({
        "id": world.id,
        "name": world.name,
        "status": "Generating",
        "seed": seed,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .map_err(|e| ApiError::Internal(format!("Failed to save metadata: {}", e)))?;

    // Spawn async generation task (fire-and-forget)
    let gen_world_id = world_id.clone();
    let gen_world_name = world.name.clone();
    tokio::spawn(async move {
        tracing::info!(
            "Async generation starting for world: {} (id: {})",
            gen_world_name,
            gen_world_id
        );

        // Run the actual world generation pipeline
        if let Err(e) = run_world_generation_internal(&gen_world_id).await {
            tracing::error!("World generation failed for {}: {}", gen_world_id, e);
        }
    });

    tracing::info!(
        "Created new world: {} (id: {}, seed: {})",
        world.name,
        world.id,
        seed
    );

    Ok((StatusCode::CREATED, Json(ApiResponse::new(world))))
}

/// GET /api/v1/worlds/{id} - Get world details
async fn get_world(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<World>>, ApiError> {
    // Normalize world ID (strip "world:" prefix if present)
    let world_id = crate::api::normalize_world_id(&id);

    // Check if world exists in storage
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // Load world from storage
    let package_path = state.storage.world_package_path(&world_id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;

    let domain_world = package.world;
    let world = World {
        id: format!("world:{}", world_id),
        name: domain_world.name.clone(),
        status: WorldStatus::Ready,
        progress: Some(1.0),
        created_at: domain_world.created_at.to_string(),
        parameters: crate::api::models::WorldParameters {
            seed: domain_world.seed,
            size: crate::api::models::WorldSize::Medium,
        },
    };

    Ok(Json(ApiResponse::new(world)))
}

/// DELETE /api/v1/worlds/{id} - Delete a world
async fn delete_world(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<WorldDeleteResponse>), crate::api::ApiError> {
    let world_id = crate::api::normalize_world_id(&id);

    // Check if world exists in storage
    if !state.storage.world_exists(&world_id) {
        return Err(crate::api::ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // Delete the world from storage
    state
        .storage
        .delete_world(&world_id)
        .map_err(|e| crate::api::ApiError::Internal(format!("Failed to delete world: {}", e)))?;

    tracing::info!("Deleted world: {}", world_id);

    Ok((
        StatusCode::NO_CONTENT,
        Json(crate::api::ApiResponse::new(DeleteResponse::deleted())),
    ))
}

/// POST /api/v1/worlds/{id}/generate - Trigger world generation
async fn trigger_generation(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Json(req): Json<GenerateWorldRequest>,
) -> Result<Json<ApiResponse<World>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    let world = World {
        id: world_id_raw,
        name: req.name.unwrap_or_else(|| "Untitled World".to_string()),
        status: WorldStatus::Generating,
        progress: Some(0.0),
        created_at: chrono::Utc::now().to_rfc3339(),
        parameters: req.parameters.unwrap_or_default(),
    };

    Ok(Json(ApiResponse::new(world)))
}

/// GET /api/v1/worlds/{id}/map - Get render-ready map data
async fn get_world_map(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<GetWorldMapParams>,
) -> Result<Json<ApiResponse<WorldMap>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);

    // Load world from storage to get seed and configuration
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    let package_path = state.storage.world_package_path(&world_id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;

    let world = &package.world;
    let seed = world.seed;

    // Build map dimensions from world config, or use defaults
    let (width, height) = world
        .config
        .as_ref()
        .map(|c| (c.width, c.height))
        .unwrap_or((256, 256));

    use crate::generation::{VoronoiConfig, VoronoiGenerator};
    use crate::terrain::{OceanDetectionConfig, OceanDetector, Polygon, PolygonGraph};

    let config = VoronoiConfig {
        width,
        height,
        num_seeds: 128,
        ..Default::default()
    };
    let mut generator = VoronoiGenerator::new(config, seed);
    let voronoi_result = generator.generate();

    // Extract polygon vertices from Voronoi
    let polygon_vertices = voronoi_result.extract_polygon_vertices();

    // Build a polygon graph for ocean detection
    // Each Voronoi cell becomes a polygon with computed elevation
    let mut graph = PolygonGraph::new();

    // Track cell centers for neighbor detection
    let mut cell_centers: Vec<(f32, f32)> = Vec::new();

    for (i, verts) in polygon_vertices.iter().enumerate() {
        if verts.len() >= 3 {
            // Compute cell center
            let center_x: f32 = verts.iter().map(|v| v.0).sum::<f32>() / verts.len() as f32;
            let center_y: f32 = verts.iter().map(|v| v.1).sum::<f32>() / verts.len() as f32;
            cell_centers.push((center_x, center_y));

            // Compute elevation based on distance from map edges
            // Edge cells have lower elevation (ocean), center cells are higher (land)
            let normalized_x = center_x / 256.0;
            let normalized_y = center_y / 256.0;

            // Distance from nearest edge (0 at edges, 1 at center)
            let edge_dist_x = (normalized_x * 2.0 - 1.0)
                .abs()
                .min(1.0 - normalized_x * 2.0 + 1.0);
            let edge_dist_y = (normalized_y * 2.0 - 1.0)
                .abs()
                .min(1.0 - normalized_y * 2.0 + 1.0);
            let edge_dist = edge_dist_x.min(edge_dist_y);

            // Add noise for variation using seeded pseudo-random based on cell index
            let noise = (((i as f32 * 12.9898).sin() * 43758.5453).fract() * 0.3
                + ((i as f32 * 78.233).cos() * 43758.5453).fract() * 0.2
                + 0.5)
                .clamp(0.0, 1.0);

            // Elevation: low at edges (ocean), higher toward center (land)
            // Scale so edges (dist ~0) become ocean, center (dist ~1) become land
            let elevation = (edge_dist * 0.7 + noise * 0.3).min(1.0);

            let mut polygon = Polygon::new(i as u32);
            polygon.elevation = elevation;
            polygon.base_elevation = elevation * 9000.0; // Convert to meters
            graph.add_polygon(polygon);
        }
    }

    // Connect neighbors based on spatial proximity (cells within threshold distance)
    let neighbor_threshold = 35.0;
    let n = cell_centers.len();
    for i in 0..n {
        let (cx1, cy1) = cell_centers[i];
        for j in (i + 1)..n {
            let (cx2, cy2) = cell_centers[j];
            let dist = ((cx1 - cx2).powi(2) + (cy1 - cy2).powi(2)).sqrt();
            if dist < neighbor_threshold {
                graph.add_edge(i as u32, j as u32);
                graph.add_edge(j as u32, i as u32);
            }
        }
    }

    // Run ocean detection
    let ocean_config = OceanDetectionConfig::default();
    let ocean_detector = OceanDetector::with_config(ocean_config);
    let coastal_ids = ocean_detector.detect_coastal_polygons(&graph);

    // Build API polygon list with ocean metadata
    let polygons: Vec<crate::api::models::Polygon> = (0..n)
        .filter_map(|i| -> Option<crate::api::models::Polygon> {
            let poly = graph.get(i as u32)?;
            let verts = &polygon_vertices[i];
            if verts.len() < 3 {
                return None;
            }

            let zone = ocean_detector.detect_zone(poly);
            let is_ocean = zone != crate::terrain::OceanZone::Land;
            let is_coastal = coastal_ids.contains(&poly.id);

            Some(crate::api::models::Polygon {
                id: format!("poly-{}", i),
                polygon_type: crate::api::models::PolygonType::Region,
                vertices: verts
                    .iter()
                    .map(|(x, y)| crate::api::models::Vertex {
                        x: *x as f64,
                        y: *y as f64,
                    })
                    .collect(),
                holes: None,
                elevation: Some(poly.elevation as f64),
                is_ocean: Some(is_ocean),
                is_coastal: Some(is_coastal),
                ocean_zone: Some(match zone {
                    crate::terrain::OceanZone::Land => "land".to_string(),
                    crate::terrain::OceanZone::ShallowOcean => "shallow".to_string(),
                    crate::terrain::OceanZone::MediumOcean => "medium".to_string(),
                    crate::terrain::OceanZone::DeepOcean => "deep".to_string(),
                }),
            })
        })
        .collect();

    let map = WorldMap {
        world_id: world_id,
        dimensions: MapDimensions {
            width: 256,
            height: 256,
        },
        scale: 1.0,
        polygons,
        biomes: Vec::new(),
        resources: Vec::new(),
        entities: Vec::new(),
        elevation_grid: None,
        metadata: MapMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    Ok(Json(ApiResponse::new(map)))
}

/// GET /api/v1/worlds/{id}/timeline - Get timeline events for a world
async fn get_world_timeline(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<TimelineResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists in storage
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // TODO: Fetch timeline from EventStore
    let response =
        TimelineResponse::new(world_id, Vec::new(), 0, params.start_year, params.end_year);

    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/{id}/events - Get events for a world
async fn get_world_events(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<EventsListResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // TODO: Fetch events from EventStore
    let response = EventsListResponse {
        events: Vec::new(),
        total: 0,
        limit: params.limit,
        offset: params.offset.unwrap_or(0),
    };

    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/{id}/history - Get history events for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - event_types: Comma-separated event types to include
/// - start_year: Start year (inclusive)
/// - end_year: End year (inclusive)
/// - entity_id: Filter by entity involvement
/// - min_significance: Minimum significance (0.0 - 1.0)
/// - tags: Comma-separated tags to filter
async fn get_world_history(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<ApiResponse<HistoryResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);

    // Parse comma-separated filters
    let event_types: Option<Vec<String>> = params
        .event_types
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
    let tags: Option<Vec<String>> = params
        .tags
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

    // TODO: Fetch events from EventStore with filters applied
    // TODO: Implement filtering:
    //   - event_types: Filter by event type
    //   - start_year/end_year: Range filter on event year
    //   - entity_id: Filter events involving this entity
    //   - min_significance: Filter by significance threshold
    //   - tags: Filter by tags

    // Placeholder response (TODO: Load from EventStore)
    let response = HistoryResponse {
        world_id: world_id.clone(),
        total_events: 0,
        events: Vec::new(),
        pagination: Pagination {
            limit,
            offset,
            has_more: false,
        },
        filters_applied: AppliedFilters {
            event_types: event_types.clone(),
            start_year: params.start_year,
            end_year: params.end_year,
            entity_id: params.entity_id.clone(),
            min_significance: params.min_significance,
            tags: tags.clone(),
        },
    };

    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/{id}/history/events - Get detailed history events for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - event_types: Comma-separated event types to include
/// - start_year: Start year (inclusive)
/// - end_year: End year (inclusive)
/// - entity_id: Filter by entity involvement
/// - min_significance: Minimum significance (0.0 - 1.0)
/// - tags: Comma-separated tags to filter
async fn get_history_events(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<ApiResponse<HistoryResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);

    // Parse comma-separated filters
    let event_types: Option<Vec<String>> = params
        .event_types
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
    let tags: Option<Vec<String>> = params
        .tags
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

    // TODO: Fetch events from EventStore with filters applied
    // For now, return empty events list (placeholder)
    let response = HistoryResponse {
        world_id: world_id.clone(),
        total_events: 0,
        events: Vec::new(),
        pagination: Pagination {
            limit,
            offset,
            has_more: false,
        },
        filters_applied: AppliedFilters {
            event_types: event_types.clone(),
            start_year: params.start_year,
            end_year: params.end_year,
            entity_id: params.entity_id.clone(),
            min_significance: params.min_significance,
            tags: tags.clone(),
        },
    };

    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/{id}/figures - Get historical figures for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - species_id: Filter by species
/// - region_id: Filter by home region
/// - min_significance: Minimum significance (0.0 - 1.0)
async fn get_world_figures(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<GetWorldFiguresParams>,
) -> Result<Json<ApiResponse<FiguresResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // TODO: Fetch figures from database with filters applied
    // TODO: Check world exists and user has access
    let figures: Vec<HistoricalFigure> = Vec::new();
    let total = 0;

    Ok(Json(ApiResponse::new(FiguresResponse::new(
        world_id,
        figures,
        total,
        params.limit.min(200),
        params.offset.unwrap_or(0),
    ))))
}

/// GET /api/v1/worlds/{id}/figures/{figure_id} - Get single figure by ID
async fn get_world_figure(
    State(state): State<crate::api::AppState>,
    Path((world_id_raw, figure_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<HistoricalFigure>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Accept both UUID and legacy ID formats (e.g., 'fig-0')
    let search_id = figure_id.clone();

    // Load figures from storage
    let figures_path = state.storage.figures_path(&world_id);
    if figures_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&figures_path) {
            // Try to parse as array of figures
            if let Ok(figures) =
                serde_json::from_str::<Vec<crate::figures::NotableFigure>>(&content)
            {
                // Try UUID match first
                if let Some(figure) = figures
                    .iter()
                    .find(|f| f.id.to_uuid().to_string() == search_id)
                {
                    let response = HistoricalFigure::from(figure);
                    return Ok(Json(ApiResponse::new(response)));
                }
                // Try legacy ID match
                if let Some(figure) = figures.iter().find(|f| f.id.to_string() == search_id) {
                    let response = HistoricalFigure::from(figure);
                    return Ok(Json(ApiResponse::new(response)));
                }
            }
            // Try to parse as single figure object
            else if let Ok(figure) =
                serde_json::from_str::<crate::figures::NotableFigure>(&content)
            {
                if figure.id.to_uuid().to_string() == search_id
                    || figure.id.to_string() == search_id
                {
                    let response = HistoricalFigure::from(&figure);
                    return Ok(Json(ApiResponse::new(response)));
                }
            }
        }
    }

    // Figure not found - return 404
    Err(ApiError::NotFound(format!(
        "Figure '{}' not found in world '{}'",
        figure_id, world_id
    )))
}

/// GET /api/v1/worlds/{id}/societies - Get societies for a world
///
/// Query params:
/// - settlement_type: Filter by settlement type (optional)
/// - species: Filter by species (optional)
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
async fn get_world_societies(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<SocietiesQueryParams>,
) -> Result<Json<ApiResponse<SocietiesResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // TODO: Fetch settlements from database grouped by species
    // TODO: Apply filters (settlement_type, species)
    // TODO: Aggregate population and settlement stats per society

    // For development: return sample societies grouped by species
    let sample_settlements = vec![
        SettlementView {
            id: "stl-human-1".to_string(),
            name: "Greenton".to_string(),
            settlement_type: Some("town".to_string()),
            population: Some(3500),
            location: GeoLocationView {
                latitude: 45.2,
                longitude: -122.5,
                elevation_m: Some(150.0),
            },
            description: Some("Human settlement on temperate grassland (river)".to_string()),
            species_id: Some("1".to_string()),
        },
        SettlementView {
            id: "stl-human-2".to_string(),
            name: "Oldham".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(450),
            location: GeoLocationView {
                latitude: 44.8,
                longitude: -123.1,
                elevation_m: Some(80.0),
            },
            description: Some("Human settlement on temperate grassland".to_string()),
            species_id: Some("1".to_string()),
        },
        SettlementView {
            id: "stl-elf-1".to_string(),
            name: "Silverglin".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(280),
            location: GeoLocationView {
                latitude: 46.5,
                longitude: -121.8,
                elevation_m: Some(320.0),
            },
            description: Some("Elf settlement in temperate deciduous forest".to_string()),
            species_id: Some("2".to_string()),
        },
        SettlementView {
            id: "stl-elf-2".to_string(),
            name: "Moonlas".to_string(),
            settlement_type: Some("hamlet".to_string()),
            population: Some(85),
            location: GeoLocationView {
                latitude: 47.1,
                longitude: -122.0,
                elevation_m: Some(410.0),
            },
            description: Some("Elf settlement in temperate mixed forest".to_string()),
            species_id: Some("2".to_string()),
        },
        SettlementView {
            id: "stl-dwarf-1".to_string(),
            name: "Ironheim".to_string(),
            settlement_type: Some("city".to_string()),
            population: Some(15000),
            location: GeoLocationView {
                latitude: 48.2,
                longitude: -124.5,
                elevation_m: Some(890.0),
            },
            description: Some("Dwarf settlement in boreal forest (fortress)".to_string()),
            species_id: Some("3".to_string()),
        },
        SettlementView {
            id: "stl-orc-1".to_string(),
            name: "Grimmar".to_string(),
            settlement_type: Some("town".to_string()),
            population: Some(2100),
            location: GeoLocationView {
                latitude: 49.1,
                longitude: -125.2,
                elevation_m: Some(220.0),
            },
            description: Some("Orc settlement on semi-arid steppe".to_string()),
            species_id: Some("4".to_string()),
        },
        SettlementView {
            id: "stl-halfling-1".to_string(),
            name: "Riverdale".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(680),
            location: GeoLocationView {
                latitude: 43.5,
                longitude: -120.8,
                elevation_m: Some(95.0),
            },
            description: Some("Halfling settlement on temperate grassland (river)".to_string()),
            species_id: Some("5".to_string()),
        },
    ];

    // Apply species filter if specified
    let filtered_settlements: Vec<SettlementView> = if let Some(ref species_filter) = params.species
    {
        sample_settlements
            .into_iter()
            .filter(|s| s.id.contains(species_filter))
            .collect()
    } else {
        sample_settlements
    };

    // Group settlements by species
    let mut human_settlements = Vec::new();
    let mut elf_settlements = Vec::new();
    let mut dwarf_settlements = Vec::new();
    let mut orc_settlements = Vec::new();
    let mut halfling_settlements = Vec::new();

    for settlement in &filtered_settlements {
        if settlement.id.contains("human") {
            human_settlements.push(settlement.clone());
        } else if settlement.id.contains("elf") {
            elf_settlements.push(settlement.clone());
        } else if settlement.id.contains("dwarf") {
            dwarf_settlements.push(settlement.clone());
        } else if settlement.id.contains("orc") {
            orc_settlements.push(settlement.clone());
        } else if settlement.id.contains("halfling") {
            halfling_settlements.push(settlement.clone());
        }
    }

    let mut societies = Vec::new();

    if !human_settlements.is_empty() {
        let total_pop: u64 = human_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&human_settlements);
        societies.push(SocietyView {
            species_id: "human".to_string(),
            species_name: "Human".to_string(),
            settlements: human_settlements,
            total_population: total_pop,
            settlement_count: 2,
            dominant_settlement_type: dominant,
        });
    }
    if !elf_settlements.is_empty() {
        let total_pop: u64 = elf_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&elf_settlements);
        societies.push(SocietyView {
            species_id: "elf".to_string(),
            species_name: "Elf".to_string(),
            settlements: elf_settlements,
            total_population: total_pop,
            settlement_count: 2,
            dominant_settlement_type: dominant,
        });
    }
    if !dwarf_settlements.is_empty() {
        let total_pop: u64 = dwarf_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&dwarf_settlements);
        societies.push(SocietyView {
            species_id: "dwarf".to_string(),
            species_name: "Dwarf".to_string(),
            settlements: dwarf_settlements,
            total_population: total_pop,
            settlement_count: 1,
            dominant_settlement_type: dominant,
        });
    }
    if !orc_settlements.is_empty() {
        let total_pop: u64 = orc_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&orc_settlements);
        societies.push(SocietyView {
            species_id: "orc".to_string(),
            species_name: "Orc".to_string(),
            settlements: orc_settlements,
            total_population: total_pop,
            settlement_count: 1,
            dominant_settlement_type: dominant,
        });
    }
    if !halfling_settlements.is_empty() {
        let total_pop: u64 = halfling_settlements
            .iter()
            .filter_map(|s| s.population)
            .sum();
        let dominant = find_dominant_type(&halfling_settlements);
        societies.push(SocietyView {
            species_id: "halfling".to_string(),
            species_name: "Halfling".to_string(),
            settlements: halfling_settlements,
            total_population: total_pop,
            settlement_count: 1,
            dominant_settlement_type: dominant,
        });
    }

    let response = SocietiesResponse::new(world_id, societies, filtered_settlements.len());
    Ok(Json(ApiResponse::new(response)))
}

/// Helper to find the most common settlement type in a list
fn find_dominant_type(settlements: &[SettlementView]) -> Option<String> {
    let mut counts = std::collections::HashMap::new();
    for s in settlements {
        if let Some(ref t) = s.settlement_type {
            *counts.entry(t.clone()).or_insert(0) += 1;
        }
    }
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(t, _)| t)
}

/// GET /api/v1/worlds/{id}/planet - Get planet data for a world
///
/// Query params:
/// - include_geography: Include geography data (default: true)
/// - include_tectonics: Include tectonic plate data (default: false)
async fn get_world_planet(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<GetWorldPlanetParams>,
) -> Result<Json<ApiResponse<PlanetResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // TODO: Load planet data from world package
    // For now, return a placeholder response with default planet values
    let planet_view = PlanetView {
        id: world_id.clone(),
        name: "Generated World".to_string(), // TODO: Load from metadata
        planet_type: PlanetType::Terrestrial,
        radius_km: None,
        mass_earths: None,
        terrain_dimensions: TerrainDimensionsView {
            width: 256,
            height: 256,
            cell_size_m: 1000.0,
        },
        axial_tilt_deg: 23.5,
        rotation_period_h: 24.0,
        orbital_period_d: 365.25,
        gravity_m_s2: None,
        has_surface_water: true,
        has_magnetic_field: true,
        is_geologically_active: true,
    };

    let mut response = PlanetResponse::new(
        world_id.clone(),
        planet_view,
        params.include_geography.unwrap_or(true),
        params.include_tectonics.unwrap_or(false),
    );

    // Include geography data if requested
    if params.include_geography.unwrap_or(true) {
        let geography = GeographyView {
            terrain_dimensions: TerrainDimensionsView {
                width: 256,
                height: 256,
                cell_size_m: 1000.0,
            },
            total_land_area_km2: Some(510_000_000.0),
            total_water_area_km2: Some(361_000_000.0),
            land_to_water_ratio: Some(0.29),
            regions: Vec::new(), // TODO: Load from world package
            rivers: RiverService::new().get_rivers_for_world(&world_id), // Loaded from storage
            settlements: Vec::new(), // TODO: Load from settlements module
            biomes: Vec::new(),  // TODO: Load from terrain/biome module
            drainage_basins: None, // TODO: Load from drainage basin module
            generation_seed: None,
            generated_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        response = response.with_geography(geography);
    }

    // Include tectonics data if requested
    if params.include_tectonics.unwrap_or(false) {
        let tectonics = TectonicsData {
            plates: Vec::new(),
            boundaries: Vec::new(),
        };
        response = response.with_tectonics(tectonics);
    }

    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/{id}/tectonics - Get tectonic plate data for a world
///
/// Returns tectonic plate information including:
/// - All tectonic plates (id, name, type, movement)
/// - All boundary segments (type, location, volcanic activity)
/// - Cell-to-plate mapping for terrain analysis
async fn get_world_tectonics(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<TectonicsResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // TODO: Fetch tectonic data from world storage
    // TODO: Check world exists and user has access
    // For now, return empty response
    let response = TectonicsResponse {
        world_id,
        plates: vec![],
        boundaries: vec![],
    };

    Ok(Json(ApiResponse::new(response)))
}

/// Tectonics response structure
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TectonicsResponse {
    pub world_id: String,
    pub plates: Vec<TectonicPlateView>,
    pub boundaries: Vec<TectonicBoundaryView>,
}

/// Simplified tectonic plate for API responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TectonicPlateView {
    pub id: String,
    pub name: Option<String>,
    pub plate_type: String,
    pub movement_direction_deg: f32,
    pub movement_speed_cm_yr: f32,
    pub area_km2: f64,
    pub cell_count: usize,
}

/// Simplified tectonic boundary for API responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TectonicBoundaryView {
    pub id: String,
    pub boundary_type: String,
    pub length_km: f64,
    pub is_volcanic: bool,
    pub volcanic_activity: Option<f32>,
    pub seismic_activity: Option<f32>,
}

// =============================================================================
// Artifacts & Cataclysms Handlers
// =============================================================================

/// Query params for artifacts endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsQueryParams {
    #[serde(default = "default_artifacts_limit")]
    pub limit: usize,
    pub offset: Option<usize>,
    pub category: Option<String>,
    pub era: Option<String>,
    pub min_significance: Option<f64>,
    pub creator_id: Option<String>,
}

fn default_artifacts_limit() -> usize {
    50
}

impl Default for ArtifactsQueryParams {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: None,
            category: None,
            era: None,
            min_significance: None,
            creator_id: None,
        }
    }
}

/// GET /api/v1/worlds/{id}/artifacts - Get artifacts for a world
async fn get_world_artifacts(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<ArtifactsQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::ArtifactsResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Delegate to artifacts module
    // For now, return sample data
    let artifacts = vec![
        crate::api::models::ArtifactView {
            id: "artifact-1".to_string(),
            name: "The Crown of Valdoria".to_string(),
            category: "crownJewel".to_string(),
            era: Some("Age of Kings".to_string()),
            created_year: 1250,
            culture: Some("Valdorian".to_string()),
            description: "The golden crown worn by the first King of Valdoria.".to_string(),
            significance: 0.85,
            condition: "worn".to_string(),
        },
        crate::api::models::ArtifactView {
            id: "artifact-2".to_string(),
            name: "Blade of the Fallen".to_string(),
            category: "weapon".to_string(),
            era: Some("Era of Strife".to_string()),
            created_year: 980,
            culture: Some("Ironblood".to_string()),
            description: "A legendary sword wielded by Korrath the Conqueror.".to_string(),
            significance: 0.75,
            condition: "damaged".to_string(),
        },
        crate::api::models::ArtifactView {
            id: "artifact-3".to_string(),
            name: "The Tome of Ages".to_string(),
            category: "document".to_string(),
            era: Some("Age of Enlightenment".to_string()),
            created_year: 1450,
            culture: Some("Scholars".to_string()),
            description: "A comprehensive chronicle of world history.".to_string(),
            significance: 0.9,
            condition: "worn".to_string(),
        },
    ];

    let total = artifacts.len();
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    let artifacts: Vec<_> = artifacts.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ApiResponse::new(
        crate::api::models::ArtifactsResponse {
            world_id,
            artifacts,
            total,
            limit,
            offset,
        },
    )))
}

/// Query params for cataclysms endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CataclysmsQueryParams {
    pub limit: usize,
    pub offset: Option<usize>,
    pub cataclysm_type: Option<String>,
    pub scope: Option<String>,
    pub min_severity: Option<f64>,
    pub region_id: Option<String>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
}

impl Default for CataclysmsQueryParams {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: None,
            cataclysm_type: None,
            scope: None,
            min_severity: None,
            region_id: None,
            start_year: None,
            end_year: None,
        }
    }
}

/// Query params for resources endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesQueryParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub category: Option<String>,
}

impl Default for ResourcesQueryParams {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: None,
            category: None,
        }
    }
}

/// GET /api/v1/worlds/{id}/wonders - Get natural wonders for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - category: Filter by wonder category (geological, hydrological, biological, atmospheric, magical, unique)
/// - wonder_type: Filter by specific wonder type
/// - include_bonuses: Include bonus details (default: true)
async fn get_world_wonders(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<WondersQueryParams>,
) -> Result<Json<ApiResponse<WondersResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Load wonders from storage or generate on-the-fly
    let wonders = if state.storage.world_exists(&world_id) {
        // TODO: Load from world package when storage integration complete
        generate_mock_wonders(
            &world_id,
            params.category.as_deref(),
            params.wonder_type.as_deref(),
        )
    } else {
        generate_mock_wonders(
            &world_id,
            params.category.as_deref(),
            params.wonder_type.as_deref(),
        )
    };

    let total = wonders.len();
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);

    // Apply pagination
    let paginated_wonders: Vec<WonderView> = wonders.into_iter().skip(offset).take(limit).collect();

    // Calculate stats
    let stats = WonderStats {
        total_wonders: total,
        by_category: std::collections::HashMap::new(), // TODO: compute from loaded wonders
        avg_influence_radius: 50.0,
    };

    let response =
        WondersResponse::new(world_id, paginated_wonders, total, limit, offset).with_stats(stats);

    Ok(Json(ApiResponse::new(response)))
}

/// Generate mock wonders for development/demo purposes.
/// In production, this will be replaced with actual world package data.
fn generate_mock_wonders(
    world_id: &str,
    category_filter: Option<&str>,
    _type_filter: Option<&str>,
) -> Vec<WonderView> {
    let all_wonders = vec![
        WonderView {
            id: format!("{}-wonder-1", world_id),
            name: "Mount Eternity".to_string(),
            wonder_type: "SacredMountain".to_string(),
            category: "geological".to_string(),
            x: 128.0,
            y: 64.0,
            influence_radius: 80.0,
            description: "A sacred mountain said to touch the heavens.".to_string(),
            bonuses: vec![
                WonderBonusView {
                    bonus_type: "defense".to_string(),
                    magnitude: 0.3,
                    radius: 100.0,
                    region_wide: true,
                },
                WonderBonusView {
                    bonus_type: "culture".to_string(),
                    magnitude: 0.2,
                    radius: 50.0,
                    region_wide: false,
                },
            ],
            primary_color: "#8B5A2B".to_string(),
            icon_type: "mountain".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-2", world_id),
            name: "The Grand Chasm".to_string(),
            wonder_type: "GrandCanyon".to_string(),
            category: "geological".to_string(),
            x: 200.0,
            y: 150.0,
            influence_radius: 60.0,
            description: "A massive rift carved by ancient rivers.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "trade".to_string(),
                magnitude: 0.4,
                radius: 80.0,
                region_wide: false,
            }],
            primary_color: "#D2691E".to_string(),
            icon_type: "canyon".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-3", world_id),
            name: "Silver Falls".to_string(),
            wonder_type: "MagnificentWaterfall".to_string(),
            category: "hydrological".to_string(),
            x: 80.0,
            y: 100.0,
            influence_radius: 40.0,
            description: "A breathtaking waterfall cascading into a crystal lake.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "food".to_string(),
                magnitude: 0.25,
                radius: 60.0,
                region_wide: true,
            }],
            primary_color: "#40A4DF".to_string(),
            icon_type: "waterfall".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-4", world_id),
            name: "The Great Lake".to_string(),
            wonder_type: "GreatLake".to_string(),
            category: "hydrological".to_string(),
            x: 160.0,
            y: 80.0,
            influence_radius: 120.0,
            description: "A vast inland sea teeming with life.".to_string(),
            bonuses: vec![
                WonderBonusView {
                    bonus_type: "food".to_string(),
                    magnitude: 0.35,
                    radius: 150.0,
                    region_wide: true,
                },
                WonderBonusView {
                    bonus_type: "trade".to_string(),
                    magnitude: 0.2,
                    radius: 100.0,
                    region_wide: true,
                },
            ],
            primary_color: "#1E90FF".to_string(),
            icon_type: "lake".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-5", world_id),
            name: "The Ancient Oak".to_string(),
            wonder_type: "AncientTree".to_string(),
            category: "biological".to_string(),
            x: 50.0,
            y: 180.0,
            influence_radius: 30.0,
            description: "A colossal tree older than human memory.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "wisdom".to_string(),
                magnitude: 0.4,
                radius: 40.0,
                region_wide: false,
            }],
            primary_color: "#228B22".to_string(),
            icon_type: "ancientTree".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-6", world_id),
            name: "Crystal Caverns".to_string(),
            wonder_type: "CrystalCavern".to_string(),
            category: "geological".to_string(),
            x: 190.0,
            y: 40.0,
            influence_radius: 25.0,
            description: "Underground caves filled with luminescent crystals.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "magic".to_string(),
                magnitude: 0.5,
                radius: 50.0,
                region_wide: false,
            }],
            primary_color: "#BA55D3".to_string(),
            icon_type: "crystal".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-7", world_id),
            name: "Fire Mountain".to_string(),
            wonder_type: "ActiveVolcano".to_string(),
            category: "geological".to_string(),
            x: 220.0,
            y: 200.0,
            influence_radius: 70.0,
            description: "An active volcano that shapes the surrounding land.".to_string(),
            bonuses: vec![
                WonderBonusView {
                    bonus_type: "production".to_string(),
                    magnitude: 0.3,
                    radius: 60.0,
                    region_wide: true,
                },
                WonderBonusView {
                    bonus_type: "danger".to_string(),
                    magnitude: -0.2,
                    radius: 80.0,
                    region_wide: false,
                },
            ],
            primary_color: "#FF4500".to_string(),
            icon_type: "volcano".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-8", world_id),
            name: "The Whispering Woods".to_string(),
            wonder_type: "AncientForest".to_string(),
            category: "biological".to_string(),
            x: 100.0,
            y: 140.0,
            influence_radius: 90.0,
            description: "An ancient forest where the trees seem to speak.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "nature".to_string(),
                magnitude: 0.35,
                radius: 100.0,
                region_wide: true,
            }],
            primary_color: "#008000".to_string(),
            icon_type: "forest".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-9", world_id),
            name: "The Northern Lights".to_string(),
            wonder_type: "AuroraBorealis".to_string(),
            category: "atmospheric".to_string(),
            x: 30.0,
            y: 20.0,
            influence_radius: 150.0,
            description: "Dancing lights that paint the sky with ethereal colors.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "magic".to_string(),
                magnitude: 0.4,
                radius: 200.0,
                region_wide: true,
            }],
            primary_color: "#00FF7F".to_string(),
            icon_type: "aurora".to_string(),
        },
        WonderView {
            id: format!("{}-wonder-10", world_id),
            name: "The Ley Nexus".to_string(),
            wonder_type: "LeyLineNexus".to_string(),
            category: "magical".to_string(),
            x: 180.0,
            y: 120.0,
            influence_radius: 100.0,
            description: "A convergence point of magical ley lines.".to_string(),
            bonuses: vec![WonderBonusView {
                bonus_type: "magic".to_string(),
                magnitude: 0.6,
                radius: 120.0,
                region_wide: true,
            }],
            primary_color: "#9400D3".to_string(),
            icon_type: "leyLine".to_string(),
        },
    ];

    // Apply category filter if specified
    match category_filter {
        Some(cat) => all_wonders
            .into_iter()
            .filter(|w| w.category == cat)
            .collect(),
        None => all_wonders,
    }
}

/// GET /api/v1/worlds/{id}/cataclysms - Get cataclysms for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - cataclysm_type: Filter by type (greatPlague, greatQuake, greatMigration, etc.)
/// - scope: Filter by scope (global, continental, regional)
/// - min_severity: Minimum severity (0.0 - 1.0)
/// - region_id: Filter by affected region
/// - start_year: Start year (inclusive)
/// - end_year: End year (inclusive)
async fn get_world_cataclysms(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<CataclysmsQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::CataclysmsResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // TODO: Filter by params
    let cataclysms = vec![
        crate::api::models::CataclysmView {
            id: "cataclysm-1".to_string(),
            name: "The Crimson Death".to_string(),
            cataclysm_type: "greatPlague".to_string(),
            year: 1347,
            duration_years: Some(50),
            severity: 0.85,
            scope: "global".to_string(),
            description: "A devastating plague swept across the continent.".to_string(),
            significance: 0.95,
            population_lost: Some(15000000),
            cultures_destroyed: Some(vec!["Old Valorian Empire".to_string()]),
            cultures_emerged: Some(vec!["New Order of Healers".to_string()]),
        },
        crate::api::models::CataclysmView {
            id: "cataclysm-2".to_string(),
            name: "The Shattering".to_string(),
            cataclysm_type: "greatQuake".to_string(),
            year: 890,
            duration_years: Some(10),
            severity: 0.8,
            scope: "continental".to_string(),
            description: "A massive earthquake split the continent.".to_string(),
            significance: 0.9,
            population_lost: Some(5000000),
            cultures_destroyed: Some(vec!["Valorian Empire".to_string()]),
            cultures_emerged: Some(vec!["Rift Dwarves".to_string()]),
        },
        crate::api::models::CataclysmView {
            id: "cataclysm-3".to_string(),
            name: "The Long Walk".to_string(),
            cataclysm_type: "greatMigration".to_string(),
            year: 450,
            duration_years: Some(100),
            severity: 0.7,
            scope: "continental".to_string(),
            description: "The horsemen migrated westward.".to_string(),
            significance: 0.75,
            population_lost: Some(3000000),
            cultures_destroyed: None,
            cultures_emerged: Some(vec!["The Horde Kingdom".to_string()]),
        },
    ];

    let total = cataclysms.len();
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    let cataclysms: Vec<_> = cataclysms.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ApiResponse::new(
        crate::api::models::CataclysmsResponse {
            world_id,
            cataclysms,
            total,
            limit,
            offset,
        },
    )))
}

/// GET /api/v1/worlds/{id}/resources - Get resource summary for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - category: Filter by resource category (optional)
async fn get_world_resources(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<ResourcesQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::ResourcesResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    // Mock resource data - backend integration pending
    let all_resources = vec![
        crate::api::models::ResourceSummary {
            resource_type: "Iron".to_string(),
            deposit_count: 24,
            total_units: 8934.0,
            avg_quality: 0.78,
            scarcity: crate::api::models::ResourceScarcity::Common,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Gold".to_string(),
            deposit_count: 8,
            total_units: 1247.0,
            avg_quality: 0.85,
            scarcity: crate::api::models::ResourceScarcity::Rare,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Gems".to_string(),
            deposit_count: 3,
            total_units: 456.0,
            avg_quality: 0.92,
            scarcity: crate::api::models::ResourceScarcity::Critical,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Copper".to_string(),
            deposit_count: 18,
            total_units: 5621.0,
            avg_quality: 0.72,
            scarcity: crate::api::models::ResourceScarcity::Common,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Stone".to_string(),
            deposit_count: 45,
            total_units: 28947.0,
            avg_quality: 0.65,
            scarcity: crate::api::models::ResourceScarcity::Abundant,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Timber".to_string(),
            deposit_count: 52,
            total_units: 45230.0,
            avg_quality: 0.70,
            scarcity: crate::api::models::ResourceScarcity::Abundant,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Coal".to_string(),
            deposit_count: 15,
            total_units: 7823.0,
            avg_quality: 0.68,
            scarcity: crate::api::models::ResourceScarcity::Common,
        },
        crate::api::models::ResourceSummary {
            resource_type: "Silver".to_string(),
            deposit_count: 6,
            total_units: 892.0,
            avg_quality: 0.81,
            scarcity: crate::api::models::ResourceScarcity::Rare,
        },
    ];

    let mut resources = all_resources;

    // Filter by category if provided
    if let Some(category) = &params.category {
        // For now, filter simple types (real impl would use world storage)
        resources.retain(|r| match category.as_str() {
            "metals" => matches!(
                r.resource_type.as_str(),
                "Iron" | "Gold" | "Copper" | "Silver"
            ),
            "minerals" => matches!(r.resource_type.as_str(), "Stone" | "Gems"),
            "organic" => matches!(r.resource_type.as_str(), "Timber"),
            "energy" => matches!(r.resource_type.as_str(), "Coal"),
            _ => true,
        });
    }

    let total = resources.len();
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);
    let resources: Vec<_> = resources.into_iter().skip(offset).take(limit).collect();

    // Build category summary
    let by_category = vec![
        crate::api::models::CategorySummary {
            category: "Metals".to_string(),
            deposit_count: 38,
            total_units: 15694.0,
        },
        crate::api::models::CategorySummary {
            category: "Minerals".to_string(),
            deposit_count: 48,
            total_units: 29403.0,
        },
        crate::api::models::CategorySummary {
            category: "Organic".to_string(),
            deposit_count: 52,
            total_units: 45230.0,
        },
    ];

    Ok(Json(ApiResponse::new(
        crate::api::models::ResourcesResponse::new(world_id, resources, by_category),
    )))
}

// =============================================================================
// Disasters Handler (WOR-22)
// =============================================================================

/// Query params for disasters endpoint
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisastersQueryParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_disasters_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by disaster type (e.g., "drought", "famine", "plague")
    #[serde(default)]
    pub disaster_type: Option<String>,
    /// Filter by severity threshold (0.0 - 1.0)
    #[serde(default)]
    pub min_severity: Option<f64>,
    /// Include resolved/ended disasters (default: false)
    #[serde(default)]
    pub include_resolved: bool,
}

fn default_disasters_limit() -> usize {
    50
}

/// GET /api/v1/worlds/{id}/disasters - Get ongoing disasters for a world
async fn get_world_disasters(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<DisastersQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::DisastersResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    let min_severity = params.min_severity.unwrap_or(0.0);

    // Generate mock disasters for the dashboard
    // In production, this would fetch from PopulationModel's active disasters
    let all_disasters = generate_mock_disasters(&world_id);

    // Calculate stats before consuming the vector
    let ongoing_count = all_disasters.iter().filter(|d| !d.is_resolved).count();
    let resolved_count = all_disasters.iter().filter(|d| d.is_resolved).count();
    let total_pop_affected = all_disasters
        .iter()
        .map(|d| d.population_affected.unwrap_or(0))
        .sum::<u64>();

    let stats = crate::api::models::DisastersStats {
        total_disasters: ongoing_count + resolved_count,
        ongoing_count,
        resolved_count,
        by_type: std::collections::HashMap::new(), // TODO: compute from filtered
        total_population_affected: total_pop_affected,
    };

    // Apply filters
    let filtered: Vec<crate::api::models::DisasterView> = all_disasters
        .into_iter()
        .filter(|d| {
            // Filter by disaster type if specified
            if let Some(ref dtype) = params.disaster_type {
                if d.disaster_type.to_lowercase() != dtype.to_lowercase() {
                    return false;
                }
            }
            // Filter by severity
            if d.severity < min_severity {
                return false;
            }
            // Filter resolved disasters unless requested
            if !params.include_resolved && d.is_resolved {
                return false;
            }
            true
        })
        .collect();

    let total = filtered.len();

    // Apply pagination
    let disasters: Vec<crate::api::models::DisasterView> =
        filtered.into_iter().skip(offset).take(limit).collect();

    let response =
        crate::api::models::DisastersResponse::new(world_id, disasters, total, limit, offset)
            .with_stats(stats);

    Ok(Json(ApiResponse::new(response)))
}

/// Generate mock disasters for development/demo purposes.
fn generate_mock_disasters(world_id: &str) -> Vec<crate::api::models::DisasterView> {
    vec![
        crate::api::models::DisasterView {
            id: format!("{}-disaster-1", world_id),
            disaster_type: "famine".to_string(),
            name: "The Great Famine".to_string(),
            description: "A devastating famine has struck the northern territories.".to_string(),
            severity: 0.85,
            start_year: 1340,
            end_year: Some(1350),
            is_resolved: false,
            affected_regions: vec![
                "Northern Plains".to_string(),
                "Eastern Highlands".to_string(),
            ],
            population_affected: Some(50000),
            recovery_estimate_years: Some(5),
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "population_decline".to_string(),
                    magnitude: 0.3,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "food_shortage".to_string(),
                    magnitude: 0.8,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-2", world_id),
            disaster_type: "plague".to_string(),
            name: "The Crimson Death".to_string(),
            description: "A deadly plague spreads through the coastal cities.".to_string(),
            severity: 0.92,
            start_year: 1347,
            end_year: None,
            is_resolved: false,
            affected_regions: vec!["Southern Shores".to_string(), "Western Forests".to_string()],
            population_affected: Some(150000),
            recovery_estimate_years: Some(10),
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "population_decline".to_string(),
                    magnitude: 0.5,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "economic_collapse".to_string(),
                    magnitude: 0.4,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-3", world_id),
            disaster_type: "drought".to_string(),
            name: "The Burning Years".to_string(),
            description: "A prolonged drought has devastated agricultural regions.".to_string(),
            severity: 0.72,
            start_year: 1280,
            end_year: Some(1295),
            is_resolved: true,
            affected_regions: vec!["Eastern Highlands".to_string()],
            population_affected: Some(25000),
            recovery_estimate_years: None,
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "food_shortage".to_string(),
                    magnitude: 0.6,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "migration".to_string(),
                    magnitude: 0.3,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-4", world_id),
            disaster_type: "earthquake".to_string(),
            name: "The Shattering".to_string(),
            description: "A massive earthquake split the western mountain range.".to_string(),
            severity: 0.78,
            start_year: 890,
            end_year: Some(892),
            is_resolved: true,
            affected_regions: vec!["Western Forests".to_string(), "Northern Plains".to_string()],
            population_affected: Some(30000),
            recovery_estimate_years: None,
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "infrastructure_damage".to_string(),
                    magnitude: 0.9,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "population_decline".to_string(),
                    magnitude: 0.2,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-5", world_id),
            disaster_type: "flood".to_string(),
            name: "The Great Deluge".to_string(),
            description: "Unprecedented flooding along the river valleys.".to_string(),
            severity: 0.65,
            start_year: 1050,
            end_year: Some(1052),
            is_resolved: true,
            affected_regions: vec!["Southern Shores".to_string()],
            population_affected: Some(15000),
            recovery_estimate_years: None,
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "infrastructure_damage".to_string(),
                    magnitude: 0.7,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "food_shortage".to_string(),
                    magnitude: 0.4,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-6", world_id),
            disaster_type: "wildfire".to_string(),
            name: "The Burning Woods".to_string(),
            description: "Massive wildfires have consumed the ancient forests.".to_string(),
            severity: 0.58,
            start_year: 1100,
            end_year: Some(1102),
            is_resolved: true,
            affected_regions: vec!["Western Forests".to_string()],
            population_affected: Some(8000),
            recovery_estimate_years: None,
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "environmental_damage".to_string(),
                    magnitude: 0.8,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "migration".to_string(),
                    magnitude: 0.2,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-7", world_id),
            disaster_type: "war".to_string(),
            name: "The War of Shadows".to_string(),
            description: "Ongoing conflict has devastated the central territories.".to_string(),
            severity: 0.88,
            start_year: 1420,
            end_year: None,
            is_resolved: false,
            affected_regions: vec![
                "Northern Plains".to_string(),
                "Eastern Highlands".to_string(),
                "Southern Shores".to_string(),
            ],
            population_affected: Some(200000),
            recovery_estimate_years: Some(15),
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "population_decline".to_string(),
                    magnitude: 0.6,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "infrastructure_damage".to_string(),
                    magnitude: 0.7,
                },
            ],
        },
        crate::api::models::DisasterView {
            id: format!("{}-disaster-8", world_id),
            disaster_type: "blizzard".to_string(),
            name: "The White Death".to_string(),
            description: "A harsh winter has gripped the northern regions.".to_string(),
            severity: 0.55,
            start_year: 1200,
            end_year: Some(1205),
            is_resolved: true,
            affected_regions: vec!["Northern Plains".to_string()],
            population_affected: Some(12000),
            recovery_estimate_years: None,
            effects: vec![
                crate::api::models::DisasterEffect {
                    effect_type: "population_decline".to_string(),
                    magnitude: 0.25,
                },
                crate::api::models::DisasterEffect {
                    effect_type: "food_shortage".to_string(),
                    magnitude: 0.5,
                },
            ],
        },
    ]
}

// =============================================================================
// Missing Handler Stubs (needed for routes registered above)
// =============================================================================

/// GET /api/v1/worlds/{id}/resources/summary - Get resource summary for a world
async fn get_world_resources_summary(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<crate::api::models::ResourcesResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Delegate to get_world_resources with default params
    get_world_resources(
        State(_state),
        Path(world_id),
        Query(ResourcesQueryParams::default()),
    )
    .await
}

/// GET /api/v1/worlds/{id}/settlements - Get settlements for a world
async fn get_world_settlements(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<SocietiesResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Reuse the societies endpoint which includes settlements
    get_world_societies(
        State(_state),
        Path(world_id),
        Query(SocietiesQueryParams::default()),
    )
    .await
}

/// GET /api/v1/worlds/{id}/settlements/map - Get settlement map data for a world
async fn get_world_settlements_map(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<WorldMap>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // TODO: Return actual settlement map data with entity markers
    // For now, return empty map
    let map = WorldMap {
        world_id,
        dimensions: MapDimensions {
            width: 256,
            height: 256,
        },
        scale: 1.0,
        polygons: Vec::new(),
        biomes: Vec::new(),
        resources: Vec::new(),
        entities: Vec::new(),
        elevation_grid: None,
        metadata: MapMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    Ok(Json(ApiResponse::new(map)))
}

/// GET /api/v1/worlds/{id}/export - Export world data (binary format)
async fn get_world_export(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<crate::types::World>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    // Validate UUID format
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists in storage
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // Load world from storage
    let package_path = state.storage.world_package_path(&world_id);

    // Try to load full package, fall back to constructing World from metadata
    let world = match crate::packaging::load_world(&package_path) {
        Ok(package) => package.world,
        Err(_) => {
            // Fall back: try to load from metadata JSON
            let metadata_path = state.storage.world_metadata_path(&world_id);
            if metadata_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let name = json
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown World")
                            .to_string();
                        let seed = json.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

                        // Construct a valid World object
                        return Ok(Json(ApiResponse::new(crate::types::World::new(name, seed))));
                    }
                }
            }
            return Err(ApiError::Internal(
                "Failed to load world package".to_string(),
            ));
        }
    };

    Ok(Json(ApiResponse::new(world)))
}

/// GET /api/v1/worlds/{id}/export.json - Export world data as JSON
async fn get_world_export_json(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<crate::types::World>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    // Delegate to regular export (same data)
    get_world_export(State(state), Path(world_id)).await
}

// =============================================================================
// Statistics Endpoint (WOR-661)
// =============================================================================

use crate::api::models::WorldStatsResponse;

/// GET /api/v1/worlds/{id}/stats - Get world statistics for dashboard
///
/// Returns aggregated statistics including:
/// - Current year
/// - Total population by species
/// - Active societies
/// - Resource summary
async fn get_world_stats(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<WorldStatsResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Fetch societies data to derive population statistics
    let societies_result = get_world_societies(
        State(_state.clone()),
        Path(world_id.clone()),
        Query(SocietiesQueryParams::default()),
    )
    .await;

    // Fetch resources summary
    let resources_result =
        get_world_resources_summary(State(_state.clone()), Path(world_id.clone())).await;

    // Build stats response
    let stats = match societies_result {
        Ok(Json(ApiResponse {
            data: societies_response,
            ..
        })) => {
            let societies = &societies_response.societies;
            let total_population: u64 = societies.iter().map(|s| s.total_population).sum();

            // Calculate population by species
            let population_by_species: Vec<crate::api::models::PopulationBySpecies> = societies
                .iter()
                .map(|s| {
                    let percentage = if total_population > 0 {
                        ((s.total_population as f64 / total_population as f64) * 100.0).round()
                            as u8
                    } else {
                        0
                    };
                    crate::api::models::PopulationBySpecies {
                        species: s.species_name.clone(),
                        population: s.total_population,
                        percentage,
                    }
                })
                .collect();

            // Build society summaries
            let society_summaries: Vec<crate::api::models::SocietySummary> = societies
                .iter()
                .map(|s| crate::api::models::SocietySummary {
                    id: s.species_id.clone(),
                    name: s.species_name.clone(),
                    species: s.species_id.clone(),
                    settlements: s.settlement_count,
                    population: s.total_population,
                })
                .collect();

            // Get resources from resources response
            let resources: Vec<crate::api::models::ResourceStats> = match &resources_result {
                Ok(Json(ApiResponse {
                    data: resources_response,
                    ..
                })) => resources_response
                    .resources
                    .iter()
                    .map(|r| crate::api::models::ResourceStats {
                        resource_type: r.resource_type.clone(),
                        total: r.total_units as u32,
                        scarcity: r.scarcity.clone(),
                    })
                    .collect(),
                Err(_) => Vec::new(),
            };

            WorldStatsResponse {
                current_year: 1247, // TODO: Derive from world metadata
                total_population,
                population_by_species,
                active_societies: societies.len(),
                societies: society_summaries,
                resources,
            }
        }
        Err(_) => {
            // Return empty stats if societies fetch fails
            WorldStatsResponse {
                current_year: 1247,
                total_population: 0,
                population_by_species: Vec::new(),
                active_societies: 0,
                societies: Vec::new(),
                resources: Vec::new(),
            }
        }
    };

    Ok(Json(ApiResponse::new(stats)))
}

// =============================================================================
// World Status and Generation Helpers
// =============================================================================

use std::path::PathBuf;
use tokio::time::{sleep, Duration};

/// Update world status in the quick-access metadata JSON
fn update_world_status(
    storage: &crate::storage::StorageManager,
    world_id: &str,
    status: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metadata_path = storage.world_metadata_path(world_id);
    if metadata_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object_mut() {
                    obj.insert("status".to_string(), serde_json::json!(status));
                    let output = serde_json::to_string_pretty(&json)?;
                    std::fs::write(&metadata_path, output)?;
                    tracing::debug!("Updated world {} status to {}", world_id, status);
                }
            }
        }
    }
    Ok(())
}

/// Wrapper to run world generation with default storage (called from tokio::spawn)
async fn run_world_generation_internal(
    world_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use the global default storage manager
    let storage = crate::storage::StorageManager::default_manager()?;
    run_world_generation(&storage, world_id).await
}

/// Run the actual world generation pipeline (internal, called from wrapper)
async fn run_world_generation(
    storage: &crate::storage::StorageManager,
    world_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting world generation pipeline for: {}", world_id);

    // Normalize world_id (strip 'world:' prefix if present)
    let normalized_id = world_id.trim_start_matches("world:");

    // Load the world package
    let package_path = storage.world_package_path(normalized_id);
    let mut package = crate::packaging::load_world(&package_path)?;

    // Generate terrain using WorldGenerator
    use crate::generation::{WorldGenConfig, WorldGenerator};
    use crate::terrain::TerrainConfig;

    let config = WorldGenConfig {
        width: 64,
        height: 64,
        sea_level: 0.4,
        terrain: TerrainConfig {
            seed: package.world.seed,
            width: 64,
            height: 64,
            cell_size: 1000.0,
            octaves: 4,
            base_elevation: 500.0,
            mountain_amplitude: 4000.0,
            sea_level: 1000.0, // 0.4 * 2500m
            enable_tectonics: false,
            tectonic_activity: 0.5,
            enable_erosion: None,
            erosion_iterations: None,
            erosion_strength: None,
        },
        rivers: Default::default(),
    };

    let mut generator = WorldGenerator::new(config);
    let _generated_world = generator.generate(package.world.seed);

    // Update world metadata to reflect completion
    package.world.updated_at = crate::types::Timestamp::now();
    package.world.current_year = 0; // Start simulation at year 0

    // Save the updated package
    crate::packaging::save_world_package(&package, &package_path)?;

    // Update world status to 'ready' in metadata
    update_world_status(storage, normalized_id, "ready")?;

    tracing::info!("World generation completed for: {}", world_id);

    Ok(())
}

// =============================================================================
// Turn API Endpoints (WOR-720)
// =============================================================================

use crate::api::models::{
    TurnAction, TurnActionRequest, TurnActionResponse, TurnConfig, TurnSpeed, TurnState,
    TurnStatistics, TurnStatus,
};

/// GET /api/v1/worlds/{id}/turn - Get current turn state
///
/// Returns the current simulation turn state including:
/// - Current turn number
/// - Current year
/// - Turn status (idle, running, paused, completed)
/// - Turn configuration
/// - Turn statistics
async fn get_world_turn(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
) -> Result<Json<ApiResponse<TurnState>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // TODO: Load turn state from world storage when persistence is implemented
    // For now, return a default turn state
    let turn_state = TurnState {
        world_id: world_id.clone(),
        current_turn: 1,
        current_year: 0, // TODO: Load from world timeline
        turn_status: TurnStatus::Idle,
        turn_config: TurnConfig::default(),
        statistics: TurnStatistics::default(),
    };

    Ok(Json(ApiResponse::new(turn_state)))
}

/// POST /api/v1/worlds/{id}/turn - Execute a turn action
/// POST /api/v1/worlds/{id}/turn/action - Alternative route for turn actions
///
/// Executes various turn-related actions:
/// - `Advance`: Advance one turn (default)
/// - `AdvanceMultiple`: Advance multiple turns (uses config.years_per_turn)
/// - `TogglePause`: Pause or resume simulation
/// - `Reset`: Reset simulation to initial state
/// - `TriggerEvent`: Manually trigger an event
/// - `UpdateConfig`: Update turn configuration
async fn execute_turn_action(
    State(state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Json(req): Json<TurnActionRequest>,
) -> Result<Json<ApiResponse<TurnActionResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;

    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!(
            "World '{}' not found",
            world_id
        )));
    }

    // Load or create turn state
    // TODO: Load actual state from storage
    let mut turn_state = TurnState {
        world_id: world_id.clone(),
        current_turn: 1,
        current_year: 0,
        turn_status: TurnStatus::Idle,
        turn_config: req.config_override.clone().unwrap_or_default(),
        statistics: TurnStatistics::default(),
    };

    // Execute the action
    let (success, message, generated_events) = match req.action {
        TurnAction::Advance | TurnAction::AdvanceMultiple => {
            // Advance the simulation
            let years = if req.action == TurnAction::AdvanceMultiple {
                turn_state.turn_config.years_per_turn
            } else {
                10 // Default years per turn
            };

            turn_state.current_year += years as i32;
            turn_state.current_turn += 1;
            turn_state.turn_status = TurnStatus::Completed;
            turn_state.statistics.total_turns_processed += 1;
            turn_state.statistics.last_processed_year = turn_state.current_year;

            // TODO: Run actual population simulation and generate events
            // This is where we would integrate with PopulationModel and EventStore

            (
                true,
                format!(
                    "Advanced {} years. Now at year {}",
                    years, turn_state.current_year
                ),
                None, // TODO: Generate actual events
            )
        }
        TurnAction::TogglePause => {
            // Toggle between paused and running
            turn_state.turn_status = match turn_state.turn_status {
                TurnStatus::Paused | TurnStatus::Idle => TurnStatus::Running,
                TurnStatus::Running => TurnStatus::Paused,
                other => other,
            };

            let message = match turn_state.turn_status {
                TurnStatus::Running => "Simulation resumed".to_string(),
                TurnStatus::Paused => "Simulation paused".to_string(),
                _ => "Status changed".to_string(),
            };

            (true, message, None)
        }
        TurnAction::Reset => {
            // Reset simulation to initial state
            turn_state.current_turn = 1;
            turn_state.current_year = 0;
            turn_state.turn_status = TurnStatus::Idle;
            turn_state.statistics = TurnStatistics::default();

            (true, "Simulation reset to initial state".to_string(), None)
        }
        TurnAction::TriggerEvent => {
            // TODO: Implement event triggering
            // This would require loading events from storage and triggering a specific event
            turn_state.statistics.total_events_generated += 1;

            (true, "Event triggered".to_string(), None)
        }
        TurnAction::UpdateConfig => {
            // Update configuration with the provided override
            if let Some(config) = req.config_override.clone() {
                turn_state.turn_config = config;
                (true, "Configuration updated".to_string(), None)
            } else {
                (false, "No configuration provided".to_string(), None)
            }
        }
    };

    // TODO: Save turn state to storage
    // state.storage.save_turn_state(&world_id, &turn_state)?;

    let response = TurnActionResponse {
        world_id: world_id.clone(),
        action_executed: req.action,
        success,
        turn_state,
        message: if success { Some(message.clone()) } else { None },
        error: if success { None } else { Some(message) },
        generated_events,
    };

    Ok(Json(ApiResponse::new(response)))
}
