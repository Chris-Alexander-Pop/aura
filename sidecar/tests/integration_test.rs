mod common;

use common::{call_method, test_registry};
use serde_json::json;

const P0_METHODS: &[&str] = &[
    "Power.GetBatteryState",
    "Power.GetProfile",
    "Network.GetStatus",
    "Network.ScanNetworks",
    "Network.ListSaved",
    "Packages.GetUpgradable",
    "Logs.Get",
    "Security.GetStatus",
    "Performance.GetMetrics",
    "DevOps.GetStatus",
    "Productivity.GetStats",
    "Automation.GetWorkflows",
    "Communication.GetUnread",
    "GameMode.IsEnabled",
    "System.GetStats",
    "Sidecar.GetVersion",
];

#[tokio::test]
async fn p0_methods_resolve_without_panic() {
    let registry = test_registry();
    for method in P0_METHODS {
        let result = call_method(&registry, method, None).await;
        assert!(
            result.is_ok(),
            "method {} failed: {:?}",
            method,
            result.err()
        );
    }
}

#[tokio::test]
async fn logs_get_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Logs.Get", None).await.expect("Logs.Get");
    assert!(value.is_array(), "Logs.Get should return a JSON array");
}

#[tokio::test]
async fn devops_get_status_has_podman_field() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetStatus", None)
        .await
        .expect("DevOps.GetStatus");
    assert!(value.get("podman_available").is_some());
}

#[tokio::test]
async fn communication_get_unread_returns_object() {
    let registry = test_registry();
    let value = call_method(&registry, "Communication.GetUnread", None)
        .await
        .expect("Communication.GetUnread");
    assert!(value.is_object());
}

#[tokio::test]
async fn storage_scan_namespace_round_trip() {
    let db = std::env::temp_dir().join(format!("ags-it-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&db);
    std::env::set_var("AURA_STORAGE_DB", db.to_string_lossy().to_string());

    let registry = test_registry();
    let ns = format!("test_ns_{}", std::process::id());

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "a", "value": { "n": 1 } })),
    )
    .await
    .expect("Storage.Set a");

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "b", "value": { "n": 2 } })),
    )
    .await
    .expect("Storage.Set b");

    let scan = call_method(
        &registry,
        "Storage.ScanNamespace",
        Some(json!({ "namespace": ns })),
    )
    .await
    .expect("Storage.ScanNamespace");

    let items = scan.get("items").and_then(|v| v.as_array()).expect("items array");
    assert_eq!(items.len(), 2);

    call_method(
        &registry,
        "Storage.Delete",
        Some(json!({ "namespace": ns, "key": "a" })),
    )
    .await
    .expect("Storage.Delete");

    let keys = call_method(
        &registry,
        "Storage.ListKeys",
        Some(json!({ "namespace": ns })),
    )
    .await
    .expect("Storage.ListKeys");

    let key_list = keys.get("keys").and_then(|v| v.as_array()).expect("keys");
    assert_eq!(key_list.len(), 1);
}

#[tokio::test]
async fn network_get_status_has_extended_fields() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.GetStatus", None)
        .await
        .expect("Network.GetStatus");
    assert!(value.get("wifi_enabled").is_some());
    assert!(value.get("connection_type").is_some());
    assert!(value.get("ethernet_connected").is_some());
}

#[tokio::test]
async fn power_get_battery_state_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Power.GetBatteryState", None)
        .await
        .expect("Power.GetBatteryState");
    assert!(value.get("percent").is_some());
    assert!(value.get("charging").is_some());
    assert!(value.get("time_remaining").is_some());
}
