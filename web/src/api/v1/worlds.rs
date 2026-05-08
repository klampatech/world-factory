//! Worlds API routes - World creation and simulation
//! 
//! Implements:
//! - POST /api/v1/worlds - Create world with 202 Accepted + async generation
//! - POST /api/v1/worlds/:id/simulate - Simulate years
//! - GET /api/v1/worlds/:id - Get world status
//! - GET /api/v1/worlds - List all worlds

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use crate::simulation::{
    handle_simulate, SimulateRequest, SimulateResponse, SimulationError,
};
use crate::territory::PolygonInfo;
use std::collections::HashMap;
use std::sync::RwLock;

/// World state stored in the world registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub id: String,
    pub name: String,
    pub polygons: Vec<PolygonInfo>,
    pub created_year: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// In-memory world registry (in production, this would be a database)
static WORLD_REGISTRY: LazyLock<RwLock<HashMap<String, WorldState>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Generation tasks for async world generation
static GENERATION_TASKS: LazyLock<RwLock<HashMap<String, GenerationTask>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Generation task state
#[derive(Debug, Clone)]
pub struct GenerationTask {
    pub world_id: String,
    pub status: GenerationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<WorldState>,
    pub error: Option<String>,
}

/// Generation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStatus {
    Pending,
    Generating,
    Complete,
    Failed,
}

/// Request body for creating a new world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorldRequest {
    /// World name (optional, auto-generated if not provided)
    pub name: Option<String>,
    /// Generation seed for reproducibility (optional)
    #[serde(default)]
    pub seed: Option<u64>,
    /// World width in tiles (default: 64, max: 128)
    #[serde(default = "default_width")]
    pub width: u32,
    /// World height in tiles (default: 64, max: 128)
    #[serde(default = "default_height")]
    pub height: u32,
    /// Number of prehistory years to simulate (default: 1000, range: 1-100000)
    #[serde(default = "default_prehistory_years")]
    pub prehistory_years: u32,
    /// Resource richness factor (0.0-1.0, default: 0.5)
    #[serde(default = "default_resource_richness")]
    pub resource_richness: f32,
    /// Disaster frequency factor (0.0-1.0, default: 0.1)
    #[serde(default = "default_disaster_frequency")]
    pub disaster_frequency: f32,
    /// Species templates for initialization (optional)
    #[serde(default)]
    pub species_templates: Option<Vec<SpeciesTemplate>>,
    /// Enable detailed events (default: true)
    #[serde(default = "default_true")]
    pub detailed_events: bool,
    /// Enable figure generation (default: true)
    #[serde(default = "default_true")]
    pub generate_figures: bool,
    /// Enable artifact generation (default: true)
    #[serde(default = "default_true")]
    pub generate_artifacts: bool,
}

/// Species template for world initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesTemplate {
    /// Species identifier
    pub id: String,
    /// Species display name
    pub name: String,
    /// Initial population count
    pub initial_population: u32,
    /// Starting society type (band, tribe, chiefdom, nation)
    #[serde(default = "default_society_type")]
    pub society_type: String,
}

impl CreateWorldRequest {
    /// Validate the request configuration
    /// Returns None if valid, Some(error_message) if invalid
    pub fn validate(&self) -> Option<String> {
        // Width validation: 1-128
        if self.width == 0 || self.width > 128 {
            return Some(format!("width must be between 1 and 128, got {}", self.width));
        }
        
        // Height validation: 1-128
        if self.height == 0 || self.height > 128 {
            return Some(format!("height must be between 1 and 128, got {}", self.height));
        }
        
        // Prehistory years validation: 1-100000
        if self.prehistory_years == 0 || self.prehistory_years > 100000 {
            return Some(format!("prehistory_years must be between 1 and 100000, got {}", self.prehistory_years));
        }
        
        // Resource richness validation: 0.0-1.0
        if self.resource_richness < 0.0 || self.resource_richness > 1.0 {
            return Some(format!("resource_richness must be between 0.0 and 1.0, got {}", self.resource_richness));
        }
        
        // Disaster frequency validation: 0.0-1.0
        if self.disaster_frequency < 0.0 || self.disaster_frequency > 1.0 {
            return Some(format!("disaster_frequency must be between 0.0 and 1.0, got {}", self.disaster_frequency));
        }
        
        // Seed is optional but if present must be non-zero
        if let Some(seed) = self.seed {
            if seed == 0 {
                return Some("seed must be non-zero if provided".to_string());
            }
        }
        
        // Species templates validation
        if let Some(ref templates) = self.species_templates {
            if templates.is_empty() {
                return Some("species_templates cannot be empty if provided".to_string());
            }
            for template in templates {
                if template.id.is_empty() {
                    return Some("species template id cannot be empty".to_string());
                }
                if template.name.is_empty() {
                    return Some("species template name cannot be empty".to_string());
                }
                if template.initial_population == 0 {
                    return Some(format!("species '{}' initial_population must be > 0", template.name));
                }
                // Validate society type
                match template.society_type.as_str() {
                    "band" | "tribe" | "chiefdom" | "nation" | "" => {}
                    _ => return Some(format!(
                        "species '{}' society_type must be one of: band, tribe, chiefdom, nation; got '{}'",
                        template.name, template.society_type
                    )),
                }
            }
        }
        
        None
    }
}

fn default_width() -> u32 { 64 }
fn default_height() -> u32 { 64 }
fn default_prehistory_years() -> u32 { 1000 }
fn default_resource_richness() -> f32 { 0.5 }
fn default_disaster_frequency() -> f32 { 0.1 }
fn default_society_type() -> String { "band".to_string() }

/// Request body for POST /api/v1/worlds/:id/simulate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateBody {
    /// Number of years to simulate (default: 10)
    #[serde(default = "default_years")]
    pub years: u32,
    /// Starting year offset (default: 0)
    #[serde(default)]
    pub start_year: i32,
    /// Optional seed for reproducibility
    #[serde(default)]
    pub seed: Option<u64>,
    /// Enable detailed events (default: true)
    #[serde(default = "default_true")]
    pub detailed_events: bool,
    /// Enable figure generation (default: true)
    #[serde(default = "default_true")]
    pub generate_figures: bool,
    /// Enable artifact generation (default: true)
    #[serde(default = "default_true")]
    pub generate_artifacts: bool,
    /// Maximum growth rate (default: 1.05)
    #[serde(default = "default_growth_rate")]
    pub max_growth_rate: f32,
}

fn default_years() -> u32 { 10 }
fn default_true() -> bool { true }
fn default_growth_rate() -> f32 { 1.05 }

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

impl ApiError {
    pub fn not_found(msg: &str) -> Self {
        ApiError {
            error: msg.to_string(),
            code: "NOT_FOUND".to_string(),
        }
    }
    
    pub fn bad_request(msg: &str) -> Self {
        ApiError {
            error: msg.to_string(),
            code: "BAD_REQUEST".to_string(),
        }
    }
    
    pub fn internal(msg: &str) -> Self {
        ApiError {
            error: msg.to_string(),
            code: "INTERNAL_ERROR".to_string(),
        }
    }
}

/// Register a world in the registry (for testing/setup)
pub fn register_world(id: &str, name: &str, polygons: Vec<PolygonInfo>) {
    let world = WorldState {
        id: id.to_string(),
        name: name.to_string(),
        polygons,
        created_year: 0,
        created_at: chrono::Utc::now(),
    };
    
    if let Ok(mut registry) = WORLD_REGISTRY.write() {
        registry.insert(id.to_string(), world);
    }
}

/// Get world by ID
/// 
/// Returns the world metadata including current generation/simulation status.
/// If the world is still generating, returns progress information.
pub fn get_world(id: &str) -> Option<WorldState> {
    WORLD_REGISTRY.read().ok()?.get(id).cloned()
}

/// Get generation status for a world
/// 
/// Returns the generation task status, or None if no task exists
pub fn get_generation_status(id: &str) -> Option<GenerationTask> {
    GENERATION_TASKS.read().ok()?.get(id).cloned()
}

/// Create a new world and start async generation
/// 
/// Returns immediately with 202 Accepted and a world_id.
/// Generation proceeds asynchronously; poll GET /api/v1/worlds/:id for status.
pub fn create_world(
    request: CreateWorldRequest,
    spawn_fn: impl FnOnce(String, CreateWorldRequest) + Send + 'static,
) -> (String, GenerationTask) {
    let world_id = uuid::Uuid::new_v4().to_string();
    let task = GenerationTask {
        world_id: world_id.clone(),
        status: GenerationStatus::Pending,
        created_at: chrono::Utc::now(),
        completed_at: None,
        result: None,
        error: None,
    };
    
    // Register the pending task
    if let Ok(mut tasks) = GENERATION_TASKS.write() {
        tasks.insert(world_id.clone(), task.clone());
    }
    
    // Spawn the generation task
    spawn_fn(world_id.clone(), request);
    
    (world_id, task)
}

/// Update generation status
pub fn update_generation_status(
    world_id: &str,
    status: GenerationStatus,
    result: Option<WorldState>,
    error: Option<String>,
) {
    if let Ok(mut tasks) = GENERATION_TASKS.write() {
        if let Some(task) = tasks.get_mut(world_id) {
            task.status = status;
            if status == GenerationStatus::Complete || status == GenerationStatus::Failed {
                task.completed_at = Some(chrono::Utc::now());
            }
            if let Some(w) = result {
                task.result = Some(w);
            }
            if let Some(e) = error {
                task.error = Some(e);
            }
        }
    }
}

/// List all worlds with their generation status
pub fn list_worlds() -> Vec<WorldMetadata> {
    let registry = match WORLD_REGISTRY.read() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    
    let tasks = match GENERATION_TASKS.read() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    
    registry.iter().map(|(id, world)| {
        let gen_task = tasks.get(id);
        let status = match gen_task {
            Some(task) => match task.status {
                // For consistency with GET /api/v1/worlds/:id:
                // - Generating → "generating"
                // - Complete → "ready"
                // - Failed → "error"
                // - Pending: if task exists but not yet started, treat as "idle"
                //   BUT if world is registered with polygons, it was already
                //   "created" so should be "ready" to avoid frontend polling mismatches
                GenerationStatus::Pending => {
                    // If world has polygons, generation completed but task
                    // wasn't updated (edge case). Treat as ready.
                    if world.polygons.is_empty() {
                        WorldPhase::Idle
                    } else {
                        WorldPhase::Ready
                    }
                }
                GenerationStatus::Generating => WorldPhase::Generating,
                GenerationStatus::Complete => WorldPhase::Ready,
                GenerationStatus::Failed => WorldPhase::Error,
            },
            // No task exists: world exists in registry, assume ready
            None => WorldPhase::Ready,
        };

        WorldMetadata {
            id: id.clone(),
            name: world.name.clone(),
            status,
            progress: if status == WorldPhase::Ready || status == WorldPhase::Error { 100 } else { 0 },
            message: match status {
                WorldPhase::Generating => "Generating world...",
                WorldPhase::Idle => "Pending...",
                WorldPhase::Ready => "World ready",
                WorldPhase::Error => "Generation failed",
            }.to_string(),
            created_at: world.created_at.to_rfc3339(),
        }
    }).collect()
}

/// World metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub id: String,
    pub name: String,
    #[serde(rename = "status")]
    pub status: WorldPhase,
    pub progress: u32,
    pub message: String,
    pub created_at: String,
}

/// World phase/status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorldPhase {
    Idle,
    Generating,
    Ready,
    Error,
}

/// 202 Accepted response for async world creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorldResponse {
    pub id: String,
    pub name: String,
    pub status: WorldPhase,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polling_url: Option<String>,
}

/// Simulate a world for a number of years
/// 
/// # Arguments
/// * `world_id` - The world identifier
/// * `body` - Simulation parameters
/// 
/// # Returns
/// * `Ok(SimulateResponse)` - Simulation results
/// * `Err((status_code, ApiError))` - Error with HTTP status
pub fn simulate_world(
    world_id: &str,
    body: SimulateBody,
) -> Result<SimulateResponse, (u16, ApiError)> {
    // Validate world_id format (should be valid UUID or alphanumeric)
    if !is_valid_world_id(world_id) {
        return Err((
            400,
            ApiError::bad_request(&format!("Invalid world ID format: {}", world_id)),
        ));
    }
    
    // Look up world
    let world = get_world(world_id).ok_or_else(|| {
        (
            404,
            ApiError::not_found(&format!("World not found: {}", world_id)),
        )
    })?;
    
    // Build simulation request
    let polygons = world.polygons.iter().map(|p| crate::simulation::handler::PolygonData {
        id: p.id,
        elevation: p.elevation,
        neighbors: p.neighbors.clone(),
        is_coastal: p.is_coastal,
        is_island: p.is_island,
    }).collect();
    
    let request = SimulateRequest {
        world_id: world_id.to_string(),
        years: body.years,
        start_year: body.start_year,
        seed: body.seed,
        polygons,
        detailed_events: body.detailed_events,
        generate_figures: body.generate_figures,
        generate_artifacts: body.generate_artifacts,
        max_growth_rate: body.max_growth_rate,
        world_width: Some((world.polygons.len() as f32).sqrt() as u32),
        world_height: Some((world.polygons.len() as f32).sqrt() as u32),
    };
    
    // Run simulation
    handle_simulate(request).map_err(|e| {
        let (status, msg) = match &e {
            SimulationError::InvalidYears => (400, e.to_string()),
            SimulationError::NoPolygons => (400, e.to_string()),
            SimulationError::DuplicatePolygonId(_) => (400, e.to_string()),
            SimulationError::InvalidNeighborReference(_, _) => (400, e.to_string()),
            SimulationError::InternalError(_) => (500, e.to_string()),
        };
        (status, ApiError::internal(&msg))
    })
}

/// Validate world ID format
fn is_valid_world_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    
    // Allow UUID format or simple alphanumeric with dashes/underscores
    id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Worlds router configuration (for integration with Axum/Warp)
pub struct WorldsRouter;

impl WorldsRouter {
    /// Get the route path for simulate endpoint
    pub fn route() -> &'static str {
        "/api/v1/worlds/:id/simulate"
    }
    
    /// Get HTTP method
    pub fn method() -> &'static str {
        "POST"
    }
}

/// Disasters for a world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disaster {
    pub id: String,
    pub name: String,
    pub disaster_type: String,
    pub severity: f32,
    pub affected_area: String,
    pub start_year: u32,
}

/// Resource summary for a world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub amount: f32,
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesSummaryResponse {
    pub resources: Vec<Resource>,
}

/// Figure for the figures endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Figure {
    pub id: String,
    pub name: String,
    pub title: String,
    pub birth_year: u32,
    pub impact_score: f32,
    pub species: String,
}

/// Detailed figure with full biography info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureDetail {
    pub id: String,
    pub name: String,
    pub figure_type: String,
    pub species: String,
    pub birth_year: u32,
    pub death_year: Option<u32>,
    pub significance: f32,
    pub description: Option<String>,
    pub achievements: Vec<String>,
    pub related_world_id: Option<String>,
}

/// In-memory figure registry
static FIGURE_REGISTRY: LazyLock<RwLock<HashMap<String, FigureDetail>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a figure in the registry
pub fn register_figure(figure: FigureDetail) {
    if let Ok(mut registry) = FIGURE_REGISTRY.write() {
        registry.insert(figure.id.clone(), figure);
    }
}

/// Get figure by ID
pub fn get_figure(id: &str) -> Option<FigureDetail> {
    FIGURE_REGISTRY.read().ok()?.get(id).cloned()
}

/// List all figures
pub fn list_figures() -> Vec<FigureDetail> {
    FIGURE_REGISTRY.read().ok().map(|r| r.values().cloned().collect()).unwrap_or_default()
}

/// Figures list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiguresResponse {
    pub figures: Vec<Figure>,
}

/// World stats response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStats {
    pub total_tiles: u32,
    pub land_tiles: u32,
    pub water_tiles: u32,
    pub species_count: u32,
    pub population_by_species: std::collections::HashMap<String, u32>,
}

impl WorldStats {
    /// Create demo stats when no real data is available
    pub fn demo_stats(polygon_count: usize) -> Self {
        let total_tiles = polygon_count as u32;
        let land_tiles = (total_tiles as f32 * 0.7) as u32;
        let water_tiles = total_tiles - land_tiles;
        
        let mut population_by_species = std::collections::HashMap::new();
        population_by_species.insert("Homo Sapiens".to_string(), 1000);
        
        WorldStats {
            total_tiles,
            land_tiles,
            water_tiles,
            species_count: 1,
            population_by_species,
        }
    }
}

/// Get active disasters for a world
/// Returns demo data when no real data is available
pub fn get_world_disasters(world_id: &str) -> Vec<Disaster> {
    // Check if world exists
    if get_world(world_id).is_none() {
        return Vec::new();
    }
    
    // Return demo disasters - in production, these would come from simulation state
    vec![
        Disaster {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Great Drought".to_string(),
            disaster_type: "drought".to_string(),
            severity: 0.7,
            affected_area: "Northern Plains".to_string(),
            start_year: 500,
        },
        Disaster {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Migrating Plague".to_string(),
            disaster_type: "disease".to_string(),
            severity: 0.5,
            affected_area: "Eastern Valleys".to_string(),
            start_year: 750,
        },
    ]
}

/// Get resource summary for a world
/// Returns demo data when no real data is available
pub fn get_world_resources_summary(world_id: &str) -> ResourcesSummaryResponse {
    // Check if world exists
    if get_world(world_id).is_none() {
        return ResourcesSummaryResponse { resources: Vec::new() };
    }
    
    // Return demo resources - in production, these would come from terrain/resources system
    ResourcesSummaryResponse {
        resources: vec![
            Resource {
                name: "Fresh Water".to_string(),
                amount: 1250.0,
                resource_type: "essential".to_string(),
            },
            Resource {
                name: "Iron Ore".to_string(),
                amount: 850.0,
                resource_type: "mineral".to_string(),
            },
            Resource {
                name: "Arable Land".to_string(),
                amount: 620.0,
                resource_type: "agricultural".to_string(),
            },
            Resource {
                name: "Timber".to_string(),
                amount: 480.0,
                resource_type: "material".to_string(),
            },
            Resource {
                name: "Stone".to_string(),
                amount: 350.0,
                resource_type: "mineral".to_string(),
            },
        ],
    }
}

/// Query params for figures endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiguresQueryParams {
    #[serde(default = "default_figures_limit")]
    pub limit: u32,
    #[serde(default = "default_figures_sort")]
    pub sort: String,
}

fn default_figures_limit() -> u32 { 5 }
fn default_figures_sort() -> String { "impact_score".to_string() }

/// Get top figures for a world
/// Returns demo data when no real data is available
pub fn get_world_figures(
    world_id: &str,
    limit: u32,
    _sort: &str, // Sort parameter, currently uses impact_score by default
) -> FiguresResponse {
    // Check if world exists
    if get_world(world_id).is_none() {
        return FiguresResponse { figures: Vec::new() };
    }
    
    // Return demo figures - in production, these would come from the figures module
    let all_figures = vec![
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Aldric the Wise".to_string(),
            title: "First King".to_string(),
            birth_year: 423,
            impact_score: 0.95,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Brynn Ironforge".to_string(),
            title: "Master Smith".to_string(),
            birth_year: 567,
            impact_score: 0.82,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Cedric the Navigator".to_string(),
            title: "Explorer".to_string(),
            birth_year: 234,
            impact_score: 0.78,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Diana of the Falls".to_string(),
            title: "Healer".to_string(),
            birth_year: 612,
            impact_score: 0.71,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Eldric Stormborn".to_string(),
            title: "Warrior Chief".to_string(),
            birth_year: 445,
            impact_score: 0.68,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Freya the Diplomat".to_string(),
            title: "Treaty Maker".to_string(),
            birth_year: 589,
            impact_score: 0.64,
            species: "Homo Sapiens".to_string(),
        },
        Figure {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Gareth Swiftarrow".to_string(),
            title: "Scout".to_string(),
            birth_year: 378,
            impact_score: 0.55,
            species: "Homo Sapiens".to_string(),
        },
    ];
    
    // Sort by impact_score descending
    let mut sorted_figures = all_figures;
    sorted_figures.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
    
    // Take up to limit
    let limited: Vec<Figure> = sorted_figures.into_iter().take(limit as usize).collect();
    
    FiguresResponse { figures: limited }
}

/// Aggregate stats response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    pub total_worlds: u32,
    pub total_polygons: u32,
    pub total_species: u32,
    pub oldest_world_created_year: Option<i32>,
    pub newest_world_created_year: Option<i32>,
}

/// Get world stats
/// Returns demo stats when no real data is available
pub fn get_world_stats(world_id: &str) -> Option<WorldStats> {
    let world = get_world(world_id)?;
    let polygon_count = world.polygons.len();
    Some(WorldStats::demo_stats(polygon_count))
}

/// History event for the events endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: String,
    pub world_id: String,
    pub tick: u32,
    #[serde(rename = "type")]
    pub event_type: String,
    pub description: String,
    pub data: Option<serde_json::Value>,
}

/// Query params for history/events endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQueryParams {
    #[serde(default = "default_history_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_history_limit() -> u32 { 100 }

/// Get simulation history/events for a world
/// Returns demo events when no real data is available
pub fn get_world_history(
    world_id: &str,
    limit: u32,
    offset: u32,
) -> Vec<HistoryEvent> {
    // Check if world exists
    if get_world(world_id).is_none() {
        return Vec::new();
    }
    
    // Return demo history events - in production, these would come from 
    // the simulation engine's event log stored per-world
    let all_events = vec![
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 1000,
            event_type: "population_growth".to_string(),
            description: "Population growth detected in the valley settlement".to_string(),
            data: Some(serde_json::json!({"settlement": "Valley", "growth_rate": 0.05})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 950,
            event_type: "migration".to_string(),
            description: "A new tribe migrated to the coastal region".to_string(),
            data: Some(serde_json::json!({"origin": "unknown", "destination": "Coast"})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 900,
            event_type: "settlement_founded".to_string(),
            description: "A new settlement was founded in the highlands".to_string(),
            data: Some(serde_json::json!({"name": "Highland Hold", "terrain": "mountain"})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 850,
            event_type: "discovery".to_string(),
            description: "Iron ore deposits discovered near the river".to_string(),
            data: Some(serde_json::json!({"resource": "iron", "location": "river_ford"})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 800,
            event_type: "natural_disaster".to_string(),
            description: "A severe drought affected the eastern plains".to_string(),
            data: Some(serde_json::json!({"type": "drought", "severity": 0.7})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 750,
            event_type: "war".to_string(),
            description: "Conflict erupted between two neighboring settlements".to_string(),
            data: Some(serde_json::json!({"parties": ["North Settlement", "South Settlement"]})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 700,
            event_type: "artifact_created".to_string(),
            description: "A ceremonial stone was created to commemorate the peace".to_string(),
            data: Some(serde_json::json!({"artifact_type": "religious", "significance": 0.6})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 650,
            event_type: "figure_born".to_string(),
            description: "A notable leader was born in the northern village".to_string(),
            data: Some(serde_json::json!({"name": "Aldric the Founder", "role": "leader"})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 600,
            event_type: "society_transition".to_string(),
            description: "The tribe transitioned from band to tribe organization".to_string(),
            data: Some(serde_json::json!({"from": "band", "to": "tribe", "population": 150})),
        },
        HistoryEvent {
            id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.to_string(),
            tick: 550,
            event_type: "territory_expansion".to_string(),
            description: "The growing tribe expanded into the western forest".to_string(),
            data: Some(serde_json::json!({"direction": "west", "new_settlements": 2})),
        },
    ];
    
    // Apply pagination
    let start = offset as usize;
    let end = (offset + limit) as usize;
    
    if start >= all_events.len() {
        return Vec::new();
    }
    
    all_events.into_iter().skip(start).take(end - start).collect()
}

/// Get aggregate statistics across all worlds
pub fn get_aggregate_stats() -> AggregateStats {
    let registry = match WORLD_REGISTRY.read() {
        Ok(r) => r,
        Err(_) => return AggregateStats {
            total_worlds: 0,
            total_polygons: 0,
            total_species: 0,
            oldest_world_created_year: None,
            newest_world_created_year: None,
        },
    };
    
    let total_worlds = registry.len() as u32;
    let total_polygons: u32 = registry.values().map(|w| w.polygons.len() as u32).sum();
    let total_species = 1; // Placeholder - would come from world simulation state
    
    let mut oldest: Option<i32> = None;
    let mut newest: Option<i32> = None;
    for world in registry.values() {
        let year = world.created_year;
        oldest = Some(oldest.map_or(year, |o| o.min(year)));
        newest = Some(newest.map_or(year, |n| n.max(year)));
    }
    
    AggregateStats {
        total_worlds,
        total_polygons,
        total_species,
        oldest_world_created_year: oldest,
        newest_world_created_year: newest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_world_id() {
        assert!(is_valid_world_id("world-1"));
        assert!(is_valid_world_id("my_world"));
        assert!(is_valid_world_id("abc123"));
        assert!(is_valid_world_id("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_invalid_world_id() {
        assert!(!is_valid_world_id(""));
        assert!(!is_valid_world_id("a".repeat(65).as_str()));
        assert!(!is_valid_world_id("world/with/slashes"));
        assert!(!is_valid_world_id("world with spaces"));
    }

    #[test]
    fn test_register_and_get_world() {
        let polygons = vec![
            PolygonInfo {
                id: 1,
                elevation: 100.0,
                neighbors: vec![2],
                is_coastal: false,
                is_island: false,
            },
        ];
        
        register_world("test-world", "Test World", polygons);
        let world = get_world("test-world");
        
        assert!(world.is_some());
        assert_eq!(world.unwrap().name, "Test World");
    }

    #[test]
    fn test_simulate_world_not_found() {
        let body = SimulateBody {
            years: 10,
            start_year: 0,
            seed: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
        };
        
        let result = simulate_world("nonexistent-world", body);
        assert!(result.is_err());
        
        let (status, _) = result.unwrap_err();
        assert_eq!(status, 404);
    }

    #[test]
    fn test_simulate_world_success() {
        let polygons = vec![
            PolygonInfo {
                id: 1,
                elevation: 100.0,
                neighbors: vec![2],
                is_coastal: false,
                is_island: false,
            },
            PolygonInfo {
                id: 2,
                elevation: 200.0,
                neighbors: vec![1],
                is_coastal: false,
                is_island: false,
            },
        ];
        
        register_world("test-world-2", "Test World 2", polygons);
        
        let body = SimulateBody {
            years: 10,
            start_year: 0,
            seed: Some(42),
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
        };
        
        let result = simulate_world("test-world-2", body);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.world_id, "test-world-2");
        assert_eq!(response.stats.years_simulated, 10);
    }

    #[test]
    fn test_simulate_world_invalid_id() {
        let body = SimulateBody {
            years: 10,
            start_year: 0,
            seed: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
            max_growth_rate: 1.05,
        };
        
        let result = simulate_world("invalid/id", body);
        assert!(result.is_err());
        
        let (status, _) = result.unwrap_err();
        assert_eq!(status, 400);
    }

    #[test]
    fn test_create_world_creates_pending_task() {
        let request = CreateWorldRequest {
            name: Some("Test World".to_string()),
            seed: Some(12345),
            width: 64,
            height: 64,
            prehistory_years: 100,
            resource_richness: 0.5,
            disaster_frequency: 0.1,
            species_templates: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        
        let (world_id, task) = create_world(request, |_id, _req| {
            // No-op spawn function for testing
        });
        
        assert!(!world_id.is_empty());
        assert_eq!(task.world_id, world_id);
        assert_eq!(task.status, GenerationStatus::Pending);
        
        // Verify task is in the registry
        let stored_task = get_generation_status(&world_id);
        assert!(stored_task.is_some());
    }

    #[test]
    fn test_create_world_request_validation_width() {
        // Width > 128 should fail
        let request = CreateWorldRequest {
            name: None,
            seed: None,
            width: 256,
            height: 64,
            prehistory_years: 100,
            resource_richness: 0.5,
            disaster_frequency: 0.1,
            species_templates: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        assert!(request.validate().is_some());
        
        // Width = 0 should fail
        let mut request = request.clone();
        request.width = 0;
        assert!(request.validate().is_some());
        
        // Width = 128 should pass
        request.width = 128;
        assert!(request.validate().is_none());
        
        // Height > 128 should fail
        request.height = 256;
        assert!(request.validate().is_some());
        
        // Valid dimensions should pass
        request.height = 64;
        assert!(request.validate().is_none());
    }

    #[test]
    fn test_create_world_request_validation_prehistory_years() {
        let mut request = CreateWorldRequest {
            name: None,
            seed: None,
            width: 64,
            height: 64,
            prehistory_years: 100001, // Too high (> 100000)
            resource_richness: 0.5,
            disaster_frequency: 0.1,
            species_templates: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        assert!(request.validate().is_some());
        
        // Zero years should fail
        request.prehistory_years = 0;
        assert!(request.validate().is_some());
        
        // Valid max (100000) should pass
        request.prehistory_years = 100000;
        assert!(request.validate().is_none());
        
        // Valid years should pass
        request.prehistory_years = 1000;
        assert!(request.validate().is_none());
    }

    #[test]
    fn test_create_world_request_validation_resource_params() {
        let mut request = CreateWorldRequest {
            name: None,
            seed: None,
            width: 64,
            height: 64,
            prehistory_years: 1000,
            resource_richness: 1.5, // Invalid - must be 0-1
            disaster_frequency: 0.1,
            species_templates: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        assert!(request.validate().is_some());
        
        // Valid resource_richness should pass
        request.resource_richness = 0.5;
        assert!(request.validate().is_none());
        
        // Invalid disaster_frequency should fail
        request.disaster_frequency = -0.5;
        assert!(request.validate().is_some());
        
        // Valid disaster_frequency should pass
        request.disaster_frequency = 0.1;
        assert!(request.validate().is_none());
    }

    #[test]
    fn test_create_world_request_validation_species_templates() {
        let mut request = CreateWorldRequest {
            name: None,
            seed: None,
            width: 64,
            height: 64,
            prehistory_years: 1000,
            resource_richness: 0.5,
            disaster_frequency: 0.1,
            species_templates: Some(vec![]), // Empty is invalid
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        assert!(request.validate().is_some());
        
        // Empty template with id should fail
        request.species_templates = Some(vec![
            SpeciesTemplate {
                id: "".to_string(),
                name: "Test".to_string(),
                initial_population: 100,
                society_type: "band".to_string(),
            }
        ]);
        assert!(request.validate().is_some());
        
        // Valid template should pass
        request.species_templates = Some(vec![
            SpeciesTemplate {
                id: "homo-sapiens".to_string(),
                name: "Homo Sapiens".to_string(),
                initial_population: 100,
                society_type: "band".to_string(),
            }
        ]);
        assert!(request.validate().is_none());
    }

    #[test]
    fn test_create_world_request_validation_seed() {
        let mut request = CreateWorldRequest {
            name: None,
            seed: Some(0), // Zero seed is invalid
            width: 64,
            height: 64,
            prehistory_years: 1000,
            resource_richness: 0.5,
            disaster_frequency: 0.1,
            species_templates: None,
            detailed_events: true,
            generate_figures: true,
            generate_artifacts: true,
        };
        assert!(request.validate().is_some());
        
        // Valid non-zero seed should pass
        request.seed = Some(12345);
        assert!(request.validate().is_none());
        
        // None seed should pass
        request.seed = None;
        assert!(request.validate().is_none());
    }

    #[test]
    fn test_list_worlds_returns_metadata() {
        // Register a test world
        let polygons = vec![
            PolygonInfo {
                id: 1,
                elevation: 100.0,
                neighbors: vec![],
                is_coastal: false,
                is_island: false,
            },
        ];
        register_world("list-test-world", "List Test World", polygons);
        
        let worlds = list_worlds();
        assert!(!worlds.is_empty());
        
        // Find our test world
        let test_world = worlds.iter().find(|w| w.id == "list-test-world");
        assert!(test_world.is_some());
        let test_world = test_world.unwrap();
        assert_eq!(test_world.name, "List Test World");
    }

    #[test]
    fn test_world_phase_serialization() {
        // Test that WorldPhase serializes correctly
        let phase = WorldPhase::Generating;
        let json = serde_json::to_string(&phase).unwrap();
        assert_eq!(json, "\"generating\""); // Note: double quotes for string literal
        
        let phase = WorldPhase::Ready;
        let json = serde_json::to_string(&phase).unwrap();
        assert_eq!(json, "\"ready\""); // Note: double quotes for string literal
    }

    #[test]
    fn test_list_world_status_consistency_with_get_world() {
        // Regression test for WOR-647: world list status should match get world status
        // 
        // When a world has polygons (generation completed), list_worlds should
        // return status "ready" to match get_world behavior.
        
        // Create world WITH polygons (simulating completed generation)
        let polygons = vec![
            PolygonInfo {
                id: 1,
                elevation: 100.0,
                neighbors: vec![],
                is_coastal: false,
                is_island: false,
            },
            PolygonInfo {
                id: 2,
                elevation: 50.0,
                neighbors: vec![1],
                is_coastal: false,
                is_island: false,
            },
        ];
        register_world("status-test-world", "Status Test World", polygons);
        
        // Verify list_worlds returns "ready" for worlds with polygons
        let worlds = list_worlds();
        let test_world = worlds.iter().find(|w| w.id == "status-test-world");
        
        assert!(test_world.is_some(), "World should be in list");
        let world_meta = test_world.unwrap();
        
        // Status should be "ready" to match GET /api/v1/worlds/:id behavior
        assert_eq!(world_meta.status, WorldPhase::Ready, 
            "List endpoint status should match individual endpoint (WOR-647)");
        assert_eq!(world_meta.progress, 100);
        assert_eq!(world_meta.message, "World ready");
    }

    #[test]
    fn test_list_world_pending_without_polygons() {
        // Worlds registered without polygons (generation not started) should show "idle"
        register_world("pending-test-world", "Pending Test World", vec![]);
        
        let worlds = list_worlds();
        let test_world = worlds.iter().find(|w| w.id == "pending-test-world");
        
        assert!(test_world.is_some());
        let world_meta = test_world.unwrap();
        
        // Without polygons, status should be "idle" (pending generation)
        assert_eq!(world_meta.status, WorldPhase::Idle);
        assert_eq!(world_meta.progress, 0);
        assert_eq!(world_meta.message, "Pending...");
    }
}
