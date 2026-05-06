//! API request and response models for World Factory
//!
//! Defines types for the API contract in docs/API_CONTRACT.md

use serde::{Deserialize, Serialize};

// =============================================================================
// Request Types
// =============================================================================

/// Query parameters for GET /api/v1/worlds/:id/map
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWorldMapParams {
    /// Viewport minimum X coordinate
    #[serde(default)]
    pub min_x: Option<f64>,
    /// Viewport minimum Y coordinate  
    #[serde(default)]
    pub min_y: Option<f64>,
    /// Viewport maximum X coordinate
    #[serde(default)]
    pub max_x: Option<f64>,
    /// Viewport maximum Y coordinate
    #[serde(default)]
    pub max_y: Option<f64>,
    /// Level of detail (0=low, 1=medium, 2=high)
    #[serde(default = "default_lod")]
    pub lod: u8,
}

fn default_lod() -> u8 {
    1
}

/// Query parameters for GET /api/v1/worlds
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorldsParams {
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
}

fn default_limit() -> usize {
    20
}

// =============================================================================
// Response Wrappers
// =============================================================================

/// Standard API response wrapper
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { success: true, data }
    }
}

/// Paginated list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>, total: usize, limit: usize, offset: usize) -> Self {
        Self { items, total, limit, offset }
    }
}

// =============================================================================
// World Domain Types  
// =============================================================================

/// World entity
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub id: String,
    pub name: String,
    pub status: WorldStatus,
    /// Generation progress (0.0 - 1.0), only meaningful when status is "generating"
    pub progress: Option<f64>,
    pub created_at: String,
    pub parameters: WorldParameters,
}

/// World generation status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorldStatus {
    Pending,
    Generating,
    Ready,
    Failed,
}

/// Parameters used to generate a world
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorldParameters {
    pub seed: u64,
    pub size: WorldSize,
    /// Optional climate parameters for customization
    #[serde(default)]
    pub climate: Option<ClimateParameters>,
    /// Optional terrain generation parameters (Phase 1)
    /// Controls Lloyd relaxation and erosion simulation
    #[serde(default)]
    pub terrain: Option<TerrainGenerationParams>,
}

/// Terrain generation parameters for Phase 1
/// Controls Lloyd relaxation for Voronoi cells and erosion simulation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerrainGenerationParams {
    /// Number of Lloyd relaxation iterations for Voronoi cells (0-5, default: 2)
    /// More iterations = more uniform cell sizes but slower generation
    #[serde(default)]
    pub lloyd_iterations: Option<u32>,
    /// Enable erosion simulation (hydraulic + thermal) (default: true)
    #[serde(default)]
    pub enable_erosion: Option<bool>,
    /// Number of erosion iterations (droplets) (default: 100_000)
    /// Higher = more realistic but slower
    #[serde(default)]
    pub erosion_iterations: Option<usize>,
    /// Erosion strength 0.0-1.0 (default: 0.3)
    #[serde(default)]
    pub erosion_strength: Option<f32>,
}

impl Default for TerrainGenerationParams {
    fn default() -> Self {
        Self {
            lloyd_iterations: Some(2),
            enable_erosion: Some(true),
            erosion_iterations: Some(100_000),
            erosion_strength: Some(0.3),
        }
    }
}

/// Climate parameters for world generation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClimateParameters {
    /// Base temperature at equator in Celsius (default: 30.0)
    #[serde(default = "default_base_temperature")]
    pub base_temperature: f32,
    /// Temperature lapse rate per 1000m in °C/km (default: -6.5)
    #[serde(default = "default_lapse_rate")]
    pub lapse_rate: f32,
    /// Latitude temperature gradient in °C per degree (default: 0.6)
    #[serde(default = "default_latitude_gradient")]
    pub latitude_gradient: f32,
}

fn default_base_temperature() -> f32 { 30.0 }
fn default_lapse_rate() -> f32 { -6.5 }
fn default_latitude_gradient() -> f32 { 0.6 }

impl Default for WorldStatus {
    fn default() -> Self {
        WorldStatus::Pending
    }
}

/// World size preset
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum WorldSize {
    #[default]
    Medium, // 256x256 terrain
    Small,  // 128x128 terrain
    Large,  // 512x512 terrain
}

// =============================================================================
// Map Data Types
// =============================================================================

/// Full render-ready map data for a world
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldMap {
    pub world_id: String,
    pub dimensions: MapDimensions,
    pub scale: f64,
    pub polygons: Vec<Polygon>,
    pub biomes: Vec<Biome>,
    pub resources: Vec<Resource>,
    pub entities: Vec<MapEntity>,
    /// Optional elevation grid for 3D rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_grid: Option<Vec<Vec<f64>>>,
    pub metadata: MapMetadata,
}

/// Map dimensions
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapDimensions {
    pub width: usize,
    pub height: usize,
}

/// Polygon representing a geographic feature
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Polygon {
    pub id: String,
    pub polygon_type: PolygonType,
    pub vertices: Vec<Vertex>,
    /// Centroid of the polygon for label positioning
    pub centroid: Option<Vertex>,
    /// Holes within the polygon (for territories with enclaves)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holes: Option<Vec<Vec<Vertex>>>,
    /// Normalized elevation (0.0 = sea level, 1.0 = max height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
    /// Whether this polygon is ocean (elevation <= ocean threshold)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ocean: Option<bool>,
    /// Whether this polygon is coastal (adjacent to both ocean and land)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_coastal: Option<bool>,
    /// Ocean depth zone: land, shallow, medium, deep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocean_zone: Option<String>,
    /// Biome type identifier for color/style mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biome_type: Option<String>,
    /// Temperature value (0.0-1.0) for heatmap visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Moisture/precipitation value (0.0-1.0) for moisture overlay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moisture: Option<f64>,
    /// Whether this polygon is a coast (bordering ocean)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_coast: Option<bool>,
    /// River volume for water rendering (0.0 = none)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub river_volume: Option<f64>,
}

/// Polygon type categories
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PolygonType {
    Territory,
    Biome,
    Region,
    Resource,
}

/// 2D vertex coordinate
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
}

/// Biome definition with visual properties
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Biome {
    pub id: String,
    pub biome_type: String,
    /// RGB color [R, G, B] with values 0-255
    pub color: [u8; 3],
    pub name: String,
}

/// Resource deposit location
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub position: Vertex,
    /// Magnitude 1-5 scale
    pub magnitude: u8,
    pub name: String,
}

/// Map entity (cities, settlements, landmarks)
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapEntity {
    pub id: String,
    pub entity_type: MapEntityType,
    pub position: Vertex,
    pub name: String,
    /// Significance 1-10 scale
    pub significance: u8,
}

/// Entity type categories
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MapEntityType {
    City,
    Settlement,
    Landmark,
    Fortress,
}

/// Map generation metadata
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapMetadata {
    pub generated_at: String,
    pub version: String,
}

// =============================================================================
// Request Payloads
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorldRequest {
    pub name: Option<String>,
    pub parameters: Option<WorldParameters>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateWorldRequest {
    pub name: Option<String>,
    pub parameters: Option<WorldParameters>,
}

// =============================================================================
// Figures API Request Types
// =============================================================================

/// Query parameters for GET /api/v1/worlds/:id/figures
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetWorldFiguresParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_figures_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by species ID
    #[serde(default)]
    pub species_id: Option<String>,
    /// Filter by home region
    #[serde(default)]
    pub region_id: Option<String>,
    /// Minimum significance (0.0 - 1.0)
    #[serde(default)]
    pub min_significance: Option<f64>,
}

fn default_figures_limit() -> usize {
    50
}

// =============================================================================
// Natural Wonders API Response Types (WOR-77)
// =============================================================================

/// Response for natural wonders list endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WondersResponse {
    pub world_id: String,
    pub wonders: Vec<WonderView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub stats: Option<WonderStats>,
}

impl WondersResponse {
    pub fn new(world_id: String, wonders: Vec<WonderView>, total: usize, limit: usize, offset: usize) -> Self {
        Self { world_id, wonders, total, limit, offset, stats: None }
    }
    
    pub fn with_stats(mut self, stats: WonderStats) -> Self {
        self.stats = Some(stats);
        self
    }
}

/// Wonder summary view for list responses
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WonderView {
    pub id: String,
    pub name: String,
    pub wonder_type: String,
    pub category: String,
    pub x: f32,
    pub y: f32,
    pub influence_radius: f32,
    pub description: String,
    pub bonuses: Vec<WonderBonusView>,
    pub primary_color: String,
    pub icon_type: String,
}

/// Wonder bonus view for API responses
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WonderBonusView {
    pub bonus_type: String,
    pub magnitude: f32,
    pub radius: f32,
    pub region_wide: bool,
}

/// Statistics about wonders in a world
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WonderStats {
    pub total_wonders: usize,
    pub by_category: std::collections::HashMap<String, usize>,
    pub avg_influence_radius: f32,
}

/// Query parameters for GET /api/v1/worlds/:id/wonders
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WondersQueryParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_wonders_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by wonder category (geological, hydrological, biological, atmospheric, magical, unique)
    #[serde(default)]
    pub category: Option<String>,
    /// Filter by wonder type
    #[serde(default)]
    pub wonder_type: Option<String>,
    /// Include bonuses in response (default: true)
    #[serde(default)]
    pub include_bonuses: bool,
}

fn default_wonders_limit() -> usize {
    50
}

/// Query parameters for GET /api/v1/worlds/:id/societies
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetWorldSocietiesParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_figures_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by region ID
    #[serde(default)]
    pub region_id: Option<String>,
    /// Filter by species ID
    #[serde(default)]
    pub species_id: Option<String>,
}

/// Query parameters for GET /api/v1/worlds/:id/planet
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetWorldPlanetParams {
    /// Include geography data (default: true)
    #[serde(default)]
    pub include_geography: Option<bool>,
    /// Include tectonic plate data (default: false)
    #[serde(default)]
    pub include_tectonics: Option<bool>,
}

// =============================================================================
// Planet/Geography API Types
// =============================================================================

/// Response for planet endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetResponse {
    pub world_id: String,
    pub planet: PlanetView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geography: Option<GeographyView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tectonics: Option<TectonicsData>,
}

impl PlanetResponse {
    pub fn new(
        world_id: String,
        planet: PlanetView,
        include_geography: bool,
        include_tectonics: bool,
    ) -> Self {
        Self {
            world_id,
            planet,
            geography: None,
            tectonics: None,
        }
    }
    
    pub fn with_geography(mut self, geography: GeographyView) -> Self {
        self.geography = Some(geography);
        self
    }
    
    pub fn with_tectonics(mut self, tectonics: TectonicsData) -> Self {
        self.tectonics = Some(tectonics);
        self
    }
}

/// Full planet view
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanetView {
    pub id: String,
    pub name: String,
    pub planet_type: PlanetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius_km: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass_earths: Option<f64>,
    pub terrain_dimensions: TerrainDimensionsView,
    pub axial_tilt_deg: f32,
    pub rotation_period_h: f32,
    pub orbital_period_d: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gravity_m_s2: Option<f32>,
    pub has_surface_water: bool,
    pub has_magnetic_field: bool,
    pub is_geologically_active: bool,
}

/// Planet type classification
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum PlanetType {
    Terrestrial,
    Desert,
    Ocean,
    Ice,
    GasGiant,
    Volcanic,
    Mixed,
}

/// Terrain grid dimensions
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerrainDimensionsView {
    pub width: u32,
    pub height: u32,
    pub cell_size_m: f32,
}

/// Tectonics data view
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TectonicsData {
    pub plates: Vec<TectonicPlateView>,
    pub boundaries: Vec<TectonicBoundaryView>,
}

/// Tectonic plate view
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TectonicPlateView {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub plate_type: String,
    pub movement_direction_deg: f32,
    pub movement_speed_cm_yr: f32,
    pub area_km2: f64,
    pub cell_count: usize,
}

/// Tectonic boundary view
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TectonicBoundaryView {
    pub id: String,
    pub boundary_type: String,
    pub length_km: f64,
    pub is_volcanic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volcanic_activity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seismic_activity: Option<f32>,
}

/// Geography data for planet surface
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeographyView {
    pub terrain_dimensions: TerrainDimensionsView,
    pub total_land_area_km2: Option<f64>,
    pub total_water_area_km2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub land_to_water_ratio: Option<f64>,
    pub regions: Vec<RegionView>,
    pub rivers: Vec<RiverView>,
    pub settlements: Vec<SettlementView>,
    pub biomes: Vec<BiomeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drainage_basins: Option<Vec<DrainageBasinView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
}

/// Region view for geography response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionView {
    pub id: String,
    pub name: String,
    pub area_km2: f64,
    pub center_lat: f64,
    pub center_lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub climate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_region_id: Option<String>,
}

/// River view for geography response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiverView {
    pub id: String,
    pub name: String,
    pub length_km: Option<f64>,
    pub source_lat: f64,
    pub source_lon: f64,
    pub mouth_lat: f64,
    pub mouth_lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drainage_basin_id: Option<String>,
}

/// Biome view for geography response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiomeView {
    pub id: String,
    pub biome_type: String,
    pub name: String,
    pub color_rgb: [u8; 3],
}

/// Drainage basin view for geography response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrainageBasinView {
    pub id: String,
    pub area_polygons: u32,
    pub outlet_type: String,
    pub outlet_id: u32,
    pub avg_elevation: f32,
    pub elevation_range: f32,
    pub river_polygon_count: u32,
    pub polygon_ids: Vec<u32>,
}

// =============================================================================
// Figures API Response Types
// =============================================================================

/// Response for figures endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiguresResponse {
    pub world_id: String,
    pub figures: Vec<HistoricalFigure>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl FiguresResponse {
    pub fn new(world_id: String, figures: Vec<HistoricalFigure>, total: usize, limit: usize, offset: usize) -> Self {
        Self { world_id, figures, total, limit, offset }
    }
}

/// Historical figure representation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalFigure {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<PersonName>,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub significance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_id: Option<String>,
}

impl From<&crate::figures::NotableFigure> for HistoricalFigure {
    fn from(figure: &crate::figures::NotableFigure) -> Self {
        let name = figure.name.as_ref().map(|n| PersonName {
            given: n.given.clone(),
            family: n.family.clone(),
            epithet: n.epithet.clone(),
            title: n.title.clone(),
        });
        
        let titles = figure.titles.clone();
        let description = figure.description.clone();
        
        Self {
            id: figure.id.to_uuid().to_string(),
            name,
            entity_type: format!("{:?}", figure.figure_type),
            birth_year: figure.birth_year,
            death_year: figure.death_year,
            birthplace_id: figure.birthplace_id.map(|id| id.to_string()),
            culture: figure.culture.clone(),
            titles,
            description,
            significance: figure.significance as f64,
            species_id: figure.species_id.map(|id| id.to_string()),
        }
    }
}

/// Person name with optional components
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersonName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epithet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Figure type enum for API responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ApiFigureType {
    Monarch,
    MilitaryLeader,
    Scholar,
    Artist,
    ReligiousLeader,
    Explorer,
    Inventor,
    Hero,
    Villain,
    FolkHero,
    Legendary,
}

impl From<crate::figures::FigureType> for ApiFigureType {
    fn from(t: crate::figures::FigureType) -> Self {
        match t {
            crate::figures::FigureType::Monarch => ApiFigureType::Monarch,
            crate::figures::FigureType::MilitaryLeader => ApiFigureType::MilitaryLeader,
            crate::figures::FigureType::Scholar => ApiFigureType::Scholar,
            crate::figures::FigureType::Artist => ApiFigureType::Artist,
            crate::figures::FigureType::ReligiousLeader => ApiFigureType::ReligiousLeader,
            crate::figures::FigureType::Explorer => ApiFigureType::Explorer,
            crate::figures::FigureType::Inventor => ApiFigureType::Inventor,
            crate::figures::FigureType::Hero => ApiFigureType::Hero,
            crate::figures::FigureType::Villain => ApiFigureType::Villain,
            crate::figures::FigureType::FolkHero => ApiFigureType::FolkHero,
            crate::figures::FigureType::Legendary => ApiFigureType::Legendary,
        }
    }
}

// =============================================================================
// Timeline API Request Types
// =============================================================================

/// Query parameters for GET /api/v1/worlds/:id/timeline and /api/v1/worlds/:id/events
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    /// Maximum number of events to return (default: 100)
    #[serde(default = "default_timeline_limit")]
    pub limit: usize,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// Sort order: 'asc' for oldest first, 'desc' for newest first (default: 'asc')
    #[serde(default)]
    pub sort: Option<String>,
    /// Comma-separated list of event types to filter
    #[serde(default)]
    pub event_types: Option<String>,
    /// Filter events involving this entity
    #[serde(default)]
    pub entity_id: Option<String>,
    /// Filter events in this region
    #[serde(default)]
    pub region_id: Option<String>,
    /// Filter events from this year onwards
    #[serde(default)]
    pub start_year: Option<i32>,
    /// Filter events up to this year
    #[serde(default)]
    pub end_year: Option<i32>,
    /// Minimum significance (0.0 - 1.0)
    #[serde(default)]
    pub min_significance: Option<f64>,
    /// Comma-separated tags to filter
    #[serde(default)]
    pub tags: Option<String>,
}

fn default_timeline_limit() -> usize {
    100
}

/// Query parameters for GET /api/v1/events/:id
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventQueryParams {
    /// Include related events in response
    #[serde(default)]
    pub include_related: Option<bool>,
}

// =============================================================================
// Timeline API Response Types
// =============================================================================

/// Response for timeline endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineResponse {
    pub world_id: String,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub events: Vec<TimelineEventView>,
    pub total_events: usize,
}

impl TimelineResponse {
    pub fn new(world_id: String, events: Vec<TimelineEventView>, total_events: usize, start_year: Option<i32>, end_year: Option<i32>) -> Self {
        Self {
            world_id,
            start_year,
            end_year,
            events,
            total_events,
        }
    }
}

/// Simplified event view for timeline responses
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventView {
    pub id: String,
    pub event_type: String,
    pub position: EventPosition,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<EventParticipant>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prerequisites: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<EventOutcome>,
    pub significance: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_entities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Event position in time
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventPosition {
    pub year: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub century: Option<String>,
}

/// Event participant in timeline view
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventParticipant {
    pub entity_id: String,
    pub name: String,
    pub entity_type: String,
    pub role: String,
}

/// Event outcome in timeline view
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventOutcome {
    pub outcome_type: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_entities: Option<Vec<String>>,
    pub magnitude: f64,
}

/// Response for single event endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResponse {
    pub event: TimelineEventView,
}

impl EventResponse {
    pub fn new(event: TimelineEventView) -> Self {
        Self { event }
    }
}

/// Paginated events list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsListResponse {
    pub events: Vec<TimelineEventView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

// =============================================================================
// History API Types
// =============================================================================

/// Query parameters for GET /api/v1/worlds/:id/history
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    /// Maximum number of events to return (default: 50, max: 200)
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// Comma-separated list of event types to include
    #[serde(default)]
    pub event_types: Option<String>,
    /// Start year (inclusive)
    #[serde(default)]
    pub start_year: Option<i32>,
    /// End year (inclusive)
    #[serde(default)]
    pub end_year: Option<i32>,
    /// Filter by entity involvement
    #[serde(default)]
    pub entity_id: Option<String>,
    /// Minimum significance (0.0 - 1.0)
    #[serde(default)]
    pub min_significance: Option<f64>,
    /// Comma-separated tags to filter
    #[serde(default)]
    pub tags: Option<String>,
}

fn default_history_limit() -> usize {
    50
}

/// Response for history endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponse {
    pub world_id: String,
    pub total_events: usize,
    pub events: Vec<HistoryEventView>,
    pub pagination: Pagination,
    pub filters_applied: AppliedFilters,
}

/// Simplified event view for history
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventView {
    pub id: String,
    pub event_type: String,
    pub year: i32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub significance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
}

/// Filters applied to the query
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_significance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// =============================================================================
// Societies API Types
// =============================================================================

/// Query parameters for GET /api/v1/worlds/:id/societies
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SocietiesQueryParams {
    /// Filter by settlement type
    #[serde(default)]
    pub settlement_type: Option<String>,
    /// Filter by species
    #[serde(default)]
    pub species: Option<String>,
    /// Maximum results (default: 50, max: 200)
    #[serde(default = "default_societies_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
}

fn default_societies_limit() -> usize {
    50
}

/// Response for planet endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocietiesResponse {
    pub world_id: String,
    pub societies: Vec<SocietyView>,
    pub total_societies: usize,
    pub total_settlements: usize,
}

impl SocietiesResponse {
    pub fn new(world_id: String, societies: Vec<SocietyView>, total_settlements: usize) -> Self {
        let total_societies = societies.len();
        Self { world_id, societies, total_societies, total_settlements }
    }
}

/// Response for planet endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocietyView {
    pub species_id: String,
    pub species_name: String,
    pub settlements: Vec<SettlementView>,
    pub total_population: u64,
    pub settlement_count: usize,
    pub dominant_settlement_type: Option<String>,
}

/// Settlement details within a society response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementView {
    pub id: String,
    pub name: String,
    pub settlement_type: Option<String>,
    pub population: Option<u64>,
    pub location: GeoLocationView,
    pub description: Option<String>,
    /// Species ID that inhabits this settlement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_id: Option<String>,
}

impl From<&crate::types::Settlement> for SettlementView {
    fn from(settlement: &crate::types::Settlement) -> Self {
        Self {
            id: settlement.id.to_uuid().to_string(),
            name: settlement.name.clone(),
            settlement_type: settlement.settlement_type.map(|t| format!("{:?}", t)),
            population: settlement.population,
            location: GeoLocationView::from(&settlement.location),
            description: settlement.description.clone(),
            species_id: settlement.species_id.map(|id| id.as_u32().to_string()),
        }
    }
}

/// Simplified location for API responses
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoLocationView {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: Option<f32>,
}


impl From<&crate::types::GeoLocation> for GeoLocationView {
    fn from(loc: &crate::types::GeoLocation) -> Self {
        Self {
            latitude: loc.latitude,
            longitude: loc.longitude,
            elevation_m: loc.elevation_m,
        }
    }
}

// =============================================================================
// Artifacts API Response Types (WOR-31)
// =============================================================================

/// Response for artifacts list endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsResponse {
    pub world_id: String,
    pub artifacts: Vec<ArtifactView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl ArtifactsResponse {
    pub fn new(world_id: String, artifacts: Vec<ArtifactView>, total: usize, limit: usize, offset: usize) -> Self {
        Self { world_id, artifacts, total, limit, offset }
    }
}

/// Artifact summary view for list responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactView {
    pub id: String,
    pub name: String,
    pub category: String,
    pub era: Option<String>,
    pub created_year: i32,
    pub culture: Option<String>,
    pub description: String,
    pub significance: f64,
    pub condition: String,
}

// =============================================================================
// Cataclysms API Response Types (WOR-31)
// =============================================================================

/// Response for cataclysms list endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CataclysmsResponse {
    pub world_id: String,
    pub cataclysms: Vec<CataclysmView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl CataclysmsResponse {
    pub fn new(world_id: String, cataclysms: Vec<CataclysmView>, total: usize, limit: usize, offset: usize) -> Self {
        Self { world_id, cataclysms, total, limit, offset }
    }
}

/// Cataclysm summary view for list responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CataclysmView {
    pub id: String,
    pub name: String,
    pub cataclysm_type: String,
    pub year: i32,
    pub duration_years: Option<i32>,
    pub severity: f64,
    pub scope: String,
    pub description: String,
    pub significance: f64,
    pub population_lost: Option<u64>,
    pub cultures_destroyed: Option<Vec<String>>,
    pub cultures_emerged: Option<Vec<String>>,
}

// =============================================================================
// Simulation API Request/Response Types (WOR-1298)
// =============================================================================

/// Request body for POST /api/v1/worlds/:id/simulate
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateWorldRequest {
    /// Number of years to simulate (default: 100, max: 10000)
    #[serde(default)]
    pub years: Option<i32>,
    /// Starting year for simulation (default: 0)
    #[serde(default)]
    pub start_year: Option<i32>,
    /// Include generated events in response (default: true)
    #[serde(default = "default_true")]
    pub include_events: bool,
    /// Include generated figures in response (default: true)
    #[serde(default = "default_true")]
    pub include_figures: bool,
    /// Random seed for reproducible simulation (optional, auto-generated if not provided)
    #[serde(default)]
    pub seed: Option<u64>,
}


fn default_true() -> bool {
    true
}

/// Response for POST /api/v1/worlds/:id/simulate
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateWorldResponse {
    pub world_id: String,
    pub start_year: i32,
    pub end_year: i32,
    pub years_simulated: i32,
    pub seed: u64,
    /// Generated timeline events
    pub events: Vec<TimelineEventView>,
    /// Generated historical figures
    pub figures: Vec<HistoricalFigure>,
    /// Population changes over the simulation period
    pub population_changes: Vec<PopulationChangeView>,
    /// Summary statistics about the simulation
    pub stats: SimulationStats,
}

/// Statistics about a simulation run
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationStats {
    pub total_events: usize,
    pub population_events: usize,
    pub political_events: usize,
    pub natural_events: usize,
    pub figures_created: usize,
    pub settlement_events: usize,
}

/// Simplified population change view for simulation responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationChangeView {
    pub settlement_id: String,
    pub old_population: u64,
    pub new_population: u64,
    pub change_amount: i64,
    pub society_type: Option<String>,
    pub years_elapsed: i32,
}
