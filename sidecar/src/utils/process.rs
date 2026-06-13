//! Subprocess helpers: timeouts, output caps, and an allowlist for sensitive services.
//!
//! When `AURA_EXEC_FIXTURE_DIR` is set, [`exec_command`] and [`run_allowlisted`] read
//! stdout/stderr/exit code from fixture files instead of spawning (integration tests).

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub const EXEC_FIXTURE_ENV: &str = "AURA_EXEC_FIXTURE_DIR";

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_MAX_OUTPUT: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct ExecOpts {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for ExecOpts {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            max_output_bytes: DEFAULT_MAX_OUTPUT,
        }
    }
}

struct AllowlistEntry {
    binary: &'static str,
    timeout: Option<Duration>,
    max_output_bytes: Option<usize>,
}

/// Binaries launcher/capture (and similar) may invoke via [`run_allowlisted`].
static ALLOWLIST: &[AllowlistEntry] = &[
    AllowlistEntry {
        binary: "gtk-launch",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "grim",
        timeout: Some(Duration::from_secs(15)),
        max_output_bytes: Some(16 * 1024 * 1024),
    },
    AllowlistEntry {
        binary: "slurp",
        timeout: Some(Duration::from_secs(120)),
        max_output_bytes: Some(4096),
    },
    AllowlistEntry {
        binary: "wl-copy",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "wf-recorder",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "pkill",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "wlsunset",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "gammastep",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "vicinae",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "pactl",
        timeout: None,
        max_output_bytes: None,
    },
    AllowlistEntry {
        binary: "which",
        timeout: Some(Duration::from_secs(5)),
        max_output_bytes: Some(4096),
    },
];

fn binary_basename(cmd: &str) -> &str {
    Path::new(cmd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(cmd)
}

pub fn opts_for_allowlisted(binary: &str) -> ExecOpts {
    let base = binary_basename(binary);
    ALLOWLIST
        .iter()
        .find(|e| e.binary == base)
        .map(|e| ExecOpts {
            timeout: e.timeout.unwrap_or(DEFAULT_TIMEOUT),
            max_output_bytes: e.max_output_bytes.unwrap_or(DEFAULT_MAX_OUTPUT),
        })
        .unwrap_or_default()
}

fn assert_allowlisted(cmd: &[&str]) -> Result<()> {
    if cmd.is_empty() {
        bail!("empty command");
    }
    let base = binary_basename(cmd[0]);
    if !ALLOWLIST.iter().any(|e| e.binary == base) {
        bail!("command not allowlisted: {base}");
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    #[serde(default)]
    mappings: Vec<FixtureMapping>,
}

#[derive(Debug, Deserialize)]
struct FixtureMapping {
    cmd: Vec<String>,
    stdout: String,
    #[serde(default)]
    stderr: Option<String>,
    #[serde(default = "default_exit_zero")]
    exit: i32,
}

fn default_exit_zero() -> i32 {
    0
}

fn exec_fixture_dir() -> Option<PathBuf> {
    std::env::var(EXEC_FIXTURE_ENV)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// True when subprocess calls should resolve from fixture files.
pub fn exec_fixtures_active() -> bool {
    exec_fixture_dir().is_some()
}

fn sanitize_fixture_key(part: &str) -> String {
    part.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn argv_fixture_key(cmd: &[&str]) -> String {
    cmd.iter()
        .map(|s| sanitize_fixture_key(s))
        .collect::<Vec<_>>()
        .join("_")
}

fn load_manifest(base: &Path) -> Option<FixtureManifest> {
    let path = base.join("manifest.json");
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn manifest_match(base: &Path, cmd: &[&str]) -> Option<(String, String, i32)> {
    let manifest = load_manifest(base)?;
    for entry in manifest.mappings {
        if entry.cmd.len() > cmd.len() {
            continue;
        }
        let matches = entry.cmd.iter().enumerate().all(|(i, part)| {
            cmd.get(i).map(|c| *c == part.as_str()).unwrap_or(false)
        });
        if !matches {
            continue;
        }
        let stdout_path = base.join(&entry.stdout);
        let stdout = std::fs::read_to_string(&stdout_path).unwrap_or_default();
        let stderr = entry
            .stderr
            .as_ref()
            .map(|rel| std::fs::read_to_string(base.join(rel)).unwrap_or_default())
            .unwrap_or_default();
        return Some((stdout, stderr, entry.exit));
    }
    None
}

fn file_fixture_candidates(base: &Path, cmd: &[&str]) -> Vec<PathBuf> {
    if cmd.is_empty() {
        return Vec::new();
    }
    let binary = sanitize_fixture_key(binary_basename(cmd[0]));
    let mut out = Vec::new();
    if cmd.len() > 1 {
        let key = argv_fixture_key(cmd);
        out.push(base.join(&binary).join(format!("{key}.stdout")));
        out.push(base.join(format!("{binary}_{key}.stdout")));
        let short = sanitize_fixture_key(cmd[1]);
        out.push(base.join(&binary).join(format!("{short}.stdout")));
    }
    out.push(base.join(format!("{binary}.stdout")));
    out
}

fn read_sidecar(path: &Path, suffix: &str) -> Option<String> {
    let side = path.with_extension(suffix.trim_start_matches('.'));
    std::fs::read_to_string(side).ok()
}

fn resolve_exec_fixture(cmd: &[&str]) -> Option<(String, String, i32)> {
    let base = exec_fixture_dir()?;
    if let Some(hit) = manifest_match(&base, cmd) {
        return Some(hit);
    }
    for stdout_path in file_fixture_candidates(&base, cmd) {
        if stdout_path.is_file() {
            let stdout = std::fs::read_to_string(&stdout_path).unwrap_or_default();
            let stderr = read_sidecar(&stdout_path, "stderr").unwrap_or_default();
            let exit = read_sidecar(&stdout_path, "exit")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            return Some((stdout, stderr, exit));
        }
    }
    None
}

/// Returns true when a `which`-style fixture exists for `name` under the active fixture dir.
pub fn exec_fixture_tool_available(name: &str) -> bool {
    let Some(base) = exec_fixture_dir() else {
        return false;
    };
    let binary = "which";
    let candidates = [
        base.join(binary).join(format!("{name}.stdout")),
        base.join(format!("{binary}_{name}.stdout")),
    ];
    candidates.iter().any(|p| p.is_file())
        || manifest_match(&base, &[binary, name]).is_some()
}

async fn run_from_fixture(cmd: &[&str], detached: bool) -> Result<Option<String>> {
    let Some((stdout, stderr, exit)) = resolve_exec_fixture(cmd) else {
        return Ok(None);
    };
    if detached {
        if exit == 0 {
            return Ok(Some(String::new()));
        }
        bail!(
            "fixture detached command failed: {}",
            cmd.first().copied().unwrap_or("command")
        );
    }
    if exit != 0 {
        let base = cmd.first().copied().unwrap_or("command");
        bail!("Command failed: {base} - {}", stderr.trim());
    }
    Ok(Some(stdout.trim().to_string()))
}

async fn run_internal(cmd: &[&str], opts: ExecOpts, detached: bool) -> Result<String> {
    if cmd.is_empty() {
        bail!("empty command");
    }

    if exec_fixture_dir().is_some() {
        match run_from_fixture(cmd, detached).await? {
            Some(out) => return Ok(out),
            None => {
                bail!(
                    "no exec fixture for `{}` (AURA_EXEC_FIXTURE_DIR set); \
                     refusing to run on host. Add mapping under tests/fixtures/exec/",
                    cmd.join(" ")
                );
            }
        }
    }

    if detached {
        Command::new(cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn detached")?;
        return Ok(String::new());
    }

    let run = async {
        let output = Command::new(cmd[0])
            .args(&cmd[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("failed to run {}", cmd[0]))?;

        if output.stdout.len() > opts.max_output_bytes {
            bail!(
                "stdout exceeded {} bytes for {}",
                opts.max_output_bytes,
                cmd[0]
            );
        }
        if output.stderr.len() > opts.max_output_bytes {
            bail!(
                "stderr exceeded {} bytes for {}",
                opts.max_output_bytes,
                cmd[0]
            );
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Command failed: {} - {}", cmd[0], stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    };

    timeout(opts.timeout, run)
        .await
        .map_err(|_| anyhow::anyhow!("command timed out after {:?}: {}", opts.timeout, cmd[0]))?
}

/// Run an allowlisted binary with per-binary timeout/output limits.
pub async fn run_allowlisted(cmd: &[&str]) -> Result<String> {
    assert_allowlisted(cmd)?;
    let opts = opts_for_allowlisted(cmd[0]);
    run_internal(cmd, opts, false).await
}

/// Spawn an allowlisted binary without waiting for completion.
pub async fn run_allowlisted_detached(cmd: &[&str]) -> Result<()> {
    assert_allowlisted(cmd)?;
    let opts = opts_for_allowlisted(cmd[0]);
    run_internal(cmd, opts, true).await.map(|_| ())
}

/// Run any command with explicit timeout/output cap (no allowlist check).
pub async fn exec_command_with_opts(cmd: &[&str], opts: ExecOpts) -> Result<String> {
    run_internal(cmd, opts, false).await
}

/// Run any command with default timeout and output cap (no allowlist check).
pub async fn exec_command(cmd: &[&str]) -> Result<String> {
    exec_command_with_opts(cmd, ExecOpts::default()).await
}

pub async fn exec_command_detached(cmd: &[&str]) -> Result<()> {
    run_internal(cmd, ExecOpts::default(), true).await.map(|_| ())
}

pub async fn exec_command_with_env(cmd: &[&str], env: &[(&str, &str)]) -> Result<String> {
    if cmd.is_empty() {
        bail!("empty command");
    }
    if exec_fixture_dir().is_some() {
        match run_from_fixture(cmd, false).await? {
            Some(out) => return Ok(out),
            None => {
                bail!(
                    "no exec fixture for `{}` (AURA_EXEC_FIXTURE_DIR set); \
                     refusing to run on host",
                    cmd.join(" ")
                );
            }
        }
    }
    let opts = ExecOpts::default();
    let run = async {
        let mut command = Command::new(cmd[0]);
        command.args(&cmd[1..]);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        for (key, value) in env {
            command.env(key, value);
        }
        let output = command.output().await?;

        if output.stdout.len() > opts.max_output_bytes || output.stderr.len() > opts.max_output_bytes
        {
            bail!("command output exceeded {} bytes", opts.max_output_bytes);
        }
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Command failed: {} - {}", cmd[0], stderr.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    };

    timeout(opts.timeout, run)
        .await
        .map_err(|_| anyhow::anyhow!("command timed out after {:?}: {}", opts.timeout, cmd[0]))?
}

/// Trait for injecting subprocess behavior in unit tests.
#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn run(&self, cmd: &[&str], opts: ExecOpts) -> Result<String>;
    async fn run_detached(&self, cmd: &[&str], opts: ExecOpts) -> Result<()>;
}

pub struct TokioCommandRunner;

#[async_trait]
impl CommandRunner for TokioCommandRunner {
    async fn run(&self, cmd: &[&str], opts: ExecOpts) -> Result<String> {
        run_internal(cmd, opts, false).await
    }

    async fn run_detached(&self, cmd: &[&str], opts: ExecOpts) -> Result<()> {
        run_internal(cmd, opts, true).await.map(|_| ())
    }
}

/// Test double for [`CommandRunner`] (integration tests and unit tests).
#[doc(hidden)]
pub struct MockCommandRunner {
    pub stdout: String,
    pub detached_ok: bool,
}

#[async_trait]
impl CommandRunner for MockCommandRunner {
    async fn run(&self, _cmd: &[&str], _opts: ExecOpts) -> Result<String> {
        Ok(self.stdout.clone())
    }

    async fn run_detached(&self, _cmd: &[&str], _opts: ExecOpts) -> Result<()> {
        if self.detached_ok {
            Ok(())
        } else {
            bail!("mock detached failure")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn exec_command_success_trims_stdout() {
        let out = exec_command(&["echo", "  hello  "]).await.unwrap();
        assert_eq!(out, "hello");
    }

    #[tokio::test]
    async fn exec_command_empty_stdout_on_success() {
        let out = exec_command(&["true"]).await.unwrap();
        assert_eq!(out, "");
    }

    #[tokio::test]
    async fn exec_command_failure_reports_command() {
        let err = exec_command(&["sh", "-c", "echo fail >&2; exit 1"])
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Command failed: sh"));
        assert!(msg.contains("fail"));
    }

    #[tokio::test]
    async fn exec_command_with_env_applies_vars() {
        let out = exec_command_with_env(
            &["sh", "-c", "printf %s \"$AURA_TEST_ENV\""],
            &[("AURA_TEST_ENV", "ok")],
        )
        .await
        .unwrap();
        assert_eq!(out, "ok");
    }

    #[tokio::test]
    async fn exec_command_detached_spawns_without_waiting() {
        exec_command_detached(&["true"]).await.unwrap();
    }

    #[tokio::test]
    async fn run_allowlisted_rejects_unknown_binary() {
        let err = run_allowlisted(&["curl", "https://example.com"])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not allowlisted"));
    }

    #[tokio::test]
    async fn run_allowlisted_gtk_launch_is_allowed() {
        // Desktop id may be missing; we only assert allowlist passes spawn attempt.
        let err = run_allowlisted(&["gtk-launch", "nonexistent-aura-test-id"])
            .await
            .unwrap_err();
        assert!(!err.to_string().contains("not allowlisted"));
    }

    #[tokio::test]
    async fn mock_runner_returns_fixture_stdout() {
        let mock = MockCommandRunner {
            stdout: "fixture".into(),
            detached_ok: true,
        };
        let out = mock
            .run(&["grim", "-"], ExecOpts::default())
            .await
            .unwrap();
        assert_eq!(out, "fixture");
    }

    #[tokio::test]
    async fn exec_fixture_manifest_wpctl_status() {
        let dir = std::env::temp_dir().join(format!("aura-exec-fixture-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            r#"{"mappings":[{"cmd":["wpctl","status"],"stdout":"wpctl_status.stdout","exit":0}]}"#,
        )
        .unwrap();
        std::fs::write(dir.join("wpctl_status.stdout"), "Audio\n Sinks:\n").unwrap();
        std::env::set_var(EXEC_FIXTURE_ENV, &dir);
        let out = exec_command(&["wpctl", "status"]).await.unwrap();
        assert!(out.contains("Audio"));
        std::env::remove_var(EXEC_FIXTURE_ENV);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn exec_fixture_file_layout_which() {
        let dir = std::env::temp_dir().join(format!("aura-exec-which-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("which")).unwrap();
        std::fs::write(dir.join("which/grim.stdout"), "/usr/bin/grim").unwrap();
        std::env::set_var(EXEC_FIXTURE_ENV, &dir);
        assert!(exec_fixture_tool_available("grim"));
        let out = exec_command(&["which", "grim"]).await.unwrap();
        assert_eq!(out, "/usr/bin/grim");
        std::env::remove_var(EXEC_FIXTURE_ENV);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn mock_runner_detached_respects_flag() {
        let ok = MockCommandRunner {
            stdout: String::new(),
            detached_ok: true,
        };
        ok.run_detached(&["wf-recorder"], ExecOpts::default())
            .await
            .unwrap();

        let bad = MockCommandRunner {
            stdout: String::new(),
            detached_ok: false,
        };
        assert!(bad
            .run_detached(&["wf-recorder"], ExecOpts::default())
            .await
            .is_err());
    }
}
