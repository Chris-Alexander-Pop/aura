use crate::services::ServiceRegistry;
use crate::utils::process;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: String,
}

/// Map journal PRIORITY (0=emerg … 7=debug) to UI bucket.
pub fn priority_to_level(priority: i64) -> String {
    match priority {
        0..=3 => "err".to_string(),
        4 => "warn".to_string(),
        5 | 6 => "info".to_string(),
        _ => "debug".to_string(),
    }
}

/// Minimum journal priority number for a filter level name.
fn min_priority_for_filter(level: &str) -> Option<&'static str> {
    match level.to_lowercase().as_str() {
        "err" | "error" => Some("err"),
        "warn" | "warning" => Some("warning"),
        "info" | "notice" => Some("info"),
        "debug" => Some("debug"),
        _ => None,
    }
}

pub fn parse_journal_json_value(v: &Value) -> Option<LogEntry> {
    let obj = v.as_object()?;
    let message = obj
        .get("MESSAGE")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    if message.is_empty() {
        return None;
    }
    let priority = obj
        .get("PRIORITY")
        .and_then(|p| p.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(6);
    let level = priority_to_level(priority);
    let service = obj
        .get("_SYSTEMD_UNIT")
        .or_else(|| obj.get("SYSLOG_IDENTIFIER"))
        .and_then(|u| u.as_str())
        .unwrap_or("journal")
        .to_string();
    let timestamp = obj
        .get("__REALTIME_TIMESTAMP")
        .and_then(|t| t.as_str())
        .map(|micros| {
            if let Ok(us) = micros.parse::<i64>() {
                let secs = us / 1_000_000;
                chrono::DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
                    .unwrap_or_else(|| micros.to_string())
            } else {
                micros.to_string()
            }
        })
        .unwrap_or_default();
    Some(LogEntry {
        timestamp,
        level,
        message,
        service,
    })
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Logs.Get", |params| async move {
        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(100);

        let priority: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("priority").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let unit: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("unit").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let grep: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("grep").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let lines_str = lines.to_string();
        let mut cmd = vec![
            "journalctl",
            "-n",
            &lines_str,
            "--no-pager",
            "-o",
            "json",
        ];
        if let Some(ref u) = unit {
            cmd.extend_from_slice(&["-u", u]);
        }
        if let Some(ref g) = grep {
            cmd.extend_from_slice(&["--grep", g]);
        }
        if let Some(ref lvl) = priority {
            if let Some(p) = min_priority_for_filter(lvl) {
                cmd.extend_from_slice(&["-p", p]);
            }
        }

        let output = process::exec_command(&cmd).await.unwrap_or_default();

        let mut entries: Vec<LogEntry> = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|v| parse_journal_json_value(&v))
            .collect();

        entries.reverse();
        Ok(serde_json::to_value(&entries)?)
    });

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
        Ok(json!({ "logs": output }))
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
        Ok(json!({ "logs": output }))
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
        Ok(json!({ "logs": output }))
    });

    registry.register("Logs.FollowLogs", |_params| async move {
        Ok(json!({ "message": "Following logs (use notifications for updates)" }))
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

        Ok(json!({ "success": true }))
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
        Ok(json!({ "success": true }))
    });

    registry.register("Logs.GetLogLevels", |_params| async move {
        Ok(json!(["emerg", "alert", "crit", "err", "warning", "notice", "info", "debug"]))
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
        Ok(json!({ "logs": output }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn priority_mapping() {
        assert_eq!(priority_to_level(3), "err");
        assert_eq!(priority_to_level(4), "warn");
        assert_eq!(priority_to_level(6), "info");
        assert_eq!(priority_to_level(7), "debug");
    }

    #[test]
    fn parse_journal_json_fixture() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/logs/journal_json.ndjson");
        let text = fs::read_to_string(path).expect("fixture");
        let entry = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|v| parse_journal_json_value(&v))
            .next()
            .expect("entry");
        assert_eq!(entry.level, "err");
        assert!(!entry.message.is_empty());
    }
}
