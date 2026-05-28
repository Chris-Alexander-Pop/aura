//! Dedicated read-only RPC shape tests for Wave 1–2 CLI services (not bulk manifest tables).

mod common;

use common::{call_method, test_registry};
use serde_json::json;

#[tokio::test]
async fn bluetooth_get_device_info_unknown_address_errors() {
    let registry = test_registry();
    let err = call_method(
        &registry,
        "Bluetooth.GetDeviceInfo",
        Some(json!({ "device_address": "FF:FF:FF:FF:FF:FF" })),
    )
    .await
    .expect_err("unknown device should not resolve");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("device not found") || msg.contains("not found"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn bluetooth_get_device_info_shape_when_device_present() {
    let registry = test_registry();
    let devices = call_method(&registry, "Bluetooth.GetDevices", None)
        .await
        .expect("Bluetooth.GetDevices");
    let Some(addr) = devices
        .as_array()
        .and_then(|a| a.first())
        .and_then(|d| d.get("address"))
        .and_then(|v| v.as_str())
    else {
        eprintln!("skip: no paired/discovered Bluetooth devices on host");
        return;
    };

    let info = call_method(
        &registry,
        "Bluetooth.GetDeviceInfo",
        Some(json!({ "device_address": addr })),
    )
    .await
    .expect("Bluetooth.GetDeviceInfo");
    let obj = info.as_object().expect("device object");
    for key in [
        "path",
        "address",
        "name",
        "alias",
        "connected",
        "paired",
        "trusted",
        "device_type",
        "services",
    ] {
        assert!(obj.contains_key(key), "missing {key}");
    }
    assert_eq!(obj.get("address").and_then(|v| v.as_str()), Some(addr));
}

#[tokio::test]
async fn network_get_status_matches_network_status_contract() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.GetStatus", None)
        .await
        .expect("Network.GetStatus");
    let obj = value.as_object().expect("object");
    for key in ["wifi_enabled", "connection_type", "ethernet_connected", "active_connection"] {
        assert!(obj.contains_key(key), "missing {key}");
    }
    let ctype = obj
        .get("connection_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        matches!(ctype, "none" | "wifi" | "ethernet"),
        "unexpected connection_type: {ctype}"
    );
}

#[tokio::test]
async fn power_get_battery_state_accepts_zero_and_full_range_percent() {
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
