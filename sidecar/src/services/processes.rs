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

        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rows.truncate(limit.max(5));

        let out: Vec<Value> = rows
            .into_iter()
            .map(|(pid, cpu, name)| {
                json!({
                    "pid": pid.as_u32(),
                    "cpu": cpu,
                    "name": name,
                })
            })
            .collect();

        Ok(json!(out))
    });

    registry.register("Process.Kill", |params| async move {
        let pid_u = params
            .as_ref()
            .and_then(|p| p.get("pid"))
            .and_then(|n| n.as_u64())
            .ok_or_else(|| anyhow::anyhow!("missing pid"))? as u32;

        if pid_u <= 1 || pid_u < 100 {
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
