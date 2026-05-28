use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTransaction {
    pub ts: String,
    pub action: String,
    pub packages: Vec<String>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn log_path() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir).join("ags-sidecar").join("transactions.jsonl"));
    }
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("ags-sidecar")
        .join("transactions.jsonl"))
}

/// Append a package transaction record (best-effort).
pub async fn log_package_transaction(
    action: &str,
    packages: Vec<String>,
    success: bool,
    error: Option<String>,
) {
    if let Err(e) = log_package_transaction_inner(action, packages, success, error).await {
        tracing::warn!("failed to log package transaction: {}", e);
    }
}

async fn log_package_transaction_inner(
    action: &str,
    packages: Vec<String>,
    success: bool,
    error: Option<String>,
) -> Result<()> {
    let path = log_path()?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let entry = PackageTransaction {
        ts: Utc::now().to_rfc3339(),
        action: action.to_string(),
        packages,
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

pub async fn read_package_transactions(limit: usize) -> Result<Vec<PackageTransaction>> {
    let path = log_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = tokio::fs::read_to_string(&path).await?;
    let mut rows: Vec<PackageTransaction> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    if rows.len() > limit {
        let skip = rows.len() - limit;
        rows = rows.split_off(skip);
    }
    Ok(rows)
}
