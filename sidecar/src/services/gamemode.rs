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
        // Reload config to restore settings
        crate::utils::process::exec_command(&["hyprctl", "reload"]).await.ok();
        Ok(serde_json::json!({ "enabled": false }))
    });

    registry.register("GameMode.IsEnabled", |_params| async move {
        let enabled = GAMEMODE_ENABLED.read().await;
        Ok(serde_json::json!({ "enabled": *enabled }))
    });
}

async fn apply_gamemode_settings(enabled: bool) -> Result<()> {
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
