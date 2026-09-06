//! Read-only aggregates for top dropdown and sidebar tiles.

use crate::services::{
    audio, bluetooth, calendar, network, notifications, power, productivity, settings,
};
use crate::services::ServiceRegistry;
use crate::types::{BatteryState, NetworkStatus};
use anyhow::{bail, Result};
use serde_json::{json, Value};

const SIDEBAR_TILE_IDS: &[&str] = &[
    "network",
    "audio",
    "bluetooth",
    "battery",
    "calendar",
    "notifications",
    "productivity",
];

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Dashboard.GetQuickStatus", |_params| async move {
        Ok(get_quick_status().await?)
    });

    registry.register("Sidebar.GetTileData", |params| async move {
        let tile = params
            .as_ref()
            .and_then(|p| p.get("tile"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing tile"))?;
        Ok(get_tile_data(tile).await?)
    });
}

async fn get_quick_status() -> Result<Value> {
    let aura_settings = settings::load_settings().await?;
    let battery: BatteryState = power::snapshot_battery().await;
    let network_status: NetworkStatus = network::snapshot_network_status().await;
    let bluetooth = bluetooth::snapshot_bluetooth_quick().await;
    let dnd = notifications::snapshot_dnd().await?;
    let next_event = calendar::snapshot_next_event().await?;
    let power_profile = power::snapshot_profile_name().await;

    Ok(json!({
        "battery": battery,
        "network": network_status,
        "bluetooth": bluetooth,
        "dnd": dnd,
        "next_event": next_event,
        "power_profile": power_profile,
        "dropdown_modules": aura_settings.dropdown_modules,
    }))
}

async fn get_tile_data(tile: &str) -> Result<Value> {
    if !SIDEBAR_TILE_IDS.contains(&tile) {
        bail!("unknown tile: {tile}");
    }

    let aura_settings = settings::load_settings().await?;
    if !aura_settings.dropdown_modules.is_empty()
        && !aura_settings.dropdown_modules.iter().any(|m| m == tile)
    {
        bail!("tile not enabled in dropdown_modules: {tile}");
    }

    let data = match tile {
        "network" => json!(network::snapshot_network_status().await),
        "audio" => audio::snapshot_audio_tile().await,
        "bluetooth" => bluetooth::snapshot_bluetooth_quick().await,
        "battery" => json!({
            "battery": power::snapshot_battery().await,
            "power_profile": power::snapshot_profile_name().await,
        }),
        "calendar" => json!({
            "events": calendar::snapshot_upcoming(5).await?,
        }),
        "notifications" => json!({
            "dnd": notifications::snapshot_dnd().await?,
        }),
        "productivity" => productivity::snapshot_productivity_stats().await?,
        _ => unreachable!(),
    };

    Ok(json!({ "tile": tile, "data": data }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_tile_allowlist_is_stable() {
        assert!(SIDEBAR_TILE_IDS.contains(&"network"));
        assert!(!SIDEBAR_TILE_IDS.contains(&"vpn"));
    }
}
