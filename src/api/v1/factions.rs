//! Factions API v1
//!
//! Provides endpoints for faction management and the faction turn system (Phase 5).

use axum::{
    routing::{get, post},
    Router,
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::models::*;
use crate::faction::{FactionRegistry, FactionTurnState, FactionAsset};

/// Query parameters for faction list
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FactionsQueryParams {
    /// Maximum number of results (default: 50, max: 200)
    #[serde(default = "default_factions_limit")]
    pub limit: usize,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<usize>,
    /// Filter by faction type
    #[serde(default)]
    pub faction_type: Option<String>,
    /// World ID filter (required)
    pub world_id: Option<String>,
}

fn default_factions_limit() -> usize {
    50
}

/// Registers faction routes under /api/v1/factions
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_factions))
        .route("/types", get(list_faction_types))
        .route("/:id", get(get_faction))
        .route("/:id/relations", get(get_faction_relations))
        .route("/:id/turn", get(get_faction_turn))
        .route("/:id/turn/advance", post(advance_faction_turn))
        .route("/:id/assets", post(add_faction_asset))
        .with_state(state)
}

/// GET /api/v1/factions - List factions for a world
async fn list_factions(
    State(state): State<crate::api::AppState>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionsListView>>, ApiError> {
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Collect all factions
    let mut factions: Vec<FactionSummaryView> = registry.factions()
        .map(FactionSummaryView::from)
        .collect();
    
    // Apply filters
    if let Some(ref ft) = params.faction_type {
        factions.retain(|f| f.faction_type == *ft);
    }
    
    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let total = factions.len();
    factions = factions.into_iter().skip(offset).take(params.limit).collect();
    
    let response = FactionsListView::new(factions).with_world_id(world_id.clone());
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/factions/types - List available faction types
async fn list_faction_types() -> Json<ApiResponse<Vec<FactionTypeView>>> {
    let types = vec![
        FactionTypeView::from_faction_type(crate::faction::FactionType::Clan),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Tribe),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Chiefdom),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Kingdom),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Empire),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Theocracy),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Republic),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Confederation),
        FactionTypeView::from_faction_type(crate::faction::FactionType::Nomadic),
    ];
    Json(ApiResponse::new(types))
}

/// GET /api/v1/factions/:id - Get a specific faction by ID
async fn get_faction(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionDetailView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;
    
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Find the faction
    let faction = registry.get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;
    
    Ok(Json(ApiResponse::new(FactionDetailView::from(faction))))
}

/// GET /api/v1/factions/:id/relations - Get faction diplomatic relations
async fn get_faction_relations(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<Vec<DiplomaticRelationView>>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;
    
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Find the faction
    let faction = registry.get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;
    
    // Build relations list
    let relations: Vec<DiplomaticRelationView> = faction.relations.iter()
        .map(|rel| {
            let target_name = registry.get(rel.target_id)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            DiplomaticRelationView {
                target_faction_id: rel.target_id.to_string(),
                target_faction_name: target_name,
                relation_type: format!("{:?}", rel.relation).to_lowercase(),
                started_year: rel.started_year.unwrap_or(rel.established_year.unwrap_or(0)),
                is_active: rel.is_active,
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(relations)))
}

/// GET /api/v1/factions/:id/turn - Get faction turn state
async fn get_faction_turn(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionTurnStateView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;
    
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Find the faction
    let faction = registry.get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;
    
    // Get turn state
    let turn_state = faction.turn_state.as_ref()
        .ok_or_else(|| ApiError::NotFound("Faction has no turn state".to_string()))?;
    
    Ok(Json(ApiResponse::new(FactionTurnStateView::from(turn_state))))
}

/// POST /api/v1/factions/:id/turn/advance - Advance faction turn
async fn advance_faction_turn(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<TurnAdvanceResponse>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;
    
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Clone for mutation
    let mut registry_clone = registry.clone();
    
    // Find and update faction
    let faction = registry_clone.get_mut(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;
    
    // Initialize turn state if needed
    if faction.turn_state.is_none() {
        faction.turn_state = Some(FactionTurnState::new(faction.founded_year.unwrap_or(0)));
    }
    
    // Advance turn
    let turn_state = faction.turn_state.as_mut().unwrap();
    let old_phase = format!("{:?}", turn_state.phase).to_lowercase();
    turn_state.end_turn();
    
    let response = TurnAdvanceResponse {
        old_phase,
        new_phase: format!("{:?}", turn_state.phase).to_lowercase(),
        turn_number: turn_state.turn_number,
        year: turn_state.year,
        resources_available: turn_state.resources,
        assets_count: turn_state.assets.len(),
        completed_goals: turn_state.check_goals().iter().map(|id| id.to_string()).collect(),
    };
    
    // Save updated registry
    drop(faction);
    state.save_faction_registry(world_id, registry_clone)
        .map_err(|e| ApiError::Internal(format!("Failed to save factions: {}", e)))?;
    
    Ok(Json(ApiResponse::new(response)))
}

/// Request body for adding a faction asset
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAssetRequest {
    pub category: String,
    pub hp: u32,
    pub location: Option<u32>,
}

/// POST /api/v1/factions/:id/assets - Add asset to faction
async fn add_faction_asset(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
    Json(request): Json<AddAssetRequest>,
) -> Result<Json<ApiResponse<FactionAssetView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;
    
    let world_id = params.world_id.as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;
    
    // Load faction registry
    let registry = state.get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;
    
    // Clone for mutation
    let mut registry_clone = registry.clone();
    
    // Find and update faction
    let faction = registry_clone.get_mut(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;
    
    // Parse asset category
    let category = match request.category.to_lowercase().as_str() {
        "force" => crate::faction::AssetCategory::Force,
        "cunning" => crate::faction::AssetCategory::Cunning,
        "wealth" => crate::faction::AssetCategory::Wealth,
        _ => return Err(ApiError::BadRequest("Invalid asset category".to_string())),
    };
    
    // Create asset
    let mut asset = FactionAsset::new(category, request.hp);
    asset.location = request.location;
    asset.purchased_year = faction.turn_state.as_ref().map(|t| t.year).unwrap_or(0);
    
    // Initialize turn state if needed
    if faction.turn_state.is_none() {
        faction.turn_state = Some(FactionTurnState::new(faction.founded_year.unwrap_or(0)));
    }
    faction.turn_state.as_mut().unwrap().add_asset(asset.clone());
    
    let response = FactionAssetView::from(&asset);
    
    // Save the updated registry
    state.save_faction_registry(world_id, registry_clone)
        .map_err(|e| ApiError::Internal(format!("Failed to save factions: {}", e)))?;
    
    Ok(Json(ApiResponse::new(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_type_list() {
        // Just verify the module compiles
        assert!(true);
    }
}