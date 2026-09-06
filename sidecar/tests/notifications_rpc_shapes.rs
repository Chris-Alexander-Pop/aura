//! Read-only `Notifications.*` RPC response shapes.

mod common;

use ags_sidecar::services::notifications::{
    dbus_monitor_fixture_path, feed_dbus_monitor_fixture, record_notification,
    reset_notifications_for_tests,
};
use common::{call_rpc, load_fixture, test_registry};
use serde_json::json;
use std::sync::Mutex;

static NOTIFICATIONS_RPC_LOCK: Mutex<()> = Mutex::new(());

fn notifications_rpc_lock() -> std::sync::MutexGuard<'static, ()> {
    NOTIFICATIONS_RPC_LOCK.lock().unwrap()
}

#[tokio::test]
async fn notifications_list_item_schema() {
    let _guard = notifications_rpc_lock();
    reset_notifications_for_tests().await;
    record_notification(
        Some(42),
        "contract-test".into(),
        1,
        "Summary".into(),
        "Body text".into(),
        None,
        1,
        vec![],
    )
    .await;

    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Notifications.List",
        Some(json!({ "limit": 10, "app_name": "contract-test" })),
    )
    .await
    .expect("Notifications.List");
    let items = value.as_array().expect("array");
    let item = items
        .iter()
        .find(|n| n.get("app_name").and_then(|v| v.as_str()) == Some("contract-test"))
        .expect("recorded notification");
    for key in [
        "id",
        "app_name",
        "summary",
        "body",
        "urgency",
        "timestamp",
        "actions",
        "closed",
    ] {
        assert!(item.get(key).is_some(), "missing notification field {key}");
    }
    assert!(item.get("actions").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn notifications_list_after_dbus_monitor_fixture() {
    let _guard = notifications_rpc_lock();
    reset_notifications_for_tests().await;
    let path = dbus_monitor_fixture_path("dbus_monitor_notify.txt");
    assert!(path.exists(), "fixture path {}", path.display());
    let fixture = load_fixture("notifications/dbus_monitor_notify.txt");
    feed_dbus_monitor_fixture(&fixture).await;

    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Notifications.List",
        Some(json!({ "limit": 5, "app_name": "firefox" })),
    )
    .await
    .expect("Notifications.List");
    let items = value.as_array().expect("array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["summary"], "Page loaded");
    assert_eq!(items[0]["body"], "example.com finished loading");
}

#[tokio::test]
async fn notifications_get_returns_recorded_item() {
    let _guard = notifications_rpc_lock();
    reset_notifications_for_tests().await;
    record_notification(
        None,
        "get-test".into(),
        0,
        "Hello".into(),
        "World".into(),
        None,
        2,
        vec![],
    )
    .await;

    let registry = test_registry();
    let listed = call_rpc(
        &registry,
        "Notifications.List",
        Some(json!({ "limit": 1, "app_name": "get-test" })),
    )
    .await
    .expect("list");
    let id = listed[0]["id"].as_u64().expect("id");

    let got = call_rpc(
        &registry,
        "Notifications.Get",
        Some(json!({ "id": id })),
    )
    .await
    .expect("Notifications.Get");
    assert_eq!(got["summary"], "Hello");
    assert_eq!(got["urgency"], 2);
}
