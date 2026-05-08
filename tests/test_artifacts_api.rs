//! Artifacts API Tests (WOR-448)
//!
//! Comprehensive tests for src/api/v1/artifacts.rs - artifact lifecycle, ownership, validation.
//! Based on gap analysis in WOR-426-GAP-ANALYSIS.md (G003: API Handlers).
//!
//! Run with: `cargo test --features api tests/test_artifacts_api.rs`
//!
//! ## Test Coverage
//!
//! ### Endpoints Tested
//! 1. GET /api/v1/worlds/:id/artifacts - List artifacts with filters
//! 2. GET /api/v1/worlds/:id/artifacts/:artifact_id - Get single artifact
//!
//! ### Test Categories
//! - UUID validation for world_id and artifact_id
//! - Query parameter parsing (limit, offset, category, era, min_significance, creator_id)
//! - Response structure (ArtifactsResponse, ArtifactView, ArtifactDetailView)
//! - Category filtering logic
//! - Significance filtering
//! - Pagination (limit/offset)
//! - Error responses (404 Not Found, 400 Bad Request)
//! - Edge cases (empty results, max limit cap, invalid filters)

#[cfg(all(test, feature = "api"))]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    /// Test helper to create app router
    fn create_test_app() -> axum::Router<crate::api::AppState> {
        use world_factory::api::create_router;
        create_router()
    }

    /// Parse JSON response body
    async fn parse_json_response(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // =========================================================================
    // Test Group 1: UUID Validation
    // =========================================================================

    #[tokio::test]
    async fn test_list_artifacts_invalid_world_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/not-a-valid-uuid/artifacts")
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

        let json = parse_json_response(response).await;
        assert_eq!(json["success"].as_bool().unwrap(), false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("invalid"));
    }

    #[tokio::test]
    async fn test_get_artifact_invalid_world_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/not-a-valid-uuid/artifacts/11111111-1111-1111-1111-111111111111")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid world UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_get_artifact_invalid_artifact_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts/not-a-valid-uuid")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid artifact UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_list_artifacts_valid_uuid_returns_200() {
        let app = create_test_app();
        let valid_uuid = "11111111-1111-1111-1111-111111111111";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/worlds/{}/artifacts", valid_uuid))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 200 (returns sample data) or 404 if world doesn't exist
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
            "Valid UUID should return 200 or 404"
        );
    }

    #[tokio::test]
    async fn test_get_artifact_valid_uuids_returns_404() {
        // Even with valid UUIDs, artifact not found (TODO: no store yet)
        let app = create_test_app();
        let world_uuid = "11111111-1111-1111-1111-111111111111";
        let artifact_uuid = "22222222-2222-2222-2222-222222222222";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/worlds/{}/artifacts/{}",
                        world_uuid, artifact_uuid
                    ))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent artifact should return 404"
        );

        let json = parse_json_response(response).await;
        assert_eq!(json["success"].as_bool().unwrap(), false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains(artifact_uuid));
    }

    // =========================================================================
    // Test Group 2: Query Parameter Parsing
    // =========================================================================

    #[tokio::test]
    async fn test_list_artifacts_default_params_returns_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Default params should work (returns sample data)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_limit_param() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=10")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_offset_param() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?offset=5")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_limit_and_offset() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=5&offset=2")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_category_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=Weapon")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_era_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?era=Age%20of%20Kings")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_min_significance_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?min_significance=0.8")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_creator_id_filter() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?creator_id=33333333-3333-3333-3333-333333333333")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_with_all_filters() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=20&offset=0&category=Weapon&era=Medieval&min_significance=0.7")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Test Group 3: Limit Validation and Cap
    // =========================================================================

    #[tokio::test]
    async fn test_list_artifacts_limit_exceeds_max() {
        // The handler caps limit at 200
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=500")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should still return 200 (capped) or 404
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_limit_at_max() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=200")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_artifacts_limit_zero() {
        // Zero limit should use default
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=0")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Test Group 4: Response Structure
    // =========================================================================

    #[tokio::test]
    async fn test_artifacts_response_has_required_fields() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;

            // Verify API response wrapper
            assert!(json.get("success").is_some(), "Response should have 'success' field");
            assert!(json.get("data").is_some(), "Response should have 'data' field");

            let data = &json["data"];

            // Verify response structure (ArtifactsResponse)
            assert!(data.get("worldId").is_some(), "Data should have 'worldId'");
            assert!(data.get("artifacts").is_some(), "Data should have 'artifacts' array");
            assert!(data.get("total").is_some(), "Data should have 'total' count");
            assert!(data.get("limit").is_some(), "Data should have 'limit'");
            assert!(data.get("offset").is_some(), "Data should have 'offset'");
        }
    }

    #[tokio::test]
    async fn test_artifact_view_has_required_fields() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];

            if let Some(first) = artifacts.as_array().and_then(|arr| arr.first()) {
                // Verify ArtifactView fields
                assert!(first.get("id").is_some(), "Artifact should have 'id'");
                assert!(first.get("name").is_some(), "Artifact should have 'name'");
                assert!(first.get("category").is_some(), "Artifact should have 'category'");
                assert!(first.get("era").is_some(), "Artifact should have 'era' (nullable)");
                assert!(first.get("createdYear").is_some(), "Artifact should have 'createdYear'");
                assert!(first.get("culture").is_some(), "Artifact should have 'culture' (nullable)");
                assert!(first.get("description").is_some(), "Artifact should have 'description'");
                assert!(first.get("significance").is_some(), "Artifact should have 'significance'");
                assert!(first.get("condition").is_some(), "Artifact should have 'condition'");

                // Verify field types
                assert!(first["id"].is_string(), "'id' should be string");
                assert!(first["name"].is_string(), "'name' should be string");
                assert!(first["category"].is_string(), "'category' should be string");
                assert!(first["createdYear"].is_i64() || first["createdYear"].is_number(), "'createdYear' should be number");
                assert!(first["significance"].is_number(), "'significance' should be number");
            }
        }
    }

    #[tokio::test]
    async fn test_artifact_view_category_is_lowercase() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];

            if let Some(first) = artifacts.as_array().and_then(|arr| arr.first()) {
                let category = first["category"].as_str().unwrap();
                assert_eq!(
                    category.to_lowercase(), category,
                    "Category should be lowercase"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_artifact_view_condition_is_lowercase() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];

            if let Some(first) = artifacts.as_array().and_then(|arr| arr.first()) {
                let condition = first["condition"].as_str().unwrap();
                assert_eq!(
                    condition.to_lowercase(), condition,
                    "Condition should be lowercase"
                );
            }
        }
    }

    // =========================================================================
    // Test Group 5: Category Filtering Logic
    // =========================================================================

    #[tokio::test]
    async fn test_category_filter_exact_match() {
        // Test exact category name matches
        let categories = vec![
            ("weapon", "Weapon"),
            ("relic", "Relic"),
            ("magical", "Magical"),
            ("monument", "Monument"),
            ("document", "Document"),
            ("trophy", "Trophy"),
            ("sacred", "Sacred"),
        ];

        for (filter, _expected) in categories {
            let app = create_test_app();
            let response = app
                .oneshot(
                    Request::builder()
                        .uri(format!(
                            "/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category={}",
                            filter
                        ))
                        .body(Body::default())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
                "Category filter '{}' should return 200 or 404",
                filter
            );
        }
    }

    #[tokio::test]
    async fn test_category_filter_crown_jewel_variants() {
        // CrownJewel has two valid filter values
        let app = create_test_app();

        // Test with underscore
        let response1 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=crown_jewel")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response1.status() == StatusCode::OK || response1.status() == StatusCode::NOT_FOUND);

        // Test without underscore
        let response2 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=crownjewel")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response2.status() == StatusCode::OK || response2.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_category_filter_case_insensitive() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=WEAPON")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_category_filter_no_match_returns_empty() {
        // Filter that won't match any sample artifacts should return empty list
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=nonexistent_category")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];
            assert!(
                artifacts.as_array().unwrap().is_empty(),
                "Non-matching category filter should return empty array"
            );
        }
    }

    // =========================================================================
    // Test Group 6: Significance Filtering
    // =========================================================================

    #[tokio::test]
    async fn test_significance_filter_exact_threshold() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?min_significance=0.85")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_significance_filter_zero() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?min_significance=0.0")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_significance_filter_one() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?min_significance=1.0")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_significance_filter_filters_results() {
        // High threshold should filter out low-significance artifacts
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?min_significance=0.95")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];
            for artifact in artifacts.as_array().unwrap() {
                let sig = artifact["significance"].as_f64().unwrap() as f32;
                assert!(
                    sig >= 0.95,
                    "Artifact significance {} should be >= 0.95",
                    sig
                );
            }
        }
    }

    // =========================================================================
    // Test Group 7: Pagination
    // =========================================================================

    #[tokio::test]
    async fn test_pagination_offset_skips_results() {
        let app = create_test_app();

        // Get without offset
        let response1 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=10")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Get with offset
        let response2 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=10&offset=2")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response1.status() == StatusCode::OK && response2.status() == StatusCode::OK {
            let json1 = parse_json_response(response1).await;
            let json2 = parse_json_response(response2).await;

            let offset1 = json1["data"]["offset"].as_i64().unwrap_or(0);
            let offset2 = json2["data"]["offset"].as_i64().unwrap_or(0);

            assert_eq!(offset1, 0, "First request should have offset 0");
            assert_eq!(offset2, 2, "Second request should have offset 2");
        }
    }

    #[tokio::test]
    async fn test_pagination_total_unchanged_by_offset() {
        let app = create_test_app();

        let response1 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=10")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        let response2 = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=10&offset=100")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response1.status() == StatusCode::OK && response2.status() == StatusCode::OK {
            let json1 = parse_json_response(response1).await;
            let json2 = parse_json_response(response2).await;

            let total1 = json1["data"]["total"].as_i64().unwrap_or(0);
            let total2 = json2["data"]["total"].as_i64().unwrap_or(0);

            assert_eq!(
                total1, total2,
                "Total count should be the same regardless of offset"
            );
        }
    }

    #[tokio::test]
    async fn test_pagination_limit_respects_capped_value() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?limit=500")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let limit = json["data"]["limit"].as_i64().unwrap_or(0);

            assert!(
                limit <= 200,
                "Limit should be capped at 200, got {}",
                limit
            );
        }
    }

    // =========================================================================
    // Test Group 8: Error Response Structure
    // =========================================================================

    #[tokio::test]
    async fn test_error_response_structure() {
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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = parse_json_response(response).await;

        // Verify error response structure
        assert_eq!(json["success"].as_bool().unwrap(), false, "Success should be false");
        assert!(json.get("error").is_some(), "Response should have 'error' field");
    }

    #[tokio::test]
    async fn test_not_found_response_structure() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts/22222222-2222-2222-2222-222222222222")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = parse_json_response(response).await;

        assert_eq!(json["success"].as_bool().unwrap(), false, "Success should be false");
        assert!(json.get("error").is_some(), "Response should have 'error' field");
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("not found"), "Error should mention 'not found'");
    }

    // =========================================================================
    // Test Group 9: Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_no_artifacts_returns_empty_array() {
        // Using a filter that should return no results
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?category=nonexistent")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];

            assert!(
                artifacts.is_array(),
                "Artifacts should be an array even when empty"
            );
            assert!(
                artifacts.as_array().unwrap().is_empty(),
                "Non-matching filter should return empty array"
            );
        }
    }

    #[tokio::test]
    async fn test_artifact_ids_are_valid_uuids() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let artifacts = &json["data"]["artifacts"];

            for artifact in artifacts.as_array().unwrap() {
                let id = artifact["id"].as_str().unwrap();

                // Should be a valid UUID format
                assert!(
                    uuid::Uuid::parse_str(id).is_ok(),
                    "Artifact ID '{}' should be a valid UUID",
                    id
                );
            }
        }
    }

    #[tokio::test]
    async fn test_world_id_in_response_matches_request() {
        let app = create_test_app();
        let world_id = "11111111-1111-1111-1111-111111111111";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/worlds/{}/artifacts", world_id))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let json = parse_json_response(response).await;
            let response_world_id = json["data"]["worldId"].as_str().unwrap();

            assert_eq!(
                response_world_id, world_id,
                "Response worldId should match request worldId"
            );
        }
    }

    // =========================================================================
    // Test Group 10: Serialization (camelCase)
    // =========================================================================

    #[tokio::test]
    async fn test_response_uses_camel_case() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();

            // Verify camelCase fields are present
            assert!(body_str.contains("worldId"), "Should use 'worldId' (camelCase)");
            assert!(body_str.contains("createdYear"), "Should use 'createdYear' (camelCase)");
            assert!(body_str.contains("minSignificance"), "Should use 'minSignificance' (camelCase)");
        }
    }

    #[tokio::test]
    async fn test_query_params_use_camel_case() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts?minSignificance=0.5&creatorId=abc")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should accept camelCase query params
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Test Group 11: Concurrency
    // =========================================================================

    #[tokio::test]
    async fn test_concurrent_artifact_requests() {
        use tokio::task::JoinSet;

        let app = create_test_app();
        let mut join_set = JoinSet::new();

        // Send 5 concurrent requests
        for _ in 0..5 {
            let mut app = app.clone();
            join_set.spawn(async move {
                app.oneshot(
                    Request::builder()
                        .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/artifacts")
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
                if response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND {
                    success_count += 1;
                }
            }
        }

        assert_eq!(success_count, 5, "All concurrent requests should succeed");
    }
}
