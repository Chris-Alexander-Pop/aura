use crate::services::ServiceRegistry;
use crate::services::MethodNotFound;
use crate::types::JsonRpcRequest;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, Request, State,
    },
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;

pub type NotifyTx = broadcast::Sender<String>;

/// Header the React UI (and curl) must send on `/api/{method}` POST.
pub const HTTP_TOKEN_HEADER: &str = "x-aura-token";

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Mutex<ServiceRegistry>>,
    pub notify_tx: NotifyTx,
    pub http_token: Arc<str>,
}

/// Production bind address (stdio sidecar spawns HTTP here).
pub const DEFAULT_HTTP_ADDR: std::net::SocketAddr =
    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 9080);

/// Resolve `ui/dist` relative to the sidecar binary location.
pub fn resolve_ui_dist() -> std::path::PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    exe_dir
        .ancestors()
        .find_map(|p| {
            let candidate = p.join("ui").join("dist");
            if candidate.exists() {
                Some(candidate)
            } else {
                None
            }
        })
        .unwrap_or_else(|| exe_dir.join("ui").join("dist"))
}

/// Loopback http(s) origins (Vite on :5173 and the WebKit UI on :9080).
pub fn origin_is_loopback(origin: &str) -> bool {
    let origin = origin.trim();
    if origin.eq_ignore_ascii_case("null") {
        return false;
    }
    let rest = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"));
    let Some(rest) = rest else {
        return false;
    };
    let hostport = rest.split('/').next().unwrap_or("");
    let host = if let Some(h) = hostport.strip_prefix('[') {
        h.split(']').next().unwrap_or("")
    } else {
        hostport.split(':').next().unwrap_or("")
    };
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

pub fn generate_http_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn resolve_http_token() -> String {
    match std::env::var("AURA_HTTP_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => generate_http_token(),
    }
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    let aa = a.as_bytes();
    let bb = b.as_bytes();
    if aa.len() != bb.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in aa.iter().zip(bb.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn forbidden_origin() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "ok": false,
            "error": "origin not allowed",
            "code": "forbidden_origin",
        })),
    )
        .into_response()
}

fn unauthorized_token() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "ok": false,
            "error": "missing or invalid X-Aura-Token",
            "code": "unauthorized",
        })),
    )
        .into_response()
}

pub fn check_origin_header(origin: Option<&HeaderValue>) -> Result<(), Response> {
    let Some(raw) = origin else {
        return Ok(());
    };
    let Ok(s) = raw.to_str() else {
        return Err(forbidden_origin());
    };
    if origin_is_loopback(s) {
        Ok(())
    } else {
        Err(forbidden_origin())
    }
}

async fn require_loopback_origin(request: Request, next: Next) -> Response {
    if let Err(resp) = check_origin_header(request.headers().get(header::ORIGIN)) {
        return resp;
    }
    next.run(request).await
}

async fn require_http_token(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let provided = request
        .headers()
        .get(HTTP_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !constant_time_eq(provided, state.http_token.as_ref()) {
        return unauthorized_token();
    }
    next.run(request).await
}

pub fn app_router(state: AppState, ui_dist: &std::path::Path) -> Router {
    let token_layer = middleware::from_fn_with_state(state.clone(), require_http_token);
    let origin_layer = middleware::from_fn(require_loopback_origin);

    let rpc = Router::new()
        .route("/api/:method", post(handle_post))
        .layer(token_layer)
        .layer(origin_layer.clone());

    Router::new()
        .route("/ws", get(ws_handler))
        .route("/docs", get(docs_handler))
        .route("/api/openapi.json", get(openapi_handler))
        .route("/api/meta", get(meta_handler).layer(origin_layer))
        .route("/api/:method", get(rpc_get_not_allowed))
        .merge(rpc)
        .nest_service("/", ServeDir::new(ui_dist))
        .with_state(state)
}

/// Bind `127.0.0.1:0` for integration tests (never competes with a dev sidecar on :9080).
pub async fn bind_ephemeral() -> Result<(tokio::net::TcpListener, std::net::SocketAddr)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    Ok((listener, addr))
}

fn persist_http_token_runtime(token: &str) {
    let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from) else {
        return;
    };
    let path = dir.join("aura-http-token");
    if std::fs::write(&path, token).is_err() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}

pub async fn serve(
    listener: tokio::net::TcpListener,
    registry: Arc<Mutex<ServiceRegistry>>,
    notify_tx: NotifyTx,
    ui_dist: std::path::PathBuf,
) -> Result<()> {
    serve_with_token(
        listener,
        registry,
        notify_tx,
        ui_dist,
        resolve_http_token().into(),
    )
    .await
}

async fn serve_with_token(
    listener: tokio::net::TcpListener,
    registry: Arc<Mutex<ServiceRegistry>>,
    notify_tx: NotifyTx,
    ui_dist: std::path::PathBuf,
    http_token: Arc<str>,
) -> Result<()> {
    let state = AppState {
        registry,
        notify_tx,
        http_token,
    };
    let app = app_router(state, &ui_dist);
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn run(registry: Arc<Mutex<ServiceRegistry>>, notify_tx: NotifyTx) -> Result<()> {
    let ui_dist = resolve_ui_dist();
    let http_token: Arc<str> = resolve_http_token().into();
    persist_http_token_runtime(http_token.as_ref());
    tracing::info!("Serving UI from: {:?}", ui_dist);
    tracing::info!(
        "HTTP server listening on {} (RPC is POST + X-Aura-Token; see /api/meta)",
        DEFAULT_HTTP_ADDR
    );

    let listener = tokio::net::TcpListener::bind(DEFAULT_HTTP_ADDR).await?;
    serve_with_token(listener, registry, notify_tx, ui_dist, http_token).await
}

// ── REST handlers ────────────────────────────────────────────────────────────

async fn openapi_handler() -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        crate::openapi::OPENAPI_JSON,
    )
        .into_response()
}

async fn docs_handler() -> Html<&'static str> {
    Html(crate::openapi::DOCS_HTML)
}

async fn meta_handler(State(state): State<AppState>) -> Response {
    let extensions = {
        let registry = state.registry.lock().await;
        registry.loaded_extensions().to_vec()
    };
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "extensions": extensions,
        "http_token": state.http_token.as_ref(),
    }))
    .into_response()
}

async fn rpc_get_not_allowed() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(json!({
            "ok": false,
            "error": "RPC is POST-only; GET is not accepted",
            "code": "method_not_allowed",
        })),
    )
        .into_response()
}

async fn handle_post(
    State(state): State<AppState>,
    Path(method): Path<String>,
    body: Bytes,
) -> Response {
    let params = if body.is_empty() {
        None
    } else {
        match serde_json::from_slice::<Value>(&body) {
            Ok(v) => Some(v),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("invalid JSON: {e}"),
                        "code": "invalid_json",
                    })),
                )
                    .into_response();
            }
        }
    };
    call_service(state, method, params).await
}

async fn call_service(state: AppState, method: String, params: Option<Value>) -> Response {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: method.clone(),
        params,
        id: Some(Value::Number(0.into())),
    };

    let handler = {
        let registry = state.registry.lock().await;
        registry.handler_for(&method)
    };

    let result = match handler {
        Some(handler) => {
            let started = std::time::Instant::now();
            let params = request.params.clone();
            let out = handler(params.clone()).await;
            crate::utils::rpc_log::log_rpc_completed(&method, params.as_ref(), started.elapsed());
            out
        }
        None => Err(anyhow::Error::new(MethodNotFound(method.clone()))),
    };

    match result {
        Ok(value) => Json(json!({ "ok": true, "data": value })).into_response(),
        Err(e) => {
            if let Some(not_found) = e.downcast_ref::<MethodNotFound>() {
                tracing::debug!("Unknown RPC method: {}", not_found.0);
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "ok": false,
                        "error": not_found.to_string(),
                        "code": "method_not_found",
                        "method": not_found.0,
                    })),
                )
                    .into_response();
            }
            let msg = e.to_string();
            tracing::warn!("Service error for {}: {}", method, msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": msg })),
            )
                .into_response()
        }
    }
}

// ── WebSocket handler ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WsQuery {
    token: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsQuery>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    if let Err(resp) = check_origin_header(headers.get(header::ORIGIN)) {
        return resp;
    }
    let provided = q.token.as_deref().unwrap_or("");
    if !constant_time_eq(provided, state.http_token.as_ref()) {
        return unauthorized_token();
    }
    ws.on_upgrade(move |socket| handle_ws(socket, state))
        .into_response()
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    crate::services::logs::ws_client_connected();
    let mut rx = state.notify_tx.subscribe();

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }
    crate::services::logs::ws_client_disconnected();
}

#[cfg(test)]
mod origin_tests {
    use super::origin_is_loopback;

    #[test]
    fn loopback_origins_accepted() {
        assert!(origin_is_loopback("http://localhost:5173"));
        assert!(origin_is_loopback("http://127.0.0.1:9080"));
        assert!(origin_is_loopback("http://[::1]:9080"));
        assert!(origin_is_loopback("https://localhost"));
    }

    #[test]
    fn remote_origins_rejected() {
        assert!(!origin_is_loopback("https://evil.example"));
        assert!(!origin_is_loopback("http://192.168.1.10"));
        assert!(!origin_is_loopback("null"));
        assert!(!origin_is_loopback("file://"));
    }
}
