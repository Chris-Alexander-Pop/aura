//! Read-only `Communication.*` RPC shapes and notification-settings storage branches.

mod common;

use ags_sidecar::contract_parsers::communication_unread_counts_schema_valid as unread_counts_schema_valid;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn communication_unread_and_notification_settings_schema() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let unread = call_rpc(&registry, "Communication.GetUnread", None)
        .await
        .expect("Communication.GetUnread");
    assert!(unread.is_object());
    assert!(unread.as_object().unwrap().is_empty());

    let default_settings = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("GetNotificationSettings");
    assert!(default_settings.is_object());

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "communication",
            "key": "notification_settings",
            "value": { "mute_all": false, "per_app": {} }
        })),
    )
    .await
    .expect("Storage.Set notification_settings");

    let settings = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("GetNotificationSettings loaded");
    assert_eq!(settings.get("mute_all"), Some(&json!(false)));
    assert!(settings.get("per_app").and_then(|v| v.as_object()).is_some());
}

#[tokio::test]
async fn communication_stub_list_methods_and_settings_branches() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let unread = call_rpc(&registry, "Communication.GetUnread", None)
        .await
        .expect("GetUnread");
    assert!(unread.is_object());
    assert!(unread.as_object().unwrap().is_empty());

    for method in [
        "Communication.GetMessages",
        "Communication.GetContacts",
        "Communication.GetConversations",
        "Communication.GetActiveCalls",
    ] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        assert!(
            value.as_array().map(|a| a.is_empty()).unwrap_or(false),
            "{method} should be empty array stub"
        );
    }

    let corrupt = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("settings default");
    assert!(corrupt.is_object());

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "communication",
            "key": "notification_settings",
            "value": "not-an-object"
        })),
    )
    .await
    .expect("Storage.Set corrupt settings");

    let normalized = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("settings normalized");
    assert_eq!(normalized, json!({}));

    let apps = call_rpc(&registry, "Communication.GetCommunicationApps", None)
        .await
        .expect("GetCommunicationApps");
    assert!(apps.is_array());
}

#[tokio::test]
async fn communication_set_notification_settings_round_trip() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Communication.SetNotificationSettings",
        Some(json!({
            "settings": {
                "mute_all": true,
                "per_app": { "discord": { "mute": true } }
            }
        })),
    )
    .await
    .expect("SetNotificationSettings");

    let loaded = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("GetNotificationSettings");
    assert_eq!(loaded.get("mute_all"), Some(&json!(true)));
    assert_eq!(
        loaded
            .get("per_app")
            .and_then(|v| v.get("discord"))
            .and_then(|v| v.get("mute")),
        Some(&json!(true))
    );
}

#[tokio::test]
async fn communication_notification_settings_array_and_null_branches() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for corrupt in [json!([]), json!(null), json!(0)] {
        call_rpc(
            &registry,
            "Storage.Set",
            Some(json!({
                "namespace": "communication",
                "key": "notification_settings",
                "value": corrupt
            })),
        )
        .await
        .expect("Storage.Set corrupt");

        let normalized = call_rpc(&registry, "Communication.GetNotificationSettings", None)
            .await
            .expect("normalized settings");
        assert_eq!(normalized, json!({}));
    }
}

#[test]
fn communication_unread_bridge_adapter_keys_schema() {
    let empty = json!({});
    assert!(unread_counts_schema_valid(&empty));

    let bridges = json!({
        "discord": 3,
        "signal": 0,
        "telegram": 12,
        "matrix": null
    });
    assert!(unread_counts_schema_valid(&bridges));
    for key in ["discord", "signal", "telegram", "matrix"] {
        assert!(bridges.get(key).is_some(), "expected bridge key {key}");
    }

    assert!(!unread_counts_schema_valid(&json!({ "discord": "many" })));
    assert!(!unread_counts_schema_valid(&json!([])));
}
