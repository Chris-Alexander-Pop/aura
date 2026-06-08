//! `Security.Offensive.Session.*` / notes / report RPC shapes (temp storage; no network).

#![cfg(feature = "offensive-security")]

mod common;

use common::{call_method_unchecked, setup_temp_storage_db, test_registry};
use serde_json::json;
use std::sync::Once;

static OFFENSIVE_TEST_HARNESS: Once = Once::new();

fn offensive_test_harness() {
    OFFENSIVE_TEST_HARNESS.call_once(|| {
        std::env::set_var("AURA_OFFENSIVE_SKIP_RATE_LIMIT", "1");
    });
}

#[tokio::test]
async fn offensive_session_create_list_shape() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let created = call_method_unchecked(
        &registry,
        "Security.Offensive.Session.Create",
        Some(json!({ "name": "lab", "target": "10.0.0.5" })),
    )
    .await
    .expect("Session.Create");

    let session_id = created["session_id"].as_str().expect("session_id");
    assert!(session_id.starts_with("session_"));
    assert_eq!(created["name"], "lab");
    assert_eq!(created["target"], "10.0.0.5");
    assert!(created["notes"].as_array().unwrap().is_empty());
    assert!(created["findings"].as_array().unwrap().is_empty());

    let list = call_method_unchecked(&registry, "Security.Offensive.Session.List", None)
        .await
        .expect("Session.List");
    let rows = list.as_array().expect("array");
    assert!(rows.iter().any(|r| r["session_id"] == session_id));
}

#[tokio::test]
async fn offensive_session_load_from_storage() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let created = call_method_unchecked(
        &registry,
        "Security.Offensive.Session.Create",
        Some(json!({ "name": "persist", "target": "example.test" })),
    )
    .await
    .expect("create");
    let session_id = created["session_id"].as_str().expect("session_id").to_string();

    let loaded = call_method_unchecked(
        &registry,
        "Security.Offensive.Session.Load",
        Some(json!({ "session_id": session_id })),
    )
    .await
    .expect("Session.Load");
    assert_eq!(loaded["name"], "persist");
    assert_eq!(loaded["target"], "example.test");

    let missing = call_method_unchecked(
        &registry,
        "Security.Offensive.Session.Load",
        Some(json!({ "session_id": "session_missing" })),
    )
    .await
    .expect_err("missing session");
    assert!(missing.to_string().contains("Session not found"));
}

#[tokio::test]
async fn offensive_notes_add_shape() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let note = call_method_unchecked(
        &registry,
        "Security.Offensive.Notes.Add",
        Some(json!({ "note": "recon complete", "category": "recon" })),
    )
    .await
    .expect("Notes.Add");
    assert_eq!(note["success"], true);
    let note_id = note["note_id"].as_str().expect("note_id");
    assert!(note_id.starts_with("note_"));
}

#[tokio::test]
async fn offensive_report_create_export_round_trip() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let session = call_method_unchecked(
        &registry,
        "Security.Offensive.Session.Create",
        Some(json!({ "name": "report-run", "target": "10.0.0.9" })),
    )
    .await
    .expect("session");
    let session_id = session["session_id"].as_str().expect("session_id");

    call_method_unchecked(
        &registry,
        "Security.Offensive.Report.AddFinding",
        Some(json!({
            "session_id": session_id,
            "finding": {
                "id": "f1",
                "title": "SSH open",
                "description": "port 22",
                "severity": "medium",
                "category": "network",
                "timestamp": 1
            }
        })),
    )
    .await
    .expect("AddFinding");

    let report = call_method_unchecked(
        &registry,
        "Security.Offensive.Report.Create",
        Some(json!({ "session_id": session_id, "template": "default" })),
    )
    .await
    .expect("Report.Create");
    assert_eq!(report["success"], true);
    let report_id = report["report_id"].as_str().expect("report_id");

    let md = call_method_unchecked(
        &registry,
        "Security.Offensive.Report.Export",
        Some(json!({ "report_id": report_id, "format": "markdown" })),
    )
    .await
    .expect("markdown export");
    let markdown = md["markdown"].as_str().expect("markdown");
    assert!(markdown.starts_with("# Pentest Report"));

    let json_export = call_method_unchecked(
        &registry,
        "Security.Offensive.Report.Export",
        Some(json!({ "report_id": report_id, "format": "json" })),
    )
    .await
    .expect("json export");
    assert_eq!(json_export["session_id"], session_id);

    let bad = call_method_unchecked(
        &registry,
        "Security.Offensive.Report.Export",
        Some(json!({ "report_id": report_id, "format": "pdf" })),
    )
    .await
    .expect_err("bad format");
    assert!(bad.to_string().contains("Unsupported format"));
}

#[tokio::test]
async fn offensive_report_create_missing_session_errors() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Security.Offensive.Report.Create",
        Some(json!({ "session_id": "session_nope" })),
    )
    .await
    .expect_err("missing session");
    assert!(err.to_string().contains("Session not found"));
}

#[tokio::test]
async fn offensive_notes_missing_note_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.Notes.Add", None)
        .await
        .expect_err("missing note");
    assert!(err.to_string().contains("Missing note"));
}
