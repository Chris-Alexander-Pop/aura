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
    if let Ok(path) = std::env::var("AURA_PACKAGE_TX_LOG") {
        return Ok(PathBuf::from(path));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOG_ENV: Mutex<()> = Mutex::new(());

    fn temp_log_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "ags-sidecar-tx-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn log_and_read_round_trip_in_temp_dir() {
        let _guard = TEST_LOG_ENV.lock().unwrap();
        let path = temp_log_path();
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AURA_PACKAGE_TX_LOG", path.to_string_lossy().to_string());

        log_package_transaction("install", vec!["foo".into()], true, None).await;
        log_package_transaction(
            "remove",
            vec!["bar".into()],
            false,
            Some("permission denied".into()),
        )
        .await;

        let rows = read_package_transactions(10).await.expect("read");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].action, "install");
        assert!(rows[0].success);
        assert_eq!(rows[0].packages, vec!["foo"]);
        assert_eq!(rows[1].action, "remove");
        assert!(!rows[1].success);
        assert_eq!(rows[1].error.as_deref(), Some("permission denied"));

        let limited = read_package_transactions(1).await.expect("read limit");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].action, "remove");

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("AURA_PACKAGE_TX_LOG");
    }

    #[tokio::test]
    async fn read_missing_log_returns_empty() {
        let _guard = TEST_LOG_ENV.lock().unwrap();
        let path = temp_log_path();
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AURA_PACKAGE_TX_LOG", path.to_string_lossy().to_string());

        let rows = read_package_transactions(5).await.expect("read");
        assert!(rows.is_empty());

        std::env::remove_var("AURA_PACKAGE_TX_LOG");
    }
}
