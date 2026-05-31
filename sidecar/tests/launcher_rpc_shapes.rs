//! Read-only `Launcher.Query` / `Launcher.Recent` RPC response shapes.

mod common;

use ags_sidecar::services::launcher::invalidate_desktop_index_cache;
use common::{call_rpc, launcher_test_lock, test_registry};
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
async fn launcher_query_empty_returns_results_array() {
    let _guard = launcher_test_lock();
    set_fixture_desktop_dirs();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Launcher.Query",
        Some(json!({ "query": "" })),
    )
    .await
    .expect("Launcher.Query");
    let results = value
        .get("results")
        .and_then(|v| v.as_array())
        .expect("results array");
    assert!(!results.is_empty(), "fixture dir should yield apps");
    let first = results.first().and_then(|v| v.as_object()).expect("object");
    for key in ["id", "name", "pinned"] {
        assert!(first.contains_key(key), "missing {key}");
    }
}

#[tokio::test]
async fn launcher_query_fuzzy_matches_firefox() {
    let _guard = launcher_test_lock();
    set_fixture_desktop_dirs();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Launcher.Query",
        Some(json!({ "query": "fire" })),
    )
    .await
    .expect("Launcher.Query fire");
    let results = value.get("results").and_then(|v| v.as_array()).unwrap();
    assert!(
        results
            .iter()
            .any(|r| r.get("id").and_then(|v| v.as_str()) == Some("firefox")),
        "expected firefox in results: {results:?}"
    );
    let firefox = results
        .iter()
        .find(|r| r.get("id").and_then(|v| v.as_str()) == Some("firefox"))
        .unwrap();
    assert!(firefox.get("score").and_then(|v| v.as_i64()).is_some());
}

#[tokio::test]
async fn launcher_query_without_desktop_dirs_is_empty() {
    let _guard = launcher_test_lock();
    std::env::set_var("AURA_LAUNCHER_DESKTOP_DIRS", "/nonexistent/aura-launcher-empty");
    invalidate_desktop_index_cache();
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Launcher.Query",
        Some(json!({ "query": "x" })),
    )
    .await
    .expect("Launcher.Query empty index");
    let results = value.get("results").and_then(|v| v.as_array()).unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn launcher_recent_returns_items_array() {
    let _guard = launcher_test_lock();
    set_fixture_desktop_dirs();
    let registry = test_registry();
    let value = call_rpc(&registry, "Launcher.Recent", None)
        .await
        .expect("Launcher.Recent");
    assert!(value.get("items").and_then(|v| v.as_array()).is_some());
}
