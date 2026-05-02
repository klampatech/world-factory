//! Cataclysm resource routes
//!
//! Handles cataclysm retrieval and listing.

use axum::{
    routing::get,
    Router,
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::api::models::*;
use crate::api::error::ApiError;
use crate::cataclysms::{Cataclysm, CataclysmType, CataclysmSeverity, CataclysmStore};

/// Query parameters for GET /api/v1/worlds/:id/cataclysms
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetCataclysmsParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_cataclysms_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by type
    #[serde(default)]
    pub cataclysm_type: Option<String>,
    /// Filter by scope (local, regional, continental, global)
    #[serde(default)]
    pub scope: Option<String>,
    /// Filter by minimum severity (0.0 - 1.0)
    #[serde(default)]
    pub min_severity: Option<f64>,
    /// Filter by region ID
    #[serde(default)]
    pub region_id: Option<String>,
    /// Start year filter
    #[serde(default)]
    pub start_year: Option<i32>,
    /// End year filter
    #[serde(default)]
    pub end_year: Option<i32>,
}

fn default_cataclysms_limit() -> usize {
    50
}

/// Registers cataclysm routes under /api/v1/worlds/:id/cataclysms
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(get_cataclysms))
        .route("/:cataclysm_id", get(get_cataclysm))
        .with_state(state)
}

/// GET /api/v1/worlds/:id/cataclysms - List cataclysms for a world
async fn get_cataclysms(
    State(_state): State<crate::api::AppState>,
    Path(world_id): Path<String>,
    Query(params): Query<GetCataclysmsParams>,
) -> Result<Json<ApiResponse<CataclysmsResponse>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    
    let limit = params.limit.min(200);
    let offset = params.offset.unwrap_or(0);
    
    // TODO: Fetch from CataclysmStore
    // For now, return sample cataclysms
    let sample_cataclysms = vec![
        Cataclysm {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&world_id).unwrap_or_default(),
            cataclysm_type: CataclysmType::GreatPlague,
            name: "The Crimson Death".to_string(),
            description: "A devastating plague swept across the continent, killing millions and reshaping civilizations.".to_string(),
            year: 1347,
            duration_years: Some(50),
            severity: 0.85,
            scope: CataclysmSeverity::Global,
            impacts: vec![
                crate::cataclysms::RegionImpact {
                    region_id: uuid::Uuid::new_v4(),
                    severity: 0.9,
                    recovery_state: crate::cataclysms::RecoveryState::Recovered,
                    start_year: 1347,
                    recovery_year: Some(1450),
                    population_loss_pct: Some(0.4),
                    cultural_damage: Some(0.2),
                    terrain_altered: false,
                    notes: None,
                }
            ],
            effects: vec![
                crate::cataclysms::CataclysmEffect {
                    description: "Massive population loss across all civilizations".to_string(),
                    magnitude: 0.9,
                    effect_type: crate::cataclysms::CataclysmEffectType::Population,
                },
                crate::cataclysms::CataclysmEffect {
                    description: "Economic collapse and famine follow the plague".to_string(),
                    magnitude: 0.7,
                    effect_type: crate::cataclysms::CataclysmEffectType::Economic,
                },
            ],
            survivors: None,
            total_population_lost: Some(15000000),
            cultures_destroyed: Some(vec!["Old Valorian Empire".to_string(), "Coastal Trading League".to_string()]),
            cultures_emerged: Some(vec!["New Order of Healers".to_string(), "Plague Survivors' Guild".to_string()]),
            related_events: None,
            significance: 0.95,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Cataclysm {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&world_id).unwrap_or_default(),
            cataclysm_type: CataclysmType::GreatQuake,
            name: "The Shattering".to_string(),
            description: "A massive earthquake split the continent, creating the Great Rift and reshaping the coastline.".to_string(),
            year: 890,
            duration_years: Some(10),
            severity: 0.8,
            scope: CataclysmSeverity::Continental,
            impacts: vec![
                crate::cataclysms::RegionImpact {
                    region_id: uuid::Uuid::new_v4(),
                    severity: 0.95,
                    recovery_state: crate::cataclysms::RecoveryState::Scarring,
                    start_year: 890,
                    recovery_year: None,
                    population_loss_pct: Some(0.25),
                    cultural_damage: Some(0.5),
                    terrain_altered: true,
                    notes: Some("The Great Rift remains to this day".to_string()),
                }
            ],
            effects: vec![
                crate::cataclysms::CataclysmEffect {
                    description: "The continent was physically split, creating new mountain ranges".to_string(),
                    magnitude: 1.0,
                    effect_type: crate::cataclysms::CataclysmEffectType::Terrain,
                },
            ],
            survivors: None,
            total_population_lost: Some(5000000),
            cultures_destroyed: Some(vec!["Valorian Empire".to_string()]),
            cultures_emerged: Some(vec!["Rift Dwarves".to_string()]),
            related_events: None,
            significance: 0.9,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
        Cataclysm {
            id: uuid::Uuid::new_v4().into(),
            world_id: uuid::Uuid::parse_str(&world_id).unwrap_or_default(),
            cataclysm_type: CataclysmType::GreatMigration,
            name: "The Long Walk".to_string(),
            description: "The horsemen of the eastern steppes migrated westward, overwhelming the old kingdoms and establishing a new order.".to_string(),
            year: 450,
            duration_years: Some(100),
            severity: 0.7,
            scope: CataclysmSeverity::Continental,
            impacts: vec![
                crate::cataclysms::RegionImpact {
                    region_id: uuid::Uuid::new_v4(),
                    severity: 0.8,
                    recovery_state: crate::cataclysms::RecoveryState::Recovered,
                    start_year: 450,
                    recovery_year: Some(700),
                    population_loss_pct: Some(0.15),
                    cultural_damage: Some(0.3),
                    terrain_altered: false,
                    notes: None,
                }
            ],
            effects: vec![
                crate::cataclysms::CataclysmEffect {
                    description: "Multiple kingdoms were conquered and absorbed".to_string(),
                    magnitude: 0.8,
                    effect_type: crate::cataclysms::CataclysmEffectType::Political,
                },
                crate::cataclysms::CataclysmEffect {
                    description: "Cultural fusion created new hybrid civilizations".to_string(),
                    magnitude: 0.6,
                    effect_type: crate::cataclysms::CataclysmEffectType::Cultural,
                },
            ],
            survivors: None,
            total_population_lost: Some(3000000),
            cultures_destroyed: None,
            cultures_emerged: Some(vec!["The Horde Kingdom".to_string(), "Steppe Alliance".to_string()]),
            related_events: None,
            significance: 0.75,
            created_at: crate::types::Timestamp::now(),
            updated_at: crate::types::Timestamp::now(),
        },
    ];
    
    // Filter cataclysms
    let filtered: Vec<CataclysmView> = sample_cataclysms
        .into_iter()
        .filter(|c| {
            if let Some(ref cat_type) = params.cataclysm_type {
                let cat_lower = cat_type.to_lowercase();
                let matches = match c.cataclysm_type {
                    CataclysmType::VolcanicEruption => cat_lower.contains("volcan"),
                    CataclysmType::MeteorStrike => cat_lower.contains("meteor") || cat_lower.contains("impact"),
                    CataclysmType::GreatQuake => cat_lower.contains("quake") || cat_lower.contains("earthquake"),
                    CataclysmType::GreatFlood => cat_lower.contains("flood"),
                    CataclysmType::Megadrought => cat_lower.contains("drought"),
                    CataclysmType::GreatPlague => cat_lower.contains("plague") || cat_lower.contains("disease"),
                    CataclysmType::IceAge => cat_lower.contains("ice") || cat_lower.contains("glacial"),
                    CataclysmType::MagicalCataclysm => cat_lower.contains("magical"),
                    CataclysmType::DivineWrath => cat_lower.contains("divine") || cat_lower.contains("god"),
                    CataclysmType::PlanarInvasion => cat_lower.contains("planar") || cat_lower.contains("invasion"),
                    CataclysmType::CivilizationalCollapse => cat_lower.contains("collapse") || cat_lower.contains("civilization"),
                    CataclysmType::GreatMigration => cat_lower.contains("migration") || cat_lower.contains("horde"),
                    CataclysmType::Blight => cat_lower.contains("blight") || cat_lower.contains("poison"),
                };
                if !matches { return false; }
            }
            if let Some(ref sc) = params.scope {
                let scope_lower = sc.to_lowercase();
                let matches = match c.scope {
                    CataclysmSeverity::Local => scope_lower == "local",
                    CataclysmSeverity::Regional => scope_lower == "regional",
                    CataclysmSeverity::Continental => scope_lower == "continental",
                    CataclysmSeverity::Global => scope_lower == "global",
                };
                if !matches { return false; }
            }
            if let Some(min_sev) = params.min_severity {
                if c.severity < min_sev as f32 { return false; }
            }
            if let Some(start) = params.start_year {
                if c.year < start { return false; }
            }
            if let Some(end) = params.end_year {
                if c.year > end { return false; }
            }
            true
        })
        .map(CataclysmView::from)
        .collect();
    
    let total = filtered.len();
    let cataclysms: Vec<CataclysmView> = filtered.into_iter().skip(offset).take(limit).collect();
    
    Ok(Json(ApiResponse::new(CataclysmsResponse::new(
        world_id,
        cataclysms,
        total,
        limit,
        offset,
    ))))
}

/// GET /api/v1/worlds/:id/cataclysms/:cataclysm_id - Get cataclysm details
async fn get_cataclysm(
    State(_state): State<crate::api::AppState>,
    Path((world_id, cataclysm_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<CataclysmDetailView>>, ApiError> {
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    uuid::Uuid::parse_str(&cataclysm_id)
        .map_err(|_| ApiError::BadRequest("Invalid cataclysm ID format".to_string()))?;
    
    // TODO: Fetch from CataclysmStore
    Err(ApiError::NotFound(format!("Cataclysm '{}' not found", cataclysm_id)))
}

// =============================================================================
// API Response Types
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
    pub severity: f32,
    pub scope: String,
    pub description: String,
    pub significance: f32,
    pub population_lost: Option<u64>,
    pub cultures_destroyed: Option<Vec<String>>,
    pub cultures_emerged: Option<Vec<String>>,
}

impl From<Cataclysm> for CataclysmView {
    fn from(cataclysm: Cataclysm) -> Self {
        Self {
            id: cataclysm.id.to_uuid().to_string(),
            name: cataclysm.name,
            cataclysm_type: format!("{:?}", cataclysm.cataclysm_type).to_lowercase(),
            year: cataclysm.year,
            duration_years: cataclysm.duration_years,
            severity: cataclysm.severity,
            scope: format!("{:?}", cataclysm.scope).to_lowercase(),
            description: cataclysm.description,
            significance: cataclysm.significance,
            population_lost: cataclysm.total_population_lost,
            cultures_destroyed: cataclysm.cultures_destroyed,
            cultures_emerged: cataclysm.cultures_emerged,
        }
    }
}

/// Detailed cataclysm view for single cataclysm response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CataclysmDetailView {
    pub id: String,
    pub name: String,
    pub cataclysm_type: String,
    pub year: i32,
    pub duration_years: Option<i32>,
    pub severity: f32,
    pub scope: String,
    pub description: String,
    pub significance: f32,
    pub impacts: Vec<ImpactView>,
    pub effects: Vec<EffectView>,
    pub total_population_lost: Option<u64>,
    pub cultures_destroyed: Option<Vec<String>>,
    pub cultures_emerged: Option<Vec<String>>,
    pub related_events: Vec<String>,
}

/// Regional impact view
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactView {
    pub region_id: String,
    pub severity: f32,
    pub recovery_state: String,
    pub start_year: i32,
    pub recovery_year: Option<i32>,
    pub population_loss_pct: Option<f32>,
    pub cultural_damage: Option<f32>,
    pub terrain_altered: bool,
    pub notes: Option<String>,
}

impl From<&crate::cataclysms::RegionImpact> for ImpactView {
    fn from(impact: &crate::cataclysms::RegionImpact) -> Self {
        Self {
            region_id: impact.region_id.to_string(),
            severity: impact.severity,
            recovery_state: format!("{:?}", impact.recovery_state).to_lowercase(),
            start_year: impact.start_year,
            recovery_year: impact.recovery_year,
            population_loss_pct: impact.population_loss_pct,
            cultural_damage: impact.cultural_damage,
            terrain_altered: impact.terrain_altered,
            notes: impact.notes.clone(),
        }
    }
}

/// Cataclysm effect view
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectView {
    pub description: String,
    pub magnitude: f32,
    pub effect_type: String,
}

impl From<&crate::cataclysms::CataclysmEffect> for EffectView {
    fn from(effect: &crate::cataclysms::CataclysmEffect) -> Self {
        Self {
            description: effect.description.clone(),
            magnitude: effect.magnitude,
            effect_type: format!("{:?}", effect.effect_type).to_lowercase(),
        }
    }
}