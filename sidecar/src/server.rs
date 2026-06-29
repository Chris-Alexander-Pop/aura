use crate::services::ServiceRegistry;
use crate::services::MethodNotFound;
use crate::types::JsonRpcRequest;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{header, Method},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

pub type NotifyTx = broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Mutex<ServiceRegistry>>,
    pub notify_tx: NotifyTx,
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

pub fn app_router(state: AppState, ui_dist: &std::path::Path) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/ws", get(ws_handler))
        .route("/docs", get(docs_handler))
        .route("/api/openapi.json", get(openapi_handler))
        .route("/api/meta", get(meta_handler))
        .route("/api/:method", get(handle_get).post(handle_post))
        .nest_service("/", ServeDir::new(ui_dist))
        .layer(cors)
        .with_state(state)
}

/// Bind `127.0.0.1:0` for integration tests (never competes with a dev sidecar on :9080).
pub async fn bind_ephemeral() -> Result<(tokio::net::TcpListener, std::net::SocketAddr)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    Ok((listener, addr))
}

pub async fn serve(
    listener: tokio::net::TcpListener,
    registry: Arc<Mutex<ServiceRegistry>>,
    notify_tx: NotifyTx,
    ui_dist: std::path::PathBuf,
) -> Result<()> {
    let state = AppState {
        registry,
        notify_tx,
    };
    let app = app_router(state, &ui_dist);
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn run(registry: Arc<Mutex<ServiceRegistry>>, notify_tx: NotifyTx) -> Result<()> {
    let ui_dist = resolve_ui_dist();
    tracing::info!("Serving UI from: {:?}", ui_dist);
    tracing::info!("HTTP server listening on {}", DEFAULT_HTTP_ADDR);

    let listener = tokio::net::TcpListener::bind(DEFAULT_HTTP_ADDR).await?;
    serve(listener, registry, notify_tx, ui_dist).await
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

async fn meta_handler() -> Response {
    #[cfg(feature = "offensive-security")]
    let offensive_enabled = true;
    #[cfg(not(feature = "offensive-security"))]
    let offensive_enabled = false;

    Json(json!({
        "offensiveEnabled": offensive_enabled,
        "version": env!("CARGO_PKG_VERSION"),
    }))
    .into_response()
}

async fn handle_get(
    State(state): State<AppState>,
    Path(method): Path<String>,
) -> Response {
    call_service(state, method, None).await
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
                    axum::http::StatusCode::BAD_REQUEST,
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
                    axum::http::StatusCode::NOT_FOUND,
                    Json(json!({
                        "ok": false,
                        "error": not_found.to_string(),
                        "code": "method_not_found",
                        "method": not_found.0,
                    })),
                )
                    .into_response();
            }
            #[cfg(feature = "offensive-security")]
            if let Some(limited) =
                e.downcast_ref::<crate::services::offensive_policy::RateLimited>()
            {
                return (
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "ok": false,
                        "error": limited.to_string(),
                        "code": "rate_limited",
                    })),
                )
                    .into_response();
            }
            let msg = e.to_string();
            tracing::warn!("Service error for {}: {}", method, msg);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": msg })),
            )
                .into_response()
        }
    }
}

// ── WebSocket handler ────────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    crate::services::logs::ws_client_connected();
    let mut rx = state.notify_tx.subscribe();

    loop {
        tokio::select! {
            // Forward push notifications to client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            // Handle client messages (ping/close)
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
