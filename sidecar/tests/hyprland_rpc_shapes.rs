//! Read-only `Hyprland.*` RPC response shapes (requires Hyprland/hyprctl on host).

mod common;

use common::{call_method, call_rpc, test_registry};

const HYPRLAND_READONLY_METHODS: &[&str] = &[
    "Hyprland.GetWorkspaces",
    "Hyprland.GetActiveWorkspace",
    "Hyprland.GetClients",
    "Hyprland.GetActiveWindow",
    "Hyprland.GetMonitors",
    "Hyprland.GetBarSnapshot",
];

#[tokio::test]
async fn hyprland_readonly_methods_resolve() {
    let registry = test_registry();
    for method in HYPRLAND_READONLY_METHODS {
        let result = call_rpc(&registry, method, None).await;
        assert!(result.is_ok(), "method {method} failed: {:?}", result.err());
    }
}

#[tokio::test]
async fn hyprland_get_workspaces_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Hyprland.GetWorkspaces", None)
        .await
        .expect("Hyprland.GetWorkspaces");
    assert!(value.is_array(), "expected array, got {value}");
    for item in value.as_array().unwrap() {
        assert!(item.get("id").and_then(|x| x.as_i64()).is_some());
        assert!(item.get("name").and_then(|x| x.as_str()).is_some());
        assert!(item.get("windows").and_then(|x| x.as_u64()).is_some());
    }
}

#[tokio::test]
async fn hyprland_get_clients_array_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Hyprland.GetClients", None)
        .await
        .expect("Hyprland.GetClients");
    for item in value.as_array().unwrap() {
        assert!(item.get("address").and_then(|x| x.as_str()).is_some());
        assert!(item.get("class").or_else(|| item.get("class_name")).is_some());
    }
}
