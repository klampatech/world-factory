//! Beings API Routes (Primal Beasts)
//!
//! Endpoints for retrieving the four primal beasts and their information.
//! These legendary creatures shape terrain and grant blessings/curses.

use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};

use crate::api::AppState;
use crate::api::error::ApiError;
use crate::api::models::*;
use crate::beasts::profiles::get_beast_profile;
use crate::beasts::{BeastElement, BeastForm, PrimalBeast, PrimalBeastInstance};

// =============================================================================
// Response Types
// =============================================================================

/// Response for beings list endpoint
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeingsListResponse {
    pub beings: Vec<BeingView>,
    pub total: usize,
}

/// Individual being (primal beast) view
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeingView {
    pub id: String,
    pub name: String,
    pub title: String,
    pub element: String,
    pub form: String,
    pub description: String,
    pub element_weakness: String,
    pub dormant_period_years: i32,
    pub profile: Option<BeingProfileView>,
}

/// Simplified profile view
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeingProfileView {
    pub power: f32,
    pub influence_radius: f32,
    pub terrain_modification_radius: f32,
    pub blessing_radius: f32,
    pub curse_radius: f32,
}

/// Registers beings routes under /api/v1/beings
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_beings))
        .route("/{id}", get(get_being))
        .with_state(state)
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/beings - List all primal beings (primal beasts)
async fn list_beings(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<BeingsListResponse>>, ApiError> {
    let beings: Vec<BeingView> = PrimalBeast::all()
        .iter()
        .map(|b| being_to_view(*b, false))
        .collect();

    let response = BeingsListResponse {
        total: beings.len(),
        beings,
    };
    Ok(Json(ApiResponse::new(response)))
}

/// GET /api/v1/beings/{id} - Get a specific primal being by ID
async fn get_being(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<BeingView>>, ApiError> {
    // Try to parse the ID as a primal beast name (case-insensitive)
    let beast = match id.to_lowercase().as_str() {
        "pyraxes" => Some(PrimalBeast::Pyraxes),
        "tidarth" => Some(PrimalBeast::Tidarth),
        "terros" => Some(PrimalBeast::Terros),
        "lumina" => Some(PrimalBeast::Lumina),
        _ => None,
    };

    match beast {
        Some(b) => {
            let being = being_to_view(b, true);
            Ok(Json(ApiResponse::new(being)))
        }
        None => Err(ApiError::NotFound(format!(
            "Primal being '{}' not found. Valid beings: Pyraxes, Tidarth, Terros, Lumina",
            id
        ))),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert a PrimalBeast to a view for API responses
fn being_to_view(beast: PrimalBeast, include_profile: bool) -> BeingView {
    use crate::beasts::slaying::calculate_dormancy_period;

    let profile = get_beast_profile(beast);

    BeingView {
        id: format!("{:?}", beast).to_lowercase(),
        name: beast.name().to_string(),
        title: beast.title().to_string(),
        element: beast.element().name().to_string(),
        form: format!("{:?}", beast.form()).to_lowercase(),
        description: format!(
            "{} is {}, dwelling in the realms of {}.",
            beast.name(),
            match beast {
                PrimalBeast::Pyraxes => "a flame wyrm of volcanic fury",
                PrimalBeast::Tidarth => "the storm serpent of endless waters",
                PrimalBeast::Terros => "the ancient mountain titan of stone",
                PrimalBeast::Lumina => "the winged spirit of renewal and life",
            },
            match beast.element() {
                BeastElement::Fire => "fire and volcanic domains",
                BeastElement::Water => "water and storm domains",
                BeastElement::Earth => "earth and mountain domains",
                BeastElement::Life => "life and renewal domains",
            }
        ),
        element_weakness: beast.element().opposing().name().to_string(),
        dormant_period_years: calculate_dormancy_period(beast),
        profile: if include_profile {
            Some(BeingProfileView {
                power: profile.power,
                influence_radius: profile.influence_radius,
                terrain_modification_radius: profile.terrain_modification_radius,
                blessing_radius: profile.blessing_radius,
                curse_radius: profile.curse_radius,
            })
        } else {
            None
        },
    }
}
