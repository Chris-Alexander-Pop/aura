//! Read-only `Bluetooth.*` RPC response shapes (host may have zero adapters/devices).

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
async fn bluetooth_list_adapters_mocked() {
    use common::ExecFixtureGuard;
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let adapters = call_method(&registry, "Bluetooth.GetAdapters", None)
        .await
        .expect("GetAdapters");
    assert!(adapters.is_array());
}

#[tokio::test]
async fn bluetooth_get_devices_mocked() {
    use common::ExecFixtureGuard;
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let devices = call_method(&registry, "Bluetooth.GetDevices", None)
        .await
        .expect("GetDevices");
    assert!(devices.is_array());
}
