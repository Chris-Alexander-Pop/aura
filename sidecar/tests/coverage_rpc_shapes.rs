//! Read-only RPC shape tests for audio, devops, system, processes, brightness coverage targets.

mod common;

use common::{call_method, test_registry};
use serde_json::json;

#[tokio::test]
async fn system_get_stats_matches_system_stats_contract() {
    let registry = test_registry();
    let value = call_method(&registry, "System.GetStats", None)
        .await
        .expect("System.GetStats");
    for key in ["cpu", "ram", "temp"] {
        let n = value
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("missing or non-numeric {key}"));
        assert!(
            (0.0..=1.0).contains(&n) || key == "temp",
            "{key} out of expected range: {n}"
        );
    }
    if let Some(gpu) = value.get("gpu") {
        assert!(gpu.is_null() || gpu.as_f64().is_some());
    }
}

#[tokio::test]
async fn devops_get_status_fields_and_types() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetStatus", None)
        .await
        .expect("DevOps.GetStatus");
    for key in [
        "podman_available",
        "docker_available",
        "kubectl_available",
        "container_runtime",
        "tool_missing",
        "container_count",
        "git_dirty_hint",
        "git_dirty_count",
    ] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    assert!(value
        .get("container_count")
        .and_then(|v| v.as_u64())
        .is_some());
}

#[tokio::test]
async fn devops_get_git_status_repo_shape() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "DevOps.GetGitStatus",
        Some(json!({ "repo_path": env!("CARGO_MANIFEST_DIR") })),
    )
    .await
    .expect("DevOps.GetGitStatus");
    assert!(value.get("branch").and_then(|v| v.as_str()).is_some());
    assert!(value.get("status").and_then(|v| v.as_str()).is_some());
    assert!(value.get("dirty").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn audio_effects_get_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Effects.GetStatus", None)
        .await
        .expect("Audio.Effects.GetStatus");
    for key in ["available", "running", "current_preset"] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
}

#[tokio::test]
async fn process_list_top_row_contract() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Process.ListTop",
        Some(json!({ "limit": 3 })),
    )
    .await
    .expect("Process.ListTop");
    let rows = value.as_array().expect("array");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row.get("pid").and_then(|v| v.as_u64()).is_some());
        assert!(row.get("cpu").and_then(|v| v.as_f64()).is_some());
        assert!(row.get("name").and_then(|v| v.as_str()).is_some());
    }
}
