mod common;

use ags_sidecar::build_registry;
use common::{call_rpc, call_method};
use serde_json::json;

const WAVE5_HYPRLAND_READONLY: &[&str] = &[
    "Hyprland.GetWorkspaces",
    "Hyprland.GetActiveWorkspace",
    "Hyprland.GetClients",
    "Hyprland.GetActiveWindow",
    "Hyprland.GetMonitors",
];

#[tokio::test]
async fn hyprland_readonly_shapes_resolve() {
    let registry = build_registry();
    for method in WAVE5_HYPRLAND_READONLY {
        let result = call_rpc(&registry, method, None).await;
        assert!(result.is_ok(), "method {method} failed: {:?}", result.err());
    }
}

#[tokio::test]
async fn hyprland_get_workspaces_returns_array() {
    let registry = build_registry();
    let v = call_method(&registry, "Hyprland.GetWorkspaces", None)
        .await
        .unwrap();
    assert!(v.is_array(), "expected array, got {v}");
    for item in v.as_array().unwrap() {
        assert!(item.get("id").and_then(|x| x.as_i64()).is_some());
        assert!(item.get("name").and_then(|x| x.as_str()).is_some());
        assert!(item.get("windows").and_then(|x| x.as_u64()).is_some());
    }
}

#[tokio::test]
async fn hyprland_get_clients_array_shape() {
    let registry = build_registry();
    let v = call_method(&registry, "Hyprland.GetClients", None)
        .await
        .unwrap();
    let arr = v.as_array().unwrap();
    for item in arr {
        assert!(item.get("address").and_then(|x| x.as_str()).is_some());
        assert!(item.get("class").or_else(|| item.get("class_name")).is_some());
    }
}
