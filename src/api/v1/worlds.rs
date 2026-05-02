//! World resource routes
//!
//! Handles world CRUD, generation triggering, and map retrieval.

use axum::{
    routing::{get, post},
    Router,
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};

use crate::api::models::*;
use crate::api::error::ApiError;
use crate::api::services::RiverService;

// Re-export WondersQueryParams for route handler
use crate::api::models::WondersQueryParams;

/// Registers world routes under /api/v1/worlds
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_worlds).post(create_world))
        .route("/:id", get(get_world))
        .route("/:id/generate", post(trigger_generation))
        .route("/:id/map", get(get_world_map))
        .route("/:id/timeline", get(get_world_timeline))
        .route("/:id/events", get(get_world_events))
        .route("/:id/history", get(get_world_history))
        .route("/:id/figures", get(get_world_figures))
        .route("/:id/societies", get(get_world_societies))
        .route("/:id/planet", get(get_world_planet))
        .route("/:id/tectonics", get(get_world_tectonics))
        .route("/:id/artifacts", get(get_world_artifacts))
        .route("/:id/cataclysms", get(get_world_cataclysms))
        .route("/:id/wonders", get(get_world_wonders))
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

fn default_worlds_limit() -> usize { 20 }
fn default_world_sort_field() -> String { "created_at".to_string() }
fn default_world_sort_dir() -> String { "desc".to_string() }

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
        return Err(ApiError::BadRequest(
            format!("Invalid sort_by: '{}'. Valid fields: {:?}", params.sort_by, valid_sort_fields)
        ));
    }
    
    // Validate sort direction
    if params.sort_dir != "asc" && params.sort_dir != "desc" {
        return Err(ApiError::BadRequest(
            "Invalid sort_dir: must be 'asc' or 'desc'".to_string()
        ));
    }
    
    // Load worlds from storage with metadata
    let stored_worlds = state.storage.list_worlds()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // Build world summaries with metadata loaded from storage
    let mut world_summaries: Vec<WorldSummary> = Vec::new();
    for stored in &stored_worlds {
        let metadata_path = state.storage.world_metadata_path(&stored.world_id);
        let (name, status, created_at) = if metadata_path.exists() {
            // Try to load from quick-access metadata JSON
            if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let name = json.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&stored.world_id)
                        .to_string();
                    let status = json.get("status")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "Pending" => WorldStatus::Pending,
                            "Generating" => WorldStatus::Generating,
                            "Ready" => WorldStatus::Ready,
                            "Failed" => WorldStatus::Failed,
                            _ => WorldStatus::Ready,
                        })
                        .unwrap_or(WorldStatus::Ready);
                    let created_at = json.get("created_at")
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
        
        let modified = stored.modified_at
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
        world_summaries.into_iter()
            .filter(|w| w.name.to_lowercase().contains(&search_lower) || w.id.to_lowercase().contains(&search_lower))
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
            "World name must be 100 characters or less".to_string()
        ));
    }
    if world_name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "World name cannot be empty".to_string()
        ));
    }
    
    // Generate or use provided seed
    let seed = req.parameters.as_ref()
        .map(|p| p.seed)
        .unwrap_or_else(|| {
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
        status: WorldStatus::Generating,  // Immediately mark as generating
        progress: Some(0.0),
        created_at: chrono::Utc::now().to_rfc3339(),
        parameters: req.parameters.clone().unwrap_or_else(|| {
            crate::api::models::WorldParameters {
                seed,
                size: crate::api::models::WorldSize::Medium,
            }
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
    std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
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
        // TODO: Call the world generation pipeline here
        // Generation will update the world package status when complete
    });
    
    tracing::info!("Created new world: {} (id: {}, seed: {})", world.name, world.id, seed);
    
    Ok((StatusCode::CREATED, Json(ApiResponse::new(world))))
}

/// GET /api/v1/worlds/:id - Get world details
async fn get_world(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<World>>, ApiError> {
    // Check if world exists in storage
    if !state.storage.world_exists(&id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", id)));
    }
    
    // Load world from storage
    let package_path = state.storage.world_package_path(&id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    let domain_world = package.world;
    let world = World {
        id: id,
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

/// POST /api/v1/worlds/:id/generate - Trigger world generation
async fn trigger_generation(
    State(_state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Json(req): Json<GenerateWorldRequest>,
) -> Result<Json<ApiResponse<World>>, ApiError> {
    uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    let world = World {
        id,
        name: req.name.unwrap_or_else(|| "Untitled World".to_string()),
        status: WorldStatus::Generating,
        progress: Some(0.0),
        created_at: chrono::Utc::now().to_rfc3339(),
        parameters: req.parameters.unwrap_or_default(),
    };
    
    Ok(Json(ApiResponse::new(world)))
}

/// GET /api/v1/worlds/:id/map - Get render-ready map data
async fn get_world_map(
    State(_state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Query(params): Query<GetWorldMapParams>,
) -> Result<Json<ApiResponse<WorldMap>>, ApiError> {
    uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    use crate::generation::{VoronoiConfig, VoronoiGenerator};
    use crate::terrain::{OceanDetector, OceanDetectionConfig, PolygonGraph, Polygon};
    
    let config = VoronoiConfig {
        width: 256,
        height: 256,
        num_seeds: 128,
        ..Default::default()
    };
    let mut generator = VoronoiGenerator::new(config, 42);
    let voronoi_result = generator.generate();
    
    // Extract polygon vertices from Voronoi
    let polygon_vertices = voronoi_result.extract_polygon_vertices();
    
    // Build a polygon graph for ocean detection
    // Each Voronoi cell becomes a polygon with computed elevation
    let mut graph = PolygonGraph::new();
    
    // Track cell centers for neighbor detection
    let mut cell_centers: Vec<(f32, f32)> = Vec::new();
    
    for (i, verts) in polygon_vertices.into_iter().enumerate() {
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
            let edge_dist_x = (normalized_x * 2.0 - 1.0).abs().min(1.0 - normalized_x * 2.0 + 1.0);
            let edge_dist_y = (normalized_y * 2.0 - 1.0).abs().min(1.0 - normalized_y * 2.0 + 1.0);
            let edge_dist = edge_dist_x.min(edge_dist_y);
            
            // Add noise for variation using seeded pseudo-random based on cell index
            let noise = (((i as f32 * 12.9898).sin() * 43758.5453).fract() 
                        * 0.3 + ((i as f32 * 78.233).cos() * 43758.5453).fract() * 0.2 + 0.5)
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
                graph.add_neighbor(i as u32, j as u32);
                graph.add_neighbor(j as u32, i as u32);
            }
        }
    }
    
    // Run ocean detection
    let ocean_config = OceanDetectionConfig::default();
    let ocean_detector = OceanDetector::with_config(ocean_config);
    let coastal_ids = ocean_detector.detect_coastal_polygons(&graph);
    
    // Build API polygon list with ocean metadata
    let polygons: Vec<Polygon> = (0..n)
        .filter_map(|i| {
            let poly = graph.get(i as u32)?;
            let verts = &polygon_vertices[i];
            if verts.len() < 3 {
                return None;
            }
            
            let zone = ocean_detector.detect_zone(poly);
            let is_ocean = zone != crate::terrain::OceanZone::Land;
            let is_coastal = coastal_ids.contains(&poly.id);
            
            Some(Polygon {
                id: format!("poly-{}", i),
                polygon_type: PolygonType::Region,
                vertices: verts.iter()
                    .map(|(x, y)| Vertex { x: *x as f64, y: *y as f64 })
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
        world_id: id,
        dimensions: MapDimensions { width: 256, height: 256 },
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

/// GET /api/v1/worlds/:id/timeline - Get timeline events for a world
async fn get_world_timeline(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<TimelineResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // TODO: Fetch timeline from EventStore
    let response = TimelineResponse::new(
        world_id,
        Vec::new(),
        0,
        params.start_year,
        params.end_year,
    );
    
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/:id/events - Get events for a world
async fn get_world_events(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<EventsListResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // TODO: Fetch events from EventStore
    let response = EventsListResponse {
        events: Vec::new(),
        total: 0,
        limit: params.limit,
        offset: params.offset.unwrap_or(0),
    };
    
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/worlds/:id/history - Get history events for a world
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
    Path(world_id): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<ApiResponse<HistoryResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    
    // Parse comma-separated filters
    let event_types: Option<Vec<String>> = params.event_types
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
    let tags: Option<Vec<String>> = params.tags
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

/// GET /api/v1/worlds/:id/figures - Get historical figures for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - species_id: Filter by species
/// - region_id: Filter by home region
/// - min_significance: Minimum significance (0.0 - 1.0)
async fn get_world_figures(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<GetWorldFiguresParams>,
) -> Result<Json<ApiResponse<FiguresResponse>>, ApiError> {
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

/// GET /api/v1/worlds/:id/societies - Get societies for a world
///
/// Query params:
/// - settlement_type: Filter by settlement type (optional)
/// - species: Filter by species (optional)
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
async fn get_world_societies(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<SocietiesQueryParams>,
) -> Result<Json<ApiResponse<SocietiesResponse>>, ApiError> {
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
            location: GeoLocationView { latitude: 45.2, longitude: -122.5, elevation_m: Some(150.0) },
            description: Some("Human settlement on temperate grassland (river)".to_string()),
            species_id: Some("1".to_string()),
        },
        SettlementView {
            id: "stl-human-2".to_string(),
            name: "Oldham".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(450),
            location: GeoLocationView { latitude: 44.8, longitude: -123.1, elevation_m: Some(80.0) },
            description: Some("Human settlement on temperate grassland".to_string()),
            species_id: Some("1".to_string()),
        },
        SettlementView {
            id: "stl-elf-1".to_string(),
            name: "Silverglin".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(280),
            location: GeoLocationView { latitude: 46.5, longitude: -121.8, elevation_m: Some(320.0) },
            description: Some("Elf settlement in temperate deciduous forest".to_string()),
            species_id: Some("2".to_string()),
        },
        SettlementView {
            id: "stl-elf-2".to_string(),
            name: "Moonlas".to_string(),
            settlement_type: Some("hamlet".to_string()),
            population: Some(85),
            location: GeoLocationView { latitude: 47.1, longitude: -122.0, elevation_m: Some(410.0) },
            description: Some("Elf settlement in temperate mixed forest".to_string()),
            species_id: Some("2".to_string()),
        },
        SettlementView {
            id: "stl-dwarf-1".to_string(),
            name: "Ironheim".to_string(),
            settlement_type: Some("city".to_string()),
            population: Some(15000),
            location: GeoLocationView { latitude: 48.2, longitude: -124.5, elevation_m: Some(890.0) },
            description: Some("Dwarf settlement in boreal forest (fortress)".to_string()),
            species_id: Some("3".to_string()),
        },
        SettlementView {
            id: "stl-orc-1".to_string(),
            name: "Grimmar".to_string(),
            settlement_type: Some("town".to_string()),
            population: Some(2100),
            location: GeoLocationView { latitude: 49.1, longitude: -125.2, elevation_m: Some(220.0) },
            description: Some("Orc settlement on semi-arid steppe".to_string()),
            species_id: Some("4".to_string()),
        },
        SettlementView {
            id: "stl-halfling-1".to_string(),
            name: "Riverdale".to_string(),
            settlement_type: Some("village".to_string()),
            population: Some(680),
            location: GeoLocationView { latitude: 43.5, longitude: -120.8, elevation_m: Some(95.0) },
            description: Some("Halfling settlement on temperate grassland (river)".to_string()),
            species_id: Some("5".to_string()),
        },
    ];
    
    // Apply species filter if specified
    let filtered_settlements: Vec<SettlementView> = if let Some(ref species_filter) = params.species {
        sample_settlements.into_iter()
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
        if settlement.id.contains("human") { human_settlements.push(settlement.clone()); }
        else if settlement.id.contains("elf") { elf_settlements.push(settlement.clone()); }
        else if settlement.id.contains("dwarf") { dwarf_settlements.push(settlement.clone()); }
        else if settlement.id.contains("orc") { orc_settlements.push(settlement.clone()); }
        else if settlement.id.contains("halfling") { halfling_settlements.push(settlement.clone()); }
    }
    
    let mut societies = Vec::new();
    
    if !human_settlements.is_empty() {
        let total_pop: u64 = human_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&human_settlements);
        societies.push(SocietyView {
            species_id: "human".to_string(), species_name: "Human".to_string(),
            settlements: human_settlements, total_population: total_pop,
            settlement_count: 2, dominant_settlement_type: dominant,
        });
    }
    if !elf_settlements.is_empty() {
        let total_pop: u64 = elf_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&elf_settlements);
        societies.push(SocietyView {
            species_id: "elf".to_string(), species_name: "Elf".to_string(),
            settlements: elf_settlements, total_population: total_pop,
            settlement_count: 2, dominant_settlement_type: dominant,
        });
    }
    if !dwarf_settlements.is_empty() {
        let total_pop: u64 = dwarf_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&dwarf_settlements);
        societies.push(SocietyView {
            species_id: "dwarf".to_string(), species_name: "Dwarf".to_string(),
            settlements: dwarf_settlements, total_population: total_pop,
            settlement_count: 1, dominant_settlement_type: dominant,
        });
    }
    if !orc_settlements.is_empty() {
        let total_pop: u64 = orc_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&orc_settlements);
        societies.push(SocietyView {
            species_id: "orc".to_string(), species_name: "Orc".to_string(),
            settlements: orc_settlements, total_population: total_pop,
            settlement_count: 1, dominant_settlement_type: dominant,
        });
    }
    if !halfling_settlements.is_empty() {
        let total_pop: u64 = halfling_settlements.iter().filter_map(|s| s.population).sum();
        let dominant = find_dominant_type(&halfling_settlements);
        societies.push(SocietyView {
            species_id: "halfling".to_string(), species_name: "Halfling".to_string(),
            settlements: halfling_settlements, total_population: total_pop,
            settlement_count: 1, dominant_settlement_type: dominant,
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

/// GET /api/v1/worlds/:id/planet - Get planet data for a world
///
/// Query params:
/// - include_geography: Include geography data (default: true)
/// - include_tectonics: Include tectonic plate data (default: false)
async fn get_world_planet(
    State(state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<GetWorldPlanetParams>,
) -> Result<Json<ApiResponse<PlanetResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", world_id)));
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
        world_id,
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
            regions: Vec::new(),   // TODO: Load from world package
            rivers: RiverService::new().get_rivers_for_world(&world_id),  // Loaded from storage
            settlements: Vec::new(), // TODO: Load from settlements module
            biomes: Vec::new(),     // TODO: Load from terrain/biome module
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

/// GET /api/v1/worlds/:id/tectonics - Get tectonic plate data for a world
///
/// Returns tectonic plate information including:
/// - All tectonic plates (id, name, type, movement)
/// - All boundary segments (type, location, volcanic activity)
/// - Cell-to-plate mapping for terrain analysis
async fn get_world_tectonics(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
) -> Result<Json<ApiResponse<TectonicsResponse>>, ApiError> {
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
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsQueryParams {
    pub limit: usize,
    pub offset: Option<usize>,
    pub category: Option<String>,
    pub era: Option<String>,
    pub min_significance: Option<f64>,
    pub creator_id: Option<String>,
}

impl Default for ArtifactsQueryParams {
    fn default() -> Self {
        Self { limit: 50, ..Default::default() }
    }
}

/// GET /api/v1/worlds/:id/artifacts - Get artifacts for a world
async fn get_world_artifacts(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<ArtifactsQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::ArtifactsResponse>>, ApiError> {
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
    
    Ok(Json(ApiResponse::new(crate::api::models::ArtifactsResponse {
        world_id,
        artifacts,
        total,
        limit,
        offset,
    })))
}

/// Query params for cataclysms endpoint  
#[derive(Debug, Deserialize, Default)]
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
        Self { limit: 50, ..Default::default() }
    }
}

/// GET /api/v1/worlds/:id/wonders - Get natural wonders for a world
///
/// Query params:
/// - limit: Max results (default: 50, max: 200)
/// - offset: Pagination offset
/// - category: Filter by wonder category (geological, hydrological, biological, atmospheric, magical, unique)
/// - wonder_type: Filter by specific wonder type
/// - include_bonuses: Include bonus details (default: true)
async fn get_world_wonders(
    State(state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<WondersQueryParams>,
) -> Result<Json<ApiResponse<WondersResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Load wonders from storage or generate on-the-fly
    let wonders = if state.storage.world_exists(&world_id) {
        // TODO: Load from world package when storage integration complete
        generate_mock_wonders(&world_id, params.category.as_deref(), params.wonder_type.as_deref())
    } else {
        generate_mock_wonders(&world_id, params.category.as_deref(), params.wonder_type.as_deref())
    };
    
    let total = wonders.len();
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    
    // Apply pagination
    let paginated_wonders: Vec<WonderView> = wonders
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    // Calculate stats
    let stats = WonderStats {
        total_wonders: total,
        by_category: std::collections::HashMap::new(), // TODO: compute from loaded wonders
        avg_influence_radius: 50.0,
    };
    
    let response = WondersResponse::new(world_id, paginated_wonders, total, limit, offset)
        .with_stats(stats);
    
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
                WonderBonusView { bonus_type: "defense".to_string(), magnitude: 0.3, radius: 100.0, region_wide: true },
                WonderBonusView { bonus_type: "culture".to_string(), magnitude: 0.2, radius: 50.0, region_wide: false },
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
            bonuses: vec![
                WonderBonusView { bonus_type: "trade".to_string(), magnitude: 0.4, radius: 80.0, region_wide: false },
            ],
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
            bonuses: vec![
                WonderBonusView { bonus_type: "food".to_string(), magnitude: 0.25, radius: 60.0, region_wide: true },
            ],
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
                WonderBonusView { bonus_type: "food".to_string(), magnitude: 0.35, radius: 150.0, region_wide: true },
                WonderBonusView { bonus_type: "trade".to_string(), magnitude: 0.2, radius: 100.0, region_wide: true },
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
            bonuses: vec![
                WonderBonusView { bonus_type: "wisdom".to_string(), magnitude: 0.4, radius: 40.0, region_wide: false },
            ],
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
            bonuses: vec![
                WonderBonusView { bonus_type: "magic".to_string(), magnitude: 0.5, radius: 50.0, region_wide: false },
            ],
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
                WonderBonusView { bonus_type: "production".to_string(), magnitude: 0.3, radius: 60.0, region_wide: true },
                WonderBonusView { bonus_type: "danger".to_string(), magnitude: -0.2, radius: 80.0, region_wide: false },
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
            bonuses: vec![
                WonderBonusView { bonus_type: "nature".to_string(), magnitude: 0.35, radius: 100.0, region_wide: true },
            ],
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
            bonuses: vec![
                WonderBonusView { bonus_type: "magic".to_string(), magnitude: 0.4, radius: 200.0, region_wide: true },
            ],
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
            bonuses: vec![
                WonderBonusView { bonus_type: "magic".to_string(), magnitude: 0.6, radius: 120.0, region_wide: true },
            ],
            primary_color: "#9400D3".to_string(),
            icon_type: "leyLine".to_string(),
        },
    ];
    
    // Apply category filter if specified
    match category_filter {
        Some(cat) => all_wonders.into_iter().filter(|w| w.category == cat).collect(),
        None => all_wonders,
    }
}

/// GET /api/v1/worlds/:id/cataclysms - Get cataclysms for a world
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
    Path(world_id): Path<String>,
    Query(params): Query<CataclysmsQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::CataclysmsResponse>>, ApiError> {
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
    
    Ok(Json(ApiResponse::new(crate::api::models::CataclysmsResponse {
        world_id,
        cataclysms,
        total,
        limit,
        offset,
    })))
}