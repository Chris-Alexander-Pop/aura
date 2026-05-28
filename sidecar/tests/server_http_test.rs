//! HTTP + WebSocket integration tests for `server.rs` / `notify.rs`.
//!
//! Binds `127.0.0.1:0` only — never :9080 — so a dev `ags-sidecar` can keep running.

mod common;

use ags_sidecar::notify;
use ags_sidecar::server;
use common::{assert_safe_rpc_method, call_method, test_registry};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;
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
    let notify_tx = notify::init_for_tests();
    let registry = Arc::new(Mutex::new(test_registry()));
    let (addr, server_task, _ui) = spawn_ephemeral_server(registry, notify_tx.clone()).await;

    let ws_url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&ws_url).await.expect("ws connect");

    // Ensure the server handler subscribed before we emit (broadcast drops early sends).
    ws.send(Message::Ping(vec![])).await.expect("ping");
    loop {
        let frame = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
            .await
            .expect("ping timeout")
            .expect("stream item")
            .expect("ws frame");
        if matches!(frame, Message::Pong(_)) {
            break;
        }
    }

    notify::emit("Test.Push", json!({ "hello": true }));

    let msg = loop {
        let frame = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
            .await
            .expect("notify timeout")
            .expect("stream item")
            .expect("ws frame");
        if let Message::Text(_) = frame {
            break frame;
        }
    };

    if let Message::Text(text) = msg {
        let v: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(v["method"], "Test.Push");
        assert_eq!(v["params"]["hello"], true);
    } else {
        panic!("expected text frame, got {:?}", msg);
    }

    server_task.abort();
}
