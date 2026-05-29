mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn calendar_crud_and_upcoming() {
    setup_temp_storage_db().await;
    let registry = build_registry();

    let now = chrono::Utc::now().timestamp();
    let created = call_method_unchecked(
        &registry,
        "Calendar.CreateEvent",
        Some(json!({
            "title": "Standup",
            "start": now + 3600,
            "end": now + 7200,
            "description": "daily",
            "reminder_minutes": 10
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    let upcoming = call_rpc(
        &registry,
        "Calendar.GetUpcomingEvents",
        Some(json!({ "limit": 5, "days": 7 })),
    )
    .await
    .unwrap();
    assert!(upcoming.as_array().unwrap().iter().any(|e| e.get("id") == Some(&json!(id))));

    let del = call_method_unchecked(
        &registry,
        "Calendar.DeleteEvent",
        Some(json!({ "event_id": id })),
    )
    .await
    .unwrap();
    assert_eq!(del.get("deleted"), Some(&json!(true)));
}
