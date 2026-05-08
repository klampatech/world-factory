mod history;
mod species;
mod settlements;
mod events;
mod figures;
mod artifacts;
mod simulation;
mod api;
pub mod wonders;
// Territory system modules
pub mod territory;
pub mod terrain;

pub use territory::{TerritorySystem, claim::{TerritoryClaim, FactionId, ContestedZone}, generator::PolygonInfo};
pub use terrain::biome_assignment::{BiomeType, BiomeAssignmentSystem, PolygonBiome};
pub use simulation::{handle_simulate, SimulateRequest, SimulateResponse};
pub use species::SpeciesData;
pub use history::generator::{PreHistoryGenerator, PreHistoryConfig, TerrainData, TerrainRegion};
pub use api::v1::worlds::{
    self, simulate_world, create_world, get_world, get_generation_status,
    list_worlds, register_world, update_generation_status,
    WorldState, WorldMetadata, WorldPhase, CreateWorldRequest, CreateWorldResponse,
    GenerationTask, GenerationStatus, get_aggregate_stats,
    // Dashboard endpoint types
    Disaster, ResourcesSummaryResponse, Resource, Figure, FiguresResponse, WorldStats,
    FiguresQueryParams, get_world_disasters, get_world_resources_summary,
    get_world_figures, get_world_stats,
    // Figure detail
    FigureDetail, register_figure, get_figure, list_figures,
    // History/events endpoint types
    HistoryEvent, HistoryQueryParams, get_world_history,
};

use axum::{
    routing::{get, post},
    Router, Json, extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared application state
#[derive(Clone)]
struct AppState {
    world_registry: Arc<RwLock<std::collections::HashMap<String, WorldState>>>,
}

/// Application error type
#[derive(Debug)]
struct AppError {
    status: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "error": self.message,
            "type": "API_ERROR"
        }));
        (StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), body).into_response()
    }
}

impl From<(u16, &str)> for AppError {
    fn from((status, message): (u16, &str)) -> Self {
        AppError { status, message: message.to_string() }
    }
}

/// GET /api/v1/worlds - List all worlds
async fn list_worlds_handler() -> impl IntoResponse {
    let worlds = worlds::list_worlds();
    Json(serde_json::json!({
        "data": {
            "totalWorlds": worlds.len(),
            "worlds": worlds
        }
    }))
}

/// GET /api/v1/worlds/:id - Get world by ID
async fn get_world_handler(Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world(&id) {
        Some(world) => {
            // Check generation status
            let gen_status = worlds::get_generation_status(&id);
            let (status, progress, message) = match gen_status {
                Some(task) => {
                    let phase = match task.status {
                        GenerationStatus::Pending => "idle",
                        GenerationStatus::Generating => "generating",
                        GenerationStatus::Complete => "ready",
                        GenerationStatus::Failed => "error",
                    };
                    let msg: String = match task.status {
                        GenerationStatus::Generating => "Generating world...".to_string(),
                        GenerationStatus::Complete => "World ready".to_string(),
                        GenerationStatus::Failed => task.error.clone().unwrap_or_else(|| "Generation failed".to_string()),
                        _ => String::new(),
                    };
                    (phase, 
                     if task.status == GenerationStatus::Complete { 100 } else { 50 },
                     msg)
                },
                None => ("ready", 100, "World ready".to_string()),
            };
            
            Ok(Json(serde_json::json!({
                "id": world.id,
                "name": world.name,
                "status": status,
                "progress": progress,
                "message": message,
                "createdAt": world.created_year.to_string(),
                "polygons": world.polygons.len()
            })))
        },
        None => {
            Err(AppError::from((404, "World not found")))
        }
    }
}

/// POST /api/v1/worlds - Create world with 202 Accepted + async generation
async fn create_world_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreateWorldRequest>,
) -> Result<(StatusCode, Json<CreateWorldResponse>), AppError> {
    // Validate request configuration
    if let Some(error_msg) = request.validate() {
        return Err(AppError {
            status: 400,
            message: format!("Invalid configuration: {}", error_msg),
        });
    }
    
    let world_id = uuid::Uuid::new_v4().to_string();
    let name = request.name.clone().unwrap_or_else(|| format!("World {}", &world_id[..8]));
    
    // Create initial world state (will be populated when generation completes)
    let world_state = WorldState {
        id: world_id.clone(),
        name: name.clone(),
        polygons: Vec::new(),
        created_year: 0, // Will be set during generation
        created_at: chrono::Utc::now(),
    };
    
    // Register the world immediately
    worlds::register_world(&world_id, &name, Vec::new());
    
    // Mark as generating
    worlds::update_generation_status(&world_id, GenerationStatus::Generating, None, None);
    
    // Spawn async generation task
    let gen_world_id = world_id.clone();
    let gen_request = request.clone();
    tokio::spawn(async move {
        run_world_generation(&gen_world_id, gen_request).await;
    });
    
    let response = CreateWorldResponse {
        id: world_id.clone(),
        name,
        status: WorldPhase::Generating,
        message: "World generation started".to_string(),
        polling_url: Some(format!("/api/v1/worlds/{}", world_id)),
    };
    
    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// POST /api/v1/worlds/:id/simulate - Simulate years
async fn simulate_world_handler(
    Path(id): Path<String>,
    Json(body): Json<worlds::SimulateBody>,
) -> Result<Json<SimulateResponse>, AppError> {
    match worlds::simulate_world(&id, body) {
        Ok(response) => Ok(Json(response)),
        Err((status, error)) => Err(AppError { 
            status, 
            message: error.error 
        }),
    }
}

/// GET /api/v1/worlds/:id/disasters - Get active disasters for a world
async fn get_world_disasters_handler(Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world(&id) {
        Some(_) => {
            let disasters = worlds::get_world_disasters(&id);
            Ok(Json(serde_json::json!({
                "data": {
                    "worldId": id,
                    "disasters": disasters
                }
            })))
        },
        None => Err(AppError::from((404, "World not found"))),
    }
}

/// GET /api/v1/worlds/:id/resources/summary - Get resource distribution summary
async fn get_world_resources_handler(Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world(&id) {
        Some(_) => {
            let summary = worlds::get_world_resources_summary(&id);
            Ok(Json(serde_json::json!({
                "data": {
                    "worldId": id,
                    "resources": summary.resources
                }
            })))
        },
        None => Err(AppError::from((404, "World not found"))),
    }
}

/// GET /api/v1/worlds/:id/figures - Get top figures for a world
async fn get_world_figures_handler(
    Path(id): Path<String>,
    Query(params): Query<worlds::FiguresQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world(&id) {
        Some(_) => {
            let response = worlds::get_world_figures(&id, params.limit, &params.sort);
            Ok(Json(serde_json::json!({
                "data": {
                    "worldId": id,
                    "figures": response.figures
                }
            })))
        },
        None => Err(AppError::from((404, "World not found"))),
    }
}

/// GET /api/v1/figures/:id - Get figure detail by ID
async fn get_figure_handler(
    Path(figure_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_figure(&figure_id) {
        Some(figure) => Ok(Json(serde_json::json!({
            "data": figure
        }))),
        None => Err(AppError::from((404, "Figure not found"))),
    }
}

/// GET /stats - Get aggregate statistics across all worlds
async fn get_stats_handler() -> impl IntoResponse {
    let stats = worlds::get_aggregate_stats();
    Json(serde_json::json!({
        "data": stats
    }))
}

/// GET /api/v1/worlds/:id/stats - Get world statistics
async fn get_world_stats_handler(Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world_stats(&id) {
        Some(stats) => Ok(Json(serde_json::json!({
            "data": {
                "worldId": id,
                "stats": stats
            }
        }))),
        None => Err(AppError::from((404, "World not found"))),
    }
}

/// GET /api/v1/worlds/:id/history - Get simulation history/events
async fn get_world_history_handler(
    Path(id): Path<String>,
    Query(params): Query<worlds::HistoryQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    match worlds::get_world(&id) {
        Some(_) => {
            let events = worlds::get_world_history(&id, params.limit, params.offset);
            Ok(Json(serde_json::json!({
                "data": {
                    "worldId": id,
                    "events": events
                }
            })))
        },
        None => Err(AppError::from((404, "World not found"))),
    }
}

/// GET /health - Health check endpoint
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "0.1.0"
    }))
}

/// Run the actual world generation asynchronously
async fn run_world_generation(world_id: &str, request: CreateWorldRequest) {
    use crate::history::generator::SocietyTransitionConfig;
    
    // Update status to generating
    update_generation_status(world_id, GenerationStatus::Generating, None, None);
    
    let result = tokio::task::spawn_blocking({
        let world_id = world_id.to_string();
        move || {
            // Generate terrain polygons based on width/height
            let polygon_count = (request.width * request.height) as usize;
            let polygons: Vec<PolygonInfo> = (0..polygon_count)
                .map(|i| {
                    let x = (i % request.width as usize) as f32;
                    let y = (i / request.width as usize) as f32;
                    let elevation = generate_elevation(x, y, request.width as f32, request.height as f32, request.seed);
                    let neighbors = get_neighbor_ids(i, request.width as usize, request.height as usize);
                    PolygonInfo {
                        id: i as u64,
                        elevation,
                        neighbors,
                        is_coastal: elevation < 0.0,
                        is_island: false,
                    }
                })
                .collect();
            
            // Create terrain data
            let terrain = TerrainData {
                regions: vec![TerrainRegion {
                    id: "main".to_string(),
                    name: "Main Region".to_string(),
                    size: polygon_count as u32,
                    fertility: 0.7,
                    resource_density: 0.5,
                }],
                polygons: polygons.clone(),
            };
            
            // Create species data
            let species_data = vec![
                SpeciesData {
                    id: "homo-sapiens".to_string(),
                    name: "Homo Sapiens".to_string(),
                    population: 1000,
                    carrying_capacity: 5000,
                    traits: vec!["intelligent".to_string(), "social".to_string()],
                },
            ];
            
            // Configure prehistory generation
            let config = PreHistoryConfig {
                world_id: world_id.clone(),
                pre_history_years: request.prehistory_years,
                population_growth_rate: 0.02,
                event_probability_base: 0.1,
                figure_significance_threshold: 0.7,
                artifact_creation_probability: 0.05,
                society_transition_thresholds: SocietyTransitionConfig {
                    band_to_tribe_population: 50,
                    tribe_to_chiefdom_population: 200,
                    chiefdom_to_nation_population: 1000,
                },
            };
            
            // Run prehistory generation
            let mut generator = PreHistoryGenerator::new(config);
            let _result = generator.generate_prehistory(species_data, terrain);
            
            // The generation is complete - in a full implementation, we'd persist the result
            // For now, we just mark it as complete
            world_id
        }
    }).await;
    
    match result {
        Ok(gen_world_id) => {
            // Get the world and update it with polygons
            if let Some(mut world) = get_world(&gen_world_id) {
                world.polygons = (0..(request.width * request.height) as usize)
                    .map(|i| {
                        let x = (i % request.width as usize) as f32;
                        let y = (i / request.width as usize) as f32;
                        let elevation = generate_elevation(x, y, request.width as f32, request.height as f32, request.seed);
                        PolygonInfo {
                            id: i as u64,
                            elevation,
                            neighbors: get_neighbor_ids(i, request.width as usize, request.height as usize),
                            is_coastal: elevation < 0.0,
                            is_island: false,
                        }
                    })
                    .collect();
                world.created_year = request.prehistory_years as i32;
                
                update_generation_status(&gen_world_id, GenerationStatus::Complete, Some(world), None);
            }
        }
        Err(e) => {
            update_generation_status(world_id, GenerationStatus::Failed, None, Some(e.to_string()));
        }
    }
}

/// Generate elevation based on position (simple noise-like function)
fn generate_elevation(x: f32, y: f32, width: f32, height: f32, seed: Option<u64>) -> f32 {
    use rand::{SeedableRng, Rng};
    use rand::rngs::SmallRng;
    
    let mut rng = SmallRng::seed_from_u64(seed.unwrap_or(42));
    
    // Simple noise approximation using sine waves
    let nx = x / width * 4.0;
    let ny = y / height * 4.0;
    
    let elevation = 
        (nx * 2.0).sin() * (ny * 1.5).cos() * 100.0 +
        (nx * 0.5).sin() * (ny * 0.7).cos() * 200.0 +
        rand::Rng::gen::<f32>(&mut rng) * 50.0 - 25.0;
    
    elevation
}

/// Get neighbor IDs for a polygon at index i
fn get_neighbor_ids(i: usize, width: usize, height: usize) -> Vec<u64> {
    let x = i % width;
    let y = i / width;
    let mut neighbors = Vec::new();
    
    if x > 0 { neighbors.push(i as u64 - 1); }
    if x < width - 1 { neighbors.push(i as u64 + 1); }
    if y > 0 { neighbors.push(i as u64 - width as u64); }
    if y < height - 1 { neighbors.push(i as u64 + width as u64); }
    
    neighbors
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("prehistory_generator=info".parse().unwrap()))
        .init();

    println!("PreHistory Generator - World Simulation System");
    println!("Starting async HTTP server on http://0.0.0.0:8080");

    let app_state = AppState {
        world_registry: Arc::new(RwLock::new(std::collections::HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/stats", get(get_stats_handler))
        .route("/api/v1/worlds", get(list_worlds_handler))
        .route("/api/v1/worlds", post(create_world_handler))
        .route("/api/v1/worlds/:id", get(get_world_handler))
        .route("/api/v1/worlds/:id/simulate", post(simulate_world_handler))
        .route("/api/v1/worlds/:id/disasters", get(get_world_disasters_handler))
        .route("/api/v1/worlds/:id/resources/summary", get(get_world_resources_handler))
        .route("/api/v1/worlds/:id/figures", get(get_world_figures_handler))
        .route("/api/v1/figures/:id", get(get_figure_handler))
        .route("/api/v1/worlds/:id/stats", get(get_world_stats_handler))
        .route("/api/v1/worlds/:id/history", get(get_world_history_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}