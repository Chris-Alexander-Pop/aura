//! Allowlisted session actions, Aura window toggles, app launch — invoked from React/WebKit only via JSON-RPC.
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::bail;
use serde_json::json;
use std::collections::HashMap;

pub(crate) fn aura_window_allowed(name: &str) -> bool {
    matches!(
        name,
        "control-center" | "sidebar" | "calendar" | "dropdown"
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

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Sidecar.GetVersion", |_p| async move {
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
        }))
    });

    registry.register("Session.Lock", |_p| async move {
        process::exec_command_detached(&["loginctl", "lock-session"]).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Session.Logout", |_p| async move {
        process::exec_command_detached(&["hyprctl", "dispatch", "exit"]).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Session.Suspend", |_p| async move {
        process::exec_command_detached(&["systemctl", "suspend"]).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Session.Reboot", |_p| async move {
        process::exec_command_detached(&["systemctl", "reboot"]).await?;
        Ok(json!({"ok": true}))
    });

    registry.register("Session.PowerOff", |_p| async move {
        process::exec_command_detached(&["systemctl", "poweroff"]).await?;
        Ok(json!({"ok": true}))
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

#[cfg(test)]
mod tests {
    use super::aura_window_allowed;

    #[test]
    fn aura_window_allowlist() {
        for allowed in ["control-center", "sidebar", "calendar", "dropdown"] {
            assert!(aura_window_allowed(allowed), "{allowed}");
        }
        assert!(!aura_window_allowed("launcher"));
        assert!(!aura_window_allowed(""));
    }
}
