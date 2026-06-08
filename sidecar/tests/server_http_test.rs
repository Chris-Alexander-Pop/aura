//! HTTP + WebSocket integration tests for `server.rs` / `notify.rs`.
//!
//! Binds `127.0.0.1:0` only — never :9080 — so a dev `ags-sidecar` can keep running.

mod common;

use ags_sidecar::server;
use common::{assert_safe_rpc_method, call_method, test_registry};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn spawn_ephemeral_server(
    registry: Arc<Mutex<ags_sidecar::services::ServiceRegistry>>,
    notify_tx: server::NotifyTx,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>, TempDir) {
    let ui_dist = TempDir::new().expect("temp ui dir");
    let (listener, addr) = server::bind_ephemeral().await.expect("bind 127.0.0.1:0");
    let ui_path = ui_dist.path().to_path_buf();
    let handle = tokio::spawn(async move {
        server::serve(listener, registry, notify_tx, ui_path)
            .await
            .expect("serve");
    });
    (addr, handle, ui_dist)
}

fn with_temp_storage_db<F: FnOnce()>(f: F) {
    let db = std::env::temp_dir().join(format!(
        "ags-http-it-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_file(&db);
    std::env::set_var("AURA_STORAGE_DB", db.to_string_lossy().to_string());
    f();
    let _ = std::fs::remove_file(&db);
}

/// Per-test notify bus (do not share `notify::init_for_tests` across parallel WS tests).
fn test_notify_bus() -> broadcast::Sender<String> {
    let (tx, _) = broadcast::channel(32);
    tx
}

type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

async fn ws_wait_until_ready(
    sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    stream: &mut WsStream,
) {
    sink.send(Message::Ping(vec![]))
        .await
        .expect("ws ping");
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("ping timeout")
            .expect("stream open")
            .expect("ws frame");
        if matches!(frame, Message::Pong(_)) {
            return;
        }
    }
}

async fn ws_recv_method(ws: &mut WsStream, method: &str) -> Value {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("notify timeout")
            .expect("stream open")
            .expect("ws frame");
        if let Message::Text(text) = frame {
            let v: Value = serde_json::from_str(&text).expect("json");
            if v.get("method").and_then(|m| m.as_str()) == Some(method) {
                return v;
            }
        }
    }
}

async fn ws_push_round_trip(
    notify_tx: &broadcast::Sender<String>,
    method: &str,
    params: Value,
) -> Value {
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx.clone()).await;

    let ws_url = format!("ws://{addr}/ws");
    let (ws, _) = connect_async(&ws_url).await.expect("ws connect");
    let (mut sink, mut stream) = ws.split();
    ws_wait_until_ready(&mut sink, &mut stream).await;

    let payload = json!({ "method": method, "params": params }).to_string();
    notify_tx
        .send(payload)
        .expect("broadcast send");

    let v = ws_recv_method(&mut stream, method).await;
    assert_eq!(v["method"], method);

    drop(sink);
    drop(stream);
    server_task.abort();
    v
}

#[tokio::test]
async fn http_get_openapi_json() {
    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .get(format!("http://{addr}/api/openapi.json"))
        .send()
        .await
        .expect("GET openapi")
        .json()
        .await
        .expect("json");

    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"]["/api/{method}"].is_object());
    assert!(body["paths"]["/api/meta"].is_object());
    assert!((body["x-rpc-methods"].as_array().unwrap().len()) >= 280);

    server_task.abort();
}

#[tokio::test]
async fn http_get_docs_html() {
    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{addr}/docs"))
        .send()
        .await
        .expect("GET docs");
    assert!(res.status().is_success());
    let html = res.text().await.expect("html");
    assert!(html.contains("swagger-ui"));
    assert!(html.contains("/api/openapi.json"));

    server_task.abort();
}

#[tokio::test]
async fn http_get_sidecar_get_version() {
    assert_safe_rpc_method("Sidecar.GetVersion");
    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/Sidecar.GetVersion");
    let body: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .expect("GET")
        .json()
        .await
        .expect("json");

    assert_eq!(body["ok"], true);
    assert!(body["data"].get("version").is_some());

    server_task.abort();
}

#[tokio::test]
async fn http_post_storage_get_temp_db() {
    assert_safe_rpc_method("Storage.Get");
    with_temp_storage_db(|| {});
    let ns = format!("http_ns_{}", std::process::id());
    let registry = Arc::new(Mutex::new(test_registry()));
    {
        let reg = registry.lock().await;
        call_method(
            &reg,
            "Storage.Set",
            Some(json!({ "namespace": ns, "key": "k", "value": { "a": 1 } })),
        )
        .await
        .expect("Storage.Set");
    }

    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/Storage.Get");
    let body: serde_json::Value = client
        .post(&url)
        .json(&json!({ "namespace": ns, "key": "k" }))
        .send()
        .await
        .expect("POST")
        .json()
        .await
        .expect("json");

    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["value"]["a"], 1);

    server_task.abort();
}

#[tokio::test]
async fn websocket_receives_notify_emit() {
    let notify_tx = test_notify_bus();
    let v = ws_push_round_trip(&notify_tx, "Test.Push", json!({ "hello": true })).await;
    assert_eq!(v["params"]["hello"], true);
}

#[tokio::test]
async fn websocket_receives_calendar_events_changed() {
    let notify_tx = test_notify_bus();
    let v = ws_push_round_trip(
        &notify_tx,
        "Calendar.EventsChanged",
        json!({ "reason": "create" }),
    )
    .await;
    assert_eq!(v["params"]["reason"], "create");
}

/// Synthetic push only (`notify::emit` payload). Live sysfs → `Power` poll → WS is `[~]` in BACKEND_TODO §0.6.
#[tokio::test]
async fn websocket_receives_power_battery_state() {
    let notify_tx = test_notify_bus();
    let v = ws_push_round_trip(
        &notify_tx,
        "Power.BatteryState",
        json!({ "percent": 72, "charging": false, "time_remaining": "3h" }),
    )
    .await;
    assert_eq!(v["params"]["percent"], 72);
    assert_eq!(v["params"]["charging"], false);
    assert_eq!(v["params"]["time_remaining"], "3h");
}

#[cfg(not(feature = "offensive-security"))]
#[tokio::test]
async fn http_get_api_meta_offensive_disabled_by_default() {
    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .get(format!("http://{addr}/api/meta"))
        .send()
        .await
        .expect("GET meta")
        .json()
        .await
        .expect("json");

    assert_eq!(body["offensiveEnabled"], false);

    server_task.abort();
}

#[cfg(feature = "offensive-security")]
#[tokio::test]
async fn http_get_api_meta_offensive_enabled_when_feature_on() {
    let (notify_tx, _) = tokio::sync::broadcast::channel(8);
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx).await;

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .get(format!("http://{addr}/api/meta"))
        .send()
        .await
        .expect("GET meta")
        .json()
        .await
        .expect("json");

    assert_eq!(body["offensiveEnabled"], true);

    server_task.abort();
}

#[tokio::test]
async fn websocket_receives_logs_line_after_follow_logs() {
    let fixture = format!(
        "{}/tests/fixtures/logs/follow_lines.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    std::env::set_var("AURA_LOGS_FOLLOW_FIXTURE", &fixture);

    let _ = test_registry();
    let notify_tx = ags_sidecar::notify::init_for_tests();
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry.clone(), notify_tx.clone()).await;

    let ws_url = format!("ws://{addr}/ws");
    let (ws, _) = connect_async(&ws_url).await.expect("ws connect");
    let (mut sink, mut stream) = ws.split();
    ws_wait_until_ready(&mut sink, &mut stream).await;

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(format!("http://{addr}/api/Logs.FollowLogs"))
        .json(&json!({ "max_lines": 2 }))
        .send()
        .await
        .expect("POST FollowLogs")
        .json()
        .await
        .expect("json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["following"], true);

    let v = ws_recv_method(&mut stream, "Logs.Line").await;
    assert!(v.get("params").and_then(|p| p.get("line")).is_some());

    drop(sink);
    drop(stream);
    server_task.abort();
    std::env::remove_var("AURA_LOGS_FOLLOW_FIXTURE");
}
