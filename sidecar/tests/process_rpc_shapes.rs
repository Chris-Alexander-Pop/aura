//! Read-only `Process.*` RPC response shapes.

mod common;

use ags_sidecar::services::processes::kill_confirmation_token;
use common::{call_method, is_denied_rpc_method, test_registry};
use serde_json::json;

#[test]
fn process_kill_blocked_in_fast_harness() {
    assert!(is_denied_rpc_method("Process.Kill"));
}

#[test]
fn kill_confirmation_token_is_unpredictable() {
    let a = kill_confirmation_token(100);
    let b = kill_confirmation_token(100);
    assert_eq!(a, b);
    assert_ne!(a, kill_confirmation_token(101));
    assert_eq!(a.len(), 64);
    assert_ne!(a, "confirm-kill-100");
}

#[tokio::test]
async fn process_list_top_row_contract() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Process.ListTop",
        Some(json!({ "limit": 3 })),
    )
    .await
    .expect("Process.ListTop");
    let rows = value.as_array().expect("array");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row.get("pid").and_then(|v| v.as_u64()).is_some());
        assert!(row.get("cpu").and_then(|v| v.as_f64()).is_some());
        assert!(row.get("name").and_then(|v| v.as_str()).is_some());
        let token = row
            .get("confirmation_token")
            .and_then(|v| v.as_str())
            .expect("confirmation_token");
        assert_eq!(token.len(), 64);
    }
}
