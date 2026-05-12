//! Static HTML page serving for multi-page web UI
//!
//! Serves landing page, map, timeline, and dashboard as static HTML files.
//! All API calls use relative paths (/api/...) for correct proxy routing.
//!
//! Resolution order for static files:
//!   1. `$WORLD_FACTORY_STATIC_DIR` environment variable (for dev/prod overrides)
//!   2. `<exe_dir>/web/static` (standard installed layout)
//!   3. Embedded content (compile-time fallback, only landing.html is embedded)

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::api::AppState;

use std::path::PathBuf;

/// Resolve the static files base directory.
///
/// Resolution order (first match wins):
///   1. `$WORLD_FACTORY_STATIC_DIR` environment variable (absolute path, dev/prod override)
///   2. `./web/static` — current working directory (local dev: `cargo run` from project root)
///   3. `<exe_dir>/web/static` — next to the binary (production install / Docker)
///
/// This ordering means local `cargo run --features api --server` works without any env var,
/// and production deployments using the standard `COPY web/static /app/web/static` in
/// Docker also work because the binary at `/app/world_generator` finds `/app/web/static`.
fn static_base_dir() -> PathBuf {
    if let Ok(env_dir) = std::env::var("WORLD_FACTORY_STATIC_DIR") {
        if !env_dir.is_empty() {
            return PathBuf::from(env_dir);
        }
    }

    // Try CWD first — works for `cargo run` from project root
    let cwd_static = PathBuf::from(".").join("web").join("static");
    if cwd_static.join("landing.html").exists() {
        return cwd_static;
    }

    // Fall back to executable's directory — works for installed binaries
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("web")
        .join("static")
}

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

/// Get the static HTML file path for a page.
fn static_file_path(page: &str) -> PathBuf {
    static_base_dir().join(page)
}

/// Load HTML content from static file.
/// Returns Ok(content) on success, Err(StatusCode) on failure.
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
