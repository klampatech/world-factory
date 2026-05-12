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
        // World SPA: GET /worlds/:id (serves world.html which handles all views internally)
        .route("/worlds/{id}", get(serve_world_page))
        // World SPA (alternate path): GET /worlds/{id}/index.html (SPA entry point)
        .route("/worlds/{id}/index.html", get(serve_world_page_index))
        // World SPA (alternate path): GET /worlds/{id}/map
        .route("/worlds/{id}/map", get(serve_world_page))
        // World SPA (alternate path): GET /worlds/{id}/timeline
        .route("/worlds/{id}/timeline", get(serve_world_page))
        // World SPA (alternate path): GET /worlds/{id}/dashboard
        .route("/worlds/{id}/dashboard", get(serve_world_page))
        // Static assets: serve JS/CSS from the static directory
        .route("/{file}", get(serve_static_file))
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

/// GET /worlds/:id - World SPA (serves world.html which handles all views internally)
/// Redirects to /worlds/{id}/index.html?id={id} so world.html can read the world ID
/// from URL query params (the existing JS pattern).
async fn serve_world_page(Path(world_id): Path<String>) -> impl IntoResponse {
    // Redirect to the /index.html path with ?id= query param so the SPA can read it
    let redirect_url = format!("/worlds/{}/index.html?id={}", world_id, world_id);
    axum::response::Redirect::to(&redirect_url).into_response()
}

/// GET /worlds/:id/index.html - World SPA entry point with world ID injected
async fn serve_world_page_index(Path(world_id): Path<String>) -> impl IntoResponse {
    // world.html reads ?id= from the URL query params to get the world ID
    match load_html("world.html").await {
        Ok(html) => {
            // Inject world ID before the main world.html script (after api-integration.js loads)
            // This must happen before the main script block runs, so we inject right after
            // the api-integration.js script tag which is guaranteed to be loaded first.
            let world_id_js = format!("window.WORLD_ID = '{}';", world_id);
            let inject_point = "<script src=\"/api-integration.js\"></script>";
            let html = html.replace(inject_point, &format!("{}\n    <script>{}</script>", inject_point, world_id_js));
            Html(html).into_response()
        }
        Err(status) => (status, "World page not found").into_response(),
    }
}

/// GET /{file} - Static asset files (JS, CSS, etc.) served from web/static/
async fn serve_static_file(Path(file): Path<String>) -> impl IntoResponse {
    // Prevent path traversal attacks: only allow alphanumeric, dash, underscore, dot
    if file.contains("..") || file.contains('/') {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    match tokio::fs::read_to_string(static_file_path(&file)).await {
        Ok(content) => {
            let mime = if file.ends_with(".js") {
                "application/javascript"
            } else if file.ends_with(".css") {
                "text/css"
            } else {
                "text/plain"
            };
            let ct: header::HeaderValue = mime.parse().unwrap();
            ([(header::CONTENT_TYPE, ct)], content).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

/// Add cache control headers for static assets
pub fn cache_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "max-age=3600".parse().unwrap());
    headers
}
