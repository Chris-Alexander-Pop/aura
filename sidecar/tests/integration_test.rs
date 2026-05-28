mod common;

use common::{call_method, test_registry};

const P0_METHODS: &[&str] = &[
    "Power.GetBatteryState",
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
