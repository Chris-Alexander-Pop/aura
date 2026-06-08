//! Mutating `Calendar.*` event CRUD and ICS import against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use ags_sidecar::notify;
use common::{call_method_unchecked, call_rpc, load_fixture, setup_temp_storage_db};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast;

static CALENDAR_STORAGE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn calendar_storage_test_lock() -> std::sync::MutexGuard<'static, ()> {
    CALENDAR_STORAGE_TEST_LOCK.lock().unwrap()
}

#[tokio::test]
async fn calendar_crud_and_upcoming() {
    let _guard = calendar_storage_test_lock();
    let _db = setup_temp_storage_db().await;
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
    let _guard = calendar_storage_test_lock();
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
async fn calendar_import_ics_edge_case_fixtures() {
    let _guard = calendar_storage_test_lock();
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    for (fixture, expected_imported) in [
        ("calendar/escaped_description.ics", 1usize),
        ("calendar/date_only.ics", 1),
        ("calendar/missing_dtstart.ics", 0),
    ] {
        let import_path = std::env::temp_dir().join(format!(
            "aura-ics-edge-{}-{}-{}.ics",
            fixture.replace('/', "-"),
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&import_path, load_fixture(fixture)).unwrap();

        let import = call_method_unchecked(
            &registry,
            "Calendar.ImportIcs",
            Some(json!({
                "file_path": import_path.to_string_lossy(),
                "calendar_id": "edge"
            })),
        )
        .await
        .unwrap();
        assert_eq!(
            import.get("imported"),
            Some(&json!(expected_imported)),
            "fixture {fixture}"
        );
        let _ = std::fs::remove_file(&import_path);
    }
}

#[tokio::test]
async fn calendar_set_reminder_and_fires_once() {
    let _guard = calendar_storage_test_lock();
    let _db = setup_temp_storage_db().await;
    ags_sidecar::services::notifications::reset_notifications_for_tests().await;
    let registry = build_registry();

    let now = chrono::Utc::now().timestamp();
    let created = call_method_unchecked(
        &registry,
        "Calendar.CreateEvent",
        Some(json!({
            "title": "Reminder RPC",
            "start": now + 300,
            "end": now + 3600,
            "description": "ping",
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    call_method_unchecked(
        &registry,
        "Calendar.SetReminder",
        Some(json!({ "event_id": id, "minutes_before": 6 })),
    )
    .await
    .expect("SetReminder");

    ags_sidecar::services::calendar::check_calendar_reminders_for_tests()
        .await
        .expect("tick");
    ags_sidecar::services::calendar::check_calendar_reminders_for_tests()
        .await
        .expect("tick again");

    let listed = call_rpc(
        &registry,
        "Notifications.List",
        Some(json!({ "limit": 5, "app_name": "aura-calendar" })),
    )
    .await
    .expect("Notifications.List");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["summary"], "Upcoming: Reminder RPC");
}

#[tokio::test]
async fn calendar_create_emits_events_changed_after_debounce() {
    let _guard = calendar_storage_test_lock();
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

    tokio::time::sleep(Duration::from_millis(350)).await;

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
