use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
enum MonitorType {
    Default,
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
}

pub fn register(registry: &mut ServiceRegistry) {
    // Detect monitors on startup
    tokio::spawn(async {
        detect_monitors().await.ok();
    });

    registry.register("Brightness.Get", |params| async move {
        let monitor_query = params
            .and_then(|p| p.get("monitor").cloned())
            .and_then(|v| serde_json::from_value::<String>(v).ok())
            .unwrap_or_else(|| "active".to_string());

        let monitors = MONITORS.read().await;
        let monitor = find_monitor(&monitors, &monitor_query);

        if let Some(m) = monitor {
            Ok(serde_json::json!({
                "monitor": m.name,
                "brightness": m.brightness
            }))
        } else {
            Ok(serde_json::json!({
                "monitor": monitor_query,
                "brightness": 0.5
            }))
        }
    });

    registry.register("Brightness.Set", |params| async move {
        let monitor_query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("monitor").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing monitor parameter"))?,
        )?;

        let value: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("value").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing value parameter"))?,
        )?;

        let monitors = MONITORS.read().await;
        let monitor = find_monitor(&monitors, &monitor_query)
            .ok_or_else(|| anyhow::anyhow!("Monitor not found: {}", monitor_query))?;

        let target_brightness = parse_brightness_value(&value, monitor.brightness)?;
        set_brightness(&monitor, target_brightness).await?;

        // Update cached brightness
        let mut monitors = MONITORS.write().await;
        if let Some(m) = monitors.get_mut(&monitor.name) {
            m.brightness = target_brightness;
        }

        Ok(serde_json::json!({
            "success": true,
            "monitor": monitor.name,
            "brightness": target_brightness
        }))
    });
}

fn find_monitor<'a>(monitors: &'a HashMap<String, Monitor>, query: &str) -> Option<&'a Monitor> {
    if query == "active" {
        // Return first monitor for now (could be enhanced to detect focused monitor)
        monitors.values().next()
    } else {
        monitors.get(query)
    }
}

pub fn parse_brightness_value(value: &str, current: f64) -> Result<f64> {
    let value = value.trim();
    
    if value.ends_with("%-") {
        let percent = value[..value.len() - 2].parse::<f64>()? / 100.0;
        Ok((current - percent).max(0.0).min(1.0))
    } else if value.starts_with("+") && value.ends_with("%") {
        let percent = value[1..value.len() - 1].parse::<f64>()? / 100.0;
        Ok((current + percent).max(0.0).min(1.0))
    } else if value.ends_with("%") {
        let percent = value[..value.len() - 1].parse::<f64>()? / 100.0;
        Ok(percent.max(0.0).min(1.0))
    } else if value.starts_with("+") {
        let increment = value[1..].parse::<f64>()?;
        Ok((current + increment).max(0.0).min(1.0))
    } else if value.ends_with("-") {
        let decrement = value[..value.len() - 1].parse::<f64>()?;
        Ok((current - decrement).max(0.0).min(1.0))
    } else {
        let absolute = value.parse::<f64>()?;
        Ok(absolute.max(0.0).min(1.0))
    }
}

async fn set_brightness(monitor: &Monitor, brightness: f64) -> Result<()> {
    let rounded = (brightness * 100.0).round() as u32;

    match &monitor.monitor_type {
        MonitorType::AppleDisplay => {
            if *APPLE_DISPLAY_PRESENT.read().await {
                process::exec_command_detached(&["asdbctl", "set", &rounded.to_string()]).await?;
            }
        }
        MonitorType::DDC { bus_num } => {
            process::exec_command_detached(&[
                "ddcutil",
                "-b",
                bus_num,
                "setvcp",
                "10",
                &rounded.to_string(),
            ])
            .await?;
        }
        MonitorType::Default => {
            process::exec_command_detached(&["brightnessctl", "s", &format!("{}%", rounded)]).await?;
        }
    }

    Ok(())
}

async fn detect_monitors() -> Result<()> {
    let mut monitors = HashMap::new();

    // Check for Apple Display
    let apple_check = process::exec_command(&["asdbctl", "get"]).await;
    let apple_present = apple_check.is_ok();
    *APPLE_DISPLAY_PRESENT.write().await = apple_present;

    // Detect DDC monitors
    let ddc_output = process::exec_command(&["ddcutil", "detect", "--brief"]).await.ok();
    let ddc_monitors = if let Some(output) = ddc_output {
        parse_ddc_monitors(&output)
    } else {
        Vec::new()
    };

    // For now, create a default monitor
    // In a full implementation, we'd detect actual monitors from the system
    let default_monitor = Monitor {
        name: "default".to_string(),
        monitor_type: MonitorType::Default,
        brightness: 0.5,
    };
    monitors.insert("default".to_string(), default_monitor);

    // Add DDC monitors
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

    *MONITORS.write().await = monitors;
    Ok(())
}

fn parse_ddc_monitors(output: &str) -> Vec<(String, String)> {
    let mut monitors = Vec::new();
    let mut current_bus: Option<String> = None;
    let mut current_connector: Option<String> = None;

    let mut flush = |monitors: &mut Vec<(String, String)>,
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
}

async fn get_ddc_brightness(bus_num: &str) -> Result<f64> {
    let output = process::exec_command(&["ddcutil", "-b", bus_num, "getvcp", "10", "--brief"]).await?;
    // Parse output like "VCP 10 C 50 100" where 50 is current, 100 is max
    if let Some((current, max)) = output
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect::<Vec<_>>()
        .get(0..2)
        .and_then(|v| Some((v[0], v[1])))
    {
        if max > 0 {
            return Ok(current as f64 / max as f64);
        }
    }
    Ok(0.5)
}
