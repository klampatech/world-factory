//! API v1 Routes
//!
//! Organized by domain:
//! - `worlds` - World listing, creation, and retrieval
//! - `events` - Timeline and event queries
//! - `artifacts` - Historical artifacts
//! - `cataclysms` - World-altering cataclysms

pub mod worlds;
pub mod events;
pub mod artifacts;
pub mod cataclysms;

use axum::Router;
use crate::api::AppState;

/// Creates the v1 router with all routes
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/worlds", worlds::routes(state.clone()))
        .nest("/events", events::routes(state.clone()))
}
