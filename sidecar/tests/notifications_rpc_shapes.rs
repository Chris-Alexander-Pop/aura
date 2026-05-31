//! Read-only `Notifications.*` RPC response shapes.

mod common;

use ags_sidecar::services::notifications::record_notification;
use common::{call_rpc, test_registry};
use serde_json::json;

#[tokio::test]
async fn notifications_list_item_schema() {
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
