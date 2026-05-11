//! Static HTML page serving for multi-page web UI
//!
//! Serves landing page, map, timeline, and dashboard as static HTML files.
//! All API calls use relative paths (/api/...) for correct proxy routing.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::api::AppState;

use std::path::PathBuf;

/// Register static page routes under the root router
pub fn routes() -> Router<AppState> {
    Router::new()
        // Landing page: GET /
        .route("/", get(serve_landing_page))
        // World overview: GET /worlds/:id
        .route("/worlds/{id}", get(serve_world_overview))
        // Map view: GET /worlds/:id/map
        .route("/worlds/{id}/map", get(serve_map_page))
        // Timeline view: GET /worlds/:id/timeline
        .route("/worlds/{id}/timeline", get(serve_timeline_page))
        // Dashboard view: GET /worlds/:id/dashboard
        .route("/worlds/{id}/dashboard", get(serve_dashboard_page))
}

/// Get the static HTML file path for a page
fn static_file_path(page: &str) -> PathBuf {
    let base = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("web")
        .join("static");
    base.join(page)
}

/// Load HTML content from static file
async fn load_html(page: &str) -> Result<String, StatusCode> {
    let path = static_file_path(page);
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// GET / - Landing page with world list and create form
async fn serve_landing_page() -> impl IntoResponse {
    match load_html("landing.html").await {
        Ok(html) => Html(html).into_response(),
        Err(status) => (status, "Landing page not found").into_response(),
    }
}

/// GET /worlds/:id - World overview with tabs (overview, map, timeline, dashboard)
async fn serve_world_overview(Path(world_id): Path<String>) -> impl IntoResponse {
    // For world overview, serve the map page with full tabbed interface
    // The overview page includes all views accessible via JS routing
    match load_html("map.html").await {
        Ok(html) => {
            // Inject world_id as window.WORLD_ID for the page to use
            let world_id_js = format!("window.WORLD_ID = '{}';", world_id);
            let html = html.replace("</script>", &format!("{}\n</script>", world_id_js));
            Html(html).into_response()
        }
        Err(status) => (status, "World page not found").into_response(),
    }
}

/// GET /worlds/:id/map - Map view with zoom/pan and export
async fn serve_map_page(Path(world_id): Path<String>) -> impl IntoResponse {
    match load_html("map.html").await {
        Ok(html) => {
            // Inject world_id as window.WORLD_ID for the page to use
            // (window.WORLD_ID is accessed by map.html's parseParams function)
            let world_id_js = format!("window.WORLD_ID = '{}';", world_id);
            let html = html.replace("</script>", &format!("{}\n</script>", world_id_js));
            Html(html).into_response()
        }
        Err(status) => (status, "Map page not found").into_response(),
    }
}

/// GET /worlds/:id/timeline - Timeline view with event filtering
async fn serve_timeline_page(Path(world_id): Path<String>) -> impl IntoResponse {
    match load_html("timeline.html").await {
        Ok(html) => {
            let html = html.replace("demo-world-1", &world_id);
            Html(html).into_response()
        }
        Err(status) => (status, "Timeline page not found").into_response(),
    }
}

/// GET /worlds/:id/dashboard - Dashboard with charts and stats
async fn serve_dashboard_page(Path(world_id): Path<String>) -> impl IntoResponse {
    match load_html("dashboard.html").await {
        Ok(html) => {
            let html = html.replace("demo-world-1", &world_id);
            Html(html).into_response()
        }
        Err(status) => (status, "Dashboard page not found").into_response(),
    }
}

/// Add cache control headers for static assets
pub fn cache_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "max-age=3600".parse().unwrap());
    headers
}
