use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothAdapter {
    pub path: String,
    pub name: String,
    pub alias: String,
    pub powered: bool,
    pub discoverable: bool,
    pub pairable: bool,
    pub discovering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

lazy_static::lazy_static! {
    static ref BLUETOOTH_STATE: RwLock<BluetoothState> = RwLock::new(BluetoothState::default());
}

#[derive(Debug, Clone, Default)]
struct BluetoothState {
    adapters: Vec<BluetoothAdapter>,
    devices: Vec<BluetoothDevice>,
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start Bluetooth monitoring
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            refresh_adapters().await.ok();
            refresh_devices().await.ok();
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
        // Start discovery on all adapters
        let state = BLUETOOTH_STATE.read().await;
        for adapter in &state.adapters {
            if adapter.powered {
                let _ = process::exec_command(&[
                    "dbus-send",
                    "--system",
                    "--print-reply",
                    "--dest=org.bluez",
                    &adapter.path,
                    "org.freedesktop.DBus.Properties.Set",
                    "string:org.bluez.Adapter1",
                    "string:Discoverable",
                    "variant:boolean:true",
                ])
                .await;

                let _ = process::exec_command(&[
                    "dbus-send",
                    "--system",
                    "--print-reply",
                    "--dest=org.bluez",
                    &adapter.path,
                    "org.bluez.Adapter1.StartDiscovery",
                ])
                .await;
            }
        }
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.StopScan", |_params| async move {
        // Stop discovery on all adapters
        let state = BLUETOOTH_STATE.read().await;
        for adapter in &state.adapters {
            if adapter.discovering {
                let _ = process::exec_command(&[
                    "dbus-send",
                    "--system",
                    "--print-reply",
                    "--dest=org.bluez",
                    &adapter.path,
                    "org.bluez.Adapter1.StopDiscovery",
                ])
                .await;
            }
        }
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Pair", |params| async move {
        let device_address: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_address").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
        )?;

        // Find device path
        let state = BLUETOOTH_STATE.read().await;
        let device = state
            .devices
            .iter()
            .find(|d| d.address == device_address)
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;

        let device_path = device.path.clone();
        drop(state);

        // Pair device
        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &device_path,
            "org.bluez.Device1.Pair",
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Connect", |params| async move {
        let device_address: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_address").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
        )?;

        // Find device path
        let state = BLUETOOTH_STATE.read().await;
        let device = state
            .devices
            .iter()
            .find(|d| d.address == device_address)
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;

        let device_path = device.path.clone();
        drop(state);

        // Connect device
        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &device_path,
            "org.bluez.Device1.Connect",
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Disconnect", |params| async move {
        let device_address: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_address").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
        )?;

        // Find device path
        let state = BLUETOOTH_STATE.read().await;
        let device = state
            .devices
            .iter()
            .find(|d| d.address == device_address)
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;

        let device_path = device.path.clone();
        drop(state);

        // Disconnect device
        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &device_path,
            "org.bluez.Device1.Disconnect",
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.Remove", |params| async move {
        let device_address: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_address").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
        )?;

        // Find device and adapter
        let state = BLUETOOTH_STATE.read().await;
        let device = state
            .devices
            .iter()
            .find(|d| d.address == device_address)
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;

        let device_path = device.path.clone();
        let adapter_path = device_path
            .rsplit('/')
            .nth(1)
            .map(|s| format!("/{}", s))
            .unwrap_or_default();
        drop(state);

        // Remove device
        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &adapter_path,
            "org.bluez.Adapter1.RemoveDevice",
            &format!("objectpath:\"{}\"", device_path),
        ])
        .await;

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

        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &adapter_path,
            "org.freedesktop.DBus.Properties.Set",
            "string:org.bluez.Adapter1",
            "string:Powered",
            &format!("variant:boolean:{}", powered),
        ])
        .await;

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

        let _ = process::exec_command(&[
            "dbus-send",
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &adapter_path,
            "org.freedesktop.DBus.Properties.Set",
            "string:org.bluez.Adapter1",
            "string:Discoverable",
            &format!("variant:boolean:{}", discoverable),
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Bluetooth.GetDeviceInfo", |params| async move {
        let device_address: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_address").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_address"))?,
        )?;

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

async fn refresh_adapters() -> Result<()> {
    // Use bluetoothctl to list adapters
    let output = process::exec_command(&["bluetoothctl", "list"]).await?;

    let mut adapters = Vec::new();
    for line in output.lines() {
        if line.starts_with("Controller") {
            // Parse: Controller 00:11:22:33:44:55 Name [default]
            if let Some(parts) = line.split_whitespace().nth(1) {
                let address = parts.to_string();
                // Get adapter properties via D-Bus
                if let Ok(adapter) = get_adapter_properties(&address).await {
                    adapters.push(adapter);
                }
            }
        }
    }

    let mut state = BLUETOOTH_STATE.write().await;
    state.adapters = adapters;
    Ok(())
}

async fn get_adapter_properties(_address: &str) -> Result<BluetoothAdapter> {
    // Find adapter path
    let _output = process::exec_command(&[
        "dbus-send",
        "--system",
        "--print-reply",
        "--dest=org.bluez",
        "/",
        "org.freedesktop.DBus.ObjectManager.GetManagedObjects",
    ])
    .await?;

    // Parse output to find adapter path
    let adapter_path = format!("/org/bluez/hci0"); // Simplified, should parse from output

    // Get properties
    let name_output = process::exec_command(&[
        "dbus-send",
        "--system",
        "--print-reply",
        "--dest=org.bluez",
        &adapter_path,
        "org.freedesktop.DBus.Properties.Get",
        "string:org.bluez.Adapter1",
        "string:Name",
    ])
    .await
    .ok()
    .and_then(|o| parse_dbus_variant(&o));

    let alias_output = process::exec_command(&[
        "dbus-send",
        "--system",
        "--print-reply",
        "--dest=org.bluez",
        &adapter_path,
        "org.freedesktop.DBus.Properties.Get",
        "string:org.bluez.Adapter1",
        "string:Alias",
    ])
    .await
    .ok()
    .and_then(|o| parse_dbus_variant(&o));

    let powered_output = process::exec_command(&[
        "dbus-send",
        "--system",
        "--print-reply",
        "--dest=org.bluez",
        &adapter_path,
        "org.freedesktop.DBus.Properties.Get",
        "string:org.bluez.Adapter1",
        "string:Powered",
    ])
    .await
    .ok()
    .and_then(|o| parse_dbus_variant(&o));

    let discoverable_output = process::exec_command(&[
        "dbus-send",
        "--system",
        "--print-reply",
        "--dest=org.bluez",
        &adapter_path,
        "org.freedesktop.DBus.Properties.Get",
        "string:org.bluez.Adapter1",
        "string:Discoverable",
    ])
    .await
    .ok()
    .and_then(|o| parse_dbus_variant(&o));

    Ok(BluetoothAdapter {
        path: adapter_path,
        name: name_output.unwrap_or_else(|| "Unknown".to_string()),
        alias: alias_output.unwrap_or_else(|| "Unknown".to_string()),
        powered: powered_output.and_then(|s| s.parse().ok()).unwrap_or(false),
        discoverable: discoverable_output
            .and_then(|s| s.parse().ok())
            .unwrap_or(false),
        pairable: false, // Would need another D-Bus call
        discovering: false, // Would need another D-Bus call
    })
}

fn parse_dbus_variant(output: &str) -> Option<String> {
    // Simple parser for D-Bus variant output
    // Format: variant string "value"
    for line in output.lines() {
        if line.contains("variant") && line.contains("string") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

async fn refresh_devices() -> Result<()> {
    // Use bluetoothctl to list devices
    let output = process::exec_command(&["bluetoothctl", "devices"]).await?;

    let mut devices = Vec::new();
    for line in output.lines() {
        if line.starts_with("Device") {
            // Parse: Device 00:11:22:33:44:55 Name
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let address = parts[1].to_string();
                let name = parts[2..].join(" ");

                // Get device properties
                if let Ok(device) = get_device_properties(&address, &name).await {
                    devices.push(device);
                }
            }
        }
    }

    let mut state = BLUETOOTH_STATE.write().await;
    state.devices = devices;
    Ok(())
}

async fn get_device_properties(address: &str, name: &str) -> Result<BluetoothDevice> {
    // Get device info via bluetoothctl
    let info_output = process::exec_command(&[
        "bluetoothctl",
        "info",
        address,
    ])
    .await?;

    let mut connected = false;
    let mut paired = false;
    let mut trusted = false;
    let mut rssi: Option<i16> = None;
    let mut battery: Option<u8> = None;
    let mut device_type = String::new();

    for line in info_output.lines() {
        if line.contains("Connected: yes") {
            connected = true;
        }
        if line.contains("Paired: yes") {
            paired = true;
        }
        if line.contains("Trusted: yes") {
            trusted = true;
        }
        if line.contains("RSSI:") {
            if let Some(rssi_str) = line.split_whitespace().nth(1) {
                rssi = rssi_str.parse().ok();
            }
        }
        if line.contains("Battery Percentage:") {
            if let Some(bat_str) = line.split_whitespace().nth(2) {
                battery = bat_str.parse().ok();
            }
        }
        if line.contains("Icon:") {
            if let Some(icon) = line.split_whitespace().nth(1) {
                device_type = icon.to_string();
            }
        }
    }

    Ok(BluetoothDevice {
        address: address.to_string(),
        path: format!("/org/bluez/hci0/dev_{}", address.replace(':', "_")),
        name: name.to_string(),
        alias: name.to_string(),
        connected,
        paired,
        trusted,
        rssi,
        battery_percentage: battery,
        device_type,
        services: Vec::new(), // Would need additional parsing
    })
}
