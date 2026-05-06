// Integration test for GET /api/v1/worlds/:id/export endpoint
// Run with: cargo test --features api --test api_world_generation export

use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use world_factory::api::{create_router, AppState};
use world_factory::storage::{StorageConfig, StorageManager};

#[tokio::test]
#[ignore] // Requires running server with existing world
async fn test_export_endpoint_returns_binary_file() {
    // This would test:
    // 1. GET /api/v1/worlds/{id}/export returns 200
    // 2. Content-Type is application/octet-stream
    // 3. Content-Disposition contains .wfw filename
    // 4. Response body is valid gzip tarball
    todo!("Implement integration test with test world fixture")
}
