//! API Endpoint Tests for World Factory (WOR-221)
//!
//! Tests 18 API handlers including happy paths and error paths.
//!
//! Run with: `cargo test --features api tests/api_endpoints_test.rs`
//!
//! ## Handlers Tested
//!
//! ### Worlds Endpoints
//! 1. GET /health - Health check
//! 2. GET /api/v1/worlds - List worlds
//! 3. POST /api/v1/worlds - Create world
//! 4. GET /api/v1/worlds/:id - Get world
//! 5. POST /api/v1/worlds/:id/generate - Trigger generation
//! 6. GET /api/v1/worlds/:id/map - Get world map
//! 7. GET /api/v1/worlds/:id/timeline - Get timeline
//! 8. GET /api/v1/worlds/:id/events - Get world events
//! 9. GET /api/v1/worlds/:id/history - Get world history
//! 10. GET /api/v1/worlds/:id/figures - Get world figures
//! 11. GET /api/v1/worlds/:id/societies - Get world societies
//! 12. GET /api/v1/worlds/:id/planet - Get world planet
//! 13. GET /api/v1/worlds/:id/artifacts - Get world artifacts
//! 14. GET /api/v1/worlds/:id/cataclysms - Get world cataclysms
//! 15. GET /api/v1/worlds/:id/wonders - Get world wonders
//! 16. GET /api/v1/worlds/:id/resources - Get world resources
//! 17. GET /api/v1/worlds/:id/disasters - Get world disasters
//!
//! ### Species Endpoints
//! 18. GET /api/v1/species - List species

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    /// Test helper to create app router
    fn create_test_app() -> axum::Router<crate::api::AppState> {
        use world_factory::api::create_router;
        create_router()
    }

    // =========================================================================
    // Handler 1: GET /health - Health Check
    // =========================================================================

    #[tokio::test]
    async fn test_health_check_returns_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Health check should return 200"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("status").unwrap().as_str().unwrap(), "ok");
        assert!(json.get("version").is_some());
    }

    // =========================================================================
    // Handler 2: GET /api/v1/worlds - List Worlds
    // =========================================================================

    #[tokio::test]
    async fn test_list_worlds_empty_returns_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "List worlds should return 200"
        );
    }

    #[tokio::test]
    async fn test_list_worlds_with_pagination_params() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds?limit=10&offset=0&sort_by=created_at&sort_dir=desc")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_worlds_invalid_sort_by_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds?sort_by=invalid_field")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid sort_by should return 400"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("success").unwrap().as_bool().unwrap(), false);
        assert!(json
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("Invalid sort_by"));
    }

    #[tokio::test]
    async fn test_list_worlds_invalid_sort_dir_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds?sort_dir=invalid")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid sort_dir should return 400"
        );
    }

    // =========================================================================
    // Handler 3: POST /api/v1/worlds - Create World
    // =========================================================================

    #[tokio::test]
    async fn test_create_world_returns_201() {
        let app = create_test_app();
        let body = serde_json::json!({
            "name": "Test World",
            "parameters": {
                "seed": 42,
                "size": "Medium"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Create world should return 201"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json.get("success").unwrap().as_bool().unwrap(), true);
        assert!(json.get("data").unwrap().get("id").is_some());
    }

    #[tokio::test]
    async fn test_create_world_empty_name_returns_400() {
        let app = create_test_app();
        let body = serde_json::json!({
            "name": "",
            "parameters": { "seed": 42 }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Empty name should return 400"
        );
    }

    #[tokio::test]
    async fn test_create_world_name_too_long_returns_400() {
        let app = create_test_app();
        let body = serde_json::json!({
            "name": "a".repeat(101),
            "parameters": { "seed": 42 }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Name > 100 chars should return 400"
        );
    }

    // =========================================================================
    // Handler 4: GET /api/v1/worlds/:id - Get World
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_not_found_returns_404() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/00000000-0000-0000-0000-000000000000")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent world should return 404"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("success").unwrap().as_bool().unwrap(), false);
        assert!(json
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("not found"));
    }

    #[tokio::test]
    async fn test_get_world_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/not-a-valid-uuid")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    // =========================================================================
    // Handler 5: POST /api/v1/worlds/:id/generate - Trigger Generation
    // =========================================================================

    #[tokio::test]
    async fn test_trigger_generation_invalid_uuid_returns_400() {
        let app = create_test_app();
        let body = serde_json::json!({ "name": "Test" });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds/invalid-uuid/generate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_trigger_generation_not_found_returns_404() {
        let app = create_test_app();
        let body = serde_json::json!({ "name": "Test" });
        let valid_uuid = "11111111-1111-1111-1111-111111111111";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/worlds/{}/generate", valid_uuid))
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // This should return 404 since world doesn't exist
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Handler 6: GET /api/v1/worlds/:id/map - Get World Map
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_map_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/map")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_map_with_params() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/map?lod=2&min_x=0&min_y=0&max_x=100&max_y=100")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be 404 since world doesn't exist, not 400
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Handler 7: GET /api/v1/worlds/:id/timeline - Get Timeline
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_timeline_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/timeline")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    // =========================================================================
    // Handler 8: GET /api/v1/worlds/:id/events - Get World Events
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_events_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/events")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_events_with_pagination() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/events?limit=50&offset=0")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 9: GET /api/v1/worlds/:id/history - Get World History
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_history_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/history")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    // =========================================================================
    // Handler 10: GET /api/v1/worlds/:id/figures - Get World Figures
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_figures_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/figures")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_figures_with_filters() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/figures?species_id=1&min_significance=0.5")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 11: GET /api/v1/worlds/:id/societies - Get World Societies
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_societies_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/societies")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    // =========================================================================
    // Handler 12: GET /api/v1/worlds/:id/planet - Get World Planet
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_planet_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/planet")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    // =========================================================================
    // Handler 13: GET /api/v1/worlds/:id/artifacts - Get World Artifacts
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_artifacts_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_artifacts_with_filters() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=20&category=Weapon&era=Medieval")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 14: GET /api/v1/worlds/:id/cataclysms - Get World Cataclysms
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_cataclysms_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_cataclysms_with_filters() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?min_severity=0.7&scope=global")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 15: GET /api/v1/worlds/:id/wonders - Get World Wonders
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_wonders_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/wonders")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_wonders_with_type_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/wonders?type=natural&limit=10")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 16: GET /api/v1/worlds/:id/resources - Get World Resources
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_resources_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/resources")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_resources_with_type_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/resources?type=mineral&min_magnitude=3")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 17: GET /api/v1/worlds/:id/disasters - Get World Disasters
    // =========================================================================

    #[tokio::test]
    async fn test_get_world_disasters_invalid_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid/disasters")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_world_disasters_with_status_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/disasters?status=active&limit=20")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // World doesn't exist
    }

    // =========================================================================
    // Handler 18: GET /api/v1/species - List Species
    // =========================================================================

    #[tokio::test]
    async fn test_list_species_returns_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/species")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "List species should return 200"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("success").unwrap().as_bool().unwrap(), true);
        assert!(json.get("data").unwrap().get("species").is_some());
    }

    #[tokio::test]
    async fn test_list_species_with_habitat_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/species?habitat=TemperateGrassland")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_species_with_trait_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/species?trait_filter=WarLike")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Error Path Tests: 404 and 500 responses
    // =========================================================================

    #[tokio::test]
    async fn test_get_species_not_found_returns_404() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/species/999999")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent species should return 404"
        );
    }

    #[tokio::test]
    async fn test_get_event_not_found_returns_404() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events/11111111-1111-1111-1111-111111111111")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent event should return 404"
        );
    }

    #[tokio::test]
    async fn test_get_factions_requires_world_id() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/factions")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 400 because world_id is required
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Missing world_id should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_faction_not_found_returns_404() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/factions/11111111-1111-1111-1111-111111111111?world_id=22222222-2222-2222-2222-222222222222")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent faction should return 404"
        );
    }

    // =========================================================================
    // Response Structure Tests
    // =========================================================================

    #[tokio::test]
    async fn test_api_response_has_correct_structure() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/species")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify API response wrapper structure
        assert!(
            json.get("success").is_some(),
            "Response should have 'success' field"
        );
        assert!(
            json.get("data").is_some(),
            "Response should have 'data' field"
        );

        // Verify data structure
        let data = json.get("data").unwrap();
        if let Some(species) = data.get("species") {
            assert!(species.is_array(), "Species should be an array");
        }
    }

    #[tokio::test]
    async fn test_error_response_has_correct_structure() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/invalid-uuid")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify error response structure
        assert_eq!(json.get("success").unwrap().as_bool().unwrap(), false);
        assert!(
            json.get("error").is_some(),
            "Error response should have 'error' field"
        );
        assert!(
            json.get("code").is_some(),
            "Error response should have 'code' field"
        );
    }

    // =========================================================================
    // Concurrency Tests
    // =========================================================================

    #[tokio::test]
    async fn test_concurrent_requests_same_endpoint() {
        use tokio::task::JoinSet;
        use tower::ServiceExt;

        let app = create_test_app();
        let mut join_set = JoinSet::new();

        // Send 10 concurrent requests to the same endpoint
        // Note: AppState contains Arc<StorageManager> so router can be cloned
        for _ in 0..10 {
            let mut app = app.clone();
            join_set.spawn(async move {
                app.oneshot(
                    Request::builder()
                        .uri("/api/v1/species")
                        .body(Body::default())
                        .unwrap(),
                )
                .await
                .unwrap()
            });
        }

        let mut success_count = 0;
        while let Some(result) = join_set.join_next().await {
            if let Ok(response) = result {
                if response.status() == StatusCode::OK {
                    success_count += 1;
                }
            }
        }

        assert_eq!(success_count, 10, "All concurrent requests should succeed");
    }
}
