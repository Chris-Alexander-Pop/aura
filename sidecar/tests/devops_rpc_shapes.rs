//! Read-only `DevOps.*` RPC response shapes.

mod common;

use common::{call_method, call_method_unchecked, test_registry};
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

#[test]
fn devops_docker_ps_fixture_parser() {
    use ags_sidecar::services::devops::parse_docker_ps_line;

    let text = common::load_fixture("devops/docker_ps.txt");
    let containers: Vec<_> = text.lines().filter_map(parse_docker_ps_line).collect();
    assert_eq!(containers.len(), 2);
    assert_eq!(containers[0].name, "my-app");
    assert_eq!(containers[0].ports, "0.0.0.0:8080->80/tcp");
    assert_eq!(containers[1].status, "Exited (0) 1 day ago");
}

#[test]
fn devops_docker_images_and_cron_fixture_parsers() {
    use ags_sidecar::contract_parsers::{parse_cron_line, parse_docker_image_line};

    let images_text = common::load_fixture("devops/docker_images.txt");
    let images: Vec<_> = images_text
        .lines()
        .filter_map(parse_docker_image_line)
        .collect();
    assert_eq!(images.len(), 2);
    assert_eq!(images[1].repository, "ags-sidecar");

    let cron_text = common::load_fixture("devops/crontab_sample.txt");
    let jobs: Vec<_> = cron_text
        .lines()
        .filter_map(|l| parse_cron_line(l, "root"))
        .collect();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].schedule, "0 0 * * *");
}

#[test]
fn devops_kubectl_pod_list_fixture_parser() {
    use ags_sidecar::services::devops::parse_kubectl_pod_list;

    let empty = common::load_fixture("devops/kubectl_pods_empty.json");
    assert_eq!(parse_kubectl_pod_list(&empty), Some(vec![]));

    let pods = common::load_fixture("devops/kubectl_pods.json");
    let list = parse_kubectl_pod_list(&pods).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].phase, "Running");
}

#[test]
fn devops_git_porcelain_fixture_parser() {
    use ags_sidecar::contract_parsers::git_status_dirty;

    assert!(!git_status_dirty(
        &common::load_fixture("devops/git_porcelain_clean.txt")
    ));
    assert!(git_status_dirty(
        &common::load_fixture("devops/git_porcelain_dirty.txt")
    ));
}

#[tokio::test]
async fn devops_get_systemd_timers_safe() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetSystemdTimers", None)
        .await
        .expect("DevOps.GetSystemdTimers");
    assert!(value.is_array());
}

#[tokio::test]
async fn devops_get_cron_jobs_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetCronJobs", None)
        .await
        .expect("DevOps.GetCronJobs");
    assert!(value.is_array());
}

#[tokio::test]
async fn devops_get_docker_containers_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetDockerContainers", None)
        .await
        .expect("DevOps.GetDockerContainers");
    let rows = value.as_array().expect("containers array");
    for row in rows {
        let obj = row.as_object().expect("container object");
        for key in ["id", "name", "image", "status", "ports"] {
            assert!(obj.contains_key(key), "missing {key}");
        }
    }
}

#[tokio::test]
async fn devops_get_podman_containers_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetPodmanContainers", None)
        .await
        .expect("DevOps.GetPodmanContainers");
    assert!(value.is_array());
}

#[tokio::test]
async fn devops_get_kubernetes_pods_wraps_json() {
    let registry = test_registry();
    match call_method(&registry, "DevOps.GetKubernetesPods", None).await {
        Ok(value) => {
            assert!(value.get("pods").and_then(|v| v.as_str()).is_some());
        }
        Err(err) => {
            let msg = err.to_string();
            assert!(
                msg.contains("kubectl") || msg.contains("connection refused"),
                "unexpected error: {msg}"
            );
        }
    }
}

#[tokio::test]
async fn devops_get_git_repos_under_manifest_dir() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "DevOps.GetGitRepos",
        Some(json!({ "path": env!("CARGO_MANIFEST_DIR") })),
    )
    .await
    .expect("DevOps.GetGitRepos");
    assert!(value.is_array());
}

#[tokio::test]
async fn devops_get_git_status_missing_repo_path_errors() {
    let registry = test_registry();
    let err = call_method(&registry, "DevOps.GetGitStatus", None)
        .await
        .expect_err("missing repo_path");
    assert!(err.to_string().contains("Missing repo_path"));
}

#[tokio::test]
async fn devops_get_container_logs_missing_name_errors() {
    let registry = test_registry();
    let err = call_method(&registry, "DevOps.GetContainerLogs", None)
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("Missing name"));
}

#[tokio::test]
async fn devops_start_container_missing_name_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "DevOps.StartContainer", None)
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("Missing name"));
}

#[tokio::test]
async fn devops_stop_container_missing_name_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "DevOps.StopContainer", None)
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("Missing name"));
}

#[tokio::test]
async fn devops_pull_image_missing_name_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "DevOps.PullImage", None)
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("Missing name"));
}
