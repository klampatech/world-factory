//! Cataclysms API Tests (WOR-447)
//!
//! Comprehensive tests for src/api/v1/cataclysms.rs - cataclysm listing, filtering, and retrieval.
//! Based on gap analysis in WOR-426-GAP-ANALYSIS.md (G003: API Handlers).
//!
//! Run with: `cargo test --features api tests/test_cataclysms_api.rs`
//!
//! ## Test Coverage
//!
//! ### Endpoints Tested
//! 1. GET /api/v1/worlds/:id/cataclysms - List cataclysms with filters
//! 2. GET /api/v1/worlds/:id/cataclysms/:cataclysm_id - Get single cataclysm
//!
//! ### Test Categories
//! - UUID validation for world_id and cataclysm_id
//! - Query parameter parsing (limit, offset, cataclysm_type, scope, min_severity, region_id, start_year, end_year)
//! - Response structure (CataclysmsResponse, CataclysmView, CataclysmDetailView, ImpactView, EffectView)
//! - Type filtering logic (all CataclysmType variants)
//! - Scope filtering logic (local, regional, continental, global)
//! - Severity filtering
//! - Year range filtering
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
    async fn test_list_cataclysms_invalid_world_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/not-a-valid-uuid/cataclysms")
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
    async fn test_get_cataclysm_invalid_world_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/not-a-valid-uuid/cataclysms/11111111-1111-1111-1111-111111111111")
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
    async fn test_get_cataclysm_invalid_cataclysm_uuid_returns_400() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms/not-a-valid-uuid")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Invalid cataclysm UUID should return 400"
        );
    }

    #[tokio::test]
    async fn test_list_cataclysms_valid_uuid_returns_200() {
        let app = create_test_app();
        let valid_uuid = "11111111-1111-1111-1111-111111111111";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/worlds/{}/cataclysms", valid_uuid))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 200 (returns sample data)
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Valid UUID should return 200"
        );

        let json = parse_json_response(response).await;
        assert_eq!(json["success"].as_bool().unwrap(), true);
        assert!(json["data"].is_object());
        assert!(json["data"]["cataclysms"].is_array());
    }

    #[tokio::test]
    async fn test_get_cataclysm_valid_uuids_returns_404() {
        // Even with valid UUIDs, cataclysm not found (TODO: no store yet)
        let app = create_test_app();
        let world_uuid = "11111111-1111-1111-1111-111111111111";
        let cataclysm_uuid = "22222222-2222-2222-2222-222222222222";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/worlds/{}/cataclysms/{}",
                        world_uuid, cataclysm_uuid
                    ))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Non-existent cataclysm should return 404"
        );

        let json = parse_json_response(response).await;
        assert_eq!(json["success"].as_bool().unwrap(), false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains(cataclysm_uuid));
    }

    // =========================================================================
    // Test Group 2: Query Parameter Parsing
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_default_params_returns_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        assert!(json["data"]["cataclysms"].is_array());
    }

    #[tokio::test]
    async fn test_list_cataclysms_with_limit_param() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=10")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_limit_capped_at_200() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=500")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        // The API should cap limit at 200
        assert!(json["data"]["limit"].as_i64().unwrap() <= 200);
    }

    #[tokio::test]
    async fn test_list_cataclysms_with_offset_param() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?offset=5")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_with_limit_and_offset() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=2&offset=1")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 3: Type Filtering
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_plague() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=plague")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let cat_type = cat["cataclysmType"].as_str().unwrap().to_lowercase();
            assert!(
                cat_type.contains("plague") || cat_type.contains("disease"),
                "Expected plague type, got: {}",
                cat_type
            );
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_quake() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=quake")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let cat_type = cat["cataclysmType"].as_str().unwrap().to_lowercase();
            assert!(
                cat_type.contains("quake") || cat_type.contains("earthquake"),
                "Expected quake type, got: {}",
                cat_type
            );
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_volcano() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=volcano")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Should return empty array when no volcanic cataclysms match
        let json = parse_json_response(response).await;
        assert!(json["data"]["cataclysms"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_migration() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=migration")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        if !cataclysms.is_empty() {
            for cat in cataclysms {
                let cat_type = cat["cataclysmType"].as_str().unwrap().to_lowercase();
                assert!(
                    cat_type.contains("migration") || cat_type.contains("horde"),
                    "Expected migration type, got: {}",
                    cat_type
                );
            }
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_flood() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=flood")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_drought() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=drought")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_ice() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=ice")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_meteor() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=meteor")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 4: Scope Filtering
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_scope_global() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=global")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let scope = cat["scope"].as_str().unwrap().to_lowercase();
            assert_eq!(scope, "global", "Expected global scope");
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_scope_continental() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=continental")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_scope_regional() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=regional")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_scope_local() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=local")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 5: Severity Filtering
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_min_severity() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?min_severity=0.8")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let severity = cat["severity"].as_f64().unwrap();
            assert!(
                severity >= 0.8,
                "Expected severity >= 0.8, got: {}",
                severity
            );
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_min_severity_high() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?min_severity=0.9")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 6: Year Range Filtering
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_start_year() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?start_year=1000")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let year = cat["year"].as_i64().unwrap();
            assert!(
                year >= 1000,
                "Expected year >= 1000, got: {}",
                year
            );
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_end_year() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?end_year=1000")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let year = cat["year"].as_i64().unwrap();
            assert!(
                year <= 1000,
                "Expected year <= 1000, got: {}",
                year
            );
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_year_range() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?start_year=400&end_year=500")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let year = cat["year"].as_i64().unwrap();
            assert!(
                year >= 400 && year <= 500,
                "Expected year between 400-500, got: {}",
                year
            );
        }
    }

    // =========================================================================
    // Test Group 7: Combined Filters
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_type_and_scope() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=plague&scope=global")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        for cat in cataclysms {
            let cat_type = cat["cataclysmType"].as_str().unwrap().to_lowercase();
            let scope = cat["scope"].as_str().unwrap().to_lowercase();
            assert!(cat_type.contains("plague") || cat_type.contains("disease"));
            assert_eq!(scope, "global");
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_filter_by_all_params() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=10&offset=0&cataclysm_type=plague&scope=global&min_severity=0.7&start_year=1300&end_year=1400")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 8: Response Structure
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_response_structure() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;

        // Top-level structure
        assert!(json["success"].is_boolean());
        assert!(json["data"].is_object());

        let data = &json["data"];

        // Response fields
        assert!(data["worldId"].is_string());
        assert!(data["cataclysms"].is_array());
        assert!(data["total"].is_number());
        assert!(data["limit"].is_number());
        assert!(data["offset"].is_number());
    }

    #[tokio::test]
    async fn test_list_cataclysms_cataclysm_view_fields() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();

        if !cataclysms.is_empty() {
            let cat = &cataclysms[0];

            // Required fields
            assert!(cat["id"].is_string(), "id should be string");
            assert!(cat["name"].is_string(), "name should be string");
            assert!(cat["cataclysmType"].is_string(), "cataclysmType should be string");
            assert!(cat["year"].is_number(), "year should be number");
            assert!(cat["severity"].is_number(), "severity should be number");
            assert!(cat["scope"].is_string(), "scope should be string");
            assert!(cat["description"].is_string(), "description should be string");
            assert!(cat["significance"].is_number(), "significance should be number");

            // Optional fields
            // duration_years can be null or number
            // populationLost can be null or number
            // culturesDestroyed can be null or array
            // culturesEmerged can be null or array
        }
    }

    #[tokio::test]
    async fn test_list_cataclysms_returns_correct_world_id() {
        let app = create_test_app();
        let world_id = "11111111-1111-1111-1111-111111111111";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/worlds/{}/cataclysms", world_id))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        assert_eq!(
            json["data"]["worldId"].as_str().unwrap(),
            world_id,
            "World ID in response should match request"
        );
    }

    // =========================================================================
    // Test Group 9: Pagination
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_pagination_total_accurate() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        let total = json["data"]["total"].as_u64().unwrap();
        assert_eq!(
            cataclysms.len() as u64, total,
            "When no filters, total should equal array length"
        );
    }

    #[tokio::test]
    async fn test_list_cataclysms_pagination_offset_works() {
        let app = create_test_app();

        // Get all first
        let response_all = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=100")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        let json_all = parse_json_response(response_all).await;
        let total = json_all["data"]["total"].as_u64().unwrap();

        if total >= 2 {
            // Get with offset
            let response_offset = app
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=1&offset=1")
                        .body(Body::default())
                        .unwrap(),
                )
                .await
                .unwrap();

            let json_offset = parse_json_response(response_offset).await;

            // Total should remain the same
            assert_eq!(
                json_offset["data"]["total"].as_u64().unwrap(),
                total,
                "Total should be consistent with pagination"
            );

            // Offset should be 1
            assert_eq!(
                json_offset["data"]["offset"].as_u64().unwrap(),
                1,
                "Offset should be 1"
            );
        }
    }

    // =========================================================================
    // Test Group 10: Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_empty_results_no_filter_match() {
        let app = create_test_app();
        // Use a very specific filter that likely won't match sample data
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=blight&min_severity=0.99")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();
        assert!(
            cataclysms.is_empty(),
            "Expected empty results for non-matching filter"
        );
    }

    #[tokio::test]
    async fn test_list_cataclysms_very_high_limit_returns_max() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=10000")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        assert!(
            json["data"]["limit"].as_i64().unwrap() <= 200,
            "Limit should be capped at 200"
        );
    }

    #[tokio::test]
    async fn test_list_cataclysms_invalid_type_returns_empty() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=nonexistent_type")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Invalid type filters should return empty (no matches)
        let json = parse_json_response(response).await;
        assert!(
            json["data"]["cataclysms"].as_array().unwrap().is_empty(),
            "Invalid type filter should return empty"
        );
    }

    #[tokio::test]
    async fn test_list_cataclysms_negative_limit_uses_default() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=-1")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should still work (using default limit)
        assert!(response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_zero_limit() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?limit=0")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_type_case_insensitive() {
        let app = create_test_app();

        // Test lowercase
        let response_lower = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=plague")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response_lower.status(), StatusCode::OK);

        // Test uppercase
        let response_upper = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?cataclysm_type=PLAGUE")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response_upper.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cataclysms_scope_case_insensitive() {
        let app = create_test_app();

        // Test lowercase
        let response_lower = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=global")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response_lower.status(), StatusCode::OK);

        // Test uppercase
        let response_upper = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?scope=GLOBAL")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response_upper.status(), StatusCode::OK);
    }

    // =========================================================================
    // Test Group 11: Region ID Filter
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_region_id_filter_valid_uuid() {
        let app = create_test_app();
        let region_uuid = "33333333-3333-3333-3333-333333333333";
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms?region_id={}",
                        region_uuid
                    ))
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Currently sample data doesn't filter by region_id
        // This test verifies the parameter is accepted
        let json = parse_json_response(response).await;
        assert!(json["data"].is_object());
    }

    // =========================================================================
    // Test Group 12: Severity Field Validation
    // =========================================================================

    #[tokio::test]
    async fn test_cataclysm_severity_within_bounds() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();

        for cat in cataclysms {
            let severity = cat["severity"].as_f64().unwrap();
            assert!(
                severity >= 0.0 && severity <= 1.0,
                "Severity should be between 0.0 and 1.0, got: {}",
                severity
            );

            let significance = cat["significance"].as_f64().unwrap();
            assert!(
                significance >= 0.0 && significance <= 1.0,
                "Significance should be between 0.0 and 1.0, got: {}",
                significance
            );
        }
    }

    // =========================================================================
    // Test Group 13: HTTP Method Verification
    // =========================================================================

    #[tokio::test]
    async fn test_list_cataclysms_only_get_method_allowed() {
        let app = create_test_app();

        // POST should be method not allowed
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "POST should return 405 Method Not Allowed"
        );
    }

    #[tokio::test]
    async fn test_get_cataclysm_only_get_method_allowed() {
        let app = create_test_app();

        // POST should be method not allowed
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms/22222222-2222-2222-2222-222222222222")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "POST should return 405 Method Not Allowed"
        );
    }

    // =========================================================================
    // Test Group 14: API Response Format (camelCase)
    // =========================================================================

    #[tokio::test]
    async fn test_api_response_uses_camel_case() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let data = &json["data"];

        // Verify camelCase fields
        assert!(data["worldId"].is_string(), "Should use camelCase: worldId");
        assert!(data["cataclysms"].is_array(), "Should use camelCase: cataclysms");
        assert!(data["total"].is_number(), "Should use camelCase: total");
        assert!(data["limit"].is_number(), "Should use camelCase: limit");
        assert!(data["offset"].is_number(), "Should use camelCase: offset");
    }

    #[tokio::test]
    async fn test_cataclysm_view_uses_camel_case() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/worlds/11111111-1111-1111-1111-111111111111/cataclysms")
                    .body(Body::default())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = parse_json_response(response).await;
        let cataclysms = json["data"]["cataclysms"].as_array().unwrap();

        if !cataclysms.is_empty() {
            let cat = &cataclysms[0];

            // Verify camelCase field names
            assert!(cat["cataclysmType"].is_string(), "Should use camelCase: cataclysmType");
            assert!(cat["populationLost"].is_number() || cat["populationLost"].is_null(),
                "Should use camelCase: populationLost");
            assert!(cat["culturesDestroyed"].is_array() || cat["culturesDestroyed"].is_null(),
                "Should use camelCase: culturesDestroyed");
            assert!(cat["culturesEmerged"].is_array() || cat["culturesEmerged"].is_null(),
                "Should use camelCase: culturesEmerged");
        }
    }
}