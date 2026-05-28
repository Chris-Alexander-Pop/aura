use serde_json::{json, Value};
use std::io::{self, Write};
use std::sync::OnceLock;
use tokio::sync::broadcast;

static NOTIFY_TX: OnceLock<broadcast::Sender<String>> = OnceLock::new();

/// Initialize the global notification bus (call once at startup).
pub fn init(tx: broadcast::Sender<String>) {
    let _ = NOTIFY_TX.set(tx);
}

/// Emit a push event to WebSocket clients (`{ method, params }`) and GTK stdin (wrapped as JSON-RPC).
pub fn emit(method: &str, params: Value) {
    let Some(tx) = NOTIFY_TX.get() else {
        return;
    };
    let msg = json!({ "method": method, "params": params }).to_string();
    let _ = tx.send(msg);
}

/// Spawn a task that writes notifications to stdout for the GTK sidecar client.
pub fn spawn_stdout_forwarder(mut rx: broadcast::Receiver<String>) {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if let Err(e) = write_stdout_notification(&msg) {
                        tracing::warn!("stdout notify failed: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!("notify lagged, skipped {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn write_stdout_notification(ws_msg: &str) -> io::Result<()> {
    let v: Value = serde_json::from_str(ws_msg)?;
    let full = json!({
        "jsonrpc": "2.0",
        "method": v.get("method").cloned().unwrap_or(Value::Null),
        "params": v.get("params").cloned().unwrap_or(Value::Null),
    });
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", serde_json::to_string(&full)?)?;
    stdout.flush()?;
    Ok(())
}

/// Initialize a no-op notification bus for integration tests.
pub fn init_for_tests() {
    let (tx, _rx) = broadcast::channel(8);
    let _ = NOTIFY_TX.set(tx);
}
