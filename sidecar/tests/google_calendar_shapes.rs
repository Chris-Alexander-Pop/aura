//! Google Calendar upsert helpers (no live network).

mod common;

use ags_sidecar::services::google_calendar::{parse_calendar_list, parse_google_event};
use common::{setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn google_event_upsert_into_storage_via_set() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let body: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/google/events_list.json")).unwrap();
    let item = &body["items"][0];
    let ev = parse_google_event(item, "primary@example.com").expect("parse");
    assert_eq!(ev.id, "gcal_evt1");

    common::call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "calendar_events",
            "key": ev.id,
            "value": serde_json::to_value(&ev).unwrap(),
        })),
    )
    .await
    .expect("Storage.Set");

    let events = common::call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("GetEvents");
    let ids: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"gcal_evt1"));
}

#[test]
fn google_fixtures_round_trip_shapes() {
    let cals = parse_calendar_list(
        &serde_json::from_str(include_str!("fixtures/google/calendar_list.json")).unwrap(),
    );
    assert_eq!(cals.len(), 2);
    let body: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/google/events_list.json")).unwrap();
    let ev = parse_google_event(&body["items"][1], "cal").unwrap();
    assert!(ev.id.starts_with("gcal_"));
    assert_eq!(ev.title, "Holiday");
}
