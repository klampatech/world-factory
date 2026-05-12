//! Biomes API Routes
//!
//! Endpoints for retrieving all available biome types and their properties.
//! Used for terrain analysis and species habitat matching.

use axum::{
    response::Json,
    routing::get,
    Router,
};

use crate::api::AppState;
use crate::api::error::ApiError;
use crate::api::models::*;
use crate::terrain::biome::BiomeType;

// =============================================================================
// Response Types
// =============================================================================

/// Response for biomes list endpoint
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiomesListResponse {
    pub biomes: Vec<BiomeTypeView>,
    pub total: usize,
}

/// Individual biome type view
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BiomeTypeView {
    pub id: u16,
    pub name: String,
    pub category: String,
    pub color_rgb: [u8; 3],
    pub temperature_range: TemperatureRange,
    pub precipitation_range: PrecipitationRange,
    pub vegetation_type: Option<String>,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemperatureRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrecipitationRange {
    pub min: f32,
    pub max: f32,
}

/// Registers biomes routes under /api/v1/biomes
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_biomes))
        .route("/{id}", get(get_biome))
        .with_state(state)
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/biomes - List all biome types
async fn list_biomes(
    State(_state): axum::extract::State<AppState>,
) -> Result<Json<ApiResponse<BiomesListResponse>>, ApiError> {
    let biomes = get_all_biome_views();
    let response = BiomesListResponse {
        total: biomes.len(),
        biomes,
    };
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/biomes/{id} - Get a specific biome type by ID
async fn get_biome(
    State(_state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u16>,
) -> Result<Json<ApiResponse<BiomeTypeView>>, ApiError> {
    let biomes = get_all_biome_views();
    biomes
        .iter()
        .find(|b| b.id == id)
        .map(|b| Json(ApiResponse::new(b.clone())))
        .ok_or_else(|| ApiError::NotFound(format!("Biome with id '{}' not found", id)))
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Static list of all biome types supported by the API
const ALL_BIOME_TYPES: &[BiomeType] = &[
    // Tropical (latitude 0-23°)
    BiomeType::TropicalRainforest,
    BiomeType::TropicalSeasonalForest,
    BiomeType::TropicalSavanna,
    BiomeType::TropicalDryForest,
    // Subtropical (latitude 23-35°)
    BiomeType::SubtropicalRainforest,
    BiomeType::SubtropicalSeasonalForest,
    BiomeType::SubtropicalSteppe,
    BiomeType::SubtropicalDesert,
    // Temperate (latitude 35-55°)
    BiomeType::TemperateRainforest,
    BiomeType::TemperateDeciduousForest,
    BiomeType::TemperateMixedForest,
    BiomeType::TemperateSteppe,
    BiomeType::TemperateDesert,
    // Continental (latitude 55-65°)
    BiomeType::BorealTaiga,
    BiomeType::BorealForest,
    BiomeType::TemperateGrassland,
    // Polar (latitude 65-90°)
    BiomeType::Tundra,
    BiomeType::Arctic,
    BiomeType::PolarDesert,
    // Mountain
    BiomeType::MontaneForest,
    BiomeType::MontaneGrassland,
    BiomeType::AlpineTundra,
    BiomeType::SnowGlacier,
    // Coastal
    BiomeType::CoastalWetland,
    BiomeType::Mangrove,
    BiomeType::CoralReef,
    BiomeType::KelpForest,
    // Ocean
    BiomeType::OpenOcean,
    // Arid
    BiomeType::HotDesert,
    BiomeType::ColdDesert,
    BiomeType::SemiAridSteppe,
    // Fantasy
    BiomeType::MagicalForest,
    BiomeType::CrystallineDesert,
    BiomeType::BioluminescentOcean,
    BiomeType::VolcanicLandscape,
    BiomeType::ToxicSwamp,
    BiomeType::FloatingIslands,
];

/// Get all biome type views for API response
fn get_all_biome_views() -> Vec<BiomeTypeView> {
    use crate::terrain::biome::BiomeColorMapping;
    
    ALL_BIOME_TYPES
        .iter()
        .map(|biome| {
            let color = BiomeColorMapping::get_color(*biome);
            let (category, temp_range, precip_range, vegetation) = get_biome_properties(biome);
            
            BiomeTypeView {
                id: *biome as u16,
                name: biome.name().to_string(),
                category,
                color_rgb: [color.0, color.1, color.2],
                temperature_range: temp_range,
                precipitation_range: precip_range,
                vegetation_type: vegetation,
            }
        })
        .collect()
}

/// Convert a BiomeType to a view for API responses
fn get_biome_properties(biome: &BiomeType) -> (String, TemperatureRange, PrecipitationRange, Option<String>) {
    match biome {
        BiomeType::TropicalRainforest => (
            "tropical".to_string(),
            TemperatureRange { min: 20.0, max: 30.0 },
            PrecipitationRange { min: 2000.0, max: 4000.0 },
            Some("DenseForest".to_string()),
        ),
        BiomeType::TropicalSeasonalForest => (
            "tropical".to_string(),
            TemperatureRange { min: 18.0, max: 28.0 },
            PrecipitationRange { min: 1000.0, max: 2000.0 },
            Some("Forest".to_string()),
        ),
        BiomeType::TropicalSavanna => (
            "tropical".to_string(),
            TemperatureRange { min: 20.0, max: 30.0 },
            PrecipitationRange { min: 500.0, max: 1500.0 },
            Some("Grass".to_string()),
        ),
        BiomeType::TropicalDryForest => (
            "tropical".to_string(),
            TemperatureRange { min: 20.0, max: 35.0 },
            PrecipitationRange { min: 250.0, max: 1000.0 },
            Some("OpenForest".to_string()),
        ),
        BiomeType::SubtropicalRainforest => (
            "subtropical".to_string(),
            TemperatureRange { min: 15.0, max: 25.0 },
            PrecipitationRange { min: 1500.0, max: 3000.0 },
            Some("DenseForest".to_string()),
        ),
        BiomeType::SubtropicalSeasonalForest => (
            "subtropical".to_string(),
            TemperatureRange { min: 12.0, max: 25.0 },
            PrecipitationRange { min: 800.0, max: 1500.0 },
            Some("Forest".to_string()),
        ),
        BiomeType::SubtropicalSteppe => (
            "subtropical".to_string(),
            TemperatureRange { min: 10.0, max: 25.0 },
            PrecipitationRange { min: 250.0, max: 500.0 },
            Some("Grass".to_string()),
        ),
        BiomeType::SubtropicalDesert => (
            "subtropical".to_string(),
            TemperatureRange { min: 20.0, max: 40.0 },
            PrecipitationRange { min: 0.0, max: 250.0 },
            Some("Desert".to_string()),
        ),
        BiomeType::TemperateRainforest => (
            "temperate".to_string(),
            TemperatureRange { min: 5.0, max: 15.0 },
            PrecipitationRange { min: 1500.0, max: 3000.0 },
            Some("DenseForest".to_string()),
        ),
        BiomeType::TemperateDeciduousForest => (
            "temperate".to_string(),
            TemperatureRange { min: 0.0, max: 20.0 },
            PrecipitationRange { min: 750.0, max: 1500.0 },
            Some("Forest".to_string()),
        ),
        BiomeType::TemperateMixedForest => (
            "temperate".to_string(),
            TemperatureRange { min: -5.0, max: 20.0 },
            PrecipitationRange { min: 500.0, max: 1200.0 },
            Some("MixedForest".to_string()),
        ),
        BiomeType::TemperateSteppe => (
            "temperate".to_string(),
            TemperatureRange { min: -5.0, max: 25.0 },
            PrecipitationRange { min: 250.0, max: 500.0 },
            Some("Grass".to_string()),
        ),
        BiomeType::TemperateDesert => (
            "temperate".to_string(),
            TemperatureRange { min: 0.0, max: 35.0 },
            PrecipitationRange { min: 0.0, max: 250.0 },
            Some("Desert".to_string()),
        ),
        BiomeType::BorealTaiga => (
            "continental".to_string(),
            TemperatureRange { min: -30.0, max: 10.0 },
            PrecipitationRange { min: 200.0, max: 600.0 },
            Some("Taiga".to_string()),
        ),
        BiomeType::BorealForest => (
            "continental".to_string(),
            TemperatureRange { min: -25.0, max: 15.0 },
            PrecipitationRange { min: 300.0, max: 900.0 },
            Some("ConiferousForest".to_string()),
        ),
        BiomeType::TemperateGrassland => (
            "continental".to_string(),
            TemperatureRange { min: -10.0, max: 25.0 },
            PrecipitationRange { min: 250.0, max: 750.0 },
            Some("Grass".to_string()),
        ),
        BiomeType::Tundra => (
            "polar".to_string(),
            TemperatureRange { min: -25.0, max: 5.0 },
            PrecipitationRange { min: 100.0, max: 400.0 },
            Some("Tundra".to_string()),
        ),
        BiomeType::Arctic => (
            "polar".to_string(),
            TemperatureRange { min: -50.0, max: 0.0 },
            PrecipitationRange { min: 50.0, max: 200.0 },
            Some("Ice".to_string()),
        ),
        BiomeType::PolarDesert => (
            "polar".to_string(),
            TemperatureRange { min: -60.0, max: -10.0 },
            PrecipitationRange { min: 0.0, max: 100.0 },
            Some("Desert".to_string()),
        ),
        BiomeType::MontaneForest => (
            "mountain".to_string(),
            TemperatureRange { min: -15.0, max: 15.0 },
            PrecipitationRange { min: 500.0, max: 2000.0 },
            Some("MountainForest".to_string()),
        ),
        BiomeType::MontaneGrassland => (
            "mountain".to_string(),
            TemperatureRange { min: -10.0, max: 15.0 },
            PrecipitationRange { min: 300.0, max: 1000.0 },
            Some("AlpineGrass".to_string()),
        ),
        BiomeType::AlpineTundra => (
            "mountain".to_string(),
            TemperatureRange { min: -20.0, max: 5.0 },
            PrecipitationRange { min: 200.0, max: 600.0 },
            Some("Tundra".to_string()),
        ),
        BiomeType::SnowGlacier => (
            "mountain".to_string(),
            TemperatureRange { min: -40.0, max: 0.0 },
            PrecipitationRange { min: 0.0, max: 500.0 },
            Some("Ice".to_string()),
        ),
        BiomeType::CoastalWetland => (
            "coastal".to_string(),
            TemperatureRange { min: -5.0, max: 30.0 },
            PrecipitationRange { min: 500.0, max: 3000.0 },
            Some("Wetland".to_string()),
        ),
        BiomeType::Mangrove => (
            "coastal".to_string(),
            TemperatureRange { min: 15.0, max: 35.0 },
            PrecipitationRange { min: 1000.0, max: 4000.0 },
            Some("Mangrove".to_string()),
        ),
        BiomeType::CoralReef => (
            "coastal".to_string(),
            TemperatureRange { min: 18.0, max: 30.0 },
            PrecipitationRange { min: 0.0, max: 0.0 },
            Some("CoralReef".to_string()),
        ),
        BiomeType::KelpForest => (
            "coastal".to_string(),
            TemperatureRange { min: 0.0, max: 20.0 },
            PrecipitationRange { min: 0.0, max: 0.0 },
            Some("KelpForest".to_string()),
        ),
        BiomeType::OpenOcean => (
            "ocean".to_string(),
            TemperatureRange { min: -2.0, max: 25.0 },
            PrecipitationRange { min: 0.0, max: 0.0 },
            Some("OpenWater".to_string()),
        ),
        BiomeType::HotDesert => (
            "arid".to_string(),
            TemperatureRange { min: 25.0, max: 50.0 },
            PrecipitationRange { min: 0.0, max: 200.0 },
            Some("Desert".to_string()),
        ),
        BiomeType::ColdDesert => (
            "arid".to_string(),
            TemperatureRange { min: -30.0, max: 15.0 },
            PrecipitationRange { min: 0.0, max: 250.0 },
            Some("Desert".to_string()),
        ),
        BiomeType::SemiAridSteppe => (
            "arid".to_string(),
            TemperatureRange { min: -5.0, max: 30.0 },
            PrecipitationRange { min: 200.0, max: 500.0 },
            Some("Steppe".to_string()),
        ),
        BiomeType::MagicalForest => (
            "fantasy".to_string(),
            TemperatureRange { min: 5.0, max: 25.0 },
            PrecipitationRange { min: 800.0, max: 2000.0 },
            Some("MagicalForest".to_string()),
        ),
        BiomeType::CrystallineDesert => (
            "fantasy".to_string(),
            TemperatureRange { min: 0.0, max: 40.0 },
            PrecipitationRange { min: 0.0, max: 100.0 },
            Some("Crystalline".to_string()),
        ),
        BiomeType::BioluminescentOcean => (
            "fantasy".to_string(),
            TemperatureRange { min: 5.0, max: 25.0 },
            PrecipitationRange { min: 0.0, max: 0.0 },
            Some("Bioluminescent".to_string()),
        ),
        BiomeType::VolcanicLandscape => (
            "fantasy".to_string(),
            TemperatureRange { min: 30.0, max: 100.0 },
            PrecipitationRange { min: 0.0, max: 500.0 },
            Some("Volcanic".to_string()),
        ),
        BiomeType::ToxicSwamp => (
            "fantasy".to_string(),
            TemperatureRange { min: 10.0, max: 35.0 },
            PrecipitationRange { min: 1000.0, max: 4000.0 },
            Some("ToxicWetland".to_string()),
        ),
        BiomeType::FloatingIslands => (
            "fantasy".to_string(),
            TemperatureRange { min: -20.0, max: 30.0 },
            PrecipitationRange { min: 500.0, max: 2500.0 },
            Some("Floating".to_string()),
        ),
    }
}
