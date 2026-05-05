//! World resource routes
//!
//! Handles world CRUD, generation triggering, and map retrieval.

use axum::{
    routing::{get, post},
    Router,
    extract::{Path, Query, State},
    response::{Json, IntoResponse, Response},
    http::{StatusCode, header::{HeaderMap, CONTENT_TYPE, CONTENT_DISPOSITION}},
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use tracing;

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
        .route("/:id/simulate", post(simulate_world))
        .route("/:id/export", get(get_world_export))
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

/// Run the world generation pipeline synchronously.
/// 
/// Generates terrain, rivers, biomes, settlements, and geography, then saves to package.
pub fn run_generation_pipeline(
    world_id: &str,
    seed: u64,
    package_path: &std::path::Path,
    metadata_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_generation_pipeline_with_config(world_id, seed, package_path, metadata_path, None)
}

/// Run the world generation pipeline with optional geography configuration.
pub fn run_generation_pipeline_with_config(
    world_id: &str,
    seed: u64,
    package_path: &std::path::Path,
    metadata_path: &std::path::Path,
    climate_config: Option<crate::world::GeographyConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::generation::{WorldGenConfig, WorldGenerator};
    use crate::settlements::{SettlementConfig, SettlementGenerator};
    use crate::world::GeographyGenerator;
    
    tracing::info!("Starting generation pipeline for world: {}", world_id);
    
    // Phase 1: Generate terrain and rivers
    let mut config = WorldGenConfig::default();
    config.width = 256;
    config.height = 256;
    
    let generator = WorldGenerator::new(config.clone());
    let terrain = generator.generate(seed);
    
    tracing::info!(
        "Terrain generated: {}x{} land={:.1}% rivers={}",
        terrain.width, terrain.height,
        terrain.land_percentage() * 100.0,
        terrain.rivers.len()
    );
    
    // Phase 2: Generate biomes
    let biome_grid = generate_biome_grid(&terrain, seed);
    
    // Phase 2b: Generate geography using GeographyGenerator with optional config
    let geography_gen = match climate_config {
        Some(cfg) => GeographyGenerator::with_config(cfg),
        None => GeographyGenerator::new(),
    };
    let elevation_data = terrain.elevation.data();
    let geographies = geography_gen.generate_grid(
        terrain.width,
        terrain.height,
        |x, y| elevation_data[y * terrain.width + x],
        &biome_grid,
        &terrain.rivers,
        seed.wrapping_add(0xDEAD),
    );
    
    tracing::info!("Geography generated: {} cells", geographies.len());
    
    // Phase 3: Generate settlements
    let river_cells: Vec<(i32, i32)> = terrain.river_cells().iter()
        .map(|v| (v.x, v.y))
        .collect();
    
    let settlement_config = SettlementConfig::default();
    let mut settlement_gen = SettlementGenerator::new(settlement_config, seed.wrapping_add(0xABCD));
    
    let elevation_data = terrain.elevation.data();
    let settlement_result = settlement_gen.generate(
        &elevation_data,
        &biome_grid,
        &[], // climate_grid - not used in this version
        config.sea_level,
        config.width,
        config.height,
        Some(&river_cells),
    );
    
    tracing::info!(
        "Settlements generated: {} total",
        settlement_result.stats.total
    );
    
    // Phase 4: Load existing world and update with generated data
    let package = crate::packaging::load_world(package_path)?;
    
    // The settlement generator already produces full Settlement domain objects
    let settlements = settlement_result.settlements;
    
    // Create updated package
    let updated_package = crate::packaging::WorldPackage {
        world: package.world,
        regions: vec![],
        settlements,
        persons: vec![],
        events: vec![],
        timelines: vec![],
        terrain: None,
        geographies: Some(geographies),
        event_store_events: vec![],
        notable_figures: vec![],
    };
    
    // Save updated package
    crate::packaging::save_world_package(&updated_package, package_path)?;
    
    // Update metadata status to Ready
    let metadata = serde_json::json!({
        "id": world_id,
        "name": updated_package.world.name,
        "status": "Ready",
        "seed": seed,
        "created_at": updated_package.world.created_at.to_string(),
        "generated_at": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    tracing::info!("Generation complete for world: {}", world_id);
    
    Ok(())
}

/// Generate biome grid for terrain cells.
fn generate_biome_grid(terrain: &crate::generation::GeneratedWorld, seed: u64) -> Vec<crate::terrain::biome::BiomeType> {
    use crate::util::Rng;
    use crate::terrain::biome_assignment::BiomeAssignmentMatrix;
    
    let mut rng = Rng::new(seed);
    let matrix = BiomeAssignmentMatrix::new();
    let mut biomes = Vec::with_capacity(terrain.width * terrain.height);
    
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let elevation = terrain.elevation.get_value_unchecked(x as i32, y as i32);
            
            // Below sea level = open ocean
            if elevation < terrain.sea_level {
                biomes.push(crate::terrain::biome::BiomeType::OpenOcean);
                continue;
            }
            
            // Calculate latitude
            let latitude = (y as f32 / terrain.height as f32) * 90.0;
            
            // Estimate temperature and precipitation
            let base_temp = 30.0 - latitude * 0.6;
            let temperature = base_temp.max(-50.0).min(50.0);
            
            // Use RNG for pseudo-precipitation
            let base = ((rng.next_f64Signed() * 0.5 + 0.5) * 2000.0) as f32;
            let precipitation = base.max(0.0).min(4000.0);
            
            // Assign biome
            let assignment = matrix.assign(elevation, latitude, precipitation, temperature);
            biomes.push(assignment.biome);
        }
    }
    
    biomes
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
                climate: None,
                terrain: None,
            }
        }),
    };
    
    // Extract climate config if provided
    let climate_config = req.parameters.as_ref()
        .and_then(|p| p.climate.as_ref())
        .map(|c| crate::world::GeographyConfig {
            base_temperature: c.base_temperature,
            lapse_rate: c.lapse_rate,
            latitude_temp_gradient: c.latitude_gradient,
            calculate_freshwater: true,
        });
    
    // Save world package to storage directory
    let package = crate::packaging::WorldPackage {
        world: domain_world,
        regions: Vec::new(),
        settlements: Vec::new(),
        persons: Vec::new(),
        events: Vec::new(),
        timelines: Vec::new(),
        terrain: None,
        geographies: None,
        event_store_events: vec![],
        notable_figures: vec![],
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
    
    // Spawn async generation task
    let gen_world_id = world_id.clone();
    let gen_world_name = world.name.clone();
    let gen_seed = seed;
    let gen_package_path = package_path.clone();
    let gen_metadata_path = metadata_path.clone();
    
    tokio::spawn(async move {
        tracing::info!(
            "Async generation starting for world: {} (id: {})",
            gen_world_name,
            gen_world_id
        );
        
        // Run the generation pipeline with optional climate config
        let result = if let Some(cfg) = climate_config.clone() {
            run_generation_pipeline_with_config(
                &gen_world_id,
                gen_seed,
                &gen_package_path,
                &gen_metadata_path,
                Some(cfg),
            )
        } else {
            run_generation_pipeline(
                &gen_world_id,
                gen_seed,
                &gen_package_path,
                &gen_metadata_path,
            )
        };
        
        match result {
            Ok(_) => {
                tracing::info!("Generation completed successfully for: {}", gen_world_id);
            }
            Err(e) => {
                tracing::error!("Generation failed for {}: {}", gen_world_id, e);
                // Update metadata to Failed status
                if let Ok(metadata) = std::fs::read_to_string(&gen_metadata_path) {
                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&metadata) {
                        json["status"] = serde_json::json!("Failed");
                        json["error"] = serde_json::json!(e.to_string());
                        let _ = std::fs::write(&gen_metadata_path, serde_json::to_string_pretty(&json).unwrap_or_default());
                    }
                }
            }
        }
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
        id,
        name: domain_world.name.clone(),
        status: WorldStatus::Ready,
        progress: Some(1.0),
        created_at: domain_world.created_at.to_string(),
        parameters: crate::api::models::WorldParameters {
            seed: domain_world.seed,
            size: crate::api::models::WorldSize::Medium,
            climate: None,
            terrain: None,
        },
    };
    
    Ok(Json(ApiResponse::new(world)))
}

/// GET /api/v1/worlds/:id/export - Download world as .wfw file
async fn get_world_export(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    // Validate UUID format
    uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists in storage
    if !state.storage.world_exists(&id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", id)));
    }
    
    // Get the package path
    let package_path = state.storage.world_package_path(&id);
    
    // Load package to get world name for filename
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world package: {}", e)))?;
    
    let world_name = package.world.name.replace(' ', "_").to_lowercase();
    let filename = format!("{}_{}.wfw", world_name, &id[..8]);
    
    // Read the .wfw file contents
    let bytes = tokio::fs::read(&package_path)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read package file: {}", e)))?;
    
    // Build response headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap()
    );
    
    Ok((StatusCode::OK, headers, bytes).into_response())
}

/// POST /api/v1/worlds/:id/generate - Trigger world generation
async fn trigger_generation(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Json(req): Json<GenerateWorldRequest>,
) -> Result<Json<ApiResponse<World>>, ApiError> {
    // Validate name if provided
    if let Some(ref name) = req.name {
        if name.len() > 100 {
            return Err(ApiError::BadRequest(
                "World name must be 100 characters or less".to_string()
            ));
        }
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "World name cannot be empty".to_string()
            ));
        }
    }

    // Extract the UUID part (strip "world:" prefix if present) for UUID validation only
    let world_id_for_uuid = id.strip_prefix("world:").unwrap_or(&id);
    
    // Validate UUID format
    uuid::Uuid::parse_str(world_id_for_uuid)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists in storage (use full ID as-is since directories use full ID)
    if !state.storage.world_exists(&id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", id)));
    }
    
    // Load existing world package (use full id with "world:" prefix)
    let package_path = state.storage.world_package_path(&id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    let world = &package.world;
    let seed = world.seed;
    let world_name = world.name.clone();
    
    // Extract climate config if provided
    let climate_config = req.parameters.as_ref()
        .and_then(|p| p.climate.as_ref())
        .map(|c| crate::world::GeographyConfig {
            base_temperature: c.base_temperature,
            lapse_rate: c.lapse_rate,
            latitude_temp_gradient: c.latitude_gradient,
            calculate_freshwater: true,
        });
    
    // Update metadata status to Generating
    let metadata_path = state.storage.world_metadata_path(&id);
    let metadata = serde_json::json!({
        "id": id,
        "name": world_name,
        "status": "Generating",
        "seed": seed,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .map_err(|e| ApiError::Internal(format!("Failed to update metadata: {}", e)))?;
    
    // Spawn async generation task
    let gen_world_id = id.clone();
    let gen_seed = seed;
    let gen_package_path = package_path.clone();
    let gen_metadata_path = metadata_path.clone();
    let gen_world_name = world_name.clone();
    
    tokio::spawn(async move {
        tracing::info!("Generation starting for world: {} (id: {})", gen_world_name, gen_world_id);
        
        // Run the generation pipeline with optional climate config
        let result = if let Some(cfg) = climate_config.clone() {
            run_generation_pipeline_with_config(
                &gen_world_id,
                gen_seed,
                &gen_package_path,
                &gen_metadata_path,
                Some(cfg),
            )
        } else {
            run_generation_pipeline(
                &gen_world_id,
                gen_seed,
                &gen_package_path,
                &gen_metadata_path,
            )
        };
        
        match result {
            Ok(_) => {
                tracing::info!("Generation completed successfully for: {}", gen_world_id);
            }
            Err(e) => {
                tracing::error!("Generation failed for {}: {}", gen_world_id, e);
                // Update metadata to Failed status
                if let Ok(metadata) = std::fs::read_to_string(&gen_metadata_path) {
                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&metadata) {
                        json["status"] = serde_json::json!("Failed");
                        json["error"] = serde_json::json!(e.to_string());
                        let _ = std::fs::write(&gen_metadata_path, serde_json::to_string_pretty(&json).unwrap_or_default());
                    }
                }
            }
        }
    });
    
    // Return the world with Generating status (use full ID with "world:" prefix for response)
    let world_response = World {
        id: id.clone(),
        name: world_name,
        status: WorldStatus::Generating,
        progress: Some(0.0),
        created_at: chrono::Utc::now().to_rfc3339(),
        parameters: req.parameters.unwrap_or_default(),
    };
    
    tracing::info!("Triggered generation for world: {} (id: {})", world_response.name, id);
    
    Ok(Json(ApiResponse::new(world_response)))
}

/// GET /api/v1/worlds/:id/map - Get render-ready map data
async fn get_world_map(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Query(params): Query<GetWorldMapParams>,
) -> Result<Json<ApiResponse<WorldMap>>, ApiError> {
    uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists in storage
    if !state.storage.world_exists(&id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", id)));
    }
    
    // Load world package from storage
    let package_path = state.storage.world_package_path(&id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    let world = &package.world;
    
    // Get world dimensions from storage or use defaults
    let (width, height) = (256usize, 256usize);
    
    // Build biomes from regions based on climate
    let biomes: Vec<crate::api::models::Biome> = package.regions.iter()
        .filter_map(|region| {
            let climate = region.climate.as_ref()?;
            let biome_type = match climate {
                crate::types::ClimateZone::Tropical => "tropical",
                crate::types::ClimateZone::Subtropical => "subtropical",
                crate::types::ClimateZone::Temperate => "temperate",
                crate::types::ClimateZone::Boreal => "boreal",
                crate::types::ClimateZone::Polar => "polar",
            };
            let color = match climate {
                crate::types::ClimateZone::Tropical => [34, 139, 34],
                crate::types::ClimateZone::Subtropical => [205, 133, 63],
                crate::types::ClimateZone::Temperate => [85, 128, 85],
                crate::types::ClimateZone::Boreal => [0, 100, 0],
                crate::types::ClimateZone::Polar => [240, 240, 250],
            };
            Some(crate::api::models::Biome {
                id: format!("biome-{}", region.id),
                biome_type: biome_type.to_string(),
                color,
                name: format!("{} Region", climate.short_name()),
            })
        })
        .collect();
    
    // Build polygons from regions (Voronoi cells per region center)
    let polygon_vertices = if package.regions.is_empty() {
        // Fall back to Voronoi generation if no regions stored
        use crate::generation::{VoronoiConfig, VoronoiGenerator};
        let config = VoronoiConfig {
            width: width as u32,
            height: height as u32,
            num_seeds: 128,
            ..Default::default()
        };
        let mut generator = VoronoiGenerator::new(config, world.seed);
        let voronoi_result = generator.generate();
        voronoi_result.extract_polygon_vertices()
    } else {
        // Use region centers to generate polygon vertices
        generate_polygons_from_regions(&package.regions, width, height)
    };
    
    // Build ocean detection from stored terrain data or compute from regions
    use crate::terrain::{OceanDetector, OceanDetectionConfig, PolygonGraph, Polygon};
    let mut graph = PolygonGraph::new();
    let mut cell_centers: Vec<(f32, f32)> = Vec::new();
    
    for (i, verts) in polygon_vertices.iter().enumerate() {
        if verts.len() >= 3 {
            let center_x: f32 = verts.iter().map(|v| v.0).sum::<f32>() / verts.len() as f32;
            let center_y: f32 = verts.iter().map(|v| v.1).sum::<f32>() / verts.len() as f32;
            cell_centers.push((center_x, center_y));
            
            // Compute elevation from region data if available
            let elevation = if i < package.regions.len() {
                package.regions[i].area_km2 as f32 / 10000.0
            } else {
                // Fall back to distance-based elevation
                let normalized_x = center_x / width as f32;
                let normalized_y = center_y / height as f32;
                let edge_dist_x = (normalized_x * 2.0 - 1.0).abs().min(1.0 - normalized_x * 2.0 + 1.0);
                let edge_dist_y = (normalized_y * 2.0 - 1.0).abs().min(1.0 - normalized_y * 2.0 + 1.0);
                edge_dist_x.min(edge_dist_y)
            };
            
            let mut polygon = Polygon::new(i as u32);
            polygon.elevation = elevation;
            polygon.base_elevation = elevation * 9000.0;
            graph.add_polygon(polygon);
        }
    }
    
    // Connect neighbors based on spatial proximity
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
    
    // Build polygons with ocean metadata
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
            let is_coast = is_coastal;
            
            // Compute centroid from vertices
            let centroid_x: f64 = verts.iter().map(|v| v.0 as f64).sum::<f64>() / verts.len() as f64;
            let centroid_y: f64 = verts.iter().map(|v| v.1 as f64).sum::<f64>() / verts.len() as f64;
            
            // Compute temperature based on latitude (y coordinate normalized to 0-1)
            let temperature = Some((centroid_y / height as f64 * 2.0 - 1.0).abs().min(1.0) as f64);
            
            // Compute moisture based on proximity to water
            let moisture = Some(if is_coastal { 0.8 } else { 0.3 + (poly.elevation as f64 * 0.3) });
            
            // River volume: only for coastal or low-elevation land
            let river_volume = Some(if is_coastal { 0.5 } else if poly.elevation < 0.3 { 0.2 } else { 0.0 });
            
            Some(crate::api::models::Polygon {
                id: format!("poly-{}", i),
                polygon_type: crate::api::models::PolygonType::Region,
                vertices: verts.iter()
                    .map(|(x, y)| crate::api::models::Vertex { x: *x as f64, y: *y as f64 })
                    .collect(),
                centroid: Some(crate::api::models::Vertex { x: centroid_x, y: centroid_y }),
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
                biome_type: Some(if is_ocean { "ocean".to_string() } else { "land".to_string() }),
                temperature,
                moisture,
                is_coast: Some(is_coast),
                river_volume,
            })
        })
        .collect();
    
    // Build resources from terrain data
    let resources: Vec<crate::api::models::Resource> = package.regions.iter()
        .enumerate()
        .filter_map(|(i, region)| {
            let resource_type = match i % 5 {
                0 => "iron",
                1 => "gold",
                2 => "copper",
                3 => "gems",
                _ => "stone",
            };
            Some(crate::api::models::Resource {
                id: format!("resource-{}", region.id),
                resource_type: resource_type.to_string(),
                position: crate::api::models::Vertex {
                    x: region.center_lon,
                    y: region.center_lat,
                },
                magnitude: ((i % 5) as u8 + 1),
                name: format!("{} Deposit", resource_type),
            })
        })
        .collect();
    
    // Build entities from settlements
    let entities: Vec<crate::api::models::MapEntity> = package.settlements.iter()
        .map(|settlement| {
            crate::api::models::MapEntity {
                id: settlement.id.to_string(),
                entity_type: crate::api::models::MapEntityType::City,
                position: crate::api::models::Vertex {
                    x: settlement.location.longitude,
                    y: settlement.location.latitude,
                },
                name: settlement.name.clone(),
                significance: 5,
            }
        })
        .collect();
    
    // Apply LOD filtering if requested
    let (polygons, biomes) = match params.lod {
        0 => (
            polygons.into_iter().step_by(4).collect(),
            biomes.into_iter().step_by(4).collect(),
        ),
        2 => (polygons, biomes),
        _ => (polygons, biomes),
    };
    
    let map = WorldMap {
        world_id: id,
        dimensions: MapDimensions { width, height },
        scale: 1.0,
        polygons,
        biomes,
        resources,
        entities,
        elevation_grid: None,
        metadata: MapMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };
    
    Ok(Json(ApiResponse::new(map)))
}

/// Generate polygon vertices from region data using Voronoi tessellation
fn generate_polygons_from_regions(
    regions: &[crate::types::Region],
    width: usize,
    height: usize,
) -> Vec<Vec<(f32, f32)>> {
    use crate::generation::{VoronoiConfig, VoronoiGenerator};
    
    let num_seeds = regions.len().max(128);
    let config = VoronoiConfig {
        width: width as u32,
        height: height as u32,
        num_seeds: num_seeds as u32,
        ..Default::default()
    };
    
    // Use world seed from regions if available (first 8 bytes as u64)
    let seed = regions.first()
        .map(|r| r.id.as_u64_pair().0)
        .unwrap_or(42);
    
    let mut generator = VoronoiGenerator::new(config, seed);
    generator.generate().extract_polygon_vertices()
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
    State(state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<ApiResponse<HistoryResponse>>, ApiError> {
    // Validate world ID format
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", world_id)));
    }
    
    // Load world package from storage
    let package_path = state.storage.world_package_path(&world_id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    // Parse filter parameters
    let event_types_filter: Option<Vec<String>> = params.event_types
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
    let tags_filter: Option<Vec<String>> = params.tags
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
    
    // Convert Event to HistoryEventView
    let all_events: Vec<HistoryEventView> = package.event_store_events.iter()
        .map(|e| {
            let location_id = e.location_id.map(|loc| loc.to_string());
            let participant_count = e.participants.as_ref().map(|p| p.len());
            
            HistoryEventView {
                id: e.id.to_string(),
                event_type: format!("{:?}", e.event_type),
                year: e.time.get_year(),
                title: e.name.clone(),
                description: Some(e.description.clone()),
                significance: e.significance.unwrap_or(0.5) as f64,
                location_id,
                participant_count,
                tags: None, // Event struct doesn't have a tags field
            }
        })
        .collect();
    
    // Apply filters
    let mut filtered_events: Vec<HistoryEventView> = all_events;
    
    // Filter by event types
    if let Some(ref types) = event_types_filter {
        filtered_events.retain(|e| {
            types.iter().any(|t| e.event_type.to_lowercase().contains(&t.to_lowercase()))
        });
    }
    
    // Filter by year range
    if let Some(start_year) = params.start_year {
        filtered_events.retain(|e| e.year >= start_year);
    }
    if let Some(end_year) = params.end_year {
        filtered_events.retain(|e| e.year <= end_year);
    }
    
    // Filter by entity involvement
    if let Some(ref entity_id) = params.entity_id {
        filtered_events.retain(|e| {
            e.location_id.as_ref().map_or(false, |id| id == entity_id)
                || e.participant_count.map_or(false, |_| true)
        });
    }
    
    // Filter by significance threshold
    if let Some(min_sig) = params.min_significance {
        filtered_events.retain(|e| e.significance >= min_sig);
    }
    
    // Filter by tags
    if let Some(ref tags) = tags_filter {
        filtered_events.retain(|e| {
            e.tags.as_ref().map_or(false, |event_tags| {
                tags.iter().any(|t| event_tags.iter().any(|et| et.to_lowercase().contains(&t.to_lowercase())))
            })
        });
    }
    
    // Sort chronologically
    filtered_events.sort_by(|a, b| a.year.cmp(&b.year));
    
    // Calculate total before pagination
    let total_events = filtered_events.len();
    
    // Apply pagination
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    let paginated_events: Vec<HistoryEventView> = filtered_events
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    let has_more = offset + paginated_events.len() < total_events;
    
    let response = HistoryResponse {
        world_id: world_id.clone(),
        total_events,
        events: paginated_events,
        pagination: Pagination {
            limit,
            offset,
            has_more,
        },
        filters_applied: AppliedFilters {
            event_types: event_types_filter,
            start_year: params.start_year,
            end_year: params.end_year,
            entity_id: params.entity_id.clone(),
            min_significance: params.min_significance,
            tags: tags_filter,
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
    State(state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<GetWorldFiguresParams>,
) -> Result<Json<ApiResponse<FiguresResponse>>, ApiError> {
    // Validate world ID format
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", world_id)));
    }
    
    // Load world package from storage
    let package_path = state.storage.world_package_path(&world_id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    // Convert NotableFigure to HistoricalFigure
    let all_figures: Vec<HistoricalFigure> = package.notable_figures.iter()
        .map(HistoricalFigure::from)
        .collect();
    
    // Apply filters
    let mut filtered_figures: Vec<HistoricalFigure> = all_figures;
    
    // Filter by species
    if let Some(ref species_id) = params.species_id {
        filtered_figures.retain(|f| {
            f.species_id.as_ref().map_or(false, |id| id == species_id)
        });
    }
    
    // Filter by region - HistoricalFigure doesn't have home_region_id, so skip this filter
    // TODO: Add home_region_id to NotableFigure if needed
    if let Some(ref region_id) = params.region_id {
        let _ = region_id; // Silence unused warning
        // filtered_figures.retain(|f| {
        //     f.home_region_id.as_ref().map_or(false, |id| id == region_id)
        // });
    }
    
    // Filter by significance
    if let Some(min_sig) = params.min_significance {
        filtered_figures.retain(|f| f.significance >= min_sig);
    }
    
    // Calculate total before pagination
    let total = filtered_figures.len();
    
    // Apply pagination
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    let paginated_figures: Vec<HistoricalFigure> = filtered_figures
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Ok(Json(ApiResponse::new(FiguresResponse::new(
        world_id,
        paginated_figures,
        total,
        limit,
        offset,
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
        world_id.clone(),
        planet_view,
        params.include_geography.unwrap_or(true),
        params.include_tectonics.unwrap_or(false),
    );
    
    // Include geography data if requested
    if params.include_geography.unwrap_or(true) {
        // Load world package to get generation data
        let package_path = state.storage.world_package_path(&world_id);
        let package = crate::packaging::load_world(&package_path)
            .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
        
        // Generate geography data using GeographyGenerator
        use crate::world::GeographyGenerator;
        use crate::generation::{WorldGenConfig, WorldGenerator};
        
        let geography_gen = GeographyGenerator::new();
        
        // Re-generate terrain to get elevation data
        let mut config = WorldGenConfig::default();
        config.width = 256;
        config.height = 256;
        let terrain_gen = WorldGenerator::new(config);
        let terrain = terrain_gen.generate(package.world.seed);
        
        // Generate biomes
        let biome_grid = generate_biome_grid(&terrain, package.world.seed);
        
        // Generate geography
        let elevation_data = terrain.elevation.data();
        let geographies = geography_gen.generate_grid(
            terrain.width,
            terrain.height,
            |x, y| elevation_data[y * terrain.width + x],
            &biome_grid,
            &terrain.rivers,
            package.world.seed.wrapping_add(0xDEAD),
        );
        
        use crate::world::DrainageType;
        
        tracing::info!("Generated {} geography entries for world {}", geographies.len(), world_id);
        
        // Calculate terrain statistics from generated data
        let land_count = geographies.iter().filter(|g| matches!(g.drainage_type, DrainageType::Exorheic)).count();
        let water_count = geographies.len() - land_count;
        
        // Create RegionView from geography data (sample for large worlds)
        let regions: Vec<RegionView> = if geographies.len() > 1000 {
            // Sample regions for large worlds to avoid huge responses
            let step = (geographies.len() / 500).max(1);
            geographies.iter().step_by(step).enumerate().map(|(i, geo)| {
                let lat = geo.latitude_deg;
                let lon = (i as f64 * 360.0 / (geographies.len() as f64 / step as f64)) % 360.0 - 180.0;
                RegionView {
                    id: format!("region-{}", i),
                    name: format!("Region {}", i + 1),
                    area_km2: 100000.0,
                    center_lat: lat as f64,
                    center_lon: lon,
                    description: None,
                    climate: Some(format!("{:?}", geo.climate_classification())),
                    parent_region_id: None,
                }
            }).collect()
        } else {
            // Full region list for small worlds
            geographies.iter().enumerate().map(|(i, geo)| {
                let lat = geo.latitude_deg;
                let lon = ((i % 256) as f64 * 360.0 / 256.0) - 180.0;
                RegionView {
                    id: format!("region-{}", i),
                    name: format!("Region {}", i + 1),
                    area_km2: 100000.0,
                    center_lat: lat as f64,
                    center_lon: lon,
                    description: None,
                    climate: Some(format!("{:?}", geo.climate_classification())),
                    parent_region_id: None,
                }
            }).collect()
        };
        
        // Load rivers from RiverService
        let rivers = RiverService::new().get_rivers_for_world(&world_id);
        
        // Load settlements from world package
        let settlements: Vec<SettlementView> = package.settlements.iter().map(|s| {
            let location = GeoLocationView {
                latitude: 0.0, // TODO: derive from location
                longitude: 0.0,
                elevation_m: None,
            };
            SettlementView {
                id: s.id.to_string(),
                name: s.name.clone(),
                settlement_type: Some(format!("{:?}", s.settlement_type)),
                population: s.population,
                location,
                description: None,
                species_id: s.species_id.map(|id| id.as_u32().to_string()),
            }
        }).collect();
        
        // Create biome views from biome grid
        let biomes: Vec<BiomeView> = biome_grid.iter().enumerate().map(|(i, b)| {
            let name = format!("{:?}", b);
            let color = b.color();
            BiomeView {
                id: format!("biome-{}", i),
                biome_type: name.clone(),
                name,
                color_rgb: [color.0, color.1, color.2],
            }
        }).collect();
        
        let geography = GeographyView {
            terrain_dimensions: TerrainDimensionsView {
                width: terrain.width as u32,
                height: terrain.height as u32,
                cell_size_m: 1000.0,
            },
            total_land_area_km2: Some((land_count as f64) * 1_000_000.0),
            total_water_area_km2: Some((water_count as f64) * 1_000_000.0),
            land_to_water_ratio: Some(land_count as f64 / geographies.len() as f64),
            regions,
            rivers,
            settlements,
            biomes,
            drainage_basins: None,  // TODO: Load from drainage basin module
            generation_seed: Some(package.world.seed),
            generated_at: Some(package.world.created_at.to_string()),
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
#[derive(Debug, Deserialize)]
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

/// POST /api/v1/worlds/:id/simulate - Run historical simulation for a world
///
/// Advances the world's history simulation by the specified number of years,
/// generating events, population changes, and historical figures.
///
/// Request body:
/// - years: Number of years to simulate (default: 100, max: 10000)
/// - startYear: Starting year for simulation (default: last world year or 0)
/// - includeEvents: Include generated events in response (default: true)
/// - includeFigures: Include generated figures in response (default: true)
/// - seed: Optional random seed for reproducible simulation
async fn simulate_world(
    State(state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Json(req): Json<SimulateWorldRequest>,
) -> Result<Json<ApiResponse<SimulateWorldResponse>>, ApiError> {
    // Validate world ID
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    // Check if world exists
    if !state.storage.world_exists(&world_id) {
        return Err(ApiError::NotFound(format!("World '{}' not found", world_id)));
    }
    
    // Load world package
    let package_path = state.storage.world_package_path(&world_id);
    let package = crate::packaging::load_world(&package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
    
    // Determine simulation parameters
    let years = req.years.unwrap_or(100).min(10000);
    let start_year = req.start_year.unwrap_or(0);
    let seed = req.seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    });
    
    tracing::info!("Starting simulation for world {}: {} years from year {}", 
                   world_id, years, start_year);
    
    // Create history generator with seed
    let config = crate::history::GeneratorConfig::default();
    let mut generator = crate::history::HistoryGenerator::with_config(config, Some(seed));
    
    // Run the simulation
    let result = generator.run_simulation(
        &package.world,
        &package.settlements,
        start_year,
        years,
    );
    
    // Build response
    let mut response = SimulateWorldResponse {
        world_id: world_id.clone(),
        start_year,
        end_year: start_year + years,
        years_simulated: years,
        seed,
        events: Vec::new(),
        figures: Vec::new(),
        population_changes: Vec::new(),
        stats: SimulationStats {
            total_events: result.events.len(),
            population_events: result.events.iter().filter(|e| matches!(e.event_type, crate::events::EventType::PopulationGrowth | crate::events::EventType::Plague)).count(),
            political_events: result.events.iter().filter(|e| matches!(e.event_type, crate::events::EventType::WarDeclared | crate::events::EventType::Treaty | crate::events::EventType::Succession)).count(),
            natural_events: result.events.iter().filter(|e| matches!(e.event_type, crate::events::EventType::Plague | crate::events::EventType::Famine | crate::events::EventType::Earthquake)).count(),
            figures_created: result.figures.len(),
            settlement_events: result.events.iter().filter(|e| matches!(e.event_type, crate::events::EventType::SettlementFounded)).count(),
        },
    };
    
    // Include events if requested
    if req.include_events {
        for event in &result.events {
            let event_view = TimelineEventView {
                id: event.id.to_uuid().to_string(),
                event_type: format!("{:?}", event.event_type),
                position: EventPosition {
                    year: event.time.get_year(),
                    season: None,
                    century: None,
                },
                title: event.name.clone(),
                description: Some(event.description.clone()),
                participants: event.participants.as_ref().map(|ps| {
                    ps.iter().map(|p| EventParticipant {
                        entity_id: p.to_string(),
                        name: "Unknown".to_string(),
                        entity_type: "unknown".to_string(),
                        role: "participant".to_string(),
                    }).collect()
                }),
                prerequisites: Vec::new(),
                outcomes: Vec::new(),
                significance: event.significance.unwrap_or(0.5) as f64,
                related_entities: None,
                tags: None,
            };
            response.events.push(event_view);
        }
    }
    
    // Include figures if requested
    if req.include_figures {
        for figure in &result.figures {
            let figure_view = HistoricalFigure {
                id: figure.id.to_uuid().to_string(),
                name: figure.name.as_ref().map(|n| PersonName {
                    given: n.given.clone(),
                    family: n.family.clone(),
                    epithet: n.epithet.clone(),
                    title: n.title.clone(),
                }),
                entity_type: format!("{:?}", figure.figure_type),
                birth_year: figure.birth_year,
                death_year: figure.death_year,
                birthplace_id: figure.birthplace_id.map(|id| id.to_string()),
                culture: figure.culture.clone(),
                titles: figure.titles.clone(),
                description: figure.description.clone(),
                significance: figure.significance as f64,
                species_id: figure.species_id.map(|id| id.to_string()),
            };
            response.figures.push(figure_view);
        }
    }
    
    // Include population changes if any were generated
    for change in &result.population_changes {
        response.population_changes.push(PopulationChangeView {
            settlement_id: change.settlement_id.to_string(),
            old_population: change.old_population,
            new_population: change.new_population,
            change_amount: change.change_amount,
            society_type: change.society_transition.map(|st| format!("{:?}", st)),
            years_elapsed: change.years_elapsed,
        });
    }
    
    // Save generated events and figures back to the world package
    let updated_package = crate::packaging::WorldPackage {
        world: package.world,
        regions: package.regions,
        settlements: package.settlements,
        persons: package.persons,
        events: package.events,
        timelines: package.timelines,
        terrain: package.terrain,
        geographies: package.geographies,
        event_store_events: result.events,
        notable_figures: result.figures,
    };
    
    // Save the updated package to persist simulation results
    crate::packaging::save_world_package(&updated_package, &package_path)
        .map_err(|e| ApiError::Internal(format!("Failed to save simulation results: {}", e)))?;
    
    tracing::info!("Simulation complete for world {}: {} events, {} figures",
                   world_id, response.events.len(), response.figures.len());
    
    Ok(Json(ApiResponse::new(response)))
}