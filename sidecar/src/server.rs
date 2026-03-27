use crate::services::ServiceRegistry;
use crate::types::JsonRpcRequest;
use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::Method,
    response::{IntoResponse, Response},
    routing::{any, get, post},
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

pub async fn run(registry: Arc<Mutex<ServiceRegistry>>) -> Result<()> {
    let (notify_tx, _) = broadcast::channel::<String>(256);

    let state = AppState {
        registry,
        notify_tx,
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Resolve ui/dist relative to the sidecar binary location
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Walk up to workspace root and find ui/dist
    let ui_dist = exe_dir
        .ancestors()
        .find_map(|p| {
            let candidate = p.join("ui").join("dist");
            if candidate.exists() {
                Some(candidate)
            } else {
                None
            }
        })
        .unwrap_or_else(|| exe_dir.join("ui").join("dist"));

    tracing::info!("Serving UI from: {:?}", ui_dist);

    let app = Router::new()
        // WebSocket push channel
        .route("/ws", get(ws_handler))
        // REST: GET /api/Service.Method
        .route("/api/:method", get(handle_get).post(handle_post))
        // Static UI files
        .nest_service("/", ServeDir::new(&ui_dist))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 9080));
    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Push a JSON notification to all connected WebSocket clients.
pub fn push_notification(tx: &NotifyTx, method: &str, params: Value) {
    let msg = json!({ "method": method, "params": params }).to_string();
    let _ = tx.send(msg);
}

// ── REST handlers ────────────────────────────────────────────────────────────

async fn handle_get(
    State(state): State<AppState>,
    Path(method): Path<String>,
) -> Response {
    call_service(state, method, None).await
}

async fn handle_post(
    State(state): State<AppState>,
    Path(method): Path<String>,
    body: Option<Json<Value>>,
) -> Response {
    let params = body.map(|Json(v)| v);
    call_service(state, method, params).await
}

async fn call_service(state: AppState, method: String, params: Option<Value>) -> Response {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: method.clone(),
        params,
        id: Some(Value::Number(0.into())),
    };

    let result = {
        let registry = state.registry.lock().await;
        registry.handle_request(request).await
    };

    match result {
        Ok(value) => Json(json!({ "ok": true, "data": value })).into_response(),
        Err(e) => {
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
}
