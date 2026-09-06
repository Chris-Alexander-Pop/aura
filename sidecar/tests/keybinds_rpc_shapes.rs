//! Read-only `Keybinds.*` RPC response shapes.

mod common;

use common::{call_rpc, test_registry};

#[tokio::test]
async fn keybinds_list_entry_schema() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Keybinds.List", None)
        .await
        .expect("Keybinds.List");
    let entries = value.as_array().expect("array");
    if let Some(row) = entries.first() {
        let obj = row.as_object().expect("object");
        for key in ["combo", "action", "bind_type", "file", "line", "category"] {
            assert!(obj.contains_key(key), "missing keybind field {key}");
        }
    }
}
