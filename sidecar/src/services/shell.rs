//! Allowlisted session actions, Aura window toggles, app launch — invoked from React/WebKit only via JSON-RPC.
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::{bail, Result};
use serde_json::json;
use std::collections::HashMap;

pub(crate) fn aura_window_allowed(name: &str) -> bool {
    matches!(
        name,
        "control-center" | "sidebar" | "calendar" | "dropdown" | "launcher"
    )
}

lazy_static::lazy_static! {
    static ref APP_MAP: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("terminal", vec!["kitty"]);
        m.insert("browser", vec!["firefox"]);
        m.insert("files", vec!["nautilus"]);
        m.insert("code", vec!["cursor"]);
        m.insert("vscode", vec!["code"]);
        m.insert("music", vec!["spotify"]);
        m.insert("discord", vec!["discord"]);
        m.insert("firefox", vec!["firefox"]);
        m
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Logout,
    Suspend,
    Reboot,
    PowerOff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    Exec { argv: Vec<String> },
    Login1 { method: &'static str, interactive: bool },
}

pub fn classify_session_stderr(stderr: &str) -> &'static str {
    let s = stderr.to_ascii_lowercase();
    if s.contains("not authorized")
        || s.contains("access denied")
        || s.contains("authentication")
        || s.contains("interactive authentication")
        || s.contains("polkit")
        || s.contains("permission denied")
        || s.contains("not permitted")
    {
        "auth_required"
    } else if s.contains("not supported") || s.contains("operation not supported") {
        "not_supported"
    } else {
        "failed"
    }
}

pub fn format_session_error(code: &str, detail: &str) -> String {
    let detail = detail
        .lines()
        .next()
        .unwrap_or(detail)
        .trim();
    let detail = if detail.len() > 240 {
        &detail[..240]
    } else {
        detail
    };
    format!("session_action_failed: {code}: {detail}")
}

fn map_session_error(err: anyhow::Error) -> anyhow::Error {
    let msg = err.to_string();
    let code = classify_session_stderr(&msg);
    anyhow::anyhow!(format_session_error(code, &msg))
}

pub fn resolve_logout_argv(session_id: Option<&str>, user: Option<&str>) -> Vec<String> {
    if let Some(sid) = session_id.filter(|s| !s.is_empty()) {
        return vec![
            "loginctl".into(),
            "terminate-session".into(),
            sid.into(),
        ];
    }
    let user = user.filter(|u| !u.is_empty()).unwrap_or("_");
    vec![
        "loginctl".into(),
        "terminate-user".into(),
        user.into(),
    ]
}

pub fn resolve_session_command(
    action: SessionAction,
    session_id: Option<&str>,
    user: Option<&str>,
) -> SessionCommand {
    match action {
        SessionAction::Logout => SessionCommand::Exec {
            argv: resolve_logout_argv(session_id, user),
        },
        SessionAction::Suspend => SessionCommand::Login1 {
            method: "Suspend",
            interactive: false,
        },
        SessionAction::Reboot => SessionCommand::Login1 {
            method: "Reboot",
            interactive: false,
        },
        SessionAction::PowerOff => SessionCommand::Login1 {
            method: "PowerOff",
            interactive: false,
        },
    }
}

async fn invoke_login1(method: &str, interactive: bool) -> Result<()> {
    use zbus::{Connection, Proxy};

    let conn = Connection::system().await?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let _: () = proxy.call(method, &(interactive,)).await?;
    Ok(())
}

async fn run_session_action(action: SessionAction) -> Result<serde_json::Value> {
    let session_id = std::env::var("XDG_SESSION_ID").ok();
    let user = std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("LOGNAME").ok());
    let cmd = resolve_session_command(
        action,
        session_id.as_deref(),
        user.as_deref(),
    );
    match cmd {
        SessionCommand::Exec { argv } => {
            let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
            process::exec_command(&refs).await.map_err(map_session_error)?;
        }
        SessionCommand::Login1 { method, interactive } => {
            invoke_login1(method, interactive)
                .await
                .map_err(map_session_error)?;
        }
    }
    Ok(json!({"ok": true}))
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Sidecar.GetVersion", |_p| async move {
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
        }))
    });

    registry.register("Session.Lock", |_p| async move {
        let argv = resolve_lock_command();
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        process::exec_command_detached(&refs).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Session.Logout", |_p| async move {
        run_session_action(SessionAction::Logout).await
    });

    registry.register("Session.Suspend", |_p| async move {
        run_session_action(SessionAction::Suspend).await
    });

    registry.register("Session.Reboot", |_p| async move {
        run_session_action(SessionAction::Reboot).await
    });

    registry.register("Session.PowerOff", |_p| async move {
        run_session_action(SessionAction::PowerOff).await
    });

    registry.register("Aura.ToggleWindow", |params| async move {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("missing name"))?;
        if !aura_window_allowed(&name) {
            bail!("window name not allowed: {name}");
        }
        process::exec_command_detached(&["ags", "request", "toggle", name.as_str()]).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Apps.Launch", |params| async move {
        let id = params
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|n| n.as_str())
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("missing id"))?;

        let argv = APP_MAP
            .get(id.as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown app id: {id}"))?;

        let refs: Vec<&str> = argv.iter().copied().collect();
        process::exec_command_detached(&refs).await?;
        Ok(json!({"ok": true}))
    });
}

pub(crate) fn app_launch_argv(app_id: &str) -> Option<Vec<&'static str>> {
    APP_MAP.get(app_id).cloned()
}

/// Prefer `hyprlock` when installed; fall back to `loginctl lock-session`.
pub fn resolve_lock_command() -> Vec<String> {
    if std::process::Command::new("which")
        .arg("hyprlock")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        vec!["hyprlock".into()]
    } else {
        vec!["loginctl".into(), "lock-session".into()]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        app_launch_argv, aura_window_allowed, classify_session_stderr, format_session_error,
        resolve_lock_command, resolve_logout_argv, resolve_session_command, SessionAction,
        SessionCommand,
    };

    #[test]
    fn aura_window_allowlist() {
        for allowed in ["control-center", "sidebar", "calendar", "dropdown"] {
            assert!(aura_window_allowed(allowed), "{allowed}");
        }
        assert!(aura_window_allowed("launcher"));
        assert!(!aura_window_allowed(""));
        assert!(!aura_window_allowed("control_center"));
    }

    #[test]
    fn app_launch_map_known_ids() {
        assert_eq!(app_launch_argv("terminal"), Some(vec!["kitty"]));
        assert_eq!(app_launch_argv("browser"), Some(vec!["firefox"]));
        assert_eq!(app_launch_argv("files"), Some(vec!["nautilus"]));
        assert_eq!(app_launch_argv("code"), Some(vec!["cursor"]));
        assert_eq!(app_launch_argv("music"), Some(vec!["spotify"]));
        assert_eq!(app_launch_argv("discord"), Some(vec!["discord"]));
        assert_eq!(app_launch_argv("firefox"), Some(vec!["firefox"]));
        assert!(app_launch_argv("unknown-app").is_none());
    }

    #[test]
    fn resolve_lock_command_hyprlock_or_loginctl() {
        let argv = resolve_lock_command();
        assert!(!argv.is_empty());
        assert!(
            argv[0] == "hyprlock"
                || (argv[0] == "loginctl" && argv.get(1).map(String::as_str) == Some("lock-session"))
        );
    }

    #[test]
    fn resolve_logout_prefers_session_id() {
        let argv = resolve_logout_argv(Some("5"), Some("alice"));
        assert_eq!(
            argv,
            vec!["loginctl", "terminate-session", "5"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn resolve_logout_falls_back_to_user() {
        let argv = resolve_logout_argv(None, Some("bob"));
        assert_eq!(
            argv,
            vec!["loginctl", "terminate-user", "bob"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn resolve_session_action_logout_is_loginctl() {
        match resolve_session_command(SessionAction::Logout, Some("2"), None) {
            SessionCommand::Exec { argv } => {
                assert_eq!(argv[0], "loginctl");
                assert_eq!(argv[1], "terminate-session");
                assert_eq!(argv[2], "2");
            }
            other => panic!("expected exec, got {other:?}"),
        }
    }

    #[test]
    fn resolve_session_action_suspend_uses_login1() {
        match resolve_session_command(SessionAction::Suspend, None, None) {
            SessionCommand::Login1 { method, interactive } => {
                assert_eq!(method, "Suspend");
                assert!(!interactive);
            }
            other => panic!("expected login1, got {other:?}"),
        }
    }

    #[test]
    fn resolve_session_action_reboot_and_poweroff() {
        for (action, method) in [
            (SessionAction::Reboot, "Reboot"),
            (SessionAction::PowerOff, "PowerOff"),
        ] {
            match resolve_session_command(action, None, None) {
                SessionCommand::Login1 { method: m, .. } => assert_eq!(m, method),
                other => panic!("expected login1, got {other:?}"),
            }
        }
    }

    #[test]
    fn classify_session_stderr_polkit() {
        assert_eq!(
            classify_session_stderr("Not authorized to perform operation"),
            "auth_required"
        );
        assert_eq!(
            classify_session_stderr("Operation not supported on this platform"),
            "not_supported"
        );
        assert_eq!(classify_session_stderr("some other fault"), "failed");
    }

    #[test]
    fn classify_session_stderr_auth_variants() {
        for msg in [
            "Access denied",
            "Authentication required",
            "Interactive authentication needed",
            "Polkit error",
            "Permission denied",
            "Operation not permitted",
        ] {
            assert_eq!(
                classify_session_stderr(msg),
                "auth_required",
                "expected auth for {msg}"
            );
        }
    }

    #[test]
    fn format_session_error_uses_first_line_and_truncates() {
        let short = format_session_error("failed", "line one\nline two");
        assert!(short.contains("line one"));
        assert!(!short.contains("line two"));
        let long_detail = "x".repeat(300);
        let truncated = format_session_error("auth_required", &long_detail);
        assert!(truncated.len() < 280);
        assert!(truncated.contains("auth_required"));
    }

    #[test]
    fn resolve_logout_no_session_or_user_uses_placeholder() {
        let argv = resolve_logout_argv(None, None);
        assert_eq!(argv[1], "terminate-user");
        assert_eq!(argv[2], "_");
    }

    #[test]
    fn classify_session_stderr_not_supported_branch() {
        assert_eq!(
            classify_session_stderr("Operation not supported on this platform"),
            "not_supported"
        );
    }
}
