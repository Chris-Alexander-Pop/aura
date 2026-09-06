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

#[tokio::test]
async fn logs_get_system_logs_mocked() {
    use common::ExecFixtureGuard;
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Logs.GetSystemLogs",
        Some(json!({ "lines": 10 })),
    )
    .await
    .expect("Logs.GetSystemLogs");
    assert!(value.get("logs").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn logs_filter_logs_mocked() {
    use common::ExecFixtureGuard;
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Logs.FilterLogs",
        Some(json!({ "level": "info", "lines": 5 })),
    )
    .await
    .expect("Logs.FilterLogs");
    assert!(value.get("logs").and_then(|v| v.as_str()).is_some());
}
