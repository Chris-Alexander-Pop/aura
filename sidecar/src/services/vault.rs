//! Vault panel backend: rclone remotes registry and backup status (read-only phase).

use crate::services::ServiceRegistry;
use crate::utils::{keyring, process, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultRemote {
    pub name: String,
    /// rclone backend type when known (e.g. `drive`, `s3`); empty when unknown.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub remote_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultListResponse {
    pub remotes: Vec<VaultRemote>,
    pub rclone_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultBackupStatus {
    /// `idle` | `running` | `error` | `unknown`
    pub state: String,
    pub engine: Option<String>,
    pub last_success_at: Option<i64>,
    pub last_error: Option<String>,
    pub in_progress: bool,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Vault.List", |_params| async move {
        let list = list_remotes().await?;
        Ok(serde_json::to_value(&list)?)
    });

    registry.register("Vault.Backup.Status", |_params| async move {
        let status = backup_status_stub();
        Ok(serde_json::to_value(&status)?)
    });

    registry.register("Vault.GetEntry", |params| async move {
        let key: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("key").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
        )?;
        validate_vault_key(&key)?;
        let value = keyring::lookup_vault_entry(&key).await?;
        Ok(json!({
            "key": key,
            "value": value,
        }))
    });

    registry.register("Vault.SetEntry", |params| async move {
        let key: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("key").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
        )?;
        let value: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("value").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing value"))?,
        )?;
        validate_vault_key(&key)?;
        keyring::store_vault_entry(&key, &value).await?;
        record_vault_audit("Vault.SetEntry", &key).await?;
        Ok(json!({ "success": true }))
    });
}

fn validate_vault_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 128 {
        anyhow::bail!("invalid vault key");
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        anyhow::bail!("invalid vault key");
    }
    Ok(())
}

async fn record_vault_audit(method: &str, key: &str) -> Result<()> {
    storage::init().await?;
    let entry = json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "method": method,
        "key": key,
        "user": std::env::var("USER").unwrap_or_else(|_| "unknown".into()),
    });
    let id = format!("audit_{}", chrono::Utc::now().timestamp_millis());
    storage::set_kv("vault_audit", &id, &entry).await?;
    Ok(())
}

/// Parse `rclone listremotes` lines (`name:`).
pub fn parse_rclone_listremotes(output: &str) -> Vec<VaultRemote> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let name = line.trim_end_matches(':').trim();
            if name.is_empty() {
                None
            } else {
                Some(VaultRemote {
                    name: name.to_string(),
                    remote_type: String::new(),
                })
            }
        })
        .collect()
}

async fn list_remotes() -> Result<VaultListResponse> {
    if let Ok(fixture) = std::env::var("AURA_VAULT_LISTREMOTES_OUTPUT") {
        let remotes = parse_rclone_listremotes(&fixture);
        return Ok(VaultListResponse {
            remotes,
            rclone_available: true,
        });
    }

    let rclone_available = process::exec_command(&["which", "rclone"]).await.is_ok();
    if !rclone_available {
        return Ok(VaultListResponse {
            remotes: Vec::new(),
            rclone_available: false,
        });
    }

    let output = process::exec_command(&["rclone", "listremotes"]).await?;
    Ok(VaultListResponse {
        remotes: parse_rclone_listremotes(&output),
        rclone_available: true,
    })
}

fn backup_status_stub() -> VaultBackupStatus {
    VaultBackupStatus {
        state: "idle".to_string(),
        engine: None,
        last_success_at: None,
        last_error: None,
        in_progress: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_listremotes_fixture() {
        let raw = "gdrive:\nnextcloud:\n# comment\n\ns3-remote:\n";
        let remotes = parse_rclone_listremotes(raw);
        assert_eq!(
            remotes,
            vec![
                VaultRemote {
                    name: "gdrive".into(),
                    remote_type: String::new(),
                },
                VaultRemote {
                    name: "nextcloud".into(),
                    remote_type: String::new(),
                },
                VaultRemote {
                    name: "s3-remote".into(),
                    remote_type: String::new(),
                },
            ]
        );
    }

    #[test]
    fn backup_status_stub_shape() {
        let status = backup_status_stub();
        assert_eq!(status.state, "idle");
        assert!(!status.in_progress);
        let v = serde_json::to_value(&status).unwrap();
        for key in ["state", "engine", "last_success_at", "last_error", "in_progress"] {
            assert!(v.get(key).is_some(), "missing {key}");
        }
    }
}
