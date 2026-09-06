//! Read-only `Power.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};

#[tokio::test]
async fn power_get_battery_state_accepts_valid_percent_range() {
    let registry = test_registry();
    let value = call_method(&registry, "Power.GetBatteryState", None)
        .await
        .expect("Power.GetBatteryState");
    let percent = value
        .get("percent")
        .and_then(|v| v.as_u64())
        .expect("percent");
    assert!(percent <= 100, "percent out of range: {percent}");
    assert!(value.get("charging").and_then(|v| v.as_bool()).is_some());
    let time = value
        .get("time_remaining")
        .and_then(|v| v.as_str())
        .expect("time_remaining string");
    assert!(!time.is_empty());
}
