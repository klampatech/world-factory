//! Artifact resource routes
//!
//! Handles artifact retrieval and listing.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::models::*;
use crate::artifacts::{Artifact, ArtifactCategory};

/// Query parameters for GET /api/v1/worlds/{id}/artifacts
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetArtifactsParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_artifacts_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by category
    #[serde(default)]
    pub category: Option<String>,
    /// Filter by era
    #[serde(default)]
    pub era: Option<String>,
    /// Filter by minimum significance (0.0 - 1.0)
    #[serde(default)]
    pub min_significance: Option<f64>,
    /// Filter by creator/figure ID
    #[serde(default)]
    pub creator_id: Option<String>,
}

fn default_artifacts_limit() -> usize {
    50
}

/// Registers artifact routes under /api/v1/worlds/{id}/artifacts
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(get_artifacts))
        .route("/:artifact_id", get(get_artifact))
        .with_state(state)
}

/// GET /api/v1/worlds/{id}/artifacts - List artifacts for a world
async fn get_artifacts(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<GetArtifactsParams>,
) -> Result<Json<ApiResponse<ArtifactsResponse>>, ApiError> {
    uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw))
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    let world_id = crate::api::normalize_world_id(&world_id_raw);

    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);

    // TODO: Fetch from ArtifactStore
    // For now, return sample artifacts
    let sample_artifacts = vec![
        Artifact {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw)).unwrap_or_default(),
            name: "The Crown of Valdoria".to_string(),
            category: ArtifactCategory::CrownJewel,
            era: Some("Age of Kings".to_string()),
            created_year: 1250,
            creator_id: None,
            culture: Some("Valdorian".to_string()),
            current_location_id: None,
            owner_id: None,
            description: "The golden crown worn by the first King of Valdoria, forged from the gold of the Sunken Kingdom.".to_string(),
            significance: 0.85,
            rarity: crate::artifacts::ArtifactRarity::Legendary,
            condition: crate::artifacts::ArtifactCondition::Worn,
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: None,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Artifact {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw)).unwrap_or_default(),
            name: "Blade of the Fallen".to_string(),
            category: ArtifactCategory::Weapon,
            era: Some("Era of Strife".to_string()),
            created_year: 980,
            creator_id: None,
            culture: Some("Ironblood".to_string()),
            current_location_id: None,
            owner_id: None,
            description: "A legendary sword wielded by the warlord Korrath the Conqueror in his campaigns of unification.".to_string(),
            significance: 0.75,
            rarity: crate::artifacts::ArtifactRarity::Rare,
            condition: crate::artifacts::ArtifactCondition::Damaged,
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: Some(vec![
                crate::artifacts::ArtifactProperty {
                    name: "Bloodweave".to_string(),
                    description: "The blade carries the echoes of every life it has taken.".to_string(),
                    property_type: crate::artifacts::ArtifactPropertyType::Cursed,
                }
            ]),
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Artifact {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw)).unwrap_or_default(),
            name: "The Tome of Ages".to_string(),
            category: ArtifactCategory::Document,
            era: Some("Age of Enlightenment".to_string()),
            created_year: 1450,
            creator_id: None,
            culture: Some("Scholars of the Ivory Tower".to_string()),
            current_location_id: None,
            owner_id: None,
            description: "A comprehensive chronicle of world history from the First Age, compiled by the Scholars of the Ivory Tower.".to_string(),
            significance: 0.9,
            rarity: crate::artifacts::ArtifactRarity::Legendary,
            condition: crate::artifacts::ArtifactCondition::Worn,
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: None,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Artifact {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw)).unwrap_or_default(),
            name: "The Sacred Reliquary".to_string(),
            category: ArtifactCategory::Sacred,
            era: Some("Age of Faith".to_string()),
            created_year: 890,
            creator_id: None,
            culture: Some("Templar Order".to_string()),
            current_location_id: None,
            owner_id: None,
            description: "A holy relic believed to contain a fragment of the divine, housed in the Grand Cathedral of the Templar Order.".to_string(),
            significance: 0.95,
            rarity: crate::artifacts::ArtifactRarity::Mythic,
            condition: crate::artifacts::ArtifactCondition::Pristine,
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: Some(vec![
                crate::artifacts::ArtifactProperty {
                    name: "Divine Light".to_string(),
                    description: "The reliquary emanates a soft golden light that is said to heal the faithful.".to_string(),
                    property_type: crate::artifacts::ArtifactPropertyType::Healing,
                }
            ]),
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Artifact {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&crate::api::normalize_world_id(&world_id_raw)).unwrap_or_default(),
            name: "The Obsidian Obelisk".to_string(),
            category: ArtifactCategory::Monument,
            era: Some("Age of Shadow".to_string()),
            created_year: 2100,
            creator_id: None,
            culture: Some("Shadow Empire".to_string()),
            current_location_id: None,
            owner_id: None,
            description: "A towering obsidian obelisk inscribed with the names of the fallen, marking the boundary of the Shadow Empire's territory.".to_string(),
            significance: 0.7,
            rarity: crate::artifacts::ArtifactRarity::Rare,
            condition: crate::artifacts::ArtifactCondition::Ruined,
            origin_event_id: None,
            related_figures: None,
            related_events: None,
            properties: None,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
    ];

    // Filter by category if specified
    let filtered: Vec<ArtifactView> = sample_artifacts
        .into_iter()
        .filter(|a| {
            if let Some(ref cat) = params.category {
                let cat_lower = cat.to_lowercase();
                let matches = match a.category {
                    ArtifactCategory::Relic => cat_lower == "relic",
                    ArtifactCategory::Weapon => cat_lower == "weapon",
                    ArtifactCategory::Magical => cat_lower == "magical",
                    ArtifactCategory::Monument => cat_lower == "monument",
                    ArtifactCategory::Document => cat_lower == "document",
                    ArtifactCategory::Trophy => cat_lower == "trophy",
                    ArtifactCategory::CrownJewel => {
                        cat_lower == "crown_jewel" || cat_lower == "crownjewel"
                    }
                    ArtifactCategory::Sacred => cat_lower == "sacred",
                };
                if !matches {
                    return false;
                }
            }
            if let Some(min_sig) = params.min_significance {
                if a.significance < min_sig as f32 {
                    return false;
                }
            }
            true
        })
        .map(ArtifactView::from)
        .collect();

    let total = filtered.len();
    let artifacts: Vec<ArtifactView> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ApiResponse::new(ArtifactsResponse::new(
        world_id, artifacts, total, limit, offset,
    ))))
}

/// GET /api/v1/worlds/{id}/artifacts/:artifact_id - Get artifact details
async fn get_artifact(
    State(_state): State<crate::api::AppState>,
    Path((world_id_raw, artifact_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<ArtifactDetailView>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    uuid::Uuid::parse_str(&artifact_id)
        .map_err(|_| ApiError::BadRequest("Invalid artifact ID format".to_string()))?;

    // TODO: Fetch from ArtifactStore
    Err(ApiError::NotFound(format!(
        "Artifact '{}' not found",
        artifact_id
    )))
}

// =============================================================================
// API Response Types
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
    pub fn new(
        world_id: String,
        artifacts: Vec<ArtifactView>,
        total: usize,
        limit: usize,
        offset: usize,
    ) -> Self {
        Self {
            world_id,
            artifacts,
            total,
            limit,
            offset,
        }
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
    pub significance: f32,
    pub condition: String,
}

impl From<Artifact> for ArtifactView {
    fn from(artifact: Artifact) -> Self {
        Self {
            id: artifact.id.to_uuid().to_string(),
            name: artifact.name,
            category: format!("{:?}", artifact.category).to_lowercase(),
            era: artifact.era,
            created_year: artifact.created_year,
            culture: artifact.culture,
            description: artifact.description,
            significance: artifact.significance,
            condition: format!("{:?}", artifact.condition).to_lowercase(),
        }
    }
}

/// Detailed artifact view for single artifact response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDetailView {
    pub id: String,
    pub name: String,
    pub category: String,
    pub era: Option<String>,
    pub created_year: i32,
    pub creator_id: Option<String>,
    pub culture: Option<String>,
    pub current_location_id: Option<String>,
    pub owner_id: Option<String>,
    pub description: String,
    pub significance: f32,
    pub condition: String,
    pub origin_event_id: Option<String>,
    pub related_figures: Vec<String>,
    pub related_events: Vec<String>,
    pub properties: Vec<PropertyView>,
}

/// Property view for artifact properties
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyView {
    pub name: String,
    pub description: String,
    pub property_type: String,
}

impl From<&crate::artifacts::ArtifactProperty> for PropertyView {
    fn from(prop: &crate::artifacts::ArtifactProperty) -> Self {
        Self {
            name: prop.name.clone(),
            description: prop.description.clone(),
            property_type: format!("{:?}", prop.property_type).to_lowercase(),
        }
    }
}
