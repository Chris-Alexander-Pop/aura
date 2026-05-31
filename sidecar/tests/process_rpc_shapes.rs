//! Read-only `Process.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};
use serde_json::json;

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
    }
}
