//! Aura shell preferences persisted in SQLite (`settings` / `aura_v1`).

use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const SCHEMA_VERSION: i32 = 1;
const SETTINGS_NS: &str = "settings";
const SETTINGS_KEY: &str = "aura_v1";
const MAX_SETTINGS_BYTES: usize = 65_536;

/// Bar section ids (must match `ui/src/components/bar/useBarLayoutStore.ts`).
const BAR_SECTION_IDS: &[&str] = &[
    "workspaces",
    "runningApps",
    "media",
    "connectivity",
    "calendar",
    "clock",
    "power",
];

/// Control center nav pane ids (subset of `ui/src/pages/control-center/navigation.ts`).
const CC_PANE_IDS: &[&str] = &[
    "packages",
    "settings",
    "network",
    "vpn",
    "bluetooth",
    "audio",
    "notifications",
    "performance",
    "keybinds",
    "security",
    "productivity",
    "automations",
    "calendar",
    "logs",
    "devops",
    "communication",
    "fitness",
    "weather",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AuraSettings {
    pub bar_section_order: Vec<String>,
    pub cc_enabled_panes: Vec<String>,
    pub theme: String,
    pub dropdown_modules: Vec<String>,
}

pub fn default_settings() -> AuraSettings {
    AuraSettings {
        bar_section_order: BAR_SECTION_IDS.iter().map(|s| (*s).to_string()).collect(),
        cc_enabled_panes: CC_PANE_IDS.iter().map(|s| (*s).to_string()).collect(),
        theme: "dark".to_string(),
        dropdown_modules: vec![],
    }
}

fn validate_id_list(ids: &[String], allowed: &[&str], field: &str) -> Result<()> {
    for id in ids {
        if !allowed.contains(&id.as_str()) {
            bail!("invalid {field} id: {id}");
        }
    }
    Ok(())
}

pub fn validate_settings(settings: &AuraSettings) -> Result<()> {
    if settings.bar_section_order.is_empty() {
        bail!("bar_section_order must not be empty");
    }
    validate_id_list(&settings.bar_section_order, BAR_SECTION_IDS, "bar_section_order")?;
    validate_id_list(&settings.cc_enabled_panes, CC_PANE_IDS, "cc_enabled_panes")?;
    for id in &settings.dropdown_modules {
        if !CC_PANE_IDS.contains(&id.as_str()) {
            bail!("invalid dropdown_modules id: {id}");
        }
    }
    if settings.theme.len() > 64 {
        bail!("theme string too long");
    }
    Ok(())
}

pub fn merge_partial(base: &AuraSettings, partial: &Map<String, Value>) -> Result<AuraSettings> {
    let mut out = base.clone();
    let mut value = serde_json::to_value(out)?;
    let obj = value.as_object_mut().expect("settings object");
    for (k, v) in partial {
        if !matches!(
            k.as_str(),
            "bar_section_order" | "cc_enabled_panes" | "theme" | "dropdown_modules"
        ) {
            bail!("unknown settings key: {k}");
        }
        obj.insert(k.clone(), v.clone());
    }
    out = serde_json::from_value(value)?;
    validate_settings(&out)?;
    Ok(out)
}

pub(crate) async fn load_settings() -> Result<AuraSettings> {
    storage::init().await?;
    if let Some(raw) = storage::get_kv(SETTINGS_NS, SETTINGS_KEY).await? {
        let settings: AuraSettings = serde_json::from_value(raw)?;
        validate_settings(&settings)?;
        return Ok(settings);
    }
    Ok(default_settings())
}

async fn save_settings(settings: &AuraSettings) -> Result<()> {
    validate_settings(settings)?;
    let serialized = serde_json::to_string(settings)?;
    if serialized.len() > MAX_SETTINGS_BYTES {
        bail!("settings document too large");
    }
    storage::set_kv(SETTINGS_NS, SETTINGS_KEY, &serde_json::to_value(settings)?).await?;
    Ok(())
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Settings.Get", |_params| async move {
        let settings = load_settings().await?;
        Ok(json!({
            "settings": settings,
            "schema_version": SCHEMA_VERSION,
        }))
    });

    registry.register("Settings.Set", |params| async move {
        let partial = params
            .as_ref()
            .and_then(|p| p.get("partial"))
            .and_then(|v| v.as_object())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("missing partial object"))?;

        let current = load_settings().await?;
        let merged = merge_partial(&current, &partial)?;
        save_settings(&merged).await?;
        Ok(json!({ "settings": merged }))
    });

    registry.register("Settings.GetSchema", |_params| async move {
        Ok(json!({
            "schema_version": SCHEMA_VERSION,
            "fields": {
                "bar_section_order": { "type": "array", "items": BAR_SECTION_IDS },
                "cc_enabled_panes": { "type": "array", "items": CC_PANE_IDS },
                "theme": { "type": "string", "examples": ["dark", "light", "catppuccin-mocha"] },
                "dropdown_modules": { "type": "array", "items": CC_PANE_IDS },
            },
        }))
    });

    registry.register("Settings.Reset", |_params| async move {
        storage::init().await?;
        let _ = storage::delete_kv(SETTINGS_NS, SETTINGS_KEY).await?;
        Ok(json!({ "ok": true }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_partial_updates_theme() {
        let base = default_settings();
        let mut partial = Map::new();
        partial.insert("theme".into(), json!("catppuccin-mocha"));
        let merged = merge_partial(&base, &partial).unwrap();
        assert_eq!(merged.theme, "catppuccin-mocha");
        assert_eq!(merged.bar_section_order, base.bar_section_order);
    }

    #[test]
    fn merge_rejects_unknown_key() {
        let base = default_settings();
        let mut partial = Map::new();
        partial.insert("evil".into(), json!(1));
        assert!(merge_partial(&base, &partial).is_err());
    }

    #[test]
    fn validate_rejects_bad_bar_id() {
        let mut s = default_settings();
        s.bar_section_order = vec!["not-a-section".into()];
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn defaults_match_allowlists() {
        let d = default_settings();
        validate_settings(&d).unwrap();
        assert_eq!(d.bar_section_order.len(), BAR_SECTION_IDS.len());
    }
}
