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

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("DevOps.GetDockerContainers", |_params| async move {
        let output = process::exec_command(&["docker", "ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}"]).await?;
        let mut containers = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                containers.push(DockerContainer {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    image: parts[2].to_string(),
                    status: parts[3].to_string(),
                    ports: parts[4].to_string(),
                });
            }
        }

        Ok(serde_json::to_value(&containers)?)
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
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                images.push(DockerImage {
                    id: parts[0].to_string(),
                    repository: parts[1].to_string(),
                    tag: parts[2].to_string(),
                    size: parts[3].to_string(),
                });
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
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                containers.push(DockerContainer {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    image: parts[2].to_string(),
                    status: parts[3].to_string(),
                    ports: String::new(),
                });
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
            "dirty": !status_output.trim().is_empty()
        }))
    });

    registry.register("DevOps.GetSystemdTimers", |_params| async move {
        let output = process::exec_command(&["systemctl", "list-timers", "--no-pager", "--no-legend"]).await?;
        let mut timers = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                timers.push(SystemdTimer {
                    name: parts[0].to_string(),
                    next_run: parts[1].to_string(),
                    last_run: parts[2].to_string(),
                    active: parts[3] == "active",
                });
            }
        }

        Ok(serde_json::to_value(&timers)?)
    });

    registry.register("DevOps.GetCronJobs", |_params| async move {
        let mut jobs = Vec::new();

        // System crontab
        if let Ok(output) = process::exec_command(&["cat", "/etc/crontab"]).await {
            for line in output.lines() {
                if !line.starts_with('#') && !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let schedule = format!("{} {} {} {} {}", parts[0], parts[1], parts[2], parts[3], parts[4]);
                        let command = parts[5..].join(" ");
                        jobs.push(CronJob {
                            schedule,
                            command,
                            user: "root".to_string(),
                        });
                    }
                }
            }
        }

        // User crontab
        if let Ok(output) = process::exec_command(&["crontab", "-l"]).await {
            for line in output.lines() {
                if !line.starts_with('#') && !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let schedule = format!("{} {} {} {} {}", parts[0], parts[1], parts[2], parts[3], parts[4]);
                        let command = parts[5..].join(" ");
                        jobs.push(CronJob {
                            schedule,
                            command,
                            user: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
                        });
                    }
                }
            }
        }

        Ok(serde_json::to_value(&jobs)?)
    });
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
