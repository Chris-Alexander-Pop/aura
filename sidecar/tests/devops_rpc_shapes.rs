//! Read-only `DevOps.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};
use serde_json::json;

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

#[test]
fn devops_systemd_timers_fixture_parser() {
    use ags_sidecar::contract_parsers::parse_systemd_timer_line;

    let text = common::load_fixture("devops/systemctl_timers.txt");
    let timers: Vec<_> = text.lines().filter_map(parse_systemd_timer_line).collect();
    assert!(!timers.is_empty());
}

#[tokio::test]
async fn devops_get_systemd_timers_safe() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetSystemdTimers", None)
        .await
        .expect("DevOps.GetSystemdTimers");
    assert!(value.is_array());
}
