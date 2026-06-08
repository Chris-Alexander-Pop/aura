use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepo {
    pub path: String,
    pub name: String,
    pub branch: String,
    pub status: String,
    pub ahead: i32,
    pub behind: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdTimer {
    pub name: String,
    pub next_run: String,
    pub last_run: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub schedule: String,
    pub command: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KubectlPodSummary {
    pub name: String,
    pub namespace: String,
    pub phase: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("DevOps.GetDockerContainers", |_params| async move {
        Ok(serde_json::to_value(&list_containers_preferred().await?)?)
    });

    registry.register("DevOps.StartContainer", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["docker", "start", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("DevOps.StopContainer", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["docker", "stop", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("DevOps.RestartContainer", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["docker", "restart", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("DevOps.GetContainerLogs", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(100);

        let output = process::exec_command(&["docker", "logs", "--tail", &lines.to_string(), &name]).await?;
        Ok(serde_json::json!({ "logs": output }))
    });

    registry.register("DevOps.GetDockerImages", |_params| async move {
        let output = process::exec_command(&["docker", "images", "--format", "{{.ID}}|{{.Repository}}|{{.Tag}}|{{.Size}}"]).await?;
        let mut images = Vec::new();

        for line in output.lines() {
            if let Some(image) = parse_docker_image_line(line) {
                images.push(image);
            }
        }

        Ok(serde_json::to_value(&images)?)
    });

    registry.register("DevOps.PullImage", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["docker", "pull", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("DevOps.RemoveImage", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["docker", "rmi", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("DevOps.GetDockerStats", |_params| async move {
        let output = process::exec_command(&["docker", "stats", "--no-stream", "--format", "{{.Container}}|{{.CPUPerc}}|{{.MemUsage}}"]).await?;
        Ok(serde_json::json!({ "stats": output }))
    });

    registry.register("DevOps.GetPodmanContainers", |_params| async move {
        let output = process::exec_command(&["podman", "ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}"]).await?;
        let mut containers = Vec::new();

        for line in output.lines() {
            if let Some(mut c) = parse_docker_ps_line(line) {
                c.ports.clear();
                containers.push(c);
            }
        }

        Ok(serde_json::to_value(&containers)?)
    });

    registry.register("DevOps.GetKubernetesPods", |_params| async move {
        let output = process::exec_command(&["kubectl", "get", "pods", "-o", "json"]).await?;
        Ok(serde_json::json!({ "pods": output }))
    });

    registry.register("DevOps.GetGitRepos", |params| async move {
        let path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("path").cloned())
                .unwrap_or(serde_json::Value::String(std::env::var("HOME").unwrap_or_default())),
        )?;

        let mut repos = Vec::new();
        scan_git_repos(PathBuf::from(path), &mut repos).await?;
        Ok(serde_json::to_value(&repos)?)
    });

    registry.register("DevOps.GetGitStatus", |params| async move {
        let repo_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("repo_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing repo_path"))?,
        )?;

        let branch_output = process::exec_command(&["git", "-C", &repo_path, "branch", "--show-current"]).await?;
        let status_output = process::exec_command(&["git", "-C", &repo_path, "status", "--porcelain"]).await?;

        Ok(serde_json::json!({
            "branch": branch_output.trim(),
            "status": status_output,
            "dirty": git_status_dirty(&status_output)
        }))
    });

    registry.register("DevOps.GetSystemdTimers", |_params| async move {
        let output = process::exec_command(&["systemctl", "list-timers", "--no-pager", "--no-legend"]).await?;
        let mut timers = Vec::new();

        for line in output.lines() {
            if let Some(timer) = parse_systemd_timer_line(line) {
                timers.push(timer);
            }
        }

        Ok(serde_json::to_value(&timers)?)
    });

    registry.register("DevOps.GetCronJobs", |_params| async move {
        let mut jobs = Vec::new();

        // System crontab
        if let Ok(output) = process::exec_command(&["cat", "/etc/crontab"]).await {
            for line in output.lines() {
                if let Some(job) = parse_cron_line(line, "root") {
                    jobs.push(job);
                }
            }
        }

        // User crontab
        if let Ok(output) = process::exec_command(&["crontab", "-l"]).await {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            for line in output.lines() {
                if let Some(job) = parse_cron_line(line, &user) {
                    jobs.push(job);
                }
            }
        }

        Ok(serde_json::to_value(&jobs)?)
    });

    registry.register("DevOps.GetStatus", |_params| async move {
        let runtime = resolve_container_runtime().await;
        let podman_available = matches!(runtime, ContainerRuntime::Podman)
            || process::exec_command(&["which", "podman"]).await.is_ok();
        let docker_available = matches!(runtime, ContainerRuntime::Docker)
            || process::exec_command(&["which", "docker"]).await.is_ok();
        let kubectl_available = process::exec_command(&["which", "kubectl"]).await.is_ok();

        let containers = list_containers_preferred().await.unwrap_or_default();
        let container_count = containers.len() as u64;

        let git_dirty_count = count_dirty_git_roots().await;
        let git_dirty_hint = git_dirty_count > 0;

        let runtime_name = match runtime {
            ContainerRuntime::Podman => "podman",
            ContainerRuntime::Docker => "docker",
            ContainerRuntime::None => "none",
        };

        Ok(serde_json::json!({
            "podman_available": podman_available,
            "docker_available": docker_available,
            "kubectl_available": kubectl_available,
            "container_runtime": runtime_name,
            "tool_missing": runtime == ContainerRuntime::None,
            "container_count": container_count,
            "git_dirty_hint": git_dirty_hint,
            "git_dirty_count": git_dirty_count,
        }))
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRuntime {
    Podman,
    Docker,
    None,
}

pub async fn resolve_container_runtime() -> ContainerRuntime {
    if process::exec_command(&["which", "podman"]).await.is_ok() {
        return ContainerRuntime::Podman;
    }
    if std::env::var("AURA_ALLOW_DOCKER").is_ok()
        && process::exec_command(&["which", "docker"]).await.is_ok()
    {
        return ContainerRuntime::Docker;
    }
    ContainerRuntime::None
}

pub async fn list_containers_preferred() -> Result<Vec<DockerContainer>> {
    match resolve_container_runtime().await {
        ContainerRuntime::Podman => {
            let output = process::exec_command(&[
                "podman",
                "ps",
                "-a",
                "--format",
                "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}",
            ])
            .await?;
            let mut containers = Vec::new();
            for line in output.lines() {
                if let Some(mut c) = parse_docker_ps_line(line) {
                    c.ports.clear();
                    containers.push(c);
                }
            }
            Ok(containers)
        }
        ContainerRuntime::Docker => {
            let output = process::exec_command(&[
                "docker",
                "ps",
                "-a",
                "--format",
                "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}",
            ])
            .await?;
            Ok(output.lines().filter_map(parse_docker_ps_line).collect())
        }
        ContainerRuntime::None => Ok(Vec::new()),
    }
}

async fn count_dirty_git_roots() -> u64 {
    let mut count = 0u64;
    let roots: Vec<PathBuf> = git_search_roots();
    for root in roots {
        if !root.join(".git").exists() {
            continue;
        }
        if let Ok(output) =
            process::exec_command(&["git", "-C", &root.to_string_lossy(), "status", "--porcelain"])
                .await
        {
            if git_status_dirty(&output) {
                count += 1;
            }
        }
    }
    count
}

fn git_search_roots() -> Vec<PathBuf> {
    if let Ok(raw) = std::env::var("AURA_GIT_ROOTS") {
        return raw
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect();
    }
    if let Ok(home) = std::env::var("HOME") {
        return vec![PathBuf::from(home)];
    }
    Vec::new()
}

pub fn parse_docker_image_line(line: &str) -> Option<DockerImage> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 4 {
        Some(DockerImage {
            id: parts[0].to_string(),
            repository: parts[1].to_string(),
            tag: parts[2].to_string(),
            size: parts[3].to_string(),
        })
    } else {
        None
    }
}

/// Parse one `systemctl list-timers --no-legend` row (simplified: name + next + last + active token).
pub fn parse_systemd_timer_line(line: &str) -> Option<SystemdTimer> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    Some(SystemdTimer {
        name: parts[0].to_string(),
        next_run: parts[1].to_string(),
        last_run: parts[2].to_string(),
        active: parts[3].eq_ignore_ascii_case("active"),
    })
}

/// Parse a non-comment crontab line into schedule + command.
pub fn parse_cron_line(line: &str, user: &str) -> Option<CronJob> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    let schedule = format!(
        "{} {} {} {} {}",
        parts[0], parts[1], parts[2], parts[3], parts[4]
    );
    let command = parts[5..].join(" ");
    Some(CronJob {
        schedule,
        command,
        user: user.to_string(),
    })
}

/// True when `git status --porcelain` has any non-whitespace output.
pub fn git_status_dirty(porcelain: &str) -> bool {
    !porcelain.trim().is_empty()
}

/// Parse `kubectl get pods -o json` output into pod summaries.
pub fn parse_kubectl_pod_list(json: &str) -> Option<Vec<KubectlPodSummary>> {
    let root: serde_json::Value = serde_json::from_str(json).ok()?;
    let items = root.get("items")?.as_array()?;
    Some(
        items
            .iter()
            .filter_map(|item| {
                let meta = item.get("metadata")?;
                let status = item.get("status")?;
                Some(KubectlPodSummary {
                    name: meta.get("name")?.as_str()?.to_string(),
                    namespace: meta
                        .get("namespace")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default")
                        .to_string(),
                    phase: status
                        .get("phase")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                })
            })
            .collect(),
    )
}

pub fn parse_docker_ps_line(line: &str) -> Option<DockerContainer> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 4 {
        Some(DockerContainer {
            id: parts[0].to_string(),
            name: parts[1].to_string(),
            image: parts[2].to_string(),
            status: parts[3].to_string(),
            ports: parts.get(4).unwrap_or(&"").to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/devops/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read_to_string(path).expect("fixture")
    }

    #[test]
    fn parse_docker_ps_fixture() {
        let text = fixture("docker_ps.txt");
        let containers: Vec<_> = text.lines().filter_map(parse_docker_ps_line).collect();
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].name, "my-app");
        assert_eq!(containers[0].ports, "0.0.0.0:8080->80/tcp");
        assert_eq!(containers[1].status, "Exited (0) 1 day ago");
    }

    #[test]
    fn parse_docker_image_and_malformed_lines() {
        let text = fixture("docker_images.txt");
        let images: Vec<_> = text.lines().filter_map(parse_docker_image_line).collect();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].repository, "nginx");
        assert!(parse_docker_image_line("bad|line").is_none());
    }

    #[test]
    fn parse_systemd_timers_fixture() {
        let text = fixture("systemctl_timers.txt");
        let timers: Vec<_> = text.lines().filter_map(parse_systemd_timer_line).collect();
        assert_eq!(timers.len(), 2);
        assert!(timers[0].active);
        assert!(!timers[1].active);
        assert!(parse_systemd_timer_line("too few").is_none());
    }

    #[test]
    fn git_search_roots_uses_env_override() {
        std::env::set_var("AURA_GIT_ROOTS", "/tmp/a:/tmp/b");
        let roots = git_search_roots();
        std::env::remove_var("AURA_GIT_ROOTS");
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0], PathBuf::from("/tmp/a"));
    }

    #[tokio::test]
    async fn resolve_container_runtime_returns_known_variant() {
        let runtime = resolve_container_runtime().await;
        assert!(matches!(
            runtime,
            ContainerRuntime::Podman | ContainerRuntime::Docker | ContainerRuntime::None
        ));
    }

    #[test]
    fn parse_cron_and_git_porcelain_fixtures() {
        let cron = fixture("crontab_sample.txt");
        let jobs: Vec<_> = cron.lines().filter_map(|l| parse_cron_line(l, "root")).collect();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].command, "/usr/bin/logrotate");
        assert_eq!(jobs[0].schedule, "0 0 * * *");
        assert_eq!(jobs[0].user, "root");
        assert!(parse_cron_line("# only comment", "root").is_none());
        assert!(parse_cron_line("0 0 * *", "root").is_none());

        assert!(!git_status_dirty(&fixture("git_porcelain_clean.txt")));
        assert!(git_status_dirty(&fixture("git_porcelain_dirty.txt")));
        assert!(!git_status_dirty("   \n  "));
    }

    #[test]
    fn parse_docker_ps_skips_malformed_lines() {
        assert!(parse_docker_ps_line("").is_none());
        assert!(parse_docker_ps_line("a|b|c").is_none());
        let line = "id|name|image|Up|0.0.0.0:80->80/tcp";
        let c = parse_docker_ps_line(line).unwrap();
        assert_eq!(c.ports, "0.0.0.0:80->80/tcp");
    }

    #[test]
    fn parse_systemd_timer_active_case_insensitive() {
        let timer = parse_systemd_timer_line("foo.timer Mon last active").unwrap();
        assert!(timer.active);
        let inactive = parse_systemd_timer_line("bar.timer Tue last INACTIVE").unwrap();
        assert!(!inactive.active);
    }

    #[test]
    fn parse_kubectl_pod_list_fixtures() {
        let empty = fixture("kubectl_pods_empty.json");
        assert_eq!(parse_kubectl_pod_list(&empty), Some(vec![]));

        let pods = fixture("kubectl_pods.json");
        let list = parse_kubectl_pod_list(&pods).expect("pod list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "nginx-abc");
        assert_eq!(list[0].phase, "Running");
        assert_eq!(list[1].namespace, "aura");
        assert_eq!(list[1].phase, "Pending");

        assert!(parse_kubectl_pod_list("{").is_none());
        assert!(parse_kubectl_pod_list(r#"{"items":"x"}"#).is_none());
    }
}

async fn scan_git_repos(path: PathBuf, repos: &mut Vec<GitRepo>) -> Result<()> {
    scan_git_repos_recursive(path, repos, 0).await
}

fn scan_git_repos_recursive(
    path: PathBuf,
    repos: &mut Vec<GitRepo>,
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
    Box::pin(async move {
        if depth > 3 {
            return Ok(()); // Limit recursion depth
        }

        let mut entries = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if entry_path.join(".git").exists() {
                    let name = entry_path.file_name().unwrap().to_string_lossy().to_string();
                    let branch = process::exec_command(&["git", "-C", &entry_path.to_string_lossy(), "branch", "--show-current"]).await.ok().unwrap_or_default();
                    let status = process::exec_command(&["git", "-C", &entry_path.to_string_lossy(), "status", "--porcelain"]).await.ok().unwrap_or_default();

                    repos.push(GitRepo {
                        path: entry_path.to_string_lossy().to_string(),
                        name,
                        branch: branch.trim().to_string(),
                        status: if status.trim().is_empty() { "clean".to_string() } else { "dirty".to_string() },
                        ahead: 0,
                        behind: 0,
                    });
                } else {
                    // Recursively scan subdirectories
                    scan_git_repos_recursive(entry_path, repos, depth + 1).await.ok();
                }
            }
        }
        Ok(())
    })
}
