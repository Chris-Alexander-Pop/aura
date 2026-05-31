//! Top processes for task-manager tile — sysinfo-backed.
use crate::services::ServiceRegistry;
use anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub status: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Process.ListTop", |params| async move {
        let limit = list_top_limit_from_params(params.as_ref());
        let rows = collect_top_process_rows(limit);
        let json_rows: Vec<Value> = rows
            .into_iter()
            .map(|r| {
                json!({
                    "pid": r.pid,
                    "cpu": r.cpu_percent,
                    "name": r.name,
                })
            })
            .collect();
        Ok(json!(json_rows))
    });

    registry.register("Process.Kill", |params| async move {
        let p = params
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing params"))?;
        let pid_u = p
            .get("pid")
            .and_then(|n| n.as_u64())
            .ok_or_else(|| anyhow::anyhow!("missing pid"))? as u32;

        let token = p
            .get("confirmation_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing confirmation_token"))?;

        if token != kill_confirmation_token(pid_u) {
            anyhow::bail!("invalid confirmation_token for pid {pid_u}");
        }

        if !kill_pid_allowed(pid_u) {
            anyhow::bail!("refusing to kill pid {pid_u}");
        }

        let st = Command::new("kill")
            .arg("-TERM")
            .arg(pid_u.to_string())
            .stdin(Stdio::null())
            .status()
            .await?;

        if !st.success() {
            anyhow::bail!("kill failed");
        }
        Ok(json!({"ok": true}))
    });
}

/// Token the UI must send after the user confirms process termination.
pub fn kill_confirmation_token(pid: u32) -> String {
    format!("confirm-kill-{pid}")
}

pub fn list_top_limit_from_params(params: Option<&Value>) -> usize {
    params
        .and_then(|p| p.get("limit"))
        .and_then(|n| n.as_u64())
        .unwrap_or(18) as usize
}

pub(crate) fn kill_pid_allowed(pid: u32) -> bool {
    pid > 1 && pid >= 100
}

/// Minimum rows returned by `Process.ListTop` even when `limit` is smaller.
pub const LIST_TOP_MIN_ROWS: usize = 5;

/// Collect top processes by CPU usage (shared with `Performance.GetProcesses`).
pub fn collect_top_process_rows(limit: usize) -> Vec<ProcessRow> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut rows: Vec<(Pid, f64, u64, String, String)> = sys
        .processes()
        .iter()
        .map(|(pid, proc)| {
            (
                *pid,
                proc.cpu_usage() as f64,
                proc.memory() / 1024,
                proc.name().to_string(),
                format!("{:?}", proc.status()),
            )
        })
        .collect();

    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let cap = limit.max(LIST_TOP_MIN_ROWS);
    rows.into_iter()
        .take(cap)
        .map(|(pid, cpu, mem, name, status)| ProcessRow {
            pid: pid.as_u32(),
            cpu_percent: cpu,
            memory_mb: mem,
            name,
            status,
        })
        .collect()
}

/// Sort by CPU descending, cap length, and serialize process rows for RPC.
pub fn build_list_top_json(rows: &mut [(Pid, f32, String)], limit: usize) -> Vec<Value> {
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let cap = limit.max(LIST_TOP_MIN_ROWS);
    rows.iter()
        .take(cap)
        .map(|(pid, cpu, name)| {
            json!({
                "pid": pid.as_u32(),
                "cpu": cpu,
                "name": name,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::{Pid, PidExt};

    #[test]
    fn refuses_low_pids() {
        assert!(!kill_pid_allowed(1));
        assert!(!kill_pid_allowed(50));
        assert!(kill_pid_allowed(100));
        assert!(kill_pid_allowed(4242));
    }

    #[test]
    fn kill_confirmation_token_is_deterministic() {
        assert_eq!(kill_confirmation_token(4242), "confirm-kill-4242");
    }

    #[test]
    fn build_list_top_json_sorts_and_enforces_minimum() {
        let pid = |n| Pid::from_u32(n);
        let mut rows = vec![
            (pid(1), 1.0, "low".into()),
            (pid(2), 50.0, "high".into()),
            (pid(3), 10.0, "mid".into()),
            (pid(4), 5.0, "a".into()),
            (pid(5), 4.0, "b".into()),
            (pid(6), 3.0, "c".into()),
        ];
        let out = build_list_top_json(&mut rows, 2);
        assert_eq!(out.len(), LIST_TOP_MIN_ROWS);
        assert_eq!(out[0].get("name").and_then(|v| v.as_str()), Some("high"));
    }
}
