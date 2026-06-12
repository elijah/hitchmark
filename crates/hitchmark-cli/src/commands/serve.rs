//! `hk serve` — lightweight HTTP API server.
//!
//! Exposes the hitchmark link store over a local REST API so that
//! browser extensions, editors, and the Obsidian plugin can query
//! and mutate links without spawning a subprocess per request.
//!
//! Default: `http://127.0.0.1:2701`
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET  | `/`                | Web dashboard (browser UI) |
//! | GET  | `/health`          | Liveness check |
//! | GET  | `/links?uri=<uri>` | List links for a resource |
//! | GET  | `/links/all`       | List all links (used by dashboard) |
//! | POST | `/links`           | Create a bidirectional link |
//! | DELETE | `/links`         | Remove a bidirectional link |
//! | GET  | `/bookmarks`       | List all bookmarks |
//! | GET  | `/uri?path=<path>` | Convert file path → hook:// URI |
//! | GET  | `/open?uri=<uri>` | Resolve and open a hook:// URI via OS |
//! | GET  | `/purple?path=<path>` | Generate purple numbers for a file |

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Query, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use hitchmark_core::{Error as CoreError, LinkStore};
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
    /// Optional PID file path
    pub pid_file: Option<String>,
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

// ── dashboard ─────────────────────────────────────────────────────────────────

const DASHBOARD_HTML: &str = include_str!("../dashboard.html");

async fn dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
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

async fn list_all_links_handler(State(store): State<SharedStore>) -> impl IntoResponse {
    let store = store.lock().unwrap();
    match store.list_all_links() {
        Ok(links) => (StatusCode::OK, Json(serde_json::to_value(links).unwrap())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_bookmarks_handler(State(store): State<SharedStore>) -> impl IntoResponse {
    let store = store.lock().unwrap();
    match store.list_bookmarks() {
        Ok(bms) => (StatusCode::OK, Json(serde_json::to_value(bms).unwrap())).into_response(),
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

/// `GET /open?uri=<uri>` — resolve and open a hook:// URI via the OS.
///
/// Used by the Windows tray and browser extensions to open URIs without
/// spawning a subprocess. Returns 200 on success, 400 on bad URI, 404 if
/// the resolved file does not exist.
async fn open_uri(
    State(store): State<SharedStore>,
    Query(q): Query<UriQuery>,
) -> impl IntoResponse {
    use hitchmark_core::{HookUri, UriType};

    let parsed = match HookUri::parse(&q.uri) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid URI: {e}") })),
            )
                .into_response();
        }
    };

    let path_to_open: String = match parsed.uri_type {
        UriType::File(ref p) => p.to_string_lossy().into_owned(),
        UriType::Bookmark(ref id) => {
            let store = store.lock().unwrap();
            match store.lookup_bookmark(id) {
                Ok(Some(p)) => p,
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({ "error": "Bookmark not found" })),
                    )
                        .into_response();
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                        .into_response();
                }
            }
        }
        UriType::XCallbackUrl(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "x-callback-url URIs cannot be opened via /open" })),
            )
                .into_response();
        }
    };

    match opener::open(&path_to_open) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "opened": path_to_open }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to open: {e}") })),
        )
            .into_response(),
    }
}

async fn purple_for_file(Query(q): Query<PathQuery>) -> impl IntoResponse {
    use hitchmark_core::{split_paragraphs, PurpleNumberGenerator};
    use std::fs;

    let content = match fs::read_to_string(&q.path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Cannot read file: {e}") })),
            )
                .into_response();
        }
    };

    let mut gen = PurpleNumberGenerator::new();
    let paragraphs = split_paragraphs(&content);
    let result: Vec<serde_json::Value> = paragraphs
        .iter()
        .filter_map(|text| {
            gen.generate(text).ok().map(|id| {
                serde_json::json!({
                    "id": id.as_str(),
                    "text": text,
                })
            })
        })
        .collect();

    (StatusCode::OK, Json(serde_json::Value::Array(result))).into_response()
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
        .route("/", get(dashboard))
        .route("/health", get(health))
        .route("/links", get(list_links).post(create_link).delete(delete_link))
        .route("/links/all", get(list_all_links_handler))
        .route("/bookmarks", get(list_bookmarks_handler))
        .route("/uri", get(file_to_uri))
        .route("/open", get(open_uri))
        .route("/purple", get(purple_for_file))
        .layer(cors_layer())
        .with_state(shared);

    println!("🔗 Hitchmark server listening on http://{addr}");
    println!("   Press Ctrl-C to stop.");

    let _pid_guard = PidFileGuard::create(args.pid_file.as_deref())?;

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
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = sigterm.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    }
}

struct PidFileGuard {
    path: Option<PathBuf>,
}

impl PidFileGuard {
    fn create(path: Option<&str>) -> anyhow::Result<Self> {
        let Some(raw) = path else {
            return Ok(Self { path: None });
        };
        let pid_path = PathBuf::from(raw);
        if let Some(parent) = pid_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&pid_path, format!("{}\n", std::process::id()))?;
        Ok(Self {
            path: Some(pid_path),
        })
    }
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("warning: failed to remove pid file {}: {e}", path.display());
            }
        }
    }
}
