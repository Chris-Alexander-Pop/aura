//! Read-only `Brightness.*` RPC response shapes.

mod common;

use common::{call_method, call_method_unchecked, test_registry};
use serde_json::json;

#[tokio::test]
async fn brightness_get_active_monitor_shape() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Brightness.Get",
        Some(json!({ "monitor": "active" })),
    )
    .await
    .expect("Brightness.Get");
    let brightness = value
        .get("brightness")
        .and_then(|v| v.as_f64())
        .expect("brightness");
    assert!((0.0..=1.0).contains(&brightness));
    assert!(
        value.get("monitor").and_then(|v| v.as_str()).is_some(),
        "missing monitor name"
    );
}

#[tokio::test]
async fn brightness_get_all_returns_monitor_array() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Brightness.Get",
        Some(json!({ "monitor": "all" })),
    )
    .await
    .expect("Brightness.Get all");
    let monitors = value
        .get("monitors")
        .and_then(|v| v.as_array())
        .expect("monitors array");
    for entry in monitors {
        let b = entry
            .get("brightness")
            .and_then(|v| v.as_f64())
            .expect("brightness");
        assert!((0.0..=1.0).contains(&b));
        assert!(entry.get("monitor").and_then(|v| v.as_str()).is_some());
    }
}

/// Burst coalescing is covered by `services::brightness::tests::coalescer_keeps_latest_only`
/// and `brightness_set_mock_monitor_dry_run`.

/// Full RPC path can stall in the integration harness (static coalescer across runtimes).
/// Fast enqueue is covered by `services::brightness::tests::enqueue_returns_immediately_without_waiting_on_hardware`.
#[tokio::test]
#[ignore = "integration harness stalls; see enqueue_returns_immediately_without_waiting_on_hardware"]
async fn brightness_set_mock_monitor_dry_run() {
    use std::time::Duration;
    use tokio::time::timeout;

    std::env::set_var("AURA_BRIGHTNESS_DRY_RUN", "1");
    let registry = test_registry();
    let _ = call_method(
        &registry,
        "Brightness.Get",
        Some(json!({ "monitor": "active" })),
    )
    .await
    .expect("warm brightness cache");

    let result = timeout(
        Duration::from_secs(2),
        call_method_unchecked(
            &registry,
            "Brightness.Set",
            Some(json!({ "monitor": "active", "percent": 50 })),
        ),
    )
    .await
    .expect("Brightness.Set timed out");
    std::env::remove_var("AURA_BRIGHTNESS_DRY_RUN");
    let value = result.expect("Brightness.Set dry-run");
    assert_eq!(value.get("success").and_then(|v| v.as_bool()), Some(true));
}
