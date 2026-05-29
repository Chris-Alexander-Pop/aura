use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunEntry {
    pub ts: String,
    pub workflow_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn log_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("AURA_AUTOMATION_RUN_LOG") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir)
            .join("ags-sidecar")
            .join("automation_runs.jsonl"));
    }
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("ags-sidecar")
        .join("automation_runs.jsonl"))
}

pub async fn log_workflow_run(workflow_id: &str, success: bool, error: Option<String>) {
    if let Err(e) = log_workflow_run_inner(workflow_id, success, error).await {
        tracing::warn!("failed to log automation run: {}", e);
    }
}

async fn log_workflow_run_inner(
    workflow_id: &str,
    success: bool,
    error: Option<String>,
) -> Result<()> {
    let path = log_path()?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let entry = WorkflowRunEntry {
        ts: Utc::now().to_rfc3339(),
        workflow_id: workflow_id.to_string(),
        success,
        error,
    };
    let line = serde_json::to_string(&entry)?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    file.write_all(line.as_bytes()).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

pub async fn read_workflow_runs(workflow_id: Option<&str>, limit: usize) -> Result<Vec<WorkflowRunEntry>> {
    let path = log_path()?;
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for line in content.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<WorkflowRunEntry>(line) {
            if workflow_id.is_none() || entry.workflow_id == workflow_id.unwrap() {
                out.push(entry);
                if out.len() >= limit {
                    break;
                }
            }
        }
    }
    out.reverse();
    Ok(out)
}
