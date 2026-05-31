//! Mutating `Calendar.*` event CRUD and ICS import against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use ags_sidecar::notify;
use common::{call_method_unchecked, call_rpc, load_fixture, setup_temp_storage_db};
use serde_json::json;
use std::path::PathBuf;
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
async fn calendar_import_ics_fixture_round_trip() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let import_path = std::env::temp_dir().join(format!(
        "aura-ics-import-{}-{}.ics",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&import_path, load_fixture("calendar/multi_event.ics")).unwrap();

    let import = call_method_unchecked(
        &registry,
        "Calendar.ImportIcs",
        Some(json!({
            "file_path": import_path.to_string_lossy(),
            "calendar_id": "imported"
        })),
    )
    .await
    .unwrap();
    assert_eq!(import.get("imported"), Some(&json!(2)));

    let events = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .unwrap();
    let ids: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"event-a@aura"));
    assert!(ids.contains(&"event-b@aura"));

    let export_path = std::env::temp_dir().join(format!(
        "aura-ics-export-{}-{}.ics",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let export = call_method_unchecked(
        &registry,
        "Calendar.ExportIcs",
        Some(json!({
            "calendar_id": "imported",
            "file_path": export_path.to_string_lossy()
        })),
    )
    .await
    .unwrap();
    assert_eq!(export.get("exported"), Some(&json!(2)));

    let body = std::fs::read_to_string(&export_path).unwrap();
    let reparsed = ags_sidecar::services::ics::parse_ics(&body).unwrap();
    assert_eq!(reparsed.len(), 2);
    let _ = std::fs::remove_file(&import_path);
    let _ = std::fs::remove_file(&export_path);
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
