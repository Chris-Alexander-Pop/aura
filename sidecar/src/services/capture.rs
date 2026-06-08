//! Screenshot and screen recording via allowlisted host tools (grim, slurp, wl-copy, wf-recorder).

/// Subprocess injection for capture tests (see [`crate::utils::process::CommandRunner`]).
pub use crate::utils::process::CommandRunner as CaptureProcess;

use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::{bail, Result};
use serde_json::json;
use std::path::{Component, Path, PathBuf};

const RECORDER_PID_FILE: &str = "recorder.pid";

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Capture.Screenshot", |params| async move {
        let mode = param_str(params.as_ref(), "mode")?;
        let output = param_str(params.as_ref(), "output")?;
        let path = params
            .as_ref()
            .and_then(|p| p.get("path"))
            .and_then(|v| v.as_str())
            .map(str::to_owned);

        run_screenshot(&mode, &output, path.as_deref()).await
    });

    registry.register("Capture.RecordStart", |_params| async move {
        record_start().await
    });

    registry.register("Capture.RecordStop", |_params| async move {
        record_stop().await
    });

    registry.register("Capture.ListDevices", |_params| async move {
        list_devices().await
    });
}

fn param_str(params: Option<&serde_json::Value>, key: &str) -> Result<String> {
    params
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("missing {key}"))
}

pub fn tool_on_path(name: &str) -> bool {
    // Sync probe for early UI hints; allowlisted `which` is also used async in RPC paths.
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn tool_on_path_async(name: &str) -> bool {
    process::run_allowlisted(&["which", name])
        .await
        .is_ok()
}

pub fn validate_screenshot_path(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    if p.as_os_str().is_empty() {
        bail!("empty path");
    }
    for comp in p.components() {
        match comp {
            Component::ParentDir => bail!("path must not contain .."),
            Component::RootDir | Component::CurDir | Component::Normal(_) | Component::Prefix(_) => {}
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let base = PathBuf::from(home);
    let full = if p.is_absolute() {
        p.to_path_buf()
    } else {
        base.join(p)
    };
    if !full.starts_with(&base) && p.is_absolute() {
        // absolute paths outside home are rejected
        bail!("path must be under home directory");
    }
    Ok(full)
}

pub fn default_screenshot_file_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let dir = PathBuf::from(home).join("Pictures");
    let name = format!(
        "aura-{}.png",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    Ok(dir.join(name))
}

/// Build argv for region selection (slurp only).
pub fn slurp_argv() -> Vec<String> {
    vec!["slurp".into()]
}

/// grim argv for region geometry from slurp stdout.
pub fn grim_region_argv(geometry: &str) -> Vec<String> {
    vec!["grim".into(), "-g".into(), geometry.into()]
}

pub fn grim_full_argv() -> Vec<String> {
    vec!["grim".into()]
}

pub fn wl_copy_argv() -> Vec<String> {
    vec!["wl-copy".into(), "--type".into(), "image/png".into()]
}

fn recorder_state_dir() -> Result<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .map_err(|_| anyhow::anyhow!("HOME not set"))?;
    Ok(base.join("ags-sidecar"))
}

pub(crate) fn recorder_state_dir_for_tests() -> Result<PathBuf> {
    recorder_state_dir()
}

fn pid_file_path() -> Result<PathBuf> {
    Ok(recorder_state_dir()?.join(RECORDER_PID_FILE))
}

pub(crate) fn pid_file_path_for_tests() -> Result<PathBuf> {
    pid_file_path()
}

async fn run_screenshot(mode: &str, output: &str, path: Option<&str>) -> Result<serde_json::Value> {
    if !tool_on_path("grim") {
        return Ok(json!({ "ok": false, "tool_missing": true, "tool": "grim" }));
    }

    match mode {
        "full" => {}
        "region" => {
            if !tool_on_path("slurp") {
                return Ok(json!({ "ok": false, "tool_missing": true, "tool": "slurp" }));
            }
        }
        "window" => {
            return Ok(json!({
                "ok": false,
                "error": "window mode not implemented in v1; use region or full"
            }));
        }
        _ => bail!("invalid mode: {mode}"),
    }

    match output {
        "clipboard" => {
            if !tool_on_path("wl-copy") {
                return Ok(json!({ "ok": false, "tool_missing": true, "tool": "wl-copy" }));
            }
            capture_to_clipboard(mode).await?;
            Ok(json!({ "ok": true, "output": "clipboard" }))
        }
        "file" => {
            let dest = match path {
                Some(p) => validate_screenshot_path(p)?,
                None => default_screenshot_file_path()?,
            };
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            capture_to_file(mode, &dest).await?;
            Ok(json!({
                "ok": true,
                "output": "file",
                "path": dest.to_string_lossy(),
            }))
        }
        _ => bail!("invalid output: {output}"),
    }
}

async fn capture_to_clipboard(mode: &str) -> Result<()> {
    match mode {
        "full" => {
            let grim = process::run_allowlisted(&["grim", "-"]).await?;
            pipe_to_wl_copy(grim.as_bytes()).await
        }
        "region" => {
            let geometry = process::run_allowlisted(&["slurp"]).await?;
            let image = process::run_allowlisted(&["grim", "-g", geometry.trim(), "-"]).await?;
            pipe_to_wl_copy(image.as_bytes()).await
        }
        _ => bail!("invalid mode"),
    }
}

async fn pipe_to_wl_copy(bytes: &[u8]) -> Result<()> {
    process::run_allowlisted(&["which", "wl-copy"]).await?;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;
    let mut child = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(bytes).await?;
    }
    let status = child.wait().await?;
    if !status.success() {
        bail!("wl-copy failed");
    }
    Ok(())
}

async fn capture_to_file(mode: &str, dest: &Path) -> Result<()> {
    match mode {
        "full" => {
            process::run_allowlisted(&["grim", dest.to_str().expect("utf8 path")]).await?;
        }
        "region" => {
            let geometry = process::run_allowlisted(&["slurp"]).await?;
            process::run_allowlisted(&[
                "grim",
                "-g",
                geometry.trim(),
                dest.to_str().expect("utf8 path"),
            ])
            .await?;
        }
        _ => bail!("invalid mode"),
    }
    Ok(())
}

async fn record_start() -> Result<serde_json::Value> {
    if !tool_on_path("wf-recorder") {
        return Ok(json!({ "ok": false, "tool_missing": true, "tool": "wf-recorder" }));
    }
    let dir = recorder_state_dir()?;
    std::fs::create_dir_all(&dir)?;
    let pid_path = pid_file_path()?;
    if pid_path.exists() {
        bail!("recording already in progress");
    }
    let out = dir.join(format!(
        "recording-{}.mp4",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    process::run_allowlisted_detached(&[
        "wf-recorder",
        "-f",
        out.to_str().expect("utf8"),
    ])
    .await?;
    // wf-recorder forks; write a marker file (best-effort pid discovery not required for stop)
    std::fs::write(&pid_path, format!("{}", std::process::id()))?;
    Ok(json!({ "ok": true, "path": out.to_string_lossy() }))
}

async fn record_stop() -> Result<serde_json::Value> {
    let pid_path = pid_file_path()?;
    if !pid_path.exists() {
        return Ok(json!({ "ok": false, "error": "not recording" }));
    }
    // Stop all wf-recorder processes for this user (allowlisted binary only).
    let _ = process::run_allowlisted(&["pkill", "-x", "wf-recorder"]).await;
    let _ = std::fs::remove_file(&pid_path);
    Ok(json!({ "ok": true }))
}

async fn list_devices() -> Result<serde_json::Value> {
    let audio = if tool_on_path_async("pactl").await {
        process::run_allowlisted(&["pactl", "list", "sources", "short"])
            .await
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let name = line.split_whitespace().nth(1)?;
                Some(json!({ "name": name, "kind": "audio" }))
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    Ok(json!({
        "audio": audio,
        "video": [],
        "tool_missing": {
            "pactl": !tool_on_path("pactl"),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_rejects_parent_dir() {
        assert!(validate_screenshot_path("../etc/passwd").is_err());
    }

    #[test]
    fn validate_path_rejects_empty() {
        assert!(validate_screenshot_path("").is_err());
    }

    #[test]
    fn validate_path_rejects_absolute_outside_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let outside = if home == "/" {
            "/etc/passwd".to_string()
        } else {
            format!("/etc/aura-capture-outside-home.png")
        };
        assert!(validate_screenshot_path(&outside).is_err());
    }

    #[test]
    fn validate_path_accepts_relative_under_home() {
        let p = validate_screenshot_path("Pictures/test.png").unwrap();
        assert!(p.to_string_lossy().contains("Pictures"));
    }

    #[test]
    fn validate_path_accepts_absolute_under_home() {
        let home = std::env::var("HOME").expect("HOME set");
        let path = format!("{home}/Pictures/aura-abs.png");
        let resolved = validate_screenshot_path(&path).unwrap();
        assert!(resolved.to_string_lossy().contains("Pictures/aura-abs.png"));
    }

    #[test]
    fn grim_region_argv_includes_geometry() {
        let argv = grim_region_argv("0,0 100x100");
        assert_eq!(argv[0], "grim");
        assert_eq!(argv[2], "0,0 100x100");
    }

    #[test]
    fn grim_full_and_slurp_argv_tables() {
        assert_eq!(grim_full_argv(), vec!["grim".to_string()]);
        assert_eq!(slurp_argv(), vec!["slurp".to_string()]);
    }

    #[test]
    fn wl_copy_argv_has_png_type() {
        let argv = wl_copy_argv();
        assert!(argv.contains(&"image/png".to_string()));
    }

    #[test]
    fn recorder_pid_file_under_xdg_data_home() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let data = tmp.path().join("data");
        std::env::set_var("XDG_DATA_HOME", data.to_str().expect("utf8"));
        let pid_path = pid_file_path_for_tests().expect("pid path");
        assert!(pid_path.starts_with(&data));
        assert!(pid_path.ends_with(RECORDER_PID_FILE));
        let state_dir = recorder_state_dir_for_tests().expect("state dir");
        assert!(state_dir.starts_with(&data));
        assert!(state_dir.ends_with("ags-sidecar"));
        std::env::remove_var("XDG_DATA_HOME");
    }
}
