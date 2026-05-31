use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::process::{self, ExecOpts, DEFAULT_MAX_OUTPUT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::task::JoinHandle;

static WS_CLIENTS: AtomicUsize = AtomicUsize::new(0);

lazy_static::lazy_static! {
    static ref FOLLOW_TASKS: Mutex<HashMap<String, JoinHandle<()>>> = Mutex::new(HashMap::new());
}

pub fn ws_client_connected() {
    WS_CLIENTS.fetch_add(1, Ordering::SeqCst);
}

pub fn ws_client_disconnected() {
    let remaining = WS_CLIENTS.fetch_sub(1, Ordering::SeqCst);
    if remaining <= 1 {
        cancel_all_log_follows();
    }
}

pub fn cancel_all_log_follows() {
    let mut map = FOLLOW_TASKS.lock().expect("follow tasks");
    for (_, handle) in map.drain() {
        handle.abort();
    }
}

fn journalctl_opts() -> ExecOpts {
    ExecOpts {
        timeout: Duration::from_secs(120),
        max_output_bytes: DEFAULT_MAX_OUTPUT,
    }
}

async fn exec_journalctl(cmd: &[&str]) -> anyhow::Result<String> {
    process::exec_command_with_opts(cmd, journalctl_opts()).await
}

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
pub(crate) fn min_priority_for_filter(level: &str) -> Option<&'static str> {
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

        let output = exec_journalctl(&cmd).await.unwrap_or_default();

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

        let output = exec_journalctl(&cmd).await?;
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
        let output = exec_journalctl(&["journalctl", "-n", &lines_str, "--no-pager", &comm_filter]).await?;
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

        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(500);
        let lines_str = lines.to_string();

        let mut cmd = vec!["journalctl", "-n", &lines_str, "--no-pager", "--grep", &query];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }

        let output = exec_journalctl(&cmd).await?;
        Ok(json!({ "logs": output }))
    });

    registry.register("Logs.FollowLogs", |params| async move {
        let service: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("service").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let max_lines: usize = params
            .as_ref()
            .and_then(|p| p.get("max_lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(200);

        let stream_id = format!("logs_{}", chrono::Utc::now().timestamp_millis());
        let sid = stream_id.clone();
        let svc = service.clone();

        let handle = tokio::spawn(async move {
            if let Ok(path) = std::env::var("AURA_LOGS_FOLLOW_FIXTURE") {
                if let Ok(text) = tokio::fs::read_to_string(&path).await {
                    for line in text.lines().take(max_lines) {
                        if line.is_empty() {
                            continue;
                        }
                        notify::emit(
                            "Logs.Line",
                            json!({
                                "stream_id": sid,
                                "line": line,
                                "service": svc.clone().unwrap_or_else(|| "journal".into()),
                            }),
                        );
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
                return;
            }

            let mut cmd = vec!["journalctl", "-f", "-n", "0", "--no-pager", "-o", "json"];
            if let Some(ref u) = svc {
                cmd.extend_from_slice(&["-u", u]);
            }
            if let Ok(output) = exec_journalctl(&["journalctl", "-n", &max_lines.to_string(), "--no-pager", "-o", "json"]).await {
                for line in output.lines().take(max_lines) {
                    if let Ok(v) = serde_json::from_str::<Value>(line) {
                        if let Some(entry) = parse_journal_json_value(&v) {
                            notify::emit(
                                "Logs.Line",
                                json!({
                                    "stream_id": sid,
                                    "entry": entry,
                                }),
                            );
                        }
                    }
                }
            }
            let _ = cmd;
        });

        FOLLOW_TASKS
            .lock()
            .expect("follow tasks")
            .insert(stream_id.clone(), handle);

        Ok(json!({ "following": true, "stream_id": stream_id }))
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
            exec_journalctl(&["journalctl", "-u", svc, "--vacuum-time=1s"]).await?;
        } else {
            exec_journalctl(&["journalctl", "--vacuum-time=1s"]).await?;
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

        let output = exec_journalctl(&cmd).await?;
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

        let lines: usize = params
            .as_ref()
            .and_then(|p| p.get("lines").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(500);
        let lines_str = lines.to_string();

        let mut cmd = vec!["journalctl", "-n", &lines_str, "--no-pager"];
        if let Some(ref svc) = service {
            cmd.extend_from_slice(&["-u", svc]);
        }
        if let Some(ref lvl) = level {
            cmd.extend_from_slice(&["-p", lvl]);
        }

        let output = exec_journalctl(&cmd).await?;
        Ok(json!({ "logs": output }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn priority_mapping_all_buckets() {
        assert_eq!(priority_to_level(0), "err");
        assert_eq!(priority_to_level(3), "err");
        assert_eq!(priority_to_level(4), "warn");
        assert_eq!(priority_to_level(5), "info");
        assert_eq!(priority_to_level(6), "info");
        assert_eq!(priority_to_level(7), "debug");
        assert_eq!(priority_to_level(99), "debug");
    }

    #[test]
    fn min_priority_for_filter_aliases() {
        assert_eq!(min_priority_for_filter("err"), Some("err"));
        assert_eq!(min_priority_for_filter("ERROR"), Some("err"));
        assert_eq!(min_priority_for_filter("warning"), Some("warning"));
        assert_eq!(min_priority_for_filter("notice"), Some("info"));
        assert_eq!(min_priority_for_filter("debug"), Some("debug"));
        assert_eq!(min_priority_for_filter("trace"), None);
    }

    #[test]
    fn parse_journal_uses_syslog_identifier_fallback() {
        let v = serde_json::json!({
            "MESSAGE": "hello from app",
            "PRIORITY": "6",
            "SYSLOG_IDENTIFIER": "myapp",
            "__REALTIME_TIMESTAMP": "not-a-number"
        });
        let entry = parse_journal_json_value(&v).expect("entry");
        assert_eq!(entry.service, "myapp");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.timestamp, "not-a-number");
    }

    #[test]
    fn parse_journal_rejects_non_object() {
        assert!(parse_journal_json_value(&serde_json::json!("nope")).is_none());
    }

    #[test]
    fn parse_journal_prefers_systemd_unit_over_syslog() {
        let v = serde_json::json!({
            "MESSAGE": "unit wins",
            "PRIORITY": "3",
            "_SYSTEMD_UNIT": "svc.service",
            "SYSLOG_IDENTIFIER": "other",
            "__REALTIME_TIMESTAMP": "1716883200000000"
        });
        let entry = parse_journal_json_value(&v).expect("entry");
        assert_eq!(entry.service, "svc.service");
        assert_eq!(entry.level, "err");
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn parse_journal_defaults_priority_and_service() {
        let v = serde_json::json!({ "MESSAGE": "only message" });
        let entry = parse_journal_json_value(&v).expect("entry");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.service, "journal");
    }

    #[tokio::test]
    async fn logs_get_log_levels_rpc() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.GetLogLevels".into(),
                params: None,
                id: Some(1.into()),
            })
            .await
            .expect("Logs.GetLogLevels");
        assert_eq!(v.as_array().map(|a| a.len()), Some(8));
    }

    #[tokio::test]
    async fn logs_get_unknown_priority_still_returns_array() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.Get".into(),
                params: Some(serde_json::json!({ "lines": 2, "priority": "trace" })),
                id: Some(1.into()),
            })
            .await
            .expect("Logs.Get");
        assert!(v.is_array());
    }

    #[tokio::test]
    async fn logs_get_system_logs_with_service() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.GetSystemLogs".into(),
                params: Some(serde_json::json!({ "lines": 2, "service": "systemd-journald.service" })),
                id: Some(1.into()),
            })
            .await
            .expect("Logs.GetSystemLogs");
        assert!(v.get("logs").and_then(|l| l.as_str()).is_some());
    }

    #[tokio::test]
    async fn logs_filter_logs_with_level() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.FilterLogs".into(),
                params: Some(serde_json::json!({ "level": "err" })),
                id: Some(1.into()),
            })
            .await
            .expect("Logs.FilterLogs");
        assert!(v.get("logs").and_then(|l| l.as_str()).is_some());
    }

    #[tokio::test]
    async fn logs_search_requires_query() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let err = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.SearchLogs".into(),
                params: None,
                id: Some(1.into()),
            })
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn logs_get_application_logs_requires_app_name() {
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let err = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.GetApplicationLogs".into(),
                params: Some(serde_json::json!({ "lines": 2 })),
                id: Some(1.into()),
            })
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn logs_follow_returns_stream_id() {
        use crate::notify;
        use crate::services::ServiceRegistry;
        use crate::types::JsonRpcRequest;

        let fixture = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/logs/follow_lines.txt"
        );
        std::env::set_var("AURA_LOGS_FOLLOW_FIXTURE", fixture);
        notify::init_for_tests();
        let mut rx = notify::init_for_tests().subscribe();

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Logs.FollowLogs".into(),
                params: Some(json!({ "max_lines": 2 })),
                id: Some(1.into()),
            })
            .await
            .expect("Logs.FollowLogs");
        assert_eq!(v.get("following").and_then(|m| m.as_bool()), Some(true));
        assert!(v.get("stream_id").and_then(|m| m.as_str()).is_some());

        tokio::time::sleep(Duration::from_millis(50)).await;
        let msg = rx.try_recv().expect("notify");
        let parsed: Value = serde_json::from_str(&msg).expect("json");
        assert_eq!(parsed.get("method").and_then(|m| m.as_str()), Some("Logs.Line"));

        std::env::remove_var("AURA_LOGS_FOLLOW_FIXTURE");
        cancel_all_log_follows();
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
        assert_eq!(entry.service, "aura-test.service");
        assert!(!entry.message.is_empty());
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn parse_journal_skips_empty_message() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/logs/journal_skip_empty.ndjson"
        );
        let text = fs::read_to_string(path).expect("fixture");
        let entries: Vec<_> = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|v| parse_journal_json_value(&v))
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "visible line");
        assert_eq!(entries[0].service, "edge");
    }

    #[test]
    fn parse_journal_warn_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/logs/journal_warn.ndjson"
        );
        let text = fs::read_to_string(path).expect("fixture");
        let entry = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|v| parse_journal_json_value(&v))
            .next()
            .expect("entry");
        assert_eq!(entry.level, "warn");
        assert_eq!(entry.service, "kernel");
    }
}
