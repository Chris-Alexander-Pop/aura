//! CalDAV fixture parsing and read-only sync (fixture file, no network).

mod common;

use ags_sidecar::services::caldav::parse_caldav_report_events;
use common::{call_method_unchecked, load_fixture, setup_temp_storage_db, test_registry};

#[test]
fn caldav_report_fixture_parses_events() {
    let xml = load_fixture("caldav/report_multistatus.xml");
    let events = parse_caldav_report_events(&xml).expect("parse");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].summary, "CalDAV fixture event");
}

#[test]
fn caldav_empty_report_returns_no_events() {
    let events = parse_caldav_report_events("<multistatus xmlns=\"DAV:\"></multistatus>").unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn calendar_sync_caldav_no_url_returns_zero_synced() {
    let _db = setup_temp_storage_db().await;
    std::env::remove_var("AURA_CALDAV_URL");
    std::env::remove_var("AURA_CALDAV_FIXTURE");

    let registry = test_registry();
    let value = common::call_method_unchecked(&registry, "Calendar.SyncCalDav", None)
        .await
        .expect("Calendar.SyncCalDav without config");
    assert_eq!(value.get("success").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(value.get("synced").and_then(|v| v.as_u64()), Some(0));
}

#[tokio::test]
async fn calendar_sync_caldav_imports_fixture() {
    let _db = setup_temp_storage_db().await;
    let fixture = format!(
        "{}/tests/fixtures/caldav/report_multistatus.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    std::env::set_var("AURA_CALDAV_FIXTURE", &fixture);
    std::env::set_var("AURA_CALDAV_URL", "http://127.0.0.1:1/caldav");

    let registry = test_registry();
    let value = common::call_method_unchecked(&registry, "Calendar.SyncCalDav", None)
        .await
        .expect("Calendar.SyncCalDav");
    assert_eq!(value.get("success").and_then(|v| v.as_bool()), Some(true));
    assert!(value.get("synced").and_then(|v| v.as_u64()).unwrap_or(0) >= 1);

    std::env::remove_var("AURA_CALDAV_FIXTURE");
    std::env::remove_var("AURA_CALDAV_URL");
}
