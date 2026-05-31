//! Read-only `Logs.*` RPC response shapes.

mod common;

use common::{call_rpc, test_registry};
use serde_json::json;

#[tokio::test]
async fn logs_get_entry_schema() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Logs.Get",
        Some(json!({ "lines": 5 })),
    )
    .await
    .expect("Logs.Get");
    let entries = value.as_array().expect("array");
    if let Some(row) = entries.first() {
        assert!(row.get("message").and_then(|v| v.as_str()).is_some());
        assert!(row.get("level").and_then(|v| v.as_str()).is_some());
        assert!(row.get("timestamp").and_then(|v| v.as_str()).is_some());
        assert!(row.get("service").and_then(|v| v.as_str()).is_some());
    }
}
