//! Hyprland keybind introspection and safe writes to Aura-managed bind file.
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const MAX_SOURCE_DEPTH: usize = 8;
const BIND_PREFIXES: &[&str] = &["bind", "bindl", "bindr", "binde", "bindm"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindEntry {
    pub combo: String,
    pub action: String,
    pub bind_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<String>,
    pub file: String,
    pub line: u32,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindCategory {
    pub id: String,
    pub title: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindValidation {
    pub duplicates: Vec<String>,
    pub unknown_dispatches: Vec<String>,
}

pub fn aura_binds_path() -> PathBuf {
    if let Ok(p) = std::env::var("AURA_KEYBINDS_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ags")
            .join("hypr")
            .join("hyprland")
            .join("aura-overrides.lua");
    }
    PathBuf::from("/tmp/aura-overrides.lua")
}

fn aura_keybinds_lua_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ags")
            .join("hypr")
            .join("hyprland")
            .join("keybinds.lua");
    }
    PathBuf::from("/tmp/keybinds.lua")
}

fn aura_hyprland_entry() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ags")
            .join("hypr")
            .join("hyprland.lua");
    }
    PathBuf::from("/tmp/hyprland.lua")
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Keybinds.List", |params| async move {
        let category: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("category").cloned())
            .and_then(|v| serde_json::from_value(v).ok());
        let mut entries = load_all_keybinds().await?;
        if let Some(cat) = category {
            entries.retain(|e| e.category == cat);
        }
        Ok(serde_json::to_value(&entries)?)
    });

    registry.register("Keybinds.GetCategories", |_params| async move {
        let entries = load_all_keybinds().await?;
        let mut counts: HashMap<String, usize> = HashMap::new();
        for e in &entries {
            *counts.entry(e.category.clone()).or_default() += 1;
        }
        let categories: Vec<KeybindCategory> = counts
            .into_iter()
            .map(|(id, count)| KeybindCategory {
                title: category_title(&id),
                id: id.clone(),
                count,
            })
            .collect();
        Ok(serde_json::to_value(categories)?)
    });

    registry.register("Keybinds.Validate", |_params| async move {
        let entries = load_all_keybinds().await?;
        Ok(serde_json::to_value(validate_entries(&entries))?)
    });

    registry.register("Keybinds.Export", |_params| async move {
        let entries = load_all_keybinds().await?;
        Ok(serde_json::to_value(&entries)?)
    });

    registry.register("Keybinds.Import", |params| async move {
        let entries: Vec<KeybindEntry> = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("entries").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing entries"))?,
        )?;
        write_aura_binds(&entries).await?;
        Ok(json!({ "success": true, "count": entries.len() }))
    });

    registry.register("Keybinds.Set", |params| async move {
        let combo: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("combo").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing combo"))?,
        )?;
        let action: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("action").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing action"))?,
        )?;
        let bind_type: String = params
            .as_ref()
            .and_then(|p| p.get("bind_type").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_else(|| "bind".to_string());

        let mut entries = load_aura_binds_only().await?;
        entries.retain(|e| e.combo != combo);
        let path = aura_binds_path();
        let category = categorize_action(&action);
        entries.push(KeybindEntry {
            combo: combo.clone(),
            action,
            bind_type,
            flags: None,
            file: path.display().to_string(),
            line: 0,
            category,
        });
        write_aura_binds(&entries).await?;
        Ok(json!({ "success": true }))
    });

    registry.register("Keybinds.Unset", |params| async move {
        let combo: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("combo").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing combo"))?,
        )?;
        let mut entries = load_aura_binds_only().await?;
        let before = entries.len();
        entries.retain(|e| e.combo != combo);
        write_aura_binds(&entries).await?;
        Ok(json!({
            "success": entries.len() < before,
        }))
    });

    registry.register("Keybinds.Reload", |_params| async move {
        process::exec_command(&["hyprctl", "reload"]).await?;
        Ok(json!({ "success": true }))
    });
}

fn category_title(id: &str) -> String {
    match id {
        "workspace" => "Workspaces",
        "window" => "Windows",
        "media" => "Media",
        "launcher" => "Launcher",
        "system" => "System",
        _ => "Other",
    }
    .to_string()
}

pub fn categorize_action(action: &str) -> String {
    let a = action.to_lowercase();
    if a.contains("workspace") || a.contains("movetoworkspace") {
        "workspace".into()
    } else if a.contains("movefocus") || a.contains("movewindow") || a.contains("togglefloating")
        || a.contains("fullscreen") || a.contains("killactive") || a.contains("closewindow")
    {
        "window".into()
    } else if a.contains("volume") || a.contains("mute") || a.contains("player") {
        "media".into()
    } else if a.contains("exec") || a.contains("launcher") {
        "launcher".into()
    } else if a.contains("exit") || a.contains("shutdown") || a.contains("lock") {
        "system".into()
    } else {
        "other".into()
    }
}

fn default_hypr_config() -> PathBuf {
    if let Ok(p) = std::env::var("HYPRLAND_CONFIG") {
        return PathBuf::from(p);
    }
    if let Ok(home) = std::env::var("HOME") {
        let lua_stub = PathBuf::from(&home)
            .join(".config")
            .join("hypr")
            .join("hyprland.lua");
        if lua_stub.exists() {
            return lua_stub;
        }
        let aura = aura_hyprland_entry();
        if aura.exists() {
            return aura;
        }
        return PathBuf::from(home)
            .join(".config")
            .join("hypr")
            .join("hyprland.conf");
    }
    PathBuf::from("/etc/hypr/hyprland.conf")
}

async fn load_all_keybinds() -> Result<Vec<KeybindEntry>> {
    let root = default_hypr_config();
    let mut files = tokio::task::spawn_blocking(move || {
        let mut out = Vec::new();
        collect_config_files(&root, 0, &mut out)?;
        Ok::<_, anyhow::Error>(out)
    })
    .await??;

    // Always include Aura Lua modules (XDG stub does not `source=` them).
    for extra in [aura_keybinds_lua_path(), aura_binds_path()] {
        if extra.exists() && !files.iter().any(|f| f == &extra) {
            files.push(extra);
        }
    }

    let mut all = Vec::new();
    for path in files {
        let text = tokio::fs::read_to_string(&path).await.unwrap_or_default();
        let display = path.display().to_string();
        if path.extension().and_then(|e| e.to_str()) == Some("lua") {
            all.extend(parse_lua_keybinds(&text, &display));
        } else {
            all.extend(parse_keybinds(&text, &display));
        }
    }
    Ok(all)
}

async fn load_aura_binds_only() -> Result<Vec<KeybindEntry>> {
    let path = aura_binds_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = tokio::fs::read_to_string(&path).await?;
    let display = path.display().to_string();
    if path.extension().and_then(|e| e.to_str()) == Some("lua") {
        Ok(parse_lua_keybinds(&text, &display))
    } else {
        Ok(parse_keybinds(&text, &display))
    }
}

fn collect_config_files(path: &Path, depth: usize, out: &mut Vec<PathBuf>) -> Result<()> {
    if depth > MAX_SOURCE_DEPTH {
        return Ok(());
    }
    if !path.exists() {
        return Ok(());
    }
    out.push(path.to_path_buf());
    let text = std::fs::read_to_string(path)?;
    for line in text.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if let Some(rest) = trimmed.strip_prefix("source") {
            let token = rest
                .trim()
                .trim_start_matches('=')
                .trim()
                .trim_matches(|c| c == '"' || c == '\'');
            if token.is_empty() {
                continue;
            }
            let mut inc = PathBuf::from(token);
            if inc.is_relative() {
                if let Some(parent) = path.parent() {
                    inc = parent.join(inc);
                }
            }
            collect_config_files(&inc, depth + 1, out)?;
        }
    }
    Ok(())
}

pub fn parse_keybinds(text: &str, file: &str) -> Vec<KeybindEntry> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let raw = line.split('#').next().unwrap_or("").trim();
        if raw.is_empty() {
            continue;
        }
        let Some((bind_type, rest)) = parse_bind_line(raw) else {
            continue;
        };
        let mut parts = rest.splitn(3, ',');
        let mod_key = parts.next().map(str::trim).unwrap_or("");
        let key = parts.next().map(str::trim).unwrap_or("");
        let action = parts
            .next()
            .map(str::trim)
            .unwrap_or("")
            .trim_matches(|c| c == ';' || c == ' ')
            .to_string();
        if mod_key.is_empty() || key.is_empty() || action.is_empty() {
            continue;
        }
        let combo = format!("{}, {}", mod_key, key);
        let category = categorize_action(&action);
        out.push(KeybindEntry {
            combo,
            action,
            bind_type: bind_type.to_string(),
            flags: None,
            file: file.to_string(),
            line: (i + 1) as u32,
            category,
        });
    }
    out
}

/// Best-effort parse of `hl.bind("SUPER + Q", …)` lines for the Lua config path.
pub fn parse_lua_keybinds(text: &str, file: &str) -> Vec<KeybindEntry> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let raw = line.split("--").next().unwrap_or("").trim();
        if raw.is_empty() {
            continue;
        }
        let Some(rest) = raw.strip_prefix("hl.bind(") else {
            continue;
        };
        let rest = rest.trim_start();
        let quote = rest.chars().next().unwrap_or('\0');
        if quote != '"' && quote != '\'' {
            continue;
        }
        let body = &rest[1..];
        let Some(end) = body.find(quote) else {
            continue;
        };
        let lua_combo = &body[..end];
        if lua_combo.is_empty() {
            continue;
        }
        let after = body[end + 1..].trim_start().trim_start_matches(',').trim();
        let action = summarize_lua_action(after);
        let combo = lua_combo_to_hyprlang(lua_combo);
        let category = categorize_action(&action);
        out.push(KeybindEntry {
            combo,
            action,
            bind_type: "bind".to_string(),
            flags: None,
            file: file.to_string(),
            line: (i + 1) as u32,
            category,
        });
    }
    out
}

fn lua_combo_to_hyprlang(combo: &str) -> String {
    let parts: Vec<&str> = combo.split('+').map(str::trim).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return combo.to_string();
    }
    if parts.len() == 1 {
        return format!(", {}", parts[0]);
    }
    let key = parts[parts.len() - 1];
    let mods = parts[..parts.len() - 1].join(" ");
    format!("{mods}, {key}")
}

fn hyprlang_combo_to_lua(combo: &str) -> String {
    let parts: Vec<&str> = combo.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return combo.to_string();
    }
    if parts.len() == 1 {
        return parts[0].to_string();
    }
    let key = parts[parts.len() - 1];
    let mods = parts[..parts.len() - 1]
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" + ");
    format!("{mods} + {key}")
}

fn summarize_lua_action(after: &str) -> String {
    let a = after.trim().trim_end_matches(',').trim().trim_end_matches(')').trim();
    if let Some(rest) = a.strip_prefix("exec(") {
        let inner = rest.trim().trim_matches(|c| c == '"' || c == '\'');
        return format!("exec, {inner}");
    }
    if let Some(rest) = a.strip_prefix("hl.dsp.exec_cmd(") {
        let inner = rest.trim().trim_matches(|c| c == '"' || c == '\'');
        return format!("exec, {inner}");
    }
    if a.contains("workspace") {
        return "workspace".to_string();
    }
    if a.contains("window.move") || a.contains("window.close") {
        return a.to_string();
    }
    if a.starts_with("function") {
        return "lua-fn".to_string();
    }
    if a.is_empty() {
        return "lua".to_string();
    }
    a.chars().take(120).collect()
}

fn action_to_lua_dispatcher(action: &str) -> String {
    let a = action.trim();
    if let Some(rest) = a.strip_prefix("exec") {
        let cmd = rest.trim().trim_start_matches(',').trim();
        let escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        return format!("hl.dsp.exec_cmd(\"{escaped}\")");
    }
    // Best-effort: unsupported classic dispatchers become a user-visible no-op.
    let escaped = a.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "hl.dsp.exec_cmd(\"notify-send -u low Aura 'Bind override needs Lua: {escaped}'\")"
    )
}

fn parse_bind_line(raw: &str) -> Option<(&str, &str)> {
    for prefix in BIND_PREFIXES {
        if let Some(rest) = raw.strip_prefix(prefix) {
            let rest = rest.trim();
            if rest.starts_with(',') || rest.starts_with(' ') {
                return Some((prefix, rest.trim_start_matches(',').trim()));
            }
            if rest.starts_with('=') {
                return Some((prefix, rest.trim_start_matches('=').trim()));
            }
        }
    }
    None
}

pub fn validate_entries(entries: &[KeybindEntry]) -> KeybindValidation {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut duplicates = Vec::new();
    let mut unknown_dispatches = Vec::new();
    let allowed = dispatch_allowlist();

    for e in entries {
        *seen.entry(e.combo.clone()).or_default() += 1;
        if let Some(cmd) = extract_dispatch(&e.action) {
            let head = cmd.split_whitespace().next().unwrap_or("").to_string();
            if !allowed.contains(&head) && !head.is_empty() {
                unknown_dispatches.push(format!("{} → {}", e.combo, cmd));
            }
        }
    }
    for (combo, count) in seen {
        if count > 1 {
            duplicates.push(format!("{combo} ({count} times)"));
        }
    }
    duplicates.sort();
    unknown_dispatches.sort();
    unknown_dispatches.dedup();
    KeybindValidation {
        duplicates,
        unknown_dispatches,
    }
}

fn extract_dispatch(action: &str) -> Option<String> {
    let a = action.trim();
    if let Some(rest) = a.strip_prefix("dispatch") {
        return Some(rest.trim().to_string());
    }
    None
}

fn dispatch_allowlist() -> HashSet<String> {
    [
        "workspace", "movetoworkspace", "movefocus", "movewindow", "togglefloating", "fullscreen",
        "killactive", "exit", "exec", "pseudo", "resizeactive", "moveactive", "cyclenext",
        "cycleprev", "swapwindow", "focuswindow", "pin", "sendshortcut", "pass", "submap",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

async fn write_aura_binds(entries: &[KeybindEntry]) -> Result<()> {
    let path = aura_binds_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if path.exists() {
        let backup = path.with_extension(format!(
            "bak.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ));
        tokio::fs::copy(&path, &backup).await.ok();
    }

    let is_lua = path.extension().and_then(|e| e.to_str()) == Some("lua");
    let mut lines = if is_lua {
        vec![
            "-- Aura-managed Hyprland binds — written by Keybinds.* RPC".to_string(),
            "-- Loaded last from hyprland.lua via pcall(require, \"hyprland/aura-overrides\").".to_string(),
            String::new(),
        ]
    } else {
        vec![
            "# Aura-managed Hyprland binds — safe to edit via Keybinds.* RPC".to_string(),
            "# Legacy hyprlang path (prefer aura-overrides.lua on Lua-config Hyprland).".to_string(),
            String::new(),
        ]
    };
    for e in entries {
        if is_lua {
            let combo = hyprlang_combo_to_lua(&e.combo);
            let disp = action_to_lua_dispatcher(&e.action);
            lines.push(format!("hl.bind(\"{combo}\", {disp})"));
        } else {
            lines.push(format!("{} = {}, {}", e.bind_type, e.combo, e.action));
        }
    }
    tokio::fs::write(&path, lines.join("\n") + "\n")
        .await
        .context("write aura keybind overrides")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/keybinds")
            .join(name)
    }

    #[test]
    fn parse_sample_binds() {
        let text = r#"
bind = SUPER, Q, killactive
binde = SUPER, left, resizeactive, -20 0
"#;
        let entries = parse_keybinds(text, "test.conf");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].combo, "SUPER, Q");
        assert_eq!(entries[0].bind_type, "bind");
    }

    #[test]
    fn parse_fixture_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/keybinds/sample.conf");
        let text = std::fs::read_to_string(path).expect("fixture");
        let entries = parse_keybinds(&text, "sample.conf");
        assert!(entries.len() >= 3);
        let v = validate_entries(&entries);
        assert!(!v.duplicates.is_empty());
    }

    #[test]
    fn validate_duplicate_combo() {
        let entries = vec![
            KeybindEntry {
                combo: "SUPER, A".into(),
                action: "dispatch workspace 1".into(),
                bind_type: "bind".into(),
                flags: None,
                file: "t".into(),
                line: 1,
                category: "workspace".into(),
            },
            KeybindEntry {
                combo: "SUPER, A".into(),
                action: "dispatch workspace 2".into(),
                bind_type: "bind".into(),
                flags: None,
                file: "t".into(),
                line: 2,
                category: "workspace".into(),
            },
        ];
        let v = validate_entries(&entries);
        assert!(!v.duplicates.is_empty());
    }

    #[test]
    fn parse_bind_line_prefixes() {
        assert_eq!(
            parse_bind_line("bind = SUPER, Q, killactive"),
            Some(("bind", "SUPER, Q, killactive"))
        );
        assert_eq!(
            parse_bind_line("binde=SUPER, left, resizeactive, -20 0"),
            Some(("binde", "SUPER, left, resizeactive, -20 0"))
        );
        assert!(parse_bind_line("exec = foot").is_none());
    }

    #[test]
    fn parse_edge_cases_fixture() {
        let text = std::fs::read_to_string(fixture("edge_cases.conf")).expect("fixture");
        let entries = parse_keybinds(&text, "edge_cases.conf");
        assert!(entries.iter().any(|e| e.bind_type == "bindl"));
        assert!(entries.iter().any(|e| e.bind_type == "bindm"));
        assert!(entries.iter().any(|e| e.category == "launcher"));
        assert!(!entries.iter().any(|e| e.combo.starts_with(", ")));
    }

    #[test]
    fn categorize_action_buckets() {
        assert_eq!(categorize_action("dispatch movetoworkspace 2"), "workspace");
        assert_eq!(categorize_action("dispatch togglefloating"), "window");
        assert_eq!(categorize_action("exec playerctl next"), "media");
        assert_eq!(categorize_action("exec wofi"), "launcher");
        assert_eq!(categorize_action("dispatch exit"), "system");
        assert_eq!(categorize_action("dispatch pseudo"), "other");
    }

    #[test]
    fn validate_unknown_dispatch() {
        let entries = vec![KeybindEntry {
            combo: "SUPER, Z".into(),
            action: "dispatch notarealcommand".into(),
            bind_type: "bind".into(),
            flags: None,
            file: "t".into(),
            line: 1,
            category: "other".into(),
        }];
        let v = validate_entries(&entries);
        assert_eq!(v.duplicates.len(), 0);
        assert!(!v.unknown_dispatches.is_empty());
    }

    #[test]
    fn collect_config_sources_fixture() {
        let root = fixture("sources/root.conf");
        let mut files = Vec::new();
        collect_config_files(&root, 0, &mut files).expect("collect");
        assert!(files.len() >= 2);
        let merged: Vec<_> = files
            .iter()
            .flat_map(|p| parse_keybinds(&std::fs::read_to_string(p).unwrap(), &p.display().to_string()))
            .collect();
        assert!(merged.iter().any(|e| e.combo == "SUPER, X"));
        assert!(merged.iter().any(|e| e.combo == "SUPER, Y"));
    }

    #[tokio::test]
    async fn write_aura_binds_uses_temp_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("aura-keybinds.conf");
        std::env::set_var("AURA_KEYBINDS_PATH", path.to_string_lossy().as_ref());

        let entries = vec![KeybindEntry {
            combo: "SUPER, T".into(),
            action: "killactive".into(),
            bind_type: "bind".into(),
            flags: None,
            file: path.display().to_string(),
            line: 0,
            category: "window".into(),
        }];
        write_aura_binds(&entries).await.expect("write");
        let text = tokio::fs::read_to_string(&path).await.expect("read");
        assert!(text.contains("Aura-managed"));
        assert!(text.contains("bind = SUPER, T, killactive"));

        std::env::remove_var("AURA_KEYBINDS_PATH");
    }
}
