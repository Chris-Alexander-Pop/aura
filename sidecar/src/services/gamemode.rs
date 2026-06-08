use crate::services::ServiceRegistry;
use anyhow::Result;
use serde_json;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref GAMEMODE_ENABLED: RwLock<bool> = RwLock::new(false);
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("GameMode.Toggle", |_params| async move {
        let mut enabled = GAMEMODE_ENABLED.write().await;
        *enabled = !*enabled;
        apply_gamemode_settings(*enabled).await?;
        Ok(serde_json::json!({ "enabled": *enabled }))
    });

    registry.register("GameMode.Enable", |_params| async move {
        let mut enabled = GAMEMODE_ENABLED.write().await;
        *enabled = true;
        apply_gamemode_settings(true).await?;
        Ok(serde_json::json!({ "enabled": true }))
    });

    registry.register("GameMode.Disable", |_params| async move {
        let mut enabled = GAMEMODE_ENABLED.write().await;
        *enabled = false;
        if !gamemode_dry_run() {
            // Reload config to restore settings
            crate::utils::process::exec_command(&["hyprctl", "reload"]).await.ok();
        }
        Ok(serde_json::json!({ "enabled": false }))
    });

    registry.register("GameMode.IsEnabled", |_params| async move {
        let enabled = GAMEMODE_ENABLED.read().await;
        Ok(serde_json::json!({ "enabled": *enabled }))
    });
}

fn gamemode_dry_run() -> bool {
    std::env::var("AURA_GAMEMODE_DRY_RUN").ok().as_deref() == Some("1")
}

/// Test-only: reset in-process toggle (process-global; serialize gamemode RPC tests).
#[doc(hidden)]
pub async fn reset_gamemode_state_for_tests() {
    let mut enabled = GAMEMODE_ENABLED.write().await;
    *enabled = false;
}

async fn apply_gamemode_settings(enabled: bool) -> Result<()> {
    if gamemode_dry_run() {
        return Ok(());
    }
    if enabled {
        // Use hyprctl to set settings
        let settings = vec![
            ("animations:enabled", "0"),
            ("decoration:shadow:enabled", "0"),
            ("decoration:blur:enabled", "0"),
            ("general:gaps_in", "0"),
            ("general:gaps_out", "0"),
            ("general:border_size", "1"),
            ("decoration:rounding", "0"),
            ("general:allow_tearing", "1"),
        ];

        for (key, value) in settings {
            let cmd = format!("{} {}", key, value);
            let _ = crate::utils::process::exec_command(&["hyprctl", "keyword", &cmd]).await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dry_run_skips_hyprctl_on_enable_and_disable() {
        std::env::set_var("AURA_GAMEMODE_DRY_RUN", "1");
        reset_gamemode_state_for_tests().await;
        apply_gamemode_settings(true).await.expect("enable dry-run");
        apply_gamemode_settings(false).await.expect("disable dry-run");
        std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
    }

    #[tokio::test]
    async fn reset_clears_enabled_flag() {
        reset_gamemode_state_for_tests().await;
        assert!(!*GAMEMODE_ENABLED.read().await);
        {
            let mut enabled = GAMEMODE_ENABLED.write().await;
            *enabled = true;
        }
        reset_gamemode_state_for_tests().await;
        assert!(!*GAMEMODE_ENABLED.read().await);
    }

    #[test]
    fn gamemode_dry_run_env_flag() {
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
        assert!(!gamemode_dry_run());
        std::env::set_var("AURA_GAMEMODE_DRY_RUN", "1");
        assert!(gamemode_dry_run());
        std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
        assert!(!gamemode_dry_run());
    }
}
