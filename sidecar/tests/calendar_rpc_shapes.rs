//! Read-only `Calendar.*` RPC shapes and storage-namespace edge cases.

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn calendar_get_events_filters_and_skips_corrupt() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for (key, start, end) in [
        ("inside", 200, 300),
        ("before", 10, 50),
        ("after", 900, 1000),
    ] {
        call_rpc(
            &registry,
            "Storage.Set",
            Some(json!({
                "namespace": "calendar_events",
                "key": key,
                "value": {
                    "id": key,
                    "title": key,
                    "start": start,
                    "end": end,
                    "description": "",
                    "calendar_id": null,
                    "reminder_minutes": null
                }
            })),
        )
        .await
        .expect("Storage.Set event");
    }

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "calendar_events",
            "key": "corrupt",
            "value": { "not_an_event": 1 }
        })),
    )
    .await
    .expect("Storage.Set corrupt");

    let filtered = call_rpc(
        &registry,
        "Calendar.GetEvents",
        Some(json!({ "start_date": 150, "end_date": 400 })),
    )
    .await
    .expect("Calendar.GetEvents");
    let ids: Vec<_> = filtered
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["inside"]);

    let all = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("Calendar.GetEvents all");
    assert_eq!(all.as_array().map(|a| a.len()), Some(3));
}

#[tokio::test]
async fn calendar_empty_namespace_and_event_shape() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("GetEvents empty");
    assert!(empty.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let calendars = call_rpc(&registry, "Calendar.GetCalendars", None)
        .await
        .expect("GetCalendars");
    assert!(calendars.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let google_status = call_rpc(&registry, "Calendar.GoogleAuthStatus", None)
        .await
        .expect("GoogleAuthStatus");
    assert_eq!(google_status.get("connected"), Some(&json!(false)));
    assert!(google_status.get("configured").is_some());

    let upcoming = call_rpc(
        &registry,
        "Calendar.GetUpcomingEvents",
        Some(json!({ "days": 3 })),
    )
    .await
    .expect("GetUpcomingEvents");
    assert!(upcoming.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "calendar_events",
            "key": "meet",
            "value": {
                "id": "meet",
                "title": "Standup",
                "start": 1000,
                "end": 1100,
                "description": "daily",
                "calendar_id": "work",
                "reminder_minutes": 5
            }
        })),
    )
    .await
    .expect("Storage.Set event");

    let events = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("GetEvents");
    let first = events.as_array().and_then(|a| a.first()).expect("one event");
    for key in [
        "id",
        "title",
        "start",
        "end",
        "description",
        "calendar_id",
        "reminder_minutes",
    ] {
        assert!(first.get(key).is_some(), "missing {key}");
    }
}
