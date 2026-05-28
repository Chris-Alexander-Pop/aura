use crate::notify;
use crate::services::ServiceRegistry;
use crate::types::{BatteryState, PowerProfile};
use crate::utils::process;
use anyhow::Result;
use serde_json;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, PartialEq)]
struct PowerServiceState {
    active_profile: PowerProfile,
    battery_percent: u8,
    battery_charging: bool,
    time_remaining: String,
}

impl Default for PowerServiceState {
    fn default() -> Self {
        Self {
            active_profile: PowerProfile::Balanced,
            battery_percent: 0,
            battery_charging: false,
            time_remaining: "Unknown".to_string(),
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: RwLock<PowerServiceState> = RwLock::new(PowerServiceState::default());
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let _ = sync_profile_from_system().await;
        let mut tick = interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            if let Err(e) = update_battery_state().await {
                tracing::warn!("Failed to update battery state: {}", e);
            }
        }
    });

    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(2));
        loop {
            tick.tick().await;
            if let Err(e) = sync_profile_from_system().await {
                tracing::debug!("Profile sync failed: {}", e);
            }
        }
    });

    registry.register("Power.SetProfile", |params| async move {
        let profile_str: String = serde_json::from_value(
            params
                .and_then(|p| p.get("profile").cloned())
                .unwrap_or(serde_json::Value::String("balanced".to_string())),
        )?;

        let profile = parse_profile_str(&profile_str);
        set_profile(profile).await?;
        emit_profile_if_changed().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Power.GetBatteryState", |_params| async move {
        Ok(serde_json::to_value(current_battery_state().await)?)
    });

    registry.register("Power.GetProfile", |_params| async move {
        let state = STATE.read().await;
        Ok(serde_json::json!({ "profile": profile_to_str(&state.active_profile) }))
    });
}

fn parse_profile_str(s: &str) -> PowerProfile {
    match s {
        "performance" => PowerProfile::Performance,
        "saver" | "power-saver" => PowerProfile::PowerSaver,
        _ => PowerProfile::Balanced,
    }
}

pub fn parse_battery_charging(status: Option<&str>) -> bool {
    status
        .map(|s| s == "Charging" || s == "Full")
        .unwrap_or(false)
}

fn profile_to_str(profile: &PowerProfile) -> &'static str {
    match profile {
        PowerProfile::Performance => "performance",
        PowerProfile::PowerSaver => "saver",
        PowerProfile::Balanced => "balanced",
    }
}

fn powerprofilesctl_mode(profile: &PowerProfile) -> &'static str {
    match profile {
        PowerProfile::Performance => "performance",
        PowerProfile::PowerSaver => "power-saver",
        PowerProfile::Balanced => "balanced",
    }
}

async fn current_battery_state() -> BatteryState {
    let state = STATE.read().await;
    BatteryState {
        percent: state.battery_percent,
        charging: state.battery_charging,
        time_remaining: state.time_remaining.clone(),
    }
}

async fn update_battery_state() -> Result<()> {
    let bat = find_battery_supply().await;
    let Some(prefix) = bat else {
        return Ok(());
    };

    let capacity = read_sysfs_u8(&format!("{prefix}/capacity")).await;
    let status = read_sysfs_string(&format!("{prefix}/status")).await;
    let charging = parse_battery_charging(status.as_deref());

    let time_remaining = compute_time_remaining(&prefix, charging).await;

    let mut state = STATE.write().await;
    let prev = state.clone();
    if let Some(cap) = capacity {
        state.battery_percent = cap.min(100);
    }
    state.battery_charging = charging;
    state.time_remaining = time_remaining;

    if prev.battery_percent != state.battery_percent
        || prev.battery_charging != state.battery_charging
        || prev.time_remaining != state.time_remaining
    {
        let battery = BatteryState {
            percent: state.battery_percent,
            charging: state.battery_charging,
            time_remaining: state.time_remaining.clone(),
        };
        notify::emit("Power.BatteryState", serde_json::to_value(battery)?);
    }

    Ok(())
}

async fn find_battery_supply() -> Option<String> {
    for name in ["BAT0", "BAT1"] {
        let path = format!("/sys/class/power_supply/{name}");
        if fs::metadata(&path).await.is_ok() {
            return Some(path);
        }
    }
    None
}

pub fn format_minutes(minutes: u64) -> String {
    if minutes == 0 {
        return "Unknown".to_string();
    }
    if minutes >= 60 {
        let h = minutes / 60;
        let m = minutes % 60;
        if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h {m}m")
        }
    } else {
        format!("{minutes}m")
    }
}

pub fn format_time_from_energy(energy_uwh: u64, power_uw: u64) -> String {
    if power_uw == 0 {
        return "Unknown".to_string();
    }
    let minutes = (energy_uwh as u128 * 60 / power_uw as u128) as u64;
    format_minutes(minutes)
}

pub fn compute_time_remaining_from_sysfs(
    charging: bool,
    time_to_full_now: Option<u64>,
    time_to_empty_now: Option<u64>,
    energy_now: Option<u64>,
    power_now: Option<u64>,
) -> String {
    if charging {
        if let Some(secs) = time_to_full_now {
            if secs > 0 && secs < u64::MAX / 2 {
                return format_minutes(secs / 60);
            }
        }
    } else if let Some(secs) = time_to_empty_now {
        if secs > 0 && secs < u64::MAX / 2 {
            return format_minutes(secs / 60);
        }
    }

    match (energy_now, power_now) {
        (Some(e), Some(p)) if p > 0 => format_time_from_energy(e, p),
        _ => "Unknown".to_string(),
    }
}

async fn compute_time_remaining(prefix: &str, charging: bool) -> String {
    let time_to_full = read_sysfs_u64(&format!("{prefix}/time_to_full_now")).await;
    let time_to_empty = read_sysfs_u64(&format!("{prefix}/time_to_empty_now")).await;
    let energy = read_sysfs_u64(&format!("{prefix}/energy_now")).await;
    let power = read_sysfs_u64(&format!("{prefix}/power_now")).await;
    compute_time_remaining_from_sysfs(charging, time_to_full, time_to_empty, energy, power)
}

async fn read_sysfs_u8(path: &str) -> Option<u8> {
    read_sysfs_string(path)
        .await?
        .trim()
        .parse()
        .ok()
}

async fn read_sysfs_u64(path: &str) -> Option<u64> {
    read_sysfs_string(path)
        .await?
        .trim()
        .parse()
        .ok()
}

async fn read_sysfs_string(path: &str) -> Option<String> {
    fs::read_to_string(path).await.ok().map(|s| s.trim().to_string())
}

async fn sync_profile_from_system() -> Result<()> {
    let profile = if process::exec_command(&["which", "powerprofilesctl"])
        .await
        .is_ok()
    {
        let out = process::exec_command(&["powerprofilesctl", "get"]).await?;
        parse_powerprofilesctl_output(&out)
    } else {
        read_caelestia_profile_file().await?
    };

    let Some(profile) = profile else {
        return Ok(());
    };

    let mut state = STATE.write().await;
    if state.active_profile != profile {
        state.active_profile = profile;
        drop(state);
        emit_profile_if_changed().await;
    }
    Ok(())
}

pub fn parse_powerprofilesctl_output(out: &str) -> Option<PowerProfile> {
    let line = out.lines().next()?.trim().to_lowercase();
    if line.contains("performance") {
        Some(PowerProfile::Performance)
    } else if line.contains("power-saver") || line.contains("power_saver") || line == "power-saver" {
        Some(PowerProfile::PowerSaver)
    } else {
        Some(PowerProfile::Balanced)
    }
}

async fn read_caelestia_profile_file() -> Result<Option<PowerProfile>> {
    let home = std::env::var("HOME")?;
    let profile_path = PathBuf::from(home)
        .join(".config")
        .join("caelestia")
        .join("current-power-profile");

    if !profile_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&profile_path).await?;
    Ok(Some(parse_profile_str(content.trim())))
}

async fn emit_profile_if_changed() -> Result<()> {
    let state = STATE.read().await;
    notify::emit(
        "Power.Profile",
        serde_json::json!({ "profile": profile_to_str(&state.active_profile) }),
    );
    Ok(())
}

async fn set_profile(profile: PowerProfile) -> Result<()> {
    if process::exec_command(&["which", "powerprofilesctl"])
        .await
        .is_ok()
    {
        let mode = powerprofilesctl_mode(&profile);
        process::exec_command(&["powerprofilesctl", "set", mode]).await?;
    } else {
        let home = std::env::var("HOME")?;
        let script_path = PathBuf::from(home)
            .join(".config")
            .join("caelestia")
            .join("set-power-profile.sh");

        let mode = profile_to_str(&profile);
        if script_path.exists() {
            process::exec_command_detached(&[script_path.to_str().unwrap(), mode]).await?;
        } else {
            tracing::warn!("No powerprofilesctl or Caelestia profile script found");
        }
    }

    let mut state = STATE.write().await;
    state.active_profile = profile;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_minutes_human() {
        assert_eq!(format_minutes(0), "Unknown");
        assert_eq!(format_minutes(45), "45m");
        assert_eq!(format_minutes(90), "1h 30m");
        assert_eq!(format_minutes(120), "2h");
        assert_eq!(format_minutes(60), "1h");
    }

    #[test]
    fn format_time_from_energy_calc() {
        // 30 Wh remaining at 10 W draw ≈ 3 hours
        let s = format_time_from_energy(30_000_000, 10_000_000);
        assert_eq!(s, "3h");
    }

    #[test]
    fn format_time_from_energy_zero_power_is_unknown() {
        assert_eq!(format_time_from_energy(30_000_000, 0), "Unknown");
    }

    #[test]
    fn parse_profile_str_aliases() {
        assert_eq!(parse_profile_str("performance"), PowerProfile::Performance);
        assert_eq!(parse_profile_str("saver"), PowerProfile::PowerSaver);
        assert_eq!(parse_profile_str("power-saver"), PowerProfile::PowerSaver);
        assert_eq!(parse_profile_str("turbo"), PowerProfile::Balanced);
        assert_eq!(profile_to_str(&PowerProfile::Balanced), "balanced");
    }

    #[test]
    fn parse_powerprofilesctl_modes() {
        assert_eq!(
            parse_powerprofilesctl_output("performance"),
            Some(PowerProfile::Performance)
        );
        assert_eq!(
            parse_powerprofilesctl_output("power-saver"),
            Some(PowerProfile::PowerSaver)
        );
        assert_eq!(
            parse_powerprofilesctl_output(" balanced \n"),
            Some(PowerProfile::Balanced)
        );
        assert_eq!(
            parse_powerprofilesctl_output("Profile: power_saver"),
            Some(PowerProfile::PowerSaver)
        );
    }

    #[test]
    fn parse_powerprofilesctl_fixture_first_line() {
        let fixture = include_str!("../../tests/fixtures/power/powerprofilesctl_get.txt");
        let first = fixture.lines().next().unwrap_or("");
        assert_eq!(
            parse_powerprofilesctl_output(first),
            Some(PowerProfile::Performance)
        );
    }

    #[test]
    fn powerprofilesctl_mode_round_trip() {
        assert_eq!(powerprofilesctl_mode(&PowerProfile::PowerSaver), "power-saver");
        assert_eq!(powerprofilesctl_mode(&PowerProfile::Balanced), "balanced");
    }

    #[test]
    fn parse_battery_charging_states() {
        assert!(parse_battery_charging(Some("Charging")));
        assert!(parse_battery_charging(Some("Full")));
        assert!(!parse_battery_charging(Some("Discharging")));
        assert!(!parse_battery_charging(None));
    }

    #[test]
    fn compute_time_remaining_zero_battery_sysfs() {
        assert_eq!(
            compute_time_remaining_from_sysfs(false, None, Some(0), None, None),
            "Unknown"
        );
        assert_eq!(
            compute_time_remaining_from_sysfs(false, None, Some(1800), None, None),
            "30m"
        );
        assert_eq!(
            compute_time_remaining_from_sysfs(true, Some(3600), None, None, None),
            "1h"
        );
    }

    #[test]
    fn parse_powerprofilesctl_empty_output() {
        assert_eq!(parse_powerprofilesctl_output(""), None);
    }
}
