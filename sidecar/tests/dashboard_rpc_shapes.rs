//! Read-only `Dashboard.*` and `Sidebar.*` RPC response shapes.

mod common;

use common::{call_rpc, ExecFixtureGuard, test_registry};
use serde_json::json;

#[tokio::test]
async fn dashboard_get_quick_status_has_aggregate_fields() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Dashboard.GetQuickStatus", None)
        .await
        .expect("Dashboard.GetQuickStatus");
    let obj = value.as_object().expect("object");

    for key in [
        "battery",
        "network",
        "bluetooth",
        "dnd",
        "next_event",
        "power_profile",
        "dropdown_modules",
    ] {
        assert!(obj.contains_key(key), "missing {key}");
    }

    let battery = obj.get("battery").and_then(|v| v.as_object()).expect("battery");
    for key in ["percent", "charging", "time_remaining"] {
        assert!(battery.contains_key(key), "battery missing {key}");
    }

    let network = obj.get("network").and_then(|v| v.as_object()).expect("network");
    for key in [
        "wifi_enabled",
        "connection_type",
        "ethernet_connected",
        "active_connection",
    ] {
        assert!(network.contains_key(key), "network missing {key}");
    }

    let bluetooth = obj
        .get("bluetooth")
        .and_then(|v| v.as_object())
        .expect("bluetooth");
    for key in ["powered", "connected_count"] {
        assert!(bluetooth.contains_key(key), "bluetooth missing {key}");
    }

    let dnd = obj.get("dnd").and_then(|v| v.as_object()).expect("dnd");
    assert!(dnd.get("enabled").and_then(|v| v.as_bool()).is_some());

    assert!(
        obj.get("next_event").map(|v| v.is_null() || v.is_object()).unwrap_or(false),
        "next_event must be null or object"
    );

    assert!(
        obj.get("power_profile").and_then(|v| v.as_str()).is_some(),
        "power_profile string"
    );

    assert!(
        obj.get("dropdown_modules").and_then(|v| v.as_array()).is_some(),
        "dropdown_modules array"
    );
}

#[tokio::test]
async fn sidebar_get_tile_data_network_shape() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "network" })),
    )
    .await
    .expect("Sidebar.GetTileData network");
    assert_eq!(
        value.get("tile").and_then(|v| v.as_str()),
        Some("network")
    );
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    for key in ["wifi_enabled", "connection_type", "ethernet_connected"] {
        assert!(data.contains_key(key), "tile data missing {key}");
    }
}

#[tokio::test]
async fn sidebar_get_tile_data_rejects_unknown_tile() {
    let registry = test_registry();
    let err = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "not-a-tile" })),
    )
    .await
    .expect_err("unknown tile");
    assert!(err.to_string().contains("unknown tile"));
}

#[tokio::test]
async fn sidebar_get_tile_data_missing_tile_param_errors() {
    let registry = test_registry();
    let err = call_rpc(&registry, "Sidebar.GetTileData", None)
        .await
        .expect_err("missing tile");
    assert!(err.to_string().contains("missing tile"));
}

#[tokio::test]
async fn sidebar_get_tile_data_audio_shape() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "audio" })),
    )
    .await
    .expect("Sidebar.GetTileData audio");
    assert_eq!(value.get("tile").and_then(|v| v.as_str()), Some("audio"));
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    for key in ["sink_count", "source_count", "default_sink"] {
        assert!(data.contains_key(key), "audio tile missing {key}");
    }
}

#[tokio::test]
async fn sidebar_get_tile_data_bluetooth_shape() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "bluetooth" })),
    )
    .await
    .expect("bluetooth tile");
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    for key in ["powered", "connected_count"] {
        assert!(data.contains_key(key), "bluetooth tile missing {key}");
    }
}

#[tokio::test]
async fn sidebar_get_tile_data_battery_shape() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "battery" })),
    )
    .await
    .expect("battery tile");
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    assert!(data.get("battery").and_then(|v| v.as_object()).is_some());
    assert!(data.get("power_profile").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn sidebar_get_tile_data_calendar_shape() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "calendar" })),
    )
    .await
    .expect("calendar tile");
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    assert!(data.get("events").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn sidebar_get_tile_data_notifications_shape() {
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "notifications" })),
    )
    .await
    .expect("notifications tile");
    let data = value.get("data").and_then(|v| v.as_object()).expect("data");
    assert!(data.get("dnd").and_then(|v| v.as_object()).is_some());
}

#[tokio::test]
async fn sidebar_get_tile_data_productivity_shape() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Sidebar.GetTileData",
        Some(json!({ "tile": "productivity" })),
    )
    .await
    .expect("productivity tile");
    assert!(value.get("data").is_some());
}
