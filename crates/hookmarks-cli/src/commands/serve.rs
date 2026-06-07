//! `hk serve` — lightweight HTTP API server.
//!
//! Exposes the hookmarks link store over a local REST API so that
//! browser extensions, editors, and the Obsidian plugin can query
//! and mutate links without spawning a subprocess per request.
//!
//! Default: `http://127.0.0.1:2701`
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET  | `/health`          | Liveness check |
//! | GET  | `/links?uri=<uri>` | List links for a resource |
//! | POST | `/links`           | Create a bidirectional link |
//! | DELETE | `/links`         | Remove a bidirectional link |
//! | GET  | `/uri?path=<path>` | Convert file path → hook:// URI |

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Query, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use hookmarks_core::{Error as CoreError, LinkStore};
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

// ── shared state ─────────────────────────────────────────────────────────────

type SharedStore = Arc<Mutex<LinkStore>>;

// ── CLI args ─────────────────────────────────────────────────────────────────

/// Arguments for `hk serve`
pub struct ServeArgs {
    /// TCP port to listen on (default 2701)
    pub port: u16,
    /// Host/IP to bind (default 127.0.0.1)
    pub host: String,
}

// ── request / response types ─────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Deserialize)]
struct UriQuery {
    uri: String,
}

#[derive(Deserialize)]
struct PathQuery {
    path: String,
}

#[derive(Deserialize)]
struct CreateLinkBody {
    uri_a: String,
    uri_b: String,
    note: Option<String>,
}

#[derive(Deserialize)]
struct DeleteLinkBody {
    uri_a: String,
    uri_b: String,
}

// ── handlers ─────────────────────────────────────────────────────────────────

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn list_links(
    State(store): State<SharedStore>,
    Query(q): Query<UriQuery>,
) -> impl IntoResponse {
    let store = store.lock().unwrap();
    match store.list_links(&q.uri) {
        Ok(links) => (StatusCode::OK, Json(serde_json::to_value(links).unwrap())).into_response(),
        Err(CoreError::InvalidUri(_)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid URI" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn create_link(
    State(store): State<SharedStore>,
    Json(body): Json<CreateLinkBody>,
) -> impl IntoResponse {
    let store = store.lock().unwrap();
    match store.create_link(&body.uri_a, &body.uri_b, body.note.as_deref()) {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(CoreError::LinkAlreadyExists { .. }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "link already exists" })),
        )
            .into_response(),
        Err(CoreError::InvalidUri(_)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid URI" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn delete_link(
    State(store): State<SharedStore>,
    Json(body): Json<DeleteLinkBody>,
) -> impl IntoResponse {
    let store = store.lock().unwrap();
    match store.delete_link(&body.uri_a, &body.uri_b) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn file_to_uri(Query(q): Query<PathQuery>) -> impl IntoResponse {
    match crate::path::path_to_uri(&q.path) {
        Ok(uri) => (
            StatusCode::OK,
            Json(serde_json::json!({ "uri": uri.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// ── CORS ─────────────────────────────────────────────────────────────────────

fn cors_layer() -> CorsLayer {
    // Allow only local origins: Obsidian (app://) and localhost browsers.
    // This is intentionally restrictive — the server binds to 127.0.0.1 only
    // by default, providing a second layer of defense.
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            let s = origin.to_str().unwrap_or("");
            s.starts_with("app://obsidian.md")
                || s.starts_with("http://localhost:")
                || s.starts_with("http://127.0.0.1:")
        }))
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(tower_http::cors::Any)
}

// ── entry point ──────────────────────────────────────────────────────────────

/// Run the HTTP server. Blocks until interrupted (Ctrl-C).
pub fn execute(args: ServeArgs, store_path: &PathBuf) -> anyhow::Result<()> {
    let store = LinkStore::open(store_path)?;
    let shared = Arc::new(Mutex::new(store));

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {e}"))?;

    let app = Router::new()
        .route("/health", get(health))
        .route("/links", get(list_links).post(create_link).delete(delete_link))
        .route("/uri", get(file_to_uri))
        .layer(cors_layer())
        .with_state(shared);

    println!("🔗 Hookmarks server listening on http://{addr}");
    println!("   Press Ctrl-C to stop.");

    tokio::runtime::Runtime::new()?.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok::<_, anyhow::Error>(())
    })?;

    println!("\n✓ Server stopped.");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl-C handler");
}
