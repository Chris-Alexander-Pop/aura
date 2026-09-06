//! `Launcher.Recent` / `Launcher.Pin` round-trip against a temporary SQLite DB.

mod common;

use ags_sidecar::build_registry;
use ags_sidecar::services::launcher::invalidate_desktop_index_cache;
use common::{
    call_method_unchecked, call_rpc, launcher_test_lock, setup_temp_storage_db, ExecFixtureGuard,
};
use serde_json::json;
use std::path::PathBuf;

fn set_fixture_desktop_dirs() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/desktop");
    std::env::set_var(
        "AURA_LAUNCHER_DESKTOP_DIRS",
        dir.to_string_lossy().to_string(),
    );
    invalidate_desktop_index_cache();
}

#[tokio::test]
async fn launcher_pin_round_trip_via_storage() {
    let _guard = launcher_test_lock();
    let _db = setup_temp_storage_db().await;
    set_fixture_desktop_dirs();
    let registry = build_registry();

    call_method_unchecked(
        &registry,
        "Launcher.Pin",
        Some(json!({ "id": "firefox", "pinned": true })),
    )
    .await
    .unwrap();

    let stored = call_rpc(
        &registry,
        "Storage.Get",
        Some(json!({ "namespace": "launcher", "key": "pins" })),
    )
    .await
    .unwrap();
    let ids = stored
        .get("value")
        .and_then(|v| v.as_array())
        .expect("pins array");
    assert!(
        ids.iter()
            .any(|v| v.as_str() == Some("firefox")),
        "firefox should be pinned: {ids:?}"
    );

    call_method_unchecked(
        &registry,
        "Launcher.Pin",
        Some(json!({ "id": "firefox", "pinned": false })),
    )
    .await
    .unwrap();

    let after = call_rpc(
        &registry,
        "Storage.Get",
        Some(json!({ "namespace": "launcher", "key": "pins" })),
    )
    .await
    .unwrap();
    let pins = after.get("value").and_then(|v| v.as_array()).unwrap();
    assert!(!pins.iter().any(|v| v.as_str() == Some("firefox")));
}

#[tokio::test]
async fn launcher_run_records_recent() {
    let _exec = ExecFixtureGuard::activate();
    let _guard = launcher_test_lock();
    let _db = setup_temp_storage_db().await;
    set_fixture_desktop_dirs();
    let registry = build_registry();

    call_method_unchecked(
        &registry,
        "Launcher.Run",
        Some(json!({ "id": "aura-test" })),
    )
    .await
    .unwrap();

    let recent = call_rpc(&registry, "Launcher.Recent", None)
        .await
        .unwrap();
    let items = recent.get("items").and_then(|v| v.as_array()).unwrap();
    assert!(
        items
            .first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            == Some("aura-test"),
        "most recent launch should be aura-test: {items:?}"
    );
}
