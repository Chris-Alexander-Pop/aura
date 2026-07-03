//! NetworkManager Wi‑Fi via `nmcli`.
//!
//! **Deferred:** captive portal detection (`Network.GetCaptivePortal` / HTTP 204 probe) and
//! 802.1X enterprise join — use NetworkManager profiles or the control-panel roadmap.

use crate::notify;
use crate::services::ServiceRegistry;
use crate::types::{AccessPoint, NetworkStatus};
use crate::utils::{keyring, process};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct NetworkState {
    status: NetworkStatus,
    networks: Vec<AccessPoint>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            status: NetworkStatus {
                wifi_enabled: true,
                active_connection: None,
                active_ssid: None,
                connection_type: "none".to_string(),
                ethernet_connected: false,
                local_ip: None,
                public_ip: None,
            },
            networks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNetwork {
    pub name: String,
    pub uuid: String,
    pub autoconnect: bool,
}

lazy_static::lazy_static! {
    static ref STATE: RwLock<NetworkState> = RwLock::new(NetworkState::default());
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(10));
        loop {
            tick.tick().await;
            if let Err(e) = update_network_state().await {
                tracing::debug!("Failed to update network state: {}", e);
            }
        }
    });

    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
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

        update_network_state().await?;
        let _ = emit_network_state().await;
        Ok(serde_json::json!({ "success": true, "wifi_enabled": enabled }))
    });

    registry.register("Network.ScanNetworks", |_params| async move {
        let _ = process::exec_command(&["nmcli", "dev", "wifi", "rescan"]).await;

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

        let password = match password {
            Some(p) => {
                if !p.is_empty() {
                    if let Err(e) = keyring::store_wifi_password(&ssid, &p).await {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": format!(
                                "Could not save Wi‑Fi password to keyring: {e}. Unlock your login keyring and try again."
                            ),
                        }));
                    }
                }
                Some(p)
            }
            None => keyring::lookup_wifi_password(&ssid).await?,
        };

        if let Some(sec) = scan_security_for_ssid(&ssid).await {
            if is_enterprise_security(&sec) {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": map_nmcli_connect_error(
                        "802.1X enterprise Wi‑Fi is not supported yet; configure the profile in NetworkManager",
                    ),
                }));
            }
        }

        let result = match password.as_deref() {
            Some(pass) if !pass.is_empty() => connect_wifi_with_password(&ssid, pass).await,
            _ => connect_open_or_saved(&ssid).await,
        };

        match result {
            Ok(()) => {
                update_network_state().await.ok();
                let _ = emit_network_state().await;
                Ok(serde_json::json!({ "success": true }))
            }
            Err(e) => Ok(serde_json::json!({
                "success": false,
                "error": map_nmcli_connect_error(&e.to_string()),
            })),
        }
    });

    registry.register("Network.Disconnect", |_params| async move {
        let conn = {
            let state = STATE.read().await;
            state.status.active_connection.clone()
        };
        if let Some(ref c) = conn {
            process::exec_command(&["nmcli", "connection", "down", c]).await?;
        }
        update_network_state().await?;
        let _ = emit_network_state().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Network.GetStatus", |_params| async move {
        Ok(serde_json::to_value(snapshot_network_status().await)?)
    });

    registry.register("Network.ListSaved", |_params| async move {
        let output = process::exec_command(&[
            "nmcli",
            "-t",
            "-f",
            "NAME,UUID,TYPE,AUTOCONNECT",
            "connection",
            "show",
        ])
        .await
        .unwrap_or_default();

        let saved = parse_saved_connections(&output);
        Ok(serde_json::to_value(saved)?)
    });

    registry.register("Network.Forget", |params| async move {
        let uuid: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("uuid").cloned())
            .and_then(|v| serde_json::from_value(v).ok());
        let name: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("name").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let target = uuid.or(name).ok_or_else(|| anyhow::anyhow!("Missing uuid or name"))?;
        process::exec_command(&["nmcli", "connection", "delete", &target]).await?;
        Ok(serde_json::json!({ "success": true }))
    });
}

pub(crate) async fn snapshot_network_status() -> NetworkStatus {
    let state = STATE.read().await;
    state.status.clone()
}

async fn scan_security_for_ssid(ssid: &str) -> Option<String> {
    let state = STATE.read().await;
    state
        .networks
        .iter()
        .find(|n| n.ssid == ssid)
        .map(|n| n.security.clone())
}

/// Map raw `nmcli` / process errors to short UI-facing messages.
pub fn map_nmcli_connect_error(raw: &str) -> String {
    let msg = extract_nmcli_error_message(raw);
    let lower = msg.to_lowercase();
    if lower.contains("802.1x") || lower.contains("802-1x") || lower.contains("enterprise") {
        return "802.1X enterprise Wi‑Fi is not supported yet; configure the profile in NetworkManager"
            .to_string();
    }
    if lower.contains("no network with ssid") {
        return "Wi‑Fi network not in range — scan again".to_string();
    }
    if lower.contains("secrets were required") {
        return "Password required for this network".to_string();
    }
    if lower.contains("passwords or encryption keys")
        || lower.contains("wrong password")
        || lower.contains("invalid secrets")
    {
        return "Incorrect Wi‑Fi password".to_string();
    }
    if lower.contains("wifi is disabled") || lower.contains("radio wifi is disabled") {
        return "Wi‑Fi is turned off — enable Wi‑Fi and try again".to_string();
    }
    if lower.contains("device") && lower.contains("not ready") {
        return "Wi‑Fi adapter is not ready — try again in a moment".to_string();
    }
    if msg.is_empty() {
        "NetworkManager rejected the connection".to_string()
    } else {
        msg
    }
}

fn extract_nmcli_error_message(raw: &str) -> String {
    let trimmed = raw.trim();
    let after_cmd = trimmed
        .strip_prefix("Command failed: nmcli - ")
        .or_else(|| trimmed.strip_prefix("Command failed: nmcli "))
        .unwrap_or(trimmed);
    after_cmd
        .trim()
        .trim_start_matches("Error: ")
        .to_string()
}

/// True when scan security string indicates an open / OWE transition network.
pub fn is_open_security(security: &str) -> bool {
    let s = security.trim();
    s.is_empty() || s == "--" || s.eq_ignore_ascii_case("open") || s.contains("OWE")
}

/// True for 802.1X / WPA-Enterprise style networks (not joinable via password-only RPC).
pub fn is_enterprise_security(security: &str) -> bool {
    let s = security.to_uppercase();
    s.contains("802.1X")
        || s.contains("802-1X")
        || s.contains("EAP")
        || s.contains("ENTERPRISE")
        || s.contains("WPA-EAP")
}

/// WPA3-SAE appears in nmcli SECURITY column; join still uses `password` with NetworkManager.
pub fn is_wpa3_security(security: &str) -> bool {
    let s = security.to_uppercase();
    s.contains("WPA3") || s.contains("SAE")
}

/// Typical success stdout from `nmcli dev wifi connect` / `connection up`.
pub fn nmcli_connect_output_success(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("successfully activated") || lower.contains("connection successfully activated")
}

async fn connect_wifi_with_password(ssid: &str, pass: &str) -> Result<()> {
    let security = scan_security_for_ssid(ssid).await;
    if security.as_ref().is_some_and(|s| is_open_security(s)) {
        return connect_open_or_saved(ssid).await;
    }

    let primary = [
        "nmcli",
        "dev",
        "wifi",
        "connect",
        ssid,
        "password",
        pass,
    ];
    match process::exec_command(&primary).await {
        Ok(out) if nmcli_connect_output_success(&out) || out.is_empty() => return Ok(()),
        Ok(_) => return Ok(()),
        Err(primary_err) => {
            let mut last_err = primary_err.to_string();

            if security.as_ref().is_some_and(|s| is_wpa3_security(s)) {
                let fallback = [
                    "nmcli",
                    "dev",
                    "wifi",
                    "connect",
                    ssid,
                    "password",
                    pass,
                    "key-mgmt",
                    "wpa-psk",
                ];
                match process::exec_command(&fallback).await {
                    Ok(out) if nmcli_connect_output_success(&out) || out.is_empty() => return Ok(()),
                    Ok(_) => return Ok(()),
                    Err(e) => last_err = e.to_string(),
                }
            }

            if connect_open_or_saved(ssid).await.is_ok() {
                return Ok(());
            }

            bail!(map_nmcli_connect_error(&last_err))
        }
    }
}

async fn connect_open_or_saved(ssid: &str) -> Result<()> {
    if let Ok(out) = process::exec_command(&["nmcli", "dev", "wifi", "connect", ssid]).await {
        if nmcli_connect_output_success(&out) || out.is_empty() {
            return Ok(());
        }
    }
    process::exec_command(&["nmcli", "connection", "up", ssid]).await?;
    Ok(())
}

pub fn parse_saved_connections(output: &str) -> Vec<SavedNetwork> {
    let mut list = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 {
            continue;
        }
        let conn_type = parts[2].trim();
        if conn_type != "802-11-wireless" && conn_type != "wifi" {
            continue;
        }
        let autoconnect = parts[3].trim().eq_ignore_ascii_case("yes");
        list.push(SavedNetwork {
            name: parts[0].to_string(),
            uuid: parts[1].to_string(),
            autoconnect,
        });
    }
    list
}

pub fn parse_networks(output: &str) -> Vec<AccessPoint> {
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
        if ssid.is_empty() || ssid == "--" {
            continue;
        }
        let bssid = parts[4].to_string();
        let security = parts[5].to_string();

        let entry = network_map.entry(ssid.clone()).or_insert_with(|| AccessPoint {
            ssid: ssid.clone(),
            bssid: bssid.clone(),
            strength,
            frequency,
            active,
            security: security.clone(),
        });

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

pub fn parse_wifi_radio_enabled(output: &str) -> bool {
    output.trim() == "enabled"
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveConnectionInfo {
    pub name: String,
    pub device: String,
    pub connection_type: String,
    pub ethernet_on_link: bool,
}

pub fn parse_first_active_connection(output: &str) -> Option<ActiveConnectionInfo> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 3 {
            continue;
        }
        let ctype = parts[2].trim();
        let (connection_type, ethernet_on_link) = connection_type_from_nmcli(ctype);
        return Some(ActiveConnectionInfo {
            name: parts[0].to_string(),
            device: parts[1].to_string(),
            connection_type,
            ethernet_on_link,
        });
    }
    None
}

pub fn connection_type_from_nmcli(ctype: &str) -> (String, bool) {
    if ctype.contains("wireless") || ctype == "802-11-wireless" {
        ("wifi".to_string(), false)
    } else if ctype.contains("ethernet") || ctype == "802-3-ethernet" {
        ("ethernet".to_string(), true)
    } else {
        ("none".to_string(), false)
    }
}

pub fn parse_active_wifi_ssid(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let p: Vec<&str> = line.split(':').collect();
        if p.len() >= 2 && p[0] == "yes" && p[1] != "--" && !p[1].is_empty() {
            Some(p[1].to_string())
        } else {
            None
        }
    })
}

pub fn parse_ip4_first_address(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let ip = line.split('/').next()?.trim();
        let ip = ip.rsplit(':').next().unwrap_or(ip).trim();
        if ip.is_empty() {
            None
        } else {
            Some(ip.to_string())
        }
    })
}

/// Accept only a single plain-text IP from a public-IP lookup response.
pub fn parse_public_ip_response(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.len() > 45 {
        return None;
    }
    if trimmed.contains('<') || trimmed.contains('\n') {
        return None;
    }
    trimmed
        .parse::<std::net::IpAddr>()
        .ok()
        .map(|ip| ip.to_string())
}

/// Compose [`NetworkStatus`] from captured `nmcli` outputs (contract / unit tests).
pub fn build_network_status_from_nmcli(
    wifi_radio: &str,
    active_connections: &str,
    wifi_devices: &str,
    device_ip: Option<&str>,
    device_list: &str,
    public_ip: Option<String>,
) -> NetworkStatus {
    let wifi_enabled = parse_wifi_radio_enabled(wifi_radio);
    let active = parse_first_active_connection(active_connections);
    let active_connection = active.as_ref().map(|a| a.name.clone());
    let mut connection_type = active
        .as_ref()
        .map(|a| a.connection_type.clone())
        .unwrap_or_else(|| "none".to_string());
    let mut ethernet_connected = active.as_ref().map(|a| a.ethernet_on_link).unwrap_or(false);

    let active_ssid = if connection_type == "wifi" {
        parse_active_wifi_ssid(wifi_devices)
    } else {
        None
    };

    let local_ip = device_ip.and_then(parse_ip4_first_address);
    merge_ethernet_from_device_list(device_list, &mut connection_type, &mut ethernet_connected);

    NetworkStatus {
        wifi_enabled,
        active_connection,
        active_ssid,
        connection_type,
        ethernet_connected,
        local_ip,
        public_ip,
    }
}

pub fn merge_ethernet_from_device_list(
    device_list: &str,
    connection_type: &mut String,
    ethernet_connected: &mut bool,
) {
    for line in device_list.lines() {
        let p: Vec<&str> = line.split(':').collect();
        if p.len() >= 3
            && (p[1].contains("ethernet") || p[1] == "802-3-ethernet")
            && p[2] == "connected"
        {
            *ethernet_connected = true;
            if connection_type == "none" {
                *connection_type = "ethernet".to_string();
            }
        }
    }
}

async fn update_network_state() -> Result<()> {
    let wifi_status = process::exec_command(&["nmcli", "radio", "wifi"]).await?;
    let wifi_enabled = parse_wifi_radio_enabled(&wifi_status);

    let active_output =
        process::exec_command(&["nmcli", "-t", "-f", "NAME,DEVICE,TYPE", "c", "show", "--active"])
            .await
            .unwrap_or_default();

    let active = parse_first_active_connection(&active_output);
    let active_connection = active.as_ref().map(|a| a.name.clone());
    let active_device = active.as_ref().map(|a| a.device.clone());
    let mut connection_type = active
        .as_ref()
        .map(|a| a.connection_type.clone())
        .unwrap_or_else(|| "none".to_string());
    let mut ethernet_connected = active.as_ref().map(|a| a.ethernet_on_link).unwrap_or(false);

    let active_ssid = if connection_type == "wifi" {
        process::exec_command(&["nmcli", "-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
            .await
            .ok()
            .and_then(|out| parse_active_wifi_ssid(&out))
    } else {
        None
    };

    let local_ip = if let Some(ref dev) = active_device {
        process::exec_command(&[
            "nmcli",
            "-t",
            "-f",
            "IP4.ADDRESS",
            "device",
            "show",
            dev,
        ])
        .await
        .ok()
        .and_then(|out| parse_ip4_first_address(&out))
    } else {
        None
    };

    let device_status = process::exec_command(&["nmcli", "-t", "-f", "DEVICE,TYPE,STATE", "device"])
        .await
        .unwrap_or_default();
    merge_ethernet_from_device_list(&device_status, &mut connection_type, &mut ethernet_connected);

    let public_ip = STATE.read().await.status.public_ip.clone();

    let new_status = NetworkStatus {
        wifi_enabled,
        active_connection,
        active_ssid,
        connection_type,
        ethernet_connected,
        local_ip,
        public_ip,
    };

    let mut state = STATE.write().await;
    let prev = state.status.clone();
    state.status = new_status.clone();

    if prev != new_status {
        notify::emit("Network.StateChanged", serde_json::to_value(&new_status)?);
    }

    Ok(())
}

async fn update_public_ip() -> Result<()> {
    let body = reqwest::Client::new()
        .get("https://ifconfig.me/ip")
        .header(reqwest::header::ACCEPT, "text/plain")
        .send()
        .await?
        .text()
        .await?;

    let Some(public_ip) = parse_public_ip_response(&body) else {
        tracing::debug!(body_len = body.len(), "public IP lookup returned non-IP body");
        return Ok(());
    };

    let mut state = STATE.write().await;
    if state.status.public_ip.as_deref() != Some(&public_ip) {
        state.status.public_ip = Some(public_ip.clone());
        let status = state.status.clone();
        drop(state);
        notify::emit("Network.StateChanged", serde_json::to_value(&status)?);
    }

    Ok(())
}

async fn emit_network_state() -> Result<()> {
    let state = STATE.read().await;
    notify::emit(
        "Network.StateChanged",
        serde_json::to_value(&state.status)?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_networks_fixture() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_wifi_list.txt");
        let networks = parse_networks(fixture);
        assert!(!networks.is_empty());
        assert!(networks.iter().any(|n| n.ssid == "TestNet"));
        assert!(!networks.iter().any(|n| n.ssid == "--"));
    }

    #[test]
    fn parse_networks_prefers_active_and_stronger_signal() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_wifi_duplicates.txt");
        let networks = parse_networks(fixture);
        let test_net = networks
            .iter()
            .find(|n| n.ssid == "TestNet")
            .expect("TestNet");
        assert!(test_net.active);
        assert_eq!(test_net.strength, 90);

        let weak_net = networks
            .iter()
            .find(|n| n.ssid == "WeakNet")
            .expect("WeakNet");
        assert!(!weak_net.active);
        assert_eq!(weak_net.strength, 80);
    }

    #[test]
    fn parse_networks_skips_short_and_hidden_ssids() {
        let input = "yes:80:5180::AA:BB:CC:DD:EE:FF:WPA2\nbad:line\n";
        let networks = parse_networks(input);
        assert!(networks.is_empty());
    }

    #[test]
    fn parse_saved_connections_fixture() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_saved_connections.txt");
        let saved = parse_saved_connections(fixture);
        assert_eq!(saved.len(), 3);
        assert!(saved.iter().any(|s| s.name == "Home" && s.autoconnect));
        assert!(saved.iter().any(|s| s.name == "Work" && !s.autoconnect));
        assert!(saved.iter().any(|s| s.name == "Guest"));
        assert!(!saved.iter().any(|s| s.name.contains("Ethernet")));
    }

    #[test]
    fn parse_saved_connections_lines() {
        let input = "Home:uuid-1:802-11-wireless:yes\nWork:uuid-2:802-11-wireless:no\n";
        let saved = parse_saved_connections(input);
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].name, "Home");
        assert!(saved[0].autoconnect);
    }

    #[test]
    fn parse_wifi_radio_disabled_fixture() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_radio_wifi.txt");
        assert!(!parse_wifi_radio_enabled(fixture));
    }

    #[test]
    fn parse_active_wifi_connection_fixture() {
        let fixture =
            include_str!("../../tests/fixtures/network/nmcli_active_connections_wifi.txt");
        let active = parse_first_active_connection(fixture).expect("wifi connection");
        assert_eq!(active.connection_type, "wifi");
        assert!(!active.ethernet_on_link);
        assert_eq!(active.name, "Home");
    }

    #[test]
    fn parse_active_ethernet_connection_fixture() {
        let fixture =
            include_str!("../../tests/fixtures/network/nmcli_active_connections_ethernet.txt");
        let active = parse_first_active_connection(fixture).expect("ethernet");
        assert_eq!(active.connection_type, "ethernet");
        assert!(active.ethernet_on_link);
    }

    #[test]
    fn parse_active_connection_skips_malformed_lines() {
        let fixture =
            include_str!("../../tests/fixtures/network/nmcli_active_connections_malformed.txt");
        let active = parse_first_active_connection(fixture).expect("valid after skip");
        assert_eq!(active.device, "wlan0");
    }

    #[test]
    fn parse_active_wifi_ssid_fixture() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_dev_wifi_active.txt");
        assert_eq!(parse_active_wifi_ssid(fixture).as_deref(), Some("CafeWiFi"));
    }

    #[test]
    fn parse_public_ip_response_accepts_ipv4_and_ipv6() {
        assert_eq!(
            parse_public_ip_response("142.117.124.114\n").as_deref(),
            Some("142.117.124.114")
        );
        assert_eq!(
            parse_public_ip_response("2001:db8::1").as_deref(),
            Some("2001:db8::1")
        );
    }

    #[test]
    fn parse_public_ip_response_rejects_html_and_garbage() {
        assert!(parse_public_ip_response("<!DOCTYPE html>").is_none());
        assert!(parse_public_ip_response("not-an-ip").is_none());
        assert!(parse_public_ip_response("").is_none());
    }

    #[test]
    fn parse_ip4_address_fixture() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_device_ip4.txt");
        assert_eq!(parse_ip4_first_address(fixture).as_deref(), Some("192.168.1.42"));
    }

    #[test]
    fn merge_ethernet_when_connection_type_none() {
        let fixture = include_str!("../../tests/fixtures/network/nmcli_device_list_ethernet.txt");
        let mut ctype = "none".to_string();
        let mut eth = false;
        merge_ethernet_from_device_list(fixture, &mut ctype, &mut eth);
        assert!(eth);
        assert_eq!(ctype, "ethernet");
    }

    #[test]
    fn map_nmcli_connect_error_not_found_fixture() {
        let raw = include_str!("../../tests/fixtures/nmcli/connect_error_not_found.txt");
        assert_eq!(
            map_nmcli_connect_error(raw),
            "Wi‑Fi network not in range — scan again"
        );
    }

    #[test]
    fn map_nmcli_connect_error_secrets_fixture() {
        let raw = include_str!("../../tests/fixtures/nmcli/connect_error_secrets.txt");
        assert_eq!(
            map_nmcli_connect_error(raw),
            "Password required for this network"
        );
    }

    #[test]
    fn nmcli_connect_success_fixture() {
        let out = include_str!("../../tests/fixtures/nmcli/connect_success.txt");
        assert!(nmcli_connect_output_success(out));
    }

    #[test]
    fn security_classifiers() {
        assert!(is_open_security("--"));
        assert!(is_open_security("open"));
        assert!(is_wpa3_security("WPA3"));
        assert!(is_enterprise_security("WPA2 802.1X"));
        assert!(!is_enterprise_security("WPA2"));
    }

    #[test]
    fn parse_saved_connections_skips_short_lines_and_non_wifi() {
        let input = "short:line\nEthernet:uuid:802-3-ethernet:yes\nCafe:u2:wifi:yes\n";
        let saved = parse_saved_connections(input);
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].name, "Cafe");
    }

    #[test]
    fn parse_saved_connections_accepts_802_11_wireless_type() {
        let input = "Guest:uuid-g:802-11-wireless:no\n";
        let saved = parse_saved_connections(input);
        assert_eq!(saved.len(), 1);
        assert!(!saved[0].autoconnect);
    }
}
