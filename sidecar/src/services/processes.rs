//! Top processes for task-manager tile — sysinfo-backed.
use crate::services::ServiceRegistry;
use anyhow;
use serde_json::{json, Value};
use std::process::Stdio;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use tokio::process::Command;

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Process.ListTop", |params| async move {
        let limit = params
            .as_ref()
            .and_then(|p| p.get("limit"))
            .and_then(|n| n.as_u64())
            .unwrap_or(18) as usize;

        let mut sys = System::new_all();
        sys.refresh_processes();

        let mut rows: Vec<(Pid, f32, String)> = sys
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let cpu = proc.cpu_usage();
                let name = proc.name().to_string();
                (*pid, cpu, name)
            })
            .collect();

        let out = build_list_top_json(&mut rows, limit);
        Ok(json!(out))
    });

    registry.register("Process.Kill", |params| async move {
        let pid_u = params
            .as_ref()
            .and_then(|p| p.get("pid"))
            .and_then(|n| n.as_u64())
            .ok_or_else(|| anyhow::anyhow!("missing pid"))? as u32;

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

pub(crate) fn kill_pid_allowed(pid: u32) -> bool {
    pid > 1 && pid >= 100
}

/// Minimum rows returned by `Process.ListTop` even when `limit` is smaller.
pub const LIST_TOP_MIN_ROWS: usize = 5;

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
