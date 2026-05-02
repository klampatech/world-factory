//! Species API Routes
//! 
//! Endpoints for retrieving species definitions and details.
//! Species data is used for settlement generation and civilization simulation.

use axum::{
    routing::get,
    Router,
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::api::models::*;
use crate::api::error::ApiError;
use crate::species::{SpeciesId, SpeciesData, SpeciesTrait, ClimateTolerance};

/// Registers species routes under /api/v1/species
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_species))
        .route("/:id", get(get_species))
        .with_state(state)
}

// =============================================================================
// Query Parameters
// =============================================================================

/// Query parameters for GET /api/v1/species
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListSpeciesParams {
    /// Filter by habitat biome type (e.g., "TemperateGrassland", "BorealForest")
    #[serde(default)]
    pub habitat: Option<String>,
    /// Filter by trait (e.g., "WarLike", "Peaceful", "Adaptable")
    #[serde(default)]
    pub trait_filter: Option<String>,
}

/// Query parameters for GET /api/v1/species/:id
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetSpeciesParams {
    /// Include name templates in response (default: true)
    #[serde(default = "default_true")]
    pub include_templates: bool,
}

fn default_true() -> bool {
    true
}

// =============================================================================
// Response Types
// =============================================================================

/// Response for species list endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesListResponse {
    pub species: Vec<SpeciesSummary>,
    pub total: usize,
}

impl SpeciesListResponse {
    pub fn new(species: Vec<SpeciesSummary>) -> Self {
        let total = species.len();
        Self { species, total }
    }
}

/// Species summary for list responses
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesSummary {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub home_biomes: Vec<String>,
    pub tolerable_biomes: Vec<String>,
    pub traits: Vec<String>,
    pub climate_tolerance: ClimateToleranceView,
}

impl From<&crate::species::Species> for SpeciesSummary {
    fn from(species: &crate::species::Species) -> Self {
        Self {
            id: species.id.0.to_string(),
            name: species.name.as_ref().to_string(),
            display_name: species.display_name.as_ref().to_string(),
            home_biomes: species.home_biomes.iter().map(|b| format!("{:?}", b)).collect(),
            tolerable_biomes: species.tolerable_biomes.iter().map(|b| format!("{:?}", b)).collect(),
            traits: species.traits.iter().map(|t| format!("{:?}", t)).collect(),
            climate_tolerance: ClimateToleranceView {
                min_temp: species.climate_tolerance.min_temp,
                max_temp: species.climate_tolerance.max_temp,
                min_precipitation: species.climate_tolerance.min_precipitation,
                max_precipitation: species.climate_tolerance.max_precipitation,
            },
        }
    }
}

/// Climate tolerance view
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClimateToleranceView {
    pub min_temp: f32,
    pub max_temp: f32,
    pub min_precipitation: f32,
    pub max_precipitation: f32,
}

/// Species detail response with full information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesDetailResponse {
    pub species: SpeciesDetail,
}

impl SpeciesDetailResponse {
    pub fn new(species: SpeciesDetail) -> Self {
        Self { species }
    }
}

/// Detailed species information
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesDetail {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub home_biomes: Vec<String>,
    pub tolerable_biomes: Vec<String>,
    pub traits: Vec<SpeciesTraitDetail>,
    pub climate_tolerance: ClimateToleranceView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_templates: Option<NameTemplatesView>,
}

/// Detailed trait information with effects
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesTraitDetail {
    pub name: String,
    pub effect: String,
}

impl From<SpeciesTrait> for SpeciesTraitDetail {
    fn from(trait_: SpeciesTrait) -> Self {
        let (name, effect) = match trait_ {
            SpeciesTrait::Aquatic => ("Aquatic".to_string(), "Can inhabit water biomes, excellent swimmers".to_string()),
            SpeciesTrait::Flying => ("Flying".to_string(), "Can traverse difficult terrain, access high locations".to_string()),
            SpeciesTrait::Subterranean => ("Subterranean".to_string(), "Prefers underground or mountain habitats".to_string()),
            SpeciesTrait::Nocturnal => ("Nocturnal".to_string(), "Active during night hours, avoiding daytime competition".to_string()),
            SpeciesTrait::PackHunter => ("PackHunter".to_string(), "Benefits from group coordination in conflicts".to_string()),
            SpeciesTrait::Nomadic => ("Nomadic".to_string(), "Does not settle permanently, moves with resources".to_string()),
            SpeciesTrait::Sedentary => ("Sedentary".to_string(), "Builds permanent settlements, stable population growth".to_string()),
            SpeciesTrait::TradeFocused => ("TradeFocused".to_string(), "Excels at commerce, gains economic bonuses".to_string()),
            SpeciesTrait::WarLike => ("WarLike".to_string(), "Strong military traditions, combat bonuses".to_string()),
            SpeciesTrait::Peaceful => ("Peaceful".to_string(), "Avoids conflict, slower to respond to aggression".to_string()),
            SpeciesTrait::Adaptable => ("Adaptable".to_string(), "+25% bonus to tolerable biome suitability".to_string()),
            SpeciesTrait::Curious => ("Curious".to_string(), "+25% innovation rate, faster discovery".to_string()),
        };
        Self { name, effect }
    }
}

/// Name templates for species
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NameTemplatesView {
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compound_patterns: Option<Vec<String>>,
}

impl From<&crate::species::NameTemplate> for NameTemplatesView {
    fn from(template: &crate::species::NameTemplate) -> Self {
        Self {
            prefixes: template.prefixes.as_ref().clone(),
            suffixes: template.suffixes.as_ref().clone(),
            compound_patterns: if template.compound_patterns.is_empty() {
                None
            } else {
                Some(template.compound_patterns.as_ref().clone())
            },
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/species - List all available species
async fn list_species(
    State(_state): State<crate::api::AppState>,
    Query(params): Query<ListSpeciesParams>,
) -> Result<Json<ApiResponse<SpeciesListResponse>>, ApiError> {
    let species_data = SpeciesData::default_species();
    
    // Start with all species
    let mut species_list: Vec<SpeciesSummary> = species_data.species
        .iter()
        .map(SpeciesSummary::from)
        .collect();
    
    // Apply habitat filter if specified
    if let Some(ref habitat) = params.habitat {
        species_list.retain(|s| {
            s.home_biomes.iter().any(|b| b.eq_ignore_ascii_case(habitat))
                || s.tolerable_biomes.iter().any(|b| b.eq_ignore_ascii_case(habitat))
        });
    }
    
    // Apply trait filter if specified
    if let Some(ref trait_filter) = params.trait_filter {
        species_list.retain(|s| {
            s.traits.iter().any(|t| t.eq_ignore_ascii_case(trait_filter))
        });
    }
    
    let response = SpeciesListResponse::new(species_list);
    
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/species/:id - Get species details by ID
async fn get_species(
    State(_state): State<crate::api::AppState>,
    Path(id): Path<String>,
    Query(params): Query<GetSpeciesParams>,
) -> Result<Json<ApiResponse<SpeciesDetailResponse>>, ApiError> {
    let species_data = SpeciesData::default_species();
    
    // Parse species ID (supports numeric IDs: 1-5 for default species)
    let species_id = id.parse::<u32>()
        .map_err(|_| ApiError::BadRequest(format!("Invalid species ID format: '{}'", id)))?;
    
    // Check for default species (IDs 1-5) or custom species
    let species = match species_id {
        1 => species_data.get(SpeciesId::HUMAN),
        2 => species_data.get(SpeciesId::ELF),
        3 => species_data.get(SpeciesId::DWARF),
        4 => species_data.get(SpeciesId::ORC),
        5 => species_data.get(SpeciesId::HALFLING),
        _ => species_data.get(SpeciesId(species_id)),
    };
    
    let species = species.ok_or_else(|| {
        ApiError::NotFound(format!("Species with ID '{}' not found", id))
    })?;
    
    // Build name templates if requested
    let name_templates = if params.include_templates {
        species_data.name_templates.get(&species.id)
            .map(NameTemplatesView::from)
    } else {
        None
    };
    
    let detail = SpeciesDetail {
        id: species.id.0.to_string(),
        name: species.name.as_ref().to_string(),
        display_name: species.display_name.as_ref().to_string(),
        home_biomes: species.home_biomes.iter().map(|b| format!("{:?}", b)).collect(),
        tolerable_biomes: species.tolerable_biomes.iter().map(|b| format!("{:?}", b)).collect(),
        traits: species.traits.iter().map(|t| SpeciesTraitDetail::from(*t)).collect(),
        climate_tolerance: ClimateToleranceView {
            min_temp: species.climate_tolerance.min_temp,
            max_temp: species.climate_tolerance.max_temp,
            min_precipitation: species.climate_tolerance.min_precipitation,
            max_precipitation: species.climate_tolerance.max_precipitation,
        },
        name_templates,
    };
    
    Ok(Json(ApiResponse::new(SpeciesDetailResponse::new(detail))))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_list_species_returns_all() {
        let app = crate::api::create_router();
        
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/species")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_get_species_by_id() {
        let app = crate::api::create_router();
        
        // Test Human (ID 1)
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/species/1")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        
        // Test invalid ID
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/species/999")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
    
    #[tokio::test]
    async fn test_filter_species_by_trait() {
        let app = crate::api::create_router();
        
        // Filter by WarLike trait
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/species?trait=WarLike")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
