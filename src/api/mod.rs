//! API Module
//!
//! REST API layer for world generation service.

pub mod error;
pub mod models;
pub mod services;

pub mod v1;

// HTTP API routes for the World Factory API
// Implements the API contract defined in docs/API_CONTRACT.md
// Uses the Axum web framework with API versioning under /api/v1/

use axum::{
    routing::get,
    Router,
    response::Json,
};

// Re-export model types for use in handlers
pub use self::models::*;
pub use self::error::ApiError;
use crate::storage::{StorageManager, StorageConfig, StorageError};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Storage manager for world persistence
    pub storage: StorageManager,
}

impl AppState {
    /// Create new AppState with default storage configuration
    pub fn new() -> Result<Self, StorageError> {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config)?;
        Ok(Self { storage })
    }
    
    /// Create AppState with custom storage configuration
    pub fn with_storage(storage: StorageManager) -> Self {
        Self { storage }
    }
}

/// Create the complete API router with all versioned routes
pub fn create_router() -> Router<AppState> {
    // Create default app state with storage
    let app_state = AppState::new().expect("Failed to initialize storage");
    
    Router::new()
        .nest("/api/v1", v1::routes(app_state))
        // Health check endpoint
        .route("/health", get(health_check))
}

/// GET /health - Health check endpoint
async fn health_check() -> impl axum::response::IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{StatusCode, Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_router();
        
        let response = app
            .oneshot(Request::builder().uri("/health").body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_worlds_empty() {
        let app = create_router();
        
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/worlds")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_uuid_returns_400() {
        let app = create_router();
        
        let response = app
            .oneshot(Request::builder()
                .uri("/api/v1/worlds/not-a-uuid")
                .body(axum::body::Body::default()).unwrap()).await
            .unwrap();
            
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}