//! Hyprland IPC via `hyprctl` for the React bar (no GJS `hyprland.ts` in WebKit).
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::{bail, Result};
use serde_json::{json, Value};

async fn hyprctl_json(args: &[&str]) -> Result<Value> {
    let mut cmd: Vec<&str> = vec!["hyprctl", "-j"];
    cmd.extend_from_slice(args);
    let s = process::exec_command(&cmd).await?;
    let v: Value = serde_json::from_str(&s).map_err(|e| anyhow::anyhow!("json: {e}"))?;
    Ok(v)
}

async fn hyprctl_json_or_empty(args: &[&str]) -> Value {
    match hyprctl_json(args).await {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!("hyprctl failed: {e}");
            json!(null)
        }
    }
}

/// `hyprctl dispatch` with a full command string, e.g. "workspace 2" or "movetoworkspace 3"
async fn hyprctl_dispatch(d: &str) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;
    let parts: Vec<&str> = d.split_ascii_whitespace().collect();
    if parts.is_empty() {
        bail!("empty dispatch");
    }
    let mut c = Command::new("hyprctl");
    c.arg("dispatch");
    for p in parts {
        c.arg(p);
    }
    c.stdin(Stdio::null());
    let st = c.status().await?;
    if !st.success() {
        bail!("hyprctl dispatch failed: {st}");
    }
    Ok(())
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Hyprland.GetWorkspaces", |_p| async move {
        Ok(hyprctl_json_or_empty(&["workspaces"]).await)
    });

    registry.register("Hyprland.GetActiveWorkspace", |_p| async move {
        Ok(hyprctl_json_or_empty(&["activeworkspace"]).await)
    });

    registry.register("Hyprland.GetClients", |_p| async move {
        Ok(hyprctl_json_or_empty(&["clients"]).await)
    });

    registry.register("Hyprland.GetActiveWindow", |_p| async move {
        Ok(hyprctl_json_or_empty(&["activewindow"]).await)
    });

    registry.register("Hyprland.GetMonitors", |_p| async move {
        Ok(hyprctl_json_or_empty(&["monitors"]).await)
    });

    registry.register("Hyprland.Dispatch", |params| async move {
        let cmd = params
            .as_ref()
            .and_then(|p| p.get("command"))
            .and_then(|c| c.as_str())
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("missing command"))?;

        hyprctl_dispatch(&cmd).await?;
        Ok(json!({"ok": true}))
    });
}
