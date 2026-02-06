use crate::services::ServiceRegistry;
use crate::utils::process;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Logs.GetSystemLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(100);

        let lines_str = lines.to_string();
        let mut cmd = vec!["journalctl", "-n", &lines_str, "--no-pager"];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "logs": output }))
    });

    registry.register("Logs.GetApplicationLogs", |params| async move {
        let app_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("app_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing app_name"))?,
        )?;

        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(100);

        let lines_str = lines.to_string();
        let comm_filter = format!("_COMM={}", app_name);
        let output = process::exec_command(&["journalctl", "-n", &lines_str, "--no-pager", &comm_filter]).await?;
        Ok(serde_json::json!({ "logs": output }))
    });

    registry.register("Logs.SearchLogs", |params| async move {
        let query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("query").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?,
        )?;

        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["journalctl", "--no-pager", "--grep", &query];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "logs": output }))
    });

    registry.register("Logs.FollowLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        // This would need to stream logs via notifications
        // For now, just return recent logs
        let mut cmd = vec!["journalctl", "-n", "50", "--no-pager", "-f"];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }

        // Note: -f follows, but we can't easily stream this via JSON-RPC
        // Would need notification-based approach
        Ok(serde_json::json!({ "message": "Following logs (use notifications for updates)" }))
    });

    registry.register("Logs.GetLogServices", |_params| async move {
        let output = process::exec_command(&["systemctl", "list-units", "--type=service", "--no-pager", "--no-legend"]).await?;
        let mut services = Vec::new();

        for line in output.lines() {
            if let Some(name) = line.split_whitespace().next() {
                services.push(name.to_string());
            }
        }

        Ok(serde_json::to_value(&services)?)
    });

    registry.register("Logs.ClearLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        if let Some(ref svc) = service {
            process::exec_command(&["journalctl", "-u", svc, "--vacuum-time=1s"]).await?;
        } else {
            process::exec_command(&["journalctl", "--vacuum-time=1s"]).await?;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Logs.ExportLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let file_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("file_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?,
        )?;

        let mut cmd = vec!["journalctl", "--no-pager", "-o", "json"];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }

        let output = process::exec_command(&cmd).await?;
        tokio::fs::write(&file_path, output).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Logs.GetLogLevels", |_params| async move {
        Ok(serde_json::json!(["emerg", "alert", "crit", "err", "warning", "notice", "info", "debug"]))
    });

    registry.register("Logs.FilterLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let level: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("level").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["journalctl", "--no-pager"];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }
        if let Some(ref lvl) = level {
            cmd.extend_from_slice(&["-p", lvl]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "logs": output }))
    });
}
