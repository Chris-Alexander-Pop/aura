//! Mutating `Calendar.*` event CRUD against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use ags_sidecar::notify;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

static CALENDAR_NOTIFY_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

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

#[tokio::test]
async fn calendar_create_emits_events_changed_after_debounce() {
    let _notify_guard = CALENDAR_NOTIFY_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .await;
    let _db = setup_temp_storage_db().await;
    let tx = notify::init_for_tests();
    let mut rx = tx.subscribe();
    while rx.try_recv().is_ok() {}

    let registry = build_registry();
    while rx.try_recv().is_ok() {}

    let now = chrono::Utc::now().timestamp();
    call_method_unchecked(
        &registry,
        "Calendar.CreateEvent",
        Some(json!({
            "title": "WS emit",
            "start": now + 3600,
            "end": now + 7200,
        })),
    )
    .await
    .expect("Calendar.CreateEvent");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for Calendar.EventsChanged");
        }
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Ok(raw)) => {
                let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
                if v.get("method").and_then(|m| m.as_str()) == Some("Calendar.EventsChanged") {
                    assert_eq!(v["params"]["reason"], "create");
                    return;
                }
            }
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                panic!("notify bus closed");
            }
            Err(_) => continue,
        }
    }
}
