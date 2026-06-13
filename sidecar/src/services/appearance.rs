//! Night light (wlsunset / gammastep) and theme helpers for web UI.

use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tokio::sync::RwLock;

const NS: &str = "appearance";
const NIGHT_KEY: &str = "night_light";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NightLightState {
    pub enabled: bool,
    #[serde(default = "default_temperature")]
    pub temperature: u32,
}

fn default_temperature() -> u32 {
    4500
}

impl Default for NightLightState {
    fn default() -> Self {
        Self {
            enabled: false,
            temperature: default_temperature(),
        }
    }
}

lazy_static::lazy_static! {
    static ref NIGHT: RwLock<NightLightState> = RwLock::new(NightLightState::default());
}

async fn load_night() -> NightLightState {
    storage::init().await.ok();
    if let Ok(Some(raw)) = storage::get_kv(NS, NIGHT_KEY).await {
        if let Ok(s) = serde_json::from_value::<NightLightState>(raw) {
            return s;
        }
    }
    NightLightState::default()
}

async fn persist_night(state: &NightLightState) -> Result<()> {
    storage::init().await?;
    storage::set_kv(NS, NIGHT_KEY, &serde_json::to_value(state)?).await?;
    Ok(())
}

fn night_tool() -> &'static str {
    match std::env::var("AURA_NIGHT_LIGHT").ok().as_deref() {
        Some("gammastep") => "gammastep",
        _ => "wlsunset",
    }
}

async fn stop_night_processes() {
    let _ = process::run_allowlisted(&["pkill", "-x", "wlsunset"]).await;
    let _ = process::run_allowlisted(&["pkill", "-x", "gammastep"]).await;
}

async fn apply_night(state: &NightLightState) -> Result<()> {
    stop_night_processes().await;
    if !state.enabled {
        return Ok(());
    }
    let tool = night_tool();
    if state.enabled {
        if tool == "gammastep" {
            process::run_allowlisted_detached(&[
                "gammastep",
                "-O",
                "6500",
                "-T",
                &state.temperature.to_string(),
            ])
            .await?;
        } else {
            process::run_allowlisted_detached(&["wlsunset", "-t", &state.temperature.to_string()])
                .await?;
        }
    }
    Ok(())
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let s = load_night().await;
        let mut guard = NIGHT.write().await;
        *guard = s;
    });

    registry.register("Appearance.GetNightLight", |_params| async move {
        let state = NIGHT.read().await.clone();
        Ok(json!(state))
    });

    registry.register("Appearance.SetNightLight", |params| async move {
        let partial = params
            .as_ref()
            .and_then(|p| p.as_object())
            .cloned()
            .unwrap_or_default();
        let mut state = NIGHT.read().await.clone();
        if let Some(v) = partial.get("enabled") {
            state.enabled = serde_json::from_value(v.clone())?;
        }
        if let Some(v) = partial.get("temperature") {
            let t: u32 = serde_json::from_value(v.clone())?;
            if !(1000..=10000).contains(&t) {
                bail!("temperature must be 1000–10000");
            }
            state.temperature = t;
        }
        apply_night(&state).await?;
        persist_night(&state).await?;
        {
            let mut guard = NIGHT.write().await;
            *guard = state.clone();
        }
        notify::emit("Appearance.NightLightChanged", json!(state));
        Ok(json!({ "ok": true, "state": state }))
    });

    registry.register("Appearance.GetTheme", |_params| async move {
        let settings = crate::services::settings::load_settings().await?;
        Ok(json!({ "theme": settings.theme }))
    });

    registry.register("Appearance.SetTheme", |params| async move {
        let theme = params
            .as_ref()
            .and_then(|p| p.get("theme"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing theme"))?;
        let mut partial = Map::new();
        partial.insert("theme".into(), Value::String(theme.to_string()));
        let merged = crate::services::settings::apply_settings_partial(partial).await?;
        notify::emit("Settings.Changed", json!({ "field": "theme" }));
        Ok(json!({ "ok": true, "theme": merged.theme }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_light_defaults() {
        let s = NightLightState::default();
        assert!(!s.enabled);
        assert_eq!(s.temperature, 4500);
    }
}
