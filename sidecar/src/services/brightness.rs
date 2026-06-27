use crate::notify;
use crate::osd;
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
enum MonitorType {
    Default { device: String },
    DDC { bus_num: String },
    AppleDisplay,
}

#[derive(Debug, Clone)]
struct Monitor {
    name: String,
    monitor_type: MonitorType,
    brightness: f64,
}

lazy_static::lazy_static! {
    static ref MONITORS: RwLock<HashMap<String, Monitor>> = RwLock::new(HashMap::new());
    static ref APPLE_DISPLAY_PRESENT: RwLock<bool> = RwLock::new(false);
    static ref ACTIVE_BACKLIGHT: RwLock<Option<String>> = RwLock::new(None);
}

async fn ensure_monitors() {
    if MONITORS.read().await.is_empty() {
        detect_monitors().await.ok();
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        detect_monitors().await.ok();
    });

    registry.register("Brightness.Get", |params| async move {
        ensure_monitors().await;

        let monitor_query = params
            .as_ref()
            .and_then(|p| p.get("monitor").cloned())
            .and_then(|v| serde_json::from_value::<String>(v).ok())
            .unwrap_or_else(|| "active".to_string());

        if monitor_query == "all" {
            let monitors = MONITORS.read().await;
            let list: Vec<Value> = monitors
                .values()
                .map(|m| {
                    json!({
                        "monitor": m.name,
                        "brightness": m.brightness
                    })
                })
                .collect();
            return Ok(json!({ "monitors": list }));
        }

        let refresh = params
            .as_ref()
            .and_then(|p| p.get("refresh"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let active = ACTIVE_BACKLIGHT.read().await.clone();
        let monitors = MONITORS.read().await;
        let monitor = find_monitor(&monitors, &monitor_query, active.as_deref());

        if let Some(m) = monitor {
            let brightness = if refresh {
                refresh_monitor_brightness(m).await.unwrap_or(m.brightness)
            } else {
                m.brightness
            };
            Ok(json!({
                "monitor": m.name,
                "brightness": brightness
            }))
        } else {
            Ok(json!({
                "monitor": monitor_query,
                "brightness": 0.5
            }))
        }
    });

    registry.register("Brightness.Set", |params| async move {
        ensure_monitors().await;

        let p = params
            .as_ref()
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("Missing params"))?;

        let monitor_query: String = serde_json::from_value(
            p.get("monitor")
                .cloned()
                .ok_or_else(|| anyhow!("Missing monitor parameter"))?,
        )?;

        let value = brightness_set_value_from_params(p)?;

        let active = ACTIVE_BACKLIGHT.read().await.clone();
        let monitors = MONITORS.read().await;
        let monitor = find_monitor(&monitors, &monitor_query, active.as_deref())
            .ok_or_else(|| anyhow!("Monitor not found: {}", monitor_query))?
            .clone();

        let target_brightness = parse_brightness_value(&value, monitor.brightness)?;
        set_brightness(&monitor, target_brightness).await?;

        let mut monitors = MONITORS.write().await;
        if let Some(m) = monitors.get_mut(&monitor.name) {
            m.brightness = target_brightness;
        }

        osd::emit_brightness(target_brightness);
        notify::emit(
            "Brightness.StateChanged",
            json!({
                "monitor": monitor.name,
                "brightness": target_brightness,
            }),
        );

        Ok(json!({
            "success": true,
            "monitor": monitor.name,
            "brightness": target_brightness
        }))
    });
}

fn brightness_set_value_from_params(params: &serde_json::Map<String, Value>) -> Result<String> {
    if let Some(v) = params.get("value") {
        return Ok(serde_json::from_value(v.clone())?);
    }
    if let Some(p) = params.get("percent") {
        let n: f64 = serde_json::from_value(p.clone())?;
        let frac = if n > 1.0 { n / 100.0 } else { n };
        let pct = (frac * 100.0).round() as i64;
        return Ok(format!("{pct}%"));
    }
    Err(anyhow!("Missing value or percent parameter"))
}

fn find_monitor<'a>(
    monitors: &'a HashMap<String, Monitor>,
    query: &str,
    active: Option<&str>,
) -> Option<&'a Monitor> {
    if query == "active" {
        if let Some(name) = active {
            if let Some(m) = monitors.get(name) {
                return Some(m);
            }
        }
        return monitors
            .values()
            .find(|m| matches!(m.monitor_type, MonitorType::Default { .. }));
    }
    monitors.get(query)
}

async fn refresh_monitor_brightness(monitor: &Monitor) -> Option<f64> {
    match &monitor.monitor_type {
        MonitorType::Default { device } => read_brightnessctl_device(device).await.ok(),
        MonitorType::DDC { bus_num } => get_ddc_brightness(bus_num).await.ok(),
        MonitorType::AppleDisplay => None,
    }
}

pub fn parse_brightness_value(value: &str, current: f64) -> Result<f64> {
    let value = value.trim();

    if value.ends_with("%-") {
        let percent = value[..value.len() - 2].parse::<f64>()? / 100.0;
        Ok((current - percent).max(0.0).min(1.0))
    } else if value.starts_with('+') && value.ends_with('%') {
        let percent = value[1..value.len() - 1].parse::<f64>()? / 100.0;
        Ok((current + percent).max(0.0).min(1.0))
    } else if value.ends_with('%') {
        let percent = value[..value.len() - 1].parse::<f64>()? / 100.0;
        Ok(percent.max(0.0).min(1.0))
    } else if value.starts_with('+') {
        let increment = value[1..].parse::<f64>()?;
        Ok((current + increment).max(0.0).min(1.0))
    } else if value.ends_with('-') {
        let decrement = value[..value.len() - 1].parse::<f64>()?;
        Ok((current - decrement).max(0.0).min(1.0))
    } else {
        let absolute = value.parse::<f64>()?;
        let frac = if absolute > 1.0 {
            absolute / 100.0
        } else {
            absolute
        };
        Ok(frac.max(0.0).min(1.0))
    }
}

/// Parse `brightnessctl -m` line: `device,class,current,NN%,max`.
pub fn parse_brightnessctl_machine_line(line: &str) -> Option<(String, f64)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 5 {
        return None;
    }
    if parts[1] != "backlight" {
        return None;
    }
    let name = parts[0].trim().to_string();
    let percent = parts[3].trim().trim_end_matches('%').parse::<f64>().ok()?;
    Some((name, (percent / 100.0).max(0.0).min(1.0)))
}

/// Parse human `brightnessctl -l` output; returns backlight devices only.
pub fn parse_brightnessctl_list(output: &str) -> Vec<(String, f64)> {
    let mut devices = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_class: Option<String> = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Device '") {
            if let Some((name, rest)) = rest.split_once("' of class '") {
                let class = rest
                    .strip_suffix(":")
                    .or_else(|| rest.strip_suffix("': "))
                    .unwrap_or(rest)
                    .trim_end_matches('\'')
                    .to_string();
                current_name = Some(name.to_string());
                current_class = Some(class);
            }
        } else if line.starts_with("Current brightness:") {
            if current_class.as_deref() != Some("backlight") {
                continue;
            }
            let Some(name) = current_name.take() else {
                continue;
            };
            current_class = None;
            if let Some(paren) = line.split('(').nth(1) {
                if let Some(pct) = paren.split('%').next() {
                    if let Ok(percent) = pct.trim().parse::<f64>() {
                        devices.push((name, (percent / 100.0).max(0.0).min(1.0)));
                    }
                }
            }
        }
    }

    devices
}

async fn set_brightness(monitor: &Monitor, brightness: f64) -> Result<()> {
    if std::env::var("AURA_BRIGHTNESS_DRY_RUN").ok().as_deref() == Some("1") {
        return Ok(());
    }

    let rounded = (brightness * 100.0).round() as u32;

    match &monitor.monitor_type {
        MonitorType::AppleDisplay => {
            if *APPLE_DISPLAY_PRESENT.read().await {
                process::exec_command_detached(&["asdbctl", "set", &rounded.to_string()]).await?;
            }
        }
        MonitorType::DDC { bus_num } => {
            process::exec_command(&[
                "ddcutil",
                "-b",
                bus_num,
                "setvcp",
                "10",
                &rounded.to_string(),
            ])
            .await?;
        }
        MonitorType::Default { device } => {
            process::exec_command(&[
                "brightnessctl",
                "-d",
                device,
                "s",
                &format!("{rounded}%"),
            ])
            .await?;
        }
    }

    Ok(())
}

async fn read_brightnessctl_device(device: &str) -> Result<f64> {
    let output = process::exec_command(&["brightnessctl", "-d", device, "-m"]).await?;
    for line in output.lines() {
        if let Some((_, b)) = parse_brightnessctl_machine_line(line) {
            return Ok(b);
        }
    }
    Ok(0.5)
}

async fn detect_monitors() -> Result<()> {
    let mut monitors = HashMap::new();

    let apple_check = process::exec_command(&["asdbctl", "get"]).await;
    let apple_present = apple_check.is_ok();
    *APPLE_DISPLAY_PRESENT.write().await = apple_present;

    let mut backlight_devices: Vec<(String, f64)> = Vec::new();
    if let Ok(list_out) = process::exec_command(&["brightnessctl", "-l"]).await {
        backlight_devices = parse_brightnessctl_list(&list_out);
    }
    if backlight_devices.is_empty() {
        if let Ok(m_out) = process::exec_command(&["brightnessctl", "-m"]).await {
            for line in m_out.lines() {
                if let Some((name, b)) = parse_brightnessctl_machine_line(line) {
                    backlight_devices.push((name, b));
                }
            }
        }
    }

    let active = backlight_devices
        .first()
        .map(|(n, _)| n.clone())
        .or_else(|| Some("default".to_string()));
    *ACTIVE_BACKLIGHT.write().await = active.clone();

    for (device, brightness) in &backlight_devices {
        let name = device.clone();
        monitors.insert(
            name.clone(),
            Monitor {
                name: name.clone(),
                monitor_type: MonitorType::Default {
                    device: device.clone(),
                },
                brightness: *brightness,
            },
        );
    }

    if let Some(first) = backlight_devices.first() {
        monitors.insert(
            "default".to_string(),
            Monitor {
                name: "default".to_string(),
                monitor_type: MonitorType::Default {
                    device: first.0.clone(),
                },
                brightness: first.1,
            },
        );
    }

    let ddc_output = process::exec_command(&["ddcutil", "detect", "--brief"]).await.ok();
    let ddc_monitors = ddc_output
        .as_deref()
        .map(parse_ddc_monitors)
        .unwrap_or_default();

    for (name, bus_num) in ddc_monitors {
        let brightness = get_ddc_brightness(&bus_num).await.unwrap_or(0.5);
        monitors.insert(
            name.clone(),
            Monitor {
                name,
                monitor_type: MonitorType::DDC { bus_num },
                brightness,
            },
        );
    }

    if apple_present {
        monitors.insert(
            "apple".to_string(),
            Monitor {
                name: "apple".to_string(),
                monitor_type: MonitorType::AppleDisplay,
                brightness: 0.5,
            },
        );
    }

    if monitors.is_empty() {
        monitors.insert(
            "default".to_string(),
            Monitor {
                name: "default".to_string(),
                monitor_type: MonitorType::Default {
                    device: "intel_backlight".to_string(),
                },
                brightness: 0.5,
            },
        );
        *ACTIVE_BACKLIGHT.write().await = Some("default".to_string());
    }

    *MONITORS.write().await = monitors;
    Ok(())
}

fn parse_ddc_monitors(output: &str) -> Vec<(String, String)> {
    let mut monitors = Vec::new();
    let mut current_bus: Option<String> = None;
    let mut current_connector: Option<String> = None;

    let flush = |monitors: &mut Vec<(String, String)>,
                 bus: &mut Option<String>,
                 connector: &mut Option<String>| {
        if let (Some(b), Some(c)) = (bus.take(), connector.take()) {
            monitors.push((c, b));
        }
    };

    for line in output.lines() {
        if line.starts_with("Display ") {
            flush(&mut monitors, &mut current_bus, &mut current_connector);
        } else if line.contains("I2C bus:") {
            if let Some(bus) = line
                .split("I2C bus:")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .map(|s| s.trim_start_matches("/dev/i2c-").to_string())
            {
                current_bus = Some(bus);
            }
        } else if line.contains("DRM connector:") {
            if let Some(connector) = line
                .split("DRM connector:")
                .nth(1)
                .map(|s| s.trim().replace("card1-", "").replace("card0-", ""))
            {
                current_connector = Some(connector);
            }
        } else if line.is_empty() {
            flush(&mut monitors, &mut current_bus, &mut current_connector);
        }
    }

    flush(&mut monitors, &mut current_bus, &mut current_connector);
    monitors
}

#[cfg(test)]
mod tests {
    use super::parse_brightness_value;

    #[test]
    fn parse_brightness_relative_and_absolute() {
        assert_eq!(parse_brightness_value("50%", 0.5).unwrap(), 0.5);
        assert_eq!(parse_brightness_value("+10%", 0.5).unwrap(), 0.6);
        assert_eq!(parse_brightness_value("10%-", 0.5).unwrap(), 0.4);
        assert_eq!(parse_brightness_value("0.8", 0.5).unwrap(), 0.8);
        assert_eq!(parse_brightness_value("+0.1", 0.5).unwrap(), 0.6);
        assert_eq!(parse_brightness_value("75", 0.5).unwrap(), 0.75);
    }

    #[test]
    fn parse_ddcutil_detect_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/brightness/ddcutil_detect.txt"
        );
        let text = std::fs::read_to_string(path).expect("fixture");
        let monitors = super::parse_ddc_monitors(&text);
        assert_eq!(monitors.len(), 2);
        assert_eq!(monitors[0].0, "HDMI-A-1");
        assert_eq!(monitors[0].1, "7");
    }

    #[test]
    fn parse_ddc_vcp_brightness_fixture() {
        let text = include_str!("../../tests/fixtures/brightness/ddcutil_getvcp.txt");
        let b = super::parse_ddc_vcp_brightness(text).expect("vcp");
        assert!((b - 0.5).abs() < f64::EPSILON);
        assert!(super::parse_ddc_vcp_brightness("VCP 10 C 0 0").is_none());
    }

    #[test]
    fn parse_brightness_value_clamps() {
        assert_eq!(parse_brightness_value("150%", 0.5).unwrap(), 1.0);
        assert!(parse_brightness_value("not-a-number", 0.5).is_err());
    }

    #[test]
    fn parse_brightnessctl_list_fixture() {
        let text = include_str!("../../tests/fixtures/brightness/brightnessctl_list.txt");
        let devices = super::parse_brightnessctl_list(text);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].0, "intel_backlight");
        assert!((devices[0].1 - 0.5).abs() < f64::EPSILON);
        assert_eq!(devices[1].0, "eDP-1");
        assert!((devices[1].1 - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_brightnessctl_machine_fixture() {
        let text = include_str!("../../tests/fixtures/brightness/brightnessctl_machine.txt");
        let lines: Vec<_> = text
            .lines()
            .filter_map(super::parse_brightnessctl_machine_line)
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "intel_backlight");
        assert!((lines[0].1 - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn brightness_set_value_from_params_accepts_percent() {
        let mut map = serde_json::Map::new();
        map.insert("percent".into(), serde_json::json!(50));
        assert_eq!(
            super::brightness_set_value_from_params(&map).unwrap(),
            "50%"
        );
    }
}

async fn get_ddc_brightness(bus_num: &str) -> Result<f64> {
    let output = process::exec_command(&["ddcutil", "-b", bus_num, "getvcp", "10", "--brief"]).await?;
    Ok(parse_ddc_vcp_brightness(&output).unwrap_or(0.5))
}

/// Parse `ddcutil getvcp 10 --brief` output (e.g. `VCP 10 C 50 100`) as 0.0–1.0.
pub fn parse_ddc_vcp_brightness(output: &str) -> Option<f64> {
    let nums: Vec<u32> = output
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if nums.len() < 2 {
        return None;
    }
    let max = *nums.last()?;
    let current = nums[nums.len() - 2];
    if max > 0 {
        Some(current as f64 / max as f64)
    } else {
        None
    }
}
