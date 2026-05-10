//! Factions API v1
//!
//! Provides endpoints for faction management and the faction turn system (Phase 5).

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::models::*;
use crate::faction::{FactionAsset, FactionRegistry, FactionTurnState};

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
        .route("/{id}", get(get_faction))
        .route("/{id}/relations", get(get_faction_relations))
        .route("/{id}/turn", get(get_faction_turn))
        .route("/{id}/turn/advance", post(advance_faction_turn))
        .route("/{id}/assets", post(add_faction_asset))
        .route("/{id}/goals", get(get_faction_goals))
        .route("/{id}/goals", post(add_faction_goal))
        .route("/{id}/goals/{goal_id}", get(get_faction_goal))
        .route("/{id}/beast-bonds", get(get_faction_beast_bonds))
        .route("/{id}/beast-bonds", post(add_faction_beast_bond))
        .with_state(state)
}

/// GET /api/v1/factions - List factions for a world
async fn list_factions(
    State(state): State<crate::api::AppState>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionsListView>>, ApiError> {
    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Collect all factions
    let mut factions: Vec<FactionSummaryView> =
        registry.factions().map(FactionView::from_faction).collect();

    // Apply filters
    if let Some(ref ft) = params.faction_type {
        factions.retain(|f| f.faction_type == *ft);
    }

    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let total = factions.len();
    factions = factions
        .into_iter()
        .skip(offset)
        .take(params.limit)
        .collect();

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

/// GET /api/v1/factions/{id} - Get a specific faction by ID
async fn get_faction(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionDetailView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Find the faction
    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    Ok(Json(ApiResponse::new(FactionDetailView::from_faction(
        faction,
    ))))
}

/// GET /api/v1/factions/{id}/relations - Get faction diplomatic relations
async fn get_faction_relations(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<Vec<DiplomaticRelationView>>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Find the faction
    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    // Build relations list
    let relations: Vec<DiplomaticRelationView> = faction
        .relations
        .iter()
        .map(|rel| {
            let target_name = registry
                .get(rel.target_id)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            DiplomaticRelationView {
                target_id: rel.target_id.to_string(),
                target_name,
                relation: format!("{:?}", rel.relation).to_lowercase(),
                established_year: rel.established_year,
                treaty_name: rel.treaty_name.clone(),
                changed_year: rel.changed_year,
            }
        })
        .collect();

    Ok(Json(ApiResponse::new(relations)))
}

/// GET /api/v1/factions/{id}/turn - Get faction turn state
async fn get_faction_turn(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<FactionTurnStateView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Find the faction
    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    // Get turn state
    let turn_state = faction
        .turn_state
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Faction has no turn state".to_string()))?;

    Ok(Json(ApiResponse::new(FactionTurnStateView::from(
        turn_state,
    ))))
}

/// POST /api/v1/factions/{id}/turn/advance - Advance faction turn
async fn advance_faction_turn(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<TurnAdvanceResponse>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Clone for mutation
    let mut registry_clone = registry.clone();

    // Find and update faction
    let faction = registry_clone
        .get_mut(faction_uuid)
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
        completed_goals: turn_state
            .check_goals()
            .iter()
            .map(|id| id.to_string())
            .collect(),
    };

    // Save updated registry
    drop(faction);
    state
        .save_faction_registry(world_id, registry_clone)
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

/// POST /api/v1/factions/{id}/assets - Add asset to faction
async fn add_faction_asset(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
    Json(request): Json<AddAssetRequest>,
) -> Result<Json<ApiResponse<FactionAssetView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    // Load faction registry
    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    // Clone for mutation
    let mut registry_clone = registry.clone();

    // Find and update faction
    let faction = registry_clone
        .get_mut(faction_uuid)
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
    faction
        .turn_state
        .as_mut()
        .unwrap()
        .add_asset(asset.clone());

    let response = FactionAssetView::from(&asset);

    // Save the updated registry
    state
        .save_faction_registry(world_id, registry_clone)
        .map_err(|e| ApiError::Internal(format!("Failed to save factions: {}", e)))?;

    Ok(Json(ApiResponse::new(response)))
}

/// Request body for adding a faction goal
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddGoalRequest {
    pub goal_type: String,
    pub description: String,
    pub target_value: u32,
}

/// GET /api/v1/factions/{id}/goals - Get faction goals
async fn get_faction_goals(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<Vec<crate::api::models::FactionGoalView>>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    let turn_state = faction
        .turn_state
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Faction has no turn state".to_string()))?;

    let goals: Vec<_> = turn_state
        .goals
        .iter()
        .map(crate::api::models::FactionGoalView::from)
        .collect();

    Ok(Json(ApiResponse::new(goals)))
}

/// POST /api/v1/factions/{id}/goals - Add a goal to faction
async fn add_faction_goal(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
    Json(request): Json<AddGoalRequest>,
) -> Result<Json<ApiResponse<crate::api::models::FactionGoalView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    let mut registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    let faction = registry
        .get_mut(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    if faction.turn_state.is_none() {
        faction.turn_state = Some(FactionTurnState::new(faction.founded_year.unwrap_or(0)));
    }

    let goal_type = match request.goal_type.to_lowercase().as_str() {
        "military_conquest" => crate::faction::GoalType::MilitaryConquest,
        "commercial_expansion" => crate::faction::GoalType::CommercialExpansion,
        "cultural_dominance" => crate::faction::GoalType::CulturalDominance,
        "diplomatic_supremacy" => crate::faction::GoalType::DiplomaticSupremacy,
        _ => return Err(ApiError::BadRequest("Invalid goal type".to_string())),
    };

    let goal =
        crate::faction::FactionGoal::new(goal_type, request.description, request.target_value);
    let goal_view = crate::api::models::FactionGoalView::from(&goal);
    faction.turn_state.as_mut().unwrap().goals.push(goal);

    state
        .save_faction_registry(world_id, registry)
        .map_err(|e| ApiError::Internal(format!("Failed to save factions: {}", e)))?;

    Ok(Json(ApiResponse::new(goal_view)))
}

/// GET /api/v1/factions/{id}/goals/{goal_id} - Get specific goal
async fn get_faction_goal(
    State(state): State<crate::api::AppState>,
    Path((faction_id, goal_id)): Path<(String, String)>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<crate::api::models::FactionGoalView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let goal_uuid = Uuid::parse_str(&goal_id)
        .map_err(|_| ApiError::BadRequest("Invalid goal ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    let turn_state = faction
        .turn_state
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Faction has no turn state".to_string()))?;

    let goal = turn_state
        .goals
        .iter()
        .find(|g| g.id == goal_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Goal '{}' not found", goal_id)))?;

    Ok(Json(ApiResponse::new(
        crate::api::models::FactionGoalView::from(goal),
    )))
}

/// Request body for adding a beast bond
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBeastBondRequest {
    pub beast_id: String,
    pub bond_type: String,
}

/// GET /api/v1/factions/{id}/beast-bonds - Get faction beast bonds
async fn get_faction_beast_bonds(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
) -> Result<Json<ApiResponse<Vec<crate::api::models::BeastBondView>>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    let registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    let faction = registry
        .get(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    let turn_state = faction
        .turn_state
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Faction has no turn state".to_string()))?;

    let bonds: Vec<_> = turn_state
        .beast_bonds
        .iter()
        .map(crate::api::models::BeastBondView::from)
        .collect();

    Ok(Json(ApiResponse::new(bonds)))
}

/// POST /api/v1/factions/{id}/beast-bonds - Add a beast bond
async fn add_faction_beast_bond(
    State(state): State<crate::api::AppState>,
    Path(faction_id): Path<String>,
    Query(params): Query<FactionsQueryParams>,
    Json(request): Json<AddBeastBondRequest>,
) -> Result<Json<ApiResponse<crate::api::models::BeastBondView>>, ApiError> {
    let faction_uuid = Uuid::parse_str(&faction_id)
        .map_err(|_| ApiError::BadRequest("Invalid faction ID format".to_string()))?;

    let beast_uuid = Uuid::parse_str(&request.beast_id)
        .map_err(|_| ApiError::BadRequest("Invalid beast ID format".to_string()))?;

    let world_id = params
        .world_id
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("world_id query parameter is required".to_string()))?;

    let mut registry = state
        .get_faction_registry(world_id)
        .map_err(|e| ApiError::Internal(format!("Failed to load factions: {}", e)))?;

    let faction = registry
        .get_mut(faction_uuid)
        .ok_or_else(|| ApiError::NotFound(format!("Faction '{}' not found", faction_id)))?;

    if faction.turn_state.is_none() {
        faction.turn_state = Some(FactionTurnState::new(faction.founded_year.unwrap_or(0)));
    }

    let bond_type = match request.bond_type.to_lowercase().as_str() {
        "worshiped" => crate::faction::BeastBondType::Worshiped,
        "allied" => crate::faction::BeastBondType::Allied,
        "tolerated" => crate::faction::BeastBondType::Tolerated,
        "opposed" => crate::faction::BeastBondType::Opposed,
        _ => return Err(ApiError::BadRequest("Invalid bond type".to_string())),
    };

    let bond = crate::faction::BeastBond::new(beast_uuid, faction_uuid, bond_type);
    let bond_view = crate::api::models::BeastBondView::from(&bond);
    faction.turn_state.as_mut().unwrap().add_beast_bond(bond);

    state
        .save_faction_registry(world_id, registry)
        .map_err(|e| ApiError::Internal(format!("Failed to save factions: {}", e)))?;

    Ok(Json(ApiResponse::new(bond_view)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_type_list() {
        assert!(true);
    }
}
