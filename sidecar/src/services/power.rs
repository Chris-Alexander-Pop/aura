use crate::services::ServiceRegistry;
use crate::types::{BatteryState, PowerProfile};
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct PowerServiceState {
    active_profile: PowerProfile,
    battery_percent: u8,
    battery_charging: bool,
}

impl Default for PowerServiceState {
    fn default() -> Self {
        Self {
            active_profile: PowerProfile::Balanced,
            battery_percent: 0,
            battery_charging: false,
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: RwLock<PowerServiceState> = RwLock::new(PowerServiceState::default());
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start battery monitoring task
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Err(e) = update_battery_state().await {
                tracing::warn!("Failed to update battery state: {}", e);
            }
        }
    });

    // Start profile file watcher
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Err(e) = check_profile_file().await {
                tracing::debug!("Profile file check failed: {}", e);
            }
        }
    });

    registry.register("Power.SetProfile", |params| async move {
        let profile_str: String = serde_json::from_value(
            params
                .and_then(|p| p.get("profile").cloned())
                .unwrap_or(serde_json::Value::String("balanced".to_string())),
        )?;

        let profile = match profile_str.as_str() {
            "performance" => PowerProfile::Performance,
            "saver" | "power-saver" => PowerProfile::PowerSaver,
            _ => PowerProfile::Balanced,
        };

        set_profile(profile).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Power.GetBatteryState", |_params| async move {
        let state = STATE.read().await;
        let battery_state = BatteryState {
            percent: state.battery_percent,
            charging: state.battery_charging,
            time_remaining: "Unknown".to_string(), // TODO: Calculate from battery info
        };
        Ok(serde_json::to_value(battery_state)?)
    });

    registry.register("Power.GetProfile", |_params| async move {
        let state = STATE.read().await;
        let profile_str = match state.active_profile {
            PowerProfile::Performance => "performance",
            PowerProfile::PowerSaver => "saver",
            PowerProfile::Balanced => "balanced",
        };
        Ok(serde_json::json!({ "profile": profile_str }))
    });
}

async fn update_battery_state() -> Result<()> {
    let capacity_paths = [
        "/sys/class/power_supply/BAT0/capacity",
        "/sys/class/power_supply/BAT1/capacity",
    ];
    let status_paths = [
        "/sys/class/power_supply/BAT0/status",
        "/sys/class/power_supply/BAT1/status",
    ];

    let mut capacity: Option<u8> = None;
    let mut charging: Option<bool> = None;

    for capacity_path in &capacity_paths {
        if let Ok(content) = fs::read_to_string(capacity_path).await {
            if let Ok(cap) = content.trim().parse::<u8>() {
                capacity = Some(cap);
                break;
            }
        }
    }

    for status_path in &status_paths {
        if let Ok(content) = fs::read_to_string(status_path).await {
            let status = content.trim();
            charging = Some(status == "Charging");
            break;
        }
    }

    let mut state = STATE.write().await;
    if let Some(cap) = capacity {
        state.battery_percent = cap;
    }
    if let Some(chg) = charging {
        state.battery_charging = chg;
    }

    Ok(())
}

async fn check_profile_file() -> Result<()> {
    let home = std::env::var("HOME")?;
    let profile_path = PathBuf::from(home)
        .join(".config")
        .join("caelestia")
        .join("current-power-profile");

    if !profile_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&profile_path).await?;
    let profile = match content.trim() {
        "saver" => PowerProfile::PowerSaver,
        "performance" => PowerProfile::Performance,
        _ => PowerProfile::Balanced,
    };

    let mut state = STATE.write().await;
    state.active_profile = profile;

    Ok(())
}

async fn set_profile(profile: PowerProfile) -> Result<()> {
    let home = std::env::var("HOME")?;
    let script_path = PathBuf::from(home)
        .join(".config")
        .join("caelestia")
        .join("set-power-profile.sh");

    let mode = match profile {
        PowerProfile::Performance => "performance",
        PowerProfile::PowerSaver => "saver",
        PowerProfile::Balanced => "balanced",
    };

    if script_path.exists() {
        process::exec_command_detached(&[
            script_path.to_str().unwrap(),
            mode,
        ])
        .await?;
    } else {
        tracing::warn!("Power profile script not found: {:?}", script_path);
    }

    let mut state = STATE.write().await;
    state.active_profile = profile;

    Ok(())
}
