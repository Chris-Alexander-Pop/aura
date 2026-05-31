//! Read-only `GameMode.*` RPC response shapes.

mod common;

use common::{assert_json_object_keys, call_rpc, test_registry};
use serde_json::json;

#[tokio::test]
async fn gamemode_is_enabled_schema() {
    let registry = test_registry();
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert!(value.get("enabled").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn gamemode_is_enabled_defaults_false() {
    let registry = test_registry();
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert_json_object_keys(&value, &["enabled"]);
    assert_eq!(value.get("enabled"), Some(&json!(false)));
}
