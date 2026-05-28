use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BluetoothAdapter {
    /// D-Bus-style path for UI compatibility (e.g. `/org/bluez/hci0`).
    pub path: String,
    pub address: String,
    pub name: String,
    pub alias: String,
    pub powered: bool,
    pub discoverable: bool,
    pub pairable: bool,
    pub discovering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BluetoothDevice {
    pub address: String,
    pub path: String,
    pub name: String,
    pub alias: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
    pub rssi: Option<i16>,
    pub battery_percentage: Option<u8>,
    pub device_type: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct BluetoothState {
    adapters: Vec<BluetoothAdapter>,
    devices: Vec<BluetoothDevice>,
}

lazy_static::lazy_static! {
    static ref BLUETOOTH_STATE: RwLock<BluetoothState> = RwLock::new(BluetoothState::default());
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(2));
        loop {
            tick.tick().await;
            let _ = refresh_all(false).await;
        }
    });

    registry.register("Bluetooth.GetAdapters", |_params| async move {
        let state = BLUETOOTH_STATE.read().await;
        Ok(serde_json::to_value(&state.adapters)?)
    });

    registry.register("Bluetooth.GetDevices", |_params| async move {
        let state = BLUETOOTH_STATE.read().await;
        Ok(serde_json::to_value(&state.devices)?)
    });

    registry.register("Bluetooth.Scan", |_params| async move {
        let macs = default_adapter_macs().await;
        let empty = macs.is_empty();
        for mac in &macs {
            let _ = bluetoothctl_on_adapter(mac, &["scan", "on"]).await;
        }
        if empty {
            let _ = process::exec_command(&["bluetoothctl", "--timeout", "5", "scan", "on"]).await;
        }
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.StopScan", |_params| async move {
        let macs = default_adapter_macs().await;
        for mac in macs {
            let _ = bluetoothctl_on_adapter(&mac, &["scan", "off"]).await;
        }
        let _ = process::exec_command(&["bluetoothctl", "scan", "off"]).await;
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Pair", |params| async move {
        let device_address = device_address_from_params(params)?;
        run_device_cmd(&device_address, &["pair"]).await?;
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Connect", |params| async move {
        let device_address = device_address_from_params(params)?;
        run_device_cmd(&device_address, &["connect"]).await?;
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Disconnect", |params| async move {
        let device_address = device_address_from_params(params)?;
        run_device_cmd(&device_address, &["disconnect"]).await?;
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Remove", |params| async move {
        let device_address = device_address_from_params(params)?;
        run_device_cmd(&device_address, &["remove"]).await?;
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.SetAdapterPower", |params| async move {
        let adapter_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("adapter_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing adapter_path"))?,
        )?;
        let powered: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("powered").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing powered"))?,
        )?;

        let mac = resolve_adapter_mac(&adapter_path).await?;
        let cmd = if powered { "on" } else { "off" };
        if let Some(ref m) = mac {
            bluetoothctl_on_adapter(m, &["power", cmd]).await?;
        } else {
            process::exec_command(&["bluetoothctl", "power", cmd]).await?;
        }
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.SetAdapterDiscoverable", |params| async move {
        let adapter_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("adapter_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing adapter_path"))?,
        )?;
        let discoverable: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("discoverable").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing discoverable"))?,
        )?;

        let mac = resolve_adapter_mac(&adapter_path).await?;
        if discoverable {
            if let Some(ref m) = mac {
                bluetoothctl_on_adapter(m, &["discoverable", "on"]).await?;
            } else {
                process::exec_command(&["bluetoothctl", "discoverable", "on"]).await?;
            }
        } else if let Some(ref m) = mac {
            bluetoothctl_on_adapter(m, &["discoverable", "off"]).await?;
        } else {
            process::exec_command(&["bluetoothctl", "discoverable", "off"]).await?;
        }
        refresh_all(true).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.GetDeviceInfo", |params| async move {
        let device_address = device_address_from_params(params)?;
        if let Ok(device) = fetch_device_info(&device_address).await {
            return Ok(serde_json::to_value(&device)?);
        }
        let state = BLUETOOTH_STATE.read().await;
        let device = state
            .devices
            .iter()
            .find(|d| d.address == device_address)
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?
            .clone();
        Ok(serde_json::to_value(&device)?)
    });
}

fn device_address_from_params(
    params: Option<serde_json::Value>,
) -> Result<String> {
    Ok(serde_json::from_value(
        params
            .and_then(|p| p.get("device_address").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
    )?)
}

async fn run_device_cmd(address: &str, args: &[&str]) -> Result<()> {
    let mut cmd = vec!["bluetoothctl"];
    cmd.push(address);
    cmd.extend_from_slice(args);
    process::exec_command(&cmd).await?;
    Ok(())
}

async fn bluetoothctl_on_adapter(mac: &str, args: &[&str]) -> Result<()> {
    let mut cmd = vec!["bluetoothctl", "-a", mac];
    cmd.extend_from_slice(args);
    process::exec_command(&cmd).await?;
    Ok(())
}

async fn default_adapter_macs() -> Vec<String> {
    BLUETOOTH_STATE
        .read()
        .await
        .adapters
        .iter()
        .map(|a| a.address.clone())
        .collect()
}

async fn resolve_adapter_mac(adapter_path: &str) -> Result<Option<String>> {
    if adapter_path.contains(':') && adapter_path.len() >= 17 {
        return Ok(Some(adapter_path.to_string()));
    }
    let state = BLUETOOTH_STATE.read().await;
    Ok(state
        .adapters
        .iter()
        .find(|a| a.path == adapter_path)
        .map(|a| a.address.clone()))
}

async fn refresh_all(force_emit: bool) -> Result<()> {
    let adapters = fetch_adapters().await.unwrap_or_default();
    let devices = fetch_devices().await.unwrap_or_default();

    let mut state = BLUETOOTH_STATE.write().await;
    let changed = state.adapters != adapters || state.devices != devices;
    state.adapters = adapters;
    state.devices = devices.clone();

    if changed || force_emit {
        notify::emit(
            "Bluetooth.StateChanged",
            serde_json::json!({
                "adapters": state.adapters,
                "devices": state.devices,
            }),
        );
    }
    Ok(())
}

async fn fetch_adapters() -> Result<Vec<BluetoothAdapter>> {
    let output = process::exec_command(&["bluetoothctl", "list"]).await?;
    let mut adapters = Vec::new();
    let mut idx = 0u32;

    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with("Controller ") {
            continue;
        }
        let rest = line.strip_prefix("Controller ").unwrap_or(line);
        let mut parts = rest.split_whitespace();
        let address = parts.next().unwrap_or("").to_string();
        if address.is_empty() || !address.contains(':') {
            continue;
        }
        let name = parts.collect::<Vec<_>>().join(" ");
        let name = name.trim_end_matches("[default]").trim().to_string();

        let details = fetch_adapter_show(&address).await;
        let path = format!("/org/bluez/hci{idx}");
        idx += 1;

        adapters.push(BluetoothAdapter {
            path,
            address: address.clone(),
            name: details.name.unwrap_or_else(|| name.clone()),
            alias: details.alias.unwrap_or(name),
            powered: details.powered.unwrap_or(false),
            discoverable: details.discoverable.unwrap_or(false),
            pairable: details.pairable.unwrap_or(true),
            discovering: details.discovering.unwrap_or(false),
        });
    }

    Ok(adapters)
}

struct AdapterShowDetails {
    name: Option<String>,
    alias: Option<String>,
    powered: Option<bool>,
    discoverable: Option<bool>,
    pairable: Option<bool>,
    discovering: Option<bool>,
}

async fn fetch_adapter_show(address: &str) -> AdapterShowDetails {
    let output = process::exec_command(&["bluetoothctl", "show", address])
        .await
        .unwrap_or_default();
    parse_show_block(&output)
}

fn parse_show_block(output: &str) -> AdapterShowDetails {
    let mut details = AdapterShowDetails {
        name: None,
        alias: None,
        powered: None,
        discoverable: None,
        pairable: None,
        discovering: None,
    };

    for line in output.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Name:") {
            details.name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("Alias:") {
            details.alias = Some(v.trim().to_string());
        } else if line.starts_with("Powered:") {
            details.powered = Some(line.contains("yes"));
        } else if line.starts_with("Discoverable:") {
            details.discoverable = Some(line.contains("yes"));
        } else if line.starts_with("Pairable:") {
            details.pairable = Some(line.contains("yes"));
        } else if line.starts_with("Discovering:") {
            details.discovering = Some(line.contains("yes"));
        }
    }
    details
}

async fn fetch_devices() -> Result<Vec<BluetoothDevice>> {
    let output = process::exec_command(&["bluetoothctl", "devices"]).await?;
    let mut devices = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with("Device ") {
            continue;
        }
        let rest = line.strip_prefix("Device ").unwrap_or(line);
        let mut parts = rest.splitn(2, ' ');
        let address = parts.next().unwrap_or("").to_string();
        if !address.contains(':') {
            continue;
        }
        let name = parts.next().unwrap_or("").trim().to_string();
        if let Ok(device) = fetch_device_info(&address).await {
            devices.push(device);
        } else {
            devices.push(BluetoothDevice {
                path: device_path_for_address(&address),
                address,
                name: name.clone(),
                alias: name,
                connected: false,
                paired: false,
                trusted: false,
                rssi: None,
                battery_percentage: None,
                device_type: String::new(),
                services: Vec::new(),
            });
        }
    }

    Ok(devices)
}

async fn fetch_device_info(address: &str) -> Result<BluetoothDevice> {
    let output = process::exec_command(&["bluetoothctl", "info", address]).await?;
    Ok(parse_device_info(address, &output))
}

pub(crate) fn parse_device_info(address: &str, output: &str) -> BluetoothDevice {
    let mut name = address.to_string();
    let mut alias = String::new();
    let mut connected = false;
    let mut paired = false;
    let mut trusted = false;
    let mut rssi: Option<i16> = None;
    let mut battery: Option<u8> = None;
    let mut device_type = String::new();

    for line in output.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Name:") {
            name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Alias:") {
            alias = v.trim().to_string();
        } else if line.starts_with("Connected:") {
            connected = line.contains("yes");
        } else if line.starts_with("Paired:") {
            paired = line.contains("yes");
        } else if line.starts_with("Trusted:") {
            trusted = line.contains("yes");
        } else if let Some(v) = line.strip_prefix("RSSI:") {
            rssi = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("Battery Percentage:") {
            let t = v.trim();
            if let Some(open) = t.rfind('(') {
                if let Some(close) = t.rfind(')') {
                    battery = t[open + 1..close].trim().parse().ok();
                }
            }
            if battery.is_none() {
                let num = t.trim_end_matches('%');
                battery = num.parse().ok();
            }
        } else if let Some(v) = line.strip_prefix("Icon:") {
            device_type = v.trim().to_string();
        }
    }

    if alias.is_empty() {
        alias = name.clone();
    }

    BluetoothDevice {
        address: address.to_string(),
        path: device_path_for_address(address),
        name,
        alias,
        connected,
        paired,
        trusted,
        rssi,
        battery_percentage: battery,
        device_type,
        services: Vec::new(),
    }
}

fn device_path_for_address(address: &str) -> String {
    format!("/org/bluez/hci0/dev_{}", address.replace(':', "_"))
}

pub(crate) fn parse_controller_list_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with("Controller ") {
        return None;
    }
    let rest = line.strip_prefix("Controller ")?;
    let mut parts = rest.split_whitespace();
    let address = parts.next()?.to_string();
    let name = parts.collect::<Vec<_>>().join(" ");
    let name = name.trim_end_matches("[default]").trim().to_string();
    Some((address, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_controller_list_fixture() {
        let fixture = include_str!("../../tests/fixtures/bluetooth/bluetoothctl_list.txt");
        let mut count = 0;
        for line in fixture.lines() {
            if let Some((addr, name)) = parse_controller_list_line(line) {
                count += 1;
                assert!(addr.contains(':'));
                assert!(!name.is_empty());
            }
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn parse_device_info_fixture() {
        let fixture = include_str!("../../tests/fixtures/bluetooth/bluetoothctl_info.txt");
        let dev = parse_device_info("AA:BB:CC:DD:EE:FF", fixture);
        assert_eq!(dev.name, "Test Headphones");
        assert!(dev.paired);
        assert!(dev.connected);
        assert_eq!(dev.battery_percentage, Some(85));
    }
}
