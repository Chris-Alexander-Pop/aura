//! Hyprland IPC via `hyprctl` for the React bar (no GJS `hyprland.ts` in WebKit).
use crate::notify;
use crate::services::ServiceRegistry;
use crate::types::{
    HyprActiveWindow, HyprActiveWorkspace, HyprClient, HyprMonitor, HyprWorkspace,
    HyprWorkspaceRef,
};
use crate::utils::process;
use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;

static HYPRLAND_EMIT_GEN: AtomicU64 = AtomicU64::new(0);
/// Last `activewindowv2` address — title-only churn on the focused window must
/// not schedule a full StateChanged (Cursor agent spinners emit activewindow*).
static LAST_ACTIVE_WINDOW_ADDR: Mutex<String> = Mutex::new(String::new());

const STATE_CHANGED_AREAS: &[&str] = &["workspaces", "clients", "active"];

async fn hyprctl_json(args: &[&str]) -> Result<Value> {
    let mut cmd: Vec<&str> = vec!["hyprctl", "-j"];
    cmd.extend_from_slice(args);
    let s = process::exec_command(&cmd).await?;
    let v: Value = serde_json::from_str(&s).map_err(|e| anyhow!("json: {e}"))?;
    Ok(v)
}

async fn hyprctl_json_or_null(args: &[&str]) -> Value {
    match hyprctl_json(args).await {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!("hyprctl failed: {e}");
            json!(null)
        }
    }
}

/// `hyprctl dispatch` with a validated command string.
async fn hyprctl_dispatch(parts: &[String]) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;
    if parts.is_empty() {
        bail!("empty dispatch");
    }
    let mut c = Command::new("hyprctl");
    c.arg("dispatch");
    for p in parts {
        c.arg(p);
    }
    c.stdin(Stdio::null());
    c.stdout(Stdio::null());
    c.stderr(Stdio::null());
    let st = c.status().await?;
    if !st.success() {
        bail!("hyprctl dispatch failed: {st}");
    }
    Ok(())
}

fn json_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_u64().map(|u| u as i64)),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// Numeric workspace id from classic `{id: 1}` or Lua-config
/// `{address: "1", type: "numbered", name: "1"}` (no `id` field).
pub fn parse_workspace_numeric_id(item: &Value) -> Option<i64> {
    item.get("id")
        .and_then(json_i64)
        .or_else(|| item.get("address").and_then(json_i64))
        .or_else(|| item.get("name").and_then(json_i64))
}

fn hypr_string_field<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

fn is_open_special_workspace(item: &Value) -> bool {
    let ws_type = hypr_string_field(item, "type");
    let address = hypr_string_field(item, "address");
    let name = hypr_string_field(item, "name");
    ws_type == "special"
        || address.starts_with("special:")
        || name.starts_with("special:")
}

pub fn parse_workspaces(raw: &Value) -> Vec<HyprWorkspace> {
    let Some(arr) = raw.as_array() else {
        return vec![];
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(id) = parse_workspace_numeric_id(item).filter(|id| *id > 0) else {
            continue;
        };
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| id.to_string());
        let windows = item
            .get("windows")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        out.push(HyprWorkspace {
            id,
            name,
            windows,
        });
    }
    out.sort_by_key(|w| w.id);
    out
}

pub fn parse_active_workspace(raw: &Value) -> Option<HyprActiveWorkspace> {
    if raw.is_null() {
        return None;
    }
    let id = parse_workspace_numeric_id(raw).filter(|id| *id > 0)?;
    let name = raw
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| id.to_string());
    Some(HyprActiveWorkspace { id, name })
}

fn parse_workspace_ref(item: &Value) -> Option<HyprWorkspaceRef> {
    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            let address = hypr_string_field(item, "address");
            if address.is_empty() {
                None
            } else {
                Some(address.to_string())
            }
        });
    if let Some(id) = parse_workspace_numeric_id(item) {
        return Some(HyprWorkspaceRef { id, name });
    }
    if is_open_special_workspace(item) {
        return Some(HyprWorkspaceRef { id: -1, name });
    }
    Some(HyprWorkspaceRef { id: 0, name })
}

fn parse_fullscreen(raw: &Value) -> u8 {
    match raw.get("fullscreen") {
        Some(Value::Bool(b)) => u8::from(*b),
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0).min(255) as u8,
        _ => 0,
    }
}

fn parse_client_monitor(item: &Value) -> i64 {
    item.get("monitor").and_then(|v| v.as_i64()).unwrap_or(-1)
}

pub fn parse_clients(raw: &Value) -> Vec<HyprClient> {
    let Some(arr) = raw.as_array() else {
        return vec![];
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(address) = item
            .get("address")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let class_name = item
            .get("class")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let workspace = item
            .get("workspace")
            .and_then(parse_workspace_ref)
            .unwrap_or(HyprWorkspaceRef {
                id: -1,
                name: None,
            });
        let floating = item.get("floating").and_then(|v| v.as_bool()).unwrap_or(false);
        let fullscreen = parse_fullscreen(item);
        let monitor = parse_client_monitor(item);
        out.push(HyprClient {
            address: address.to_string(),
            title,
            class_name,
            workspace,
            floating,
            fullscreen,
            monitor,
        });
    }
    out
}

pub fn parse_active_window(raw: &Value) -> Option<HyprActiveWindow> {
    if raw.is_null() {
        return None;
    }
    let address = raw.get("address")?.as_str()?;
    if address.is_empty() {
        return None;
    }
    let title = raw
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let class_name = raw
        .get("class")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let workspace = raw
        .get("workspace")
        .and_then(parse_workspace_ref)
        .unwrap_or(HyprWorkspaceRef {
            id: -1,
            name: None,
        });
    let floating = raw.get("floating").and_then(|v| v.as_bool()).unwrap_or(false);
    let fullscreen = parse_fullscreen(raw);
    let monitor = parse_client_monitor(raw);
    Some(HyprActiveWindow {
        address: address.to_string(),
        title,
        class_name,
        workspace,
        floating,
        fullscreen,
        monitor,
    })
}

/// Workspace ids currently visible on a monitor (active + open special).
fn visible_workspace_ids(mon: &HyprMonitor) -> Vec<i64> {
    let mut ids = Vec::with_capacity(2);
    if mon.active_workspace.id != 0 {
        ids.push(mon.active_workspace.id);
    }
    // Hyprland reports `specialWorkspace.id == 0` when no special is shown.
    if mon.special_workspace.id != 0 {
        ids.push(mon.special_workspace.id);
    }
    ids
}

/// Monitor ids whose *visible* workspace has a fullscreen (or maximized) client.
///
/// Fullscreen clients on inactive workspaces do not suppress the shell — only the
/// workspace (or open special) you are looking at.
pub fn fullscreen_monitor_ids(clients: &[HyprClient], monitors: &[HyprMonitor]) -> Vec<i64> {
    if monitors.is_empty() {
        // Fallback when monitor list is unavailable: any fullscreen client.
        let mut ids: Vec<i64> = clients
            .iter()
            .filter(|c| c.fullscreen > 0)
            .map(|c| c.monitor)
            .filter(|&id| id >= 0)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        return ids;
    }

    let mut ids: Vec<i64> = Vec::new();
    for mon in monitors {
        let visible = visible_workspace_ids(mon);
        if visible.is_empty() {
            continue;
        }
        let has_fs = clients.iter().any(|c| {
            c.fullscreen > 0 && c.monitor == mon.id && visible.contains(&c.workspace.id)
        });
        if has_fs {
            ids.push(mon.id);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn parse_monitors(raw: &Value) -> Vec<HyprMonitor> {
    let Some(arr) = raw.as_array() else {
        return vec![];
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(name) = item
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let active_workspace = item
            .get("activeWorkspace")
            .or_else(|| item.get("active_workspace"))
            .and_then(parse_workspace_ref)
            .unwrap_or(HyprWorkspaceRef {
                id: -1,
                name: None,
            });
        let special_workspace = item
            .get("specialWorkspace")
            .or_else(|| item.get("special_workspace"))
            .and_then(parse_workspace_ref)
            .unwrap_or(HyprWorkspaceRef {
                id: 0,
                name: None,
            });
        let focused = item
            .get("focused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let x = item.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
        let y = item.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
        let width = item.get("width").and_then(|v| v.as_i64()).unwrap_or(0);
        let height = item.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
        out.push(HyprMonitor {
            name: name.to_string(),
            id,
            active_workspace,
            special_workspace,
            focused,
            x,
            y,
            width,
            height,
        });
    }
    out
}

/// Validate a bar-safe `hyprctl dispatch` command before execution.
pub fn validate_dispatch(command: &str) -> Result<Vec<String>> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        bail!("dispatch_denied: empty command");
    }
    if trimmed.contains(';')
        || trimmed.contains('|')
        || trimmed.contains('`')
        || trimmed.contains('\n')
        || trimmed.contains('\r')
    {
        bail!("dispatch_denied: shell metacharacters forbidden");
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let verb = parts[0];
    match verb {
        "workspace" | "movetoworkspace" => {
            if parts.len() != 2 {
                bail!("dispatch_denied: {verb} requires one argument");
            }
            validate_workspace_arg(parts[1])?;
        }
        "focuswindow" => {
            if parts.len() != 2 {
                bail!("dispatch_denied: focuswindow requires address:…");
            }
            let arg = parts[1];
            let hex = arg
                .strip_prefix("address:")
                .ok_or_else(|| anyhow!("dispatch_denied: focuswindow requires address: prefix"))?;
            validate_hex_address(hex)?;
        }
        "togglefloating" | "togglespecialfloating" | "pin" | "unpin" | "killactive" => {
            if parts.len() != 1 {
                bail!("dispatch_denied: {verb} takes no arguments");
            }
        }
        "togglespecialworkspace" => {
            if parts.len() != 2 {
                bail!("dispatch_denied: togglespecialworkspace requires one argument");
            }
            validate_special_workspace_arg(parts[1])?;
        }
        "movefocus" | "swapwindow" => {
            if parts.len() != 2 || !matches!(parts[1], "l" | "r" | "u" | "d") {
                bail!("dispatch_denied: {verb} requires direction l|r|u|d");
            }
        }
        "exec" | "execr" | "execdispatcher" | "keyword" | "pass" | "sendshortcut" => {
            bail!("dispatch_denied: verb not allowlisted: {verb}");
        }
        _ => bail!("dispatch_denied: verb not allowlisted: {verb}"),
    }
    Ok(parts.iter().map(|s| s.to_string()).collect())
}

fn validate_special_workspace_arg(arg: &str) -> Result<()> {
    if arg.is_empty() || arg.len() > 64 {
        bail!("dispatch_denied: invalid special workspace argument");
    }
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Ok(());
    }
    bail!("dispatch_denied: invalid special workspace argument")
}

fn validate_workspace_arg(arg: &str) -> Result<()> {
    if arg.is_empty() || arg.len() > 64 {
        bail!("dispatch_denied: invalid workspace argument");
    }
    if arg.chars().all(|c| c.is_ascii_digit()) {
        return Ok(());
    }
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Ok(());
    }
    bail!("dispatch_denied: invalid workspace argument")
}

fn validate_hex_address(hex: &str) -> Result<()> {
    let h = hex.strip_prefix("0x").unwrap_or(hex);
    if h.is_empty() || h.len() > 32 {
        bail!("dispatch_denied: invalid window address");
    }
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("dispatch_denied: invalid window address");
    }
    Ok(())
}

pub fn hyprland_events_enabled() -> bool {
    std::env::var("AURA_HYPRLAND_EVENTS")
        .ok()
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true)
}

fn hyprland_socket_path() -> Option<PathBuf> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let path = PathBuf::from(runtime)
        .join("hypr")
        .join(sig)
        .join(".socket2.sock");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Parse numeric workspace id from Hyprland socket2 `workspace>>` / `workspacev2>>` payload.
/// `workspace>>NAME` (name may be numeric); `workspacev2>>ID,NAME`.
pub fn parse_workspace_event_id(data: &str) -> Option<i64> {
    let trimmed = data.trim();
    let id_part = trimmed.split(',').next().unwrap_or(trimmed).trim();
    id_part.parse::<i64>().ok().filter(|&id| id > 0)
}

fn workspace_event_name(event: &str, data: &str, id: i64) -> String {
    if event == "workspacev2" {
        if let Some((_, name)) = data.split_once(',') {
            let name = name.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }
    } else {
        let name = data.trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    id.to_string()
}

/// Hyprland socket2 event names that should invalidate bar state.
///
/// Title animation events are excluded:
/// - `windowtitle` / `windowtitlev2` — every spinner frame
/// - `activewindow` — Hyprland re-emits this when the focused title changes
///
/// Focus changes use `activewindowv2` (address-only) with change detection in
/// [`note_hyprland_event_line`]. Titles still refresh on real focus changes and
/// the bar safety-net poll.
pub fn event_triggers_state_changed(event: &str) -> bool {
    matches!(
        event,
        "workspace"
            | "workspacev2"
            | "focusedmon"
            | "activewindowv2"
            | "openwindow"
            | "closewindow"
            | "movewindow"
            | "createworkspace"
            | "createworkspacev2"
            | "destroyworkspace"
            | "destroyworkspacev2"
            | "float"
            | "pin"
            | "fullscreen"
            // Special/scratch toggles (SUPER+S / SUPER+D). Closing onto an empty
            // regular workspace often emits only these — no workspace* change and
            // activewindowv2 may be empty (ignored), so the bar overlay stuck open.
            | "activespecial"
            | "activespecialv2"
            // Output hotplug — GDK alone often keeps phantom monitors, so Aura
            // must resync per-monitor shell windows from Hyprland's list.
            | "monitoradded"
            | "monitorremoved"
            | "monitoraddedv2"
            | "monitorremovedv2"
    )
}

fn active_window_address_changed(addr: &str) -> bool {
    let addr = addr.trim();
    if addr.is_empty() {
        return false;
    }
    let Ok(mut last) = LAST_ACTIVE_WINDOW_ADDR.lock() else {
        return true;
    };
    if last.as_str() == addr {
        return false;
    }
    last.clear();
    last.push_str(addr);
    true
}

pub fn note_hyprland_event_line(line: &str) {
    let Some((event, data)) = line.split_once(">>") else {
        return;
    };
    if event == "workspace" || event == "workspacev2" {
        if let Some(id) = parse_workspace_event_id(data) {
            notify::emit(
                "Hyprland.WorkspaceActive",
                json!({ "id": id, "name": workspace_event_name(event, data, id) }),
            );
        }
    }
    if event == "activewindowv2" {
        if active_window_address_changed(data) {
            schedule_hyprland_state_emit();
        }
        return;
    }
    if event_triggers_state_changed(event) {
        schedule_hyprland_state_emit();
    }
}

fn schedule_hyprland_state_emit() {
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    let gen = HYPRLAND_EMIT_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    tokio::spawn(async move {
        // Coalesce bursty socket2 events into one hyprctl pass.
        tokio::time::sleep(Duration::from_millis(50)).await;
        if HYPRLAND_EMIT_GEN.load(Ordering::Relaxed) != gen {
            return;
        }
        let (clients_raw, mon_raw) = tokio::join!(
            hyprctl_json_or_null(&["clients"]),
            hyprctl_json_or_null(&["monitors"]),
        );
        let clients = parse_clients(&clients_raw);
        let monitors = parse_monitors(&mon_raw);
        let fullscreen_monitor_ids = fullscreen_monitor_ids(&clients, &monitors);
        let monitor_tags: Vec<Value> = monitors
            .iter()
            .map(|m| json!({ "id": m.id, "name": m.name }))
            .collect();
        notify::emit(
            "Hyprland.StateChanged",
            json!({
                "areas": STATE_CHANGED_AREAS,
                "fullscreen_monitor_ids": fullscreen_monitor_ids,
                "monitors": monitor_tags,
            }),
        );
    });
}

async fn run_hyprland_event_loop() -> Result<()> {
    let path = hyprland_socket_path().ok_or_else(|| anyhow!("hyprland socket missing"))?;
    let stream = UnixStream::connect(&path).await?;
    let mut lines = BufReader::new(stream).lines();
    while let Some(line) = lines.next_line().await? {
        note_hyprland_event_line(&line);
    }
    Ok(())
}

/// Background Hyprland socket2 listener → debounced `Hyprland.StateChanged`.
pub fn spawn_event_listener() {
    if !hyprland_events_enabled() {
        tracing::debug!("hyprland event listener disabled (AURA_HYPRLAND_EVENTS)");
        return;
    }
    if hyprland_socket_path().is_none() {
        tracing::debug!("hyprland event listener skipped (no socket)");
        return;
    }
    tokio::spawn(async {
        loop {
            if let Err(e) = run_hyprland_event_loop().await {
                tracing::debug!("hyprland events ended: {e}");
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

async fn bar_snapshot() -> Value {
    let (ws_raw, active_raw, clients_raw, win_raw, mon_raw) = tokio::join!(
        hyprctl_json_or_null(&["workspaces"]),
        hyprctl_json_or_null(&["activeworkspace"]),
        hyprctl_json_or_null(&["clients"]),
        hyprctl_json_or_null(&["activewindow"]),
        hyprctl_json_or_null(&["monitors"]),
    );
    let clients = parse_clients(&clients_raw);
    let monitors = parse_monitors(&mon_raw);
    let fullscreen_monitor_ids = fullscreen_monitor_ids(&clients, &monitors);
    json!({
        "workspaces": parse_workspaces(&ws_raw),
        "active_workspace": parse_active_workspace(&active_raw),
        "clients": clients,
        "active_window": parse_active_window(&win_raw),
        "monitors": monitors,
        "fullscreen_monitor_ids": fullscreen_monitor_ids,
    })
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Hyprland.GetWorkspaces", |_p| async move {
        let raw = hyprctl_json_or_null(&["workspaces"]).await;
        let workspaces = parse_workspaces(&raw);
        Ok(serde_json::to_value(workspaces)?)
    });

    registry.register("Hyprland.GetBarSnapshot", |_p| async move {
        Ok(bar_snapshot().await)
    });

    registry.register("Hyprland.GetActiveWorkspace", |_p| async move {
        let raw = hyprctl_json_or_null(&["activeworkspace"]).await;
        let ws = parse_active_workspace(&raw);
        Ok(serde_json::to_value(ws)?)
    });

    registry.register("Hyprland.GetClients", |_p| async move {
        let raw = hyprctl_json_or_null(&["clients"]).await;
        let clients = parse_clients(&raw);
        Ok(serde_json::to_value(clients)?)
    });

    registry.register("Hyprland.GetActiveWindow", |_p| async move {
        let raw = hyprctl_json_or_null(&["activewindow"]).await;
        let win = parse_active_window(&raw);
        Ok(serde_json::to_value(win)?)
    });

    registry.register("Hyprland.GetMonitors", |_p| async move {
        let raw = hyprctl_json_or_null(&["monitors"]).await;
        let monitors = parse_monitors(&raw);
        Ok(serde_json::to_value(monitors)?)
    });

    registry.register("Hyprland.Dispatch", |params| async move {
        let cmd = params
            .as_ref()
            .and_then(|p| p.get("command"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow!("missing command"))?;

        let parts = validate_dispatch(cmd)?;
        hyprctl_dispatch(&parts).await?;
        schedule_hyprland_state_emit();
        Ok(json!({"ok": true}))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HyprMonitor, HyprWorkspaceRef};
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hyprland")
            .join(name);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        serde_json::from_str(&text).expect("fixture json")
    }

    fn hyprland_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hyprland")
            .join(name)
    }

    #[test]
    fn parse_workspaces_fixture() {
        let raw = fixture("workspaces.json");
        let ws = parse_workspaces(&raw);
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[0].id, 1);
        assert_eq!(ws[0].windows, 2);
    }

    #[test]
    fn parse_clients_fixture() {
        let raw = fixture("clients.json");
        let clients = parse_clients(&raw);
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].class_name, "firefox");
        assert_eq!(clients[0].workspace.id, 1);
        assert_eq!(clients[0].fullscreen, 0);
        assert_eq!(clients[0].monitor, -1);
    }

    #[test]
    fn parse_clients_fullscreen_fixture() {
        let raw = fixture("clients_fullscreen.json");
        let clients = parse_clients(&raw);
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].fullscreen, 1);
        assert_eq!(clients[0].monitor, 0);
        let monitors = parse_monitors(&fixture("monitors.json"));
        assert_eq!(fullscreen_monitor_ids(&clients, &monitors), vec![0]);
    }

    #[test]
    fn fullscreen_monitor_ids_ignores_inactive_workspace() {
        let clients = parse_clients(&fixture("clients_fullscreen.json"));
        // Same monitor, but active workspace is 2 while fullscreen client is on 1.
        let monitors = vec![HyprMonitor {
            name: "eDP-1".into(),
            id: 0,
            active_workspace: HyprWorkspaceRef {
                id: 2,
                name: Some("2".into()),
            },
            special_workspace: HyprWorkspaceRef {
                id: 0,
                name: None,
            },
            focused: true,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }];
        assert!(fullscreen_monitor_ids(&clients, &monitors).is_empty());
    }

    #[test]
    fn fullscreen_monitor_ids_includes_open_special() {
        let mut clients = parse_clients(&fixture("clients_fullscreen.json"));
        clients[0].workspace = HyprWorkspaceRef {
            id: -95,
            name: Some("special:communication".into()),
        };
        let monitors = vec![HyprMonitor {
            name: "eDP-1".into(),
            id: 0,
            active_workspace: HyprWorkspaceRef {
                id: 1,
                name: Some("1".into()),
            },
            special_workspace: HyprWorkspaceRef {
                id: -95,
                name: Some("special:communication".into()),
            },
            focused: true,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }];
        assert_eq!(fullscreen_monitor_ids(&clients, &monitors), vec![0]);
    }

    #[test]
    fn parse_active_window_fixture() {
        let raw = fixture("activewindow.json");
        let win = parse_active_window(&raw).unwrap();
        assert_eq!(win.address, "0x1a2b3c");
    }

    #[test]
    fn parse_active_workspace_fixture() {
        let raw = fixture("activeworkspace.json");
        let ws = parse_active_workspace(&raw).unwrap();
        assert_eq!(ws.id, 2);
    }

    #[test]
    fn parse_monitors_fixture() {
        let raw = fixture("monitors.json");
        let mon = parse_monitors(&raw);
        assert_eq!(mon.len(), 1);
        assert_eq!(mon[0].name, "HDMI-A-1");
    }

    #[test]
    fn parse_clients_skips_empty_address_fixture() {
        let raw = fixture("clients_skip_empty_address.json");
        let clients = parse_clients(&raw);
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].class_name, "kitty");
        assert!(clients[0].floating);
    }

    #[test]
    fn parse_workspaces_non_array_returns_empty() {
        let raw = fixture("workspaces_not_array.json");
        assert!(parse_workspaces(&raw).is_empty());
    }

    #[test]
    fn parse_active_window_null_fixture() {
        let raw = fixture("activewindow_null.json");
        assert!(parse_active_window(&raw).is_none());
    }

    #[test]
    fn parse_monitors_snake_case_active_workspace() {
        let raw = fixture("monitors_snake_case.json");
        let mon = parse_monitors(&raw);
        assert_eq!(mon.len(), 1);
        assert_eq!(mon[0].active_workspace.id, 3);
        assert_eq!(mon[0].active_workspace.name.as_deref(), Some("3"));
    }

    #[test]
    fn hyprland_fixture_paths_exist() {
        for name in [
            "workspaces.json",
            "clients.json",
            "clients_fullscreen.json",
            "activewindow.json",
            "activeworkspace.json",
            "monitors.json",
            "clients_skip_empty_address.json",
            "workspaces_not_array.json",
            "activewindow_null.json",
            "monitors_snake_case.json",
            "workspaces_address.json",
            "clients_address_workspace.json",
            "monitors_address_workspace.json",
            "activeworkspace_address.json",
        ] {
            let path = hyprland_fixture_path(name);
            assert!(path.exists(), "missing fixture {}", path.display());
        }
    }

    #[test]
    fn dispatch_allowlist_accepts_bar_commands() {
        assert!(validate_dispatch("workspace 3").is_ok());
        assert!(validate_dispatch("focuswindow address:0x1a2b3c").is_ok());
    }

    #[test]
    fn dispatch_allowlist_rejects_injection() {
        assert!(validate_dispatch("exec kitty").unwrap_err().to_string().contains("dispatch_denied"));
        assert!(validate_dispatch("workspace 1; exec rm").unwrap_err().to_string().contains("dispatch_denied"));
        assert!(validate_dispatch("keyword monitor ,add,auto").unwrap_err().to_string().contains("dispatch_denied"));
    }

    #[test]
    fn event_line_triggers_state_changed() {
        note_hyprland_event_line("workspace>>3");
        assert!(event_triggers_state_changed("workspace"));
        assert!(!event_triggers_state_changed("bell"));
    }
}
