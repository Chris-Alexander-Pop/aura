use crate::services::ServiceRegistry;
use crate::types::AccessPoint;
use crate::utils::process;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct NetworkState {
    wifi_enabled: bool,
    active_connection: Option<String>,
    local_ip: Option<String>,
    public_ip: Option<String>,
    networks: Vec<AccessPoint>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            active_connection: None,
            local_ip: None,
            public_ip: None,
            networks: Vec::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: RwLock<NetworkState> = RwLock::new(NetworkState::default());
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start monitoring tasks
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Err(e) = update_network_state().await {
                tracing::debug!("Failed to update network state: {}", e);
            }
        }
    });

    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = update_public_ip().await {
                tracing::debug!("Failed to update public IP: {}", e);
            }
        }
    });

    registry.register("Network.ToggleWifi", |params| async move {
        let enabled: bool = serde_json::from_value(
            params
                .and_then(|p| p.get("enabled").cloned())
                .unwrap_or(serde_json::Value::Bool(true)),
        )?;

        let cmd = if enabled { "on" } else { "off" };
        process::exec_command(&["nmcli", "radio", "wifi", cmd]).await?;

        let mut state = STATE.write().await;
        state.wifi_enabled = enabled;

        Ok(serde_json::json!({ "success": true, "wifi_enabled": enabled }))
    });

    registry.register("Network.ScanNetworks", |_params| async move {
        // Trigger rescan
        let _ = process::exec_command(&["nmcli", "dev", "wifi", "rescan"]).await;

        // Get networks
        let output = process::exec_command(&[
            "nmcli",
            "-g",
            "ACTIVE,SIGNAL,FREQ,SSID,BSSID,SECURITY",
            "d",
            "w",
        ])
        .await?;

        let networks = parse_networks(&output);
        let mut state = STATE.write().await;
        state.networks = networks.clone();

        Ok(serde_json::to_value(networks)?)
    });

    registry.register("Network.Connect", |params| async move {
        let params_value = params.clone();
        let ssid: String = serde_json::from_value(
            params_value
                .and_then(|p| p.get("ssid").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing ssid parameter"))?,
        )?;

        let password: Option<String> = params
            .and_then(|p| p.get("password").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        if let Some(pass) = password {
            process::exec_command(&[
                "nmcli",
                "dev",
                "wifi",
                "connect",
                &ssid,
                "password",
                &pass,
            ])
            .await?;
        } else {
            process::exec_command(&["nmcli", "conn", "up", &ssid]).await?;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Network.Disconnect", |_params| async move {
        let state = STATE.read().await;
        if let Some(ref conn) = state.active_connection {
            process::exec_command(&["nmcli", "connection", "down", conn]).await?;
        }
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Network.GetStatus", |_params| async move {
        let state = STATE.read().await;
        Ok(serde_json::json!({
            "wifi_enabled": state.wifi_enabled,
            "active_connection": state.active_connection,
            "local_ip": state.local_ip,
            "public_ip": state.public_ip,
        }))
    });
}

fn parse_networks(output: &str) -> Vec<AccessPoint> {
    let mut network_map: HashMap<String, AccessPoint> = HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 6 {
            continue;
        }

        let active = parts[0] == "yes";
        let strength: i32 = parts[1].parse().unwrap_or(0);
        let frequency: i32 = parts[2].parse().unwrap_or(0);
        let ssid = parts[3].to_string();
        let bssid = parts[4].to_string();
        let security = parts[5].to_string();

        // Group by SSID, prioritize active connections
        let entry = network_map.entry(ssid.clone()).or_insert_with(|| AccessPoint {
            ssid: ssid.clone(),
            bssid: bssid.clone(),
            strength,
            frequency,
            active,
            security: security.clone(),
        });

        // Update if this is active or has better signal
        if active && !entry.active {
            *entry = AccessPoint {
                ssid,
                bssid,
                strength,
                frequency,
                active: true,
                security,
            };
        } else if !entry.active && !active && strength > entry.strength {
            entry.strength = strength;
            entry.bssid = bssid;
            entry.frequency = frequency;
            entry.security = security;
        }
    }

    network_map.into_values().collect()
}

async fn update_network_state() -> Result<()> {
    // Check WiFi status
    let wifi_status = process::exec_command(&["nmcli", "radio", "wifi"]).await?;
    let wifi_enabled = wifi_status.trim() == "enabled";

    // Get active connection
    let active_output = process::exec_command(&["nmcli", "-t", "-f", "NAME,DEVICE", "c", "show", "--active"]).await?;
    let active_connection = active_output
        .lines()
        .next()
        .and_then(|line| line.split(':').next().map(|s| s.to_string()));

    // Get local IP
    let local_ip_output = process::exec_command(&[
        "sh",
        "-c",
        "ip -4 addr show wlan0 2>/dev/null | grep -oP '(?<=inet\\s)\\d+(\\.\\d+){3}' || ip -4 addr show | grep -oP '(?<=inet\\s)\\d+(\\.\\d+){3}' | head -1",
    ])
    .await
    .ok();
    let local_ip = local_ip_output.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    let mut state = STATE.write().await;
    state.wifi_enabled = wifi_enabled;
    state.active_connection = active_connection;
    state.local_ip = local_ip;

    Ok(())
}

async fn update_public_ip() -> Result<()> {
    let public_ip = reqwest::get("https://ifconfig.me")
        .await?
        .text()
        .await?
        .trim()
        .to_string();

    let mut state = STATE.write().await;
    state.public_ip = Some(public_ip);

    Ok(())
}
