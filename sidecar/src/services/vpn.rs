use crate::services::ServiceRegistry;
use crate::types::{VpnProfile, VpnState, VpnStatus};
use crate::utils::{keyring, process};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct VpnServiceState {
    active_profile_id: Option<String>,
    state: VpnState,
    message: String,
    connection_log: Vec<String>,
}

impl Default for VpnServiceState {
    fn default() -> Self {
        Self {
            active_profile_id: None,
            state: VpnState::Disconnected,
            message: "Disconnected".to_string(),
            connection_log: Vec::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref VPN_STATE: RwLock<VpnServiceState> = RwLock::new(VpnServiceState::default());
    static ref VPN_PROFILES: Vec<VpnProfile> = vec![
        VpnProfile {
            id: "education".to_string(),
            name: "Campus".to_string(),
            icon: "school".to_string(),
            display_name: "Campus VPN".to_string(),
            interface: "tun0".to_string(),
            requires_credentials: true,
        },
        VpnProfile {
            id: "personal".to_string(),
            name: "Personal".to_string(),
            icon: "person".to_string(),
            display_name: "Personal VPN".to_string(),
            interface: "example-exit".to_string(),
            requires_credentials: false,
        },
        VpnProfile {
            id: "home".to_string(),
            name: "Home".to_string(),
            icon: "home".to_string(),
            display_name: "Home VPN".to_string(),
            interface: "tun0".to_string(),
            requires_credentials: true,
        },
    ];
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start connection monitoring
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            check_connection_status().await.ok();
        }
    });

    registry.register("Vpn.Connect", |params| async move {
        let profile_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("profile_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing profile_id parameter"))?,
        )?;

        let auth: Option<HashMap<String, String>> = params
            .as_ref()
            .and_then(|p| p.get("auth").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        connect_vpn(&profile_id, auth).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Vpn.Disconnect", |_params| async move {
        disconnect_vpn().await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Vpn.GetProfiles", |_params| async move {
        Ok(serde_json::to_value(VPN_PROFILES.clone())?)
    });

    registry.register("Vpn.GetStatus", |_params| async move {
        let state = VPN_STATE.read().await;
        let status = VpnStatus {
            state: state.state.clone(),
            message: state.message.clone(),
            profile_id: state.active_profile_id.clone(),
        };
        Ok(serde_json::to_value(status)?)
    });
}

async fn connect_vpn(profile_id: &str, auth: Option<HashMap<String, String>>) -> Result<()> {
    let profile = VPN_PROFILES
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", profile_id))?;

    // Disconnect any existing connection
    disconnect_vpn().await.ok();

    let mut state = VPN_STATE.write().await;
    state.active_profile_id = Some(profile_id.to_string());
    state.state = VpnState::Connecting;
    state.message = "Connecting...".to_string();
    state.connection_log.clear();
    drop(state);

    if profile.requires_credentials {
        // Load credentials from keyring or use provided auth
        let mut username = auth
            .as_ref()
            .and_then(|a| a.get("user").cloned());

        let mut password = auth
            .as_ref()
            .and_then(|a| a.get("pass").cloned());

        let mut mfa = auth
            .as_ref()
            .and_then(|a| a.get("mfa").cloned());

        // Fallback to keyring if not provided
        if username.is_none() {
            if let Ok(Some(u)) = keyring::lookup_vpn_credential(profile_id, "vpn_user").await {
                username = Some(u);
            }
        }

        if password.is_none() {
            if let Ok(Some(p)) = keyring::lookup_vpn_credential(profile_id, "vpn_password").await {
                password = Some(p);
            }
        }

        if mfa.is_none() {
            if let Ok(Some(m)) = keyring::lookup_vpn_credential(profile_id, "vpn_mfa").await {
                mfa = Some(m);
            }
        }

        let mfa = mfa.unwrap_or_else(|| "push".to_string());

        if username.is_none() || password.is_none() {
            let mut state = VPN_STATE.write().await;
            state.state = VpnState::Error;
            state.message = "Missing credentials".to_string();
            return Err(anyhow::anyhow!("Missing credentials"));
        }

        // Persist provided credentials into keyring for future use
        if let Some(a) = auth.as_ref() {
            if let Some(user) = a.get("user") {
                let _ = keyring::store_vpn_credential(profile_id, "vpn_user", user).await;
            }
            if let Some(pass) = a.get("pass") {
                let _ = keyring::store_vpn_credential(profile_id, "vpn_password", pass).await;
            }
            if let Some(m) = a.get("mfa") {
                let _ = keyring::store_vpn_credential(profile_id, "vpn_mfa", m).await;
            }
        }

        // Execute VPN connection command
        if profile_id == "home" {
            // OpenVPN
            let auth_file = format!("/tmp/caelestia-vpn-auth-{}.txt", profile_id);
            tokio::fs::write(&auth_file, format!("{}\n{}", username.unwrap(), password.unwrap())).await?;

            let home = std::env::var("HOME")?;
            let config_path = format!("{}/.config/quickshell/caelestia/assets/vpn/home.ovpn", home);

            Command::new("sudo")
                .args(&[
                    "/usr/bin/openvpn",
                    "--config",
                    &config_path,
                    "--auth-user-pass",
                    &auth_file,
                    "--script-security",
                    "2",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
        } else {
            // OpenConnect
            let mut cmd = Command::new("sudo");
            cmd.args(&[
                "/usr/bin/openconnect",
                "vpn.example.edu",
                "--user",
                &username.unwrap(),
                "--authgroup=example-group",
                "--passwd-on-stdin",
                "--disable-ipv6",
                "--script=/etc/vpnc/vpnc-script",
            ]);

            if let Some(pass) = password {
                cmd.stdin(Stdio::piped());
                let mut child = cmd.spawn()?;
                if let Some(mut stdin) = child.stdin.take() {
                    use tokio::io::AsyncWriteExt;
                    let input = format!("{}\n{}\n", pass, mfa);
                    stdin.write_all(input.as_bytes()).await?;
                }
                // Don't wait for completion - it runs in background
            }
        }
    } else {
        // Non-credential VPN (NetworkManager)
        if profile_id == "personal" {
            process::exec_command_detached(&["nmcli", "connection", "up", "example-exit"]).await?;
        }
    }

    Ok(())
}

async fn disconnect_vpn() -> Result<()> {
    let state = VPN_STATE.read().await;
    let profile_id = state.active_profile_id.clone();
    drop(state);

    if let Some(profile) = profile_id.and_then(|id| VPN_PROFILES.iter().find(|p| p.id == id)) {
        if profile.requires_credentials {
            // Kill OpenConnect or OpenVPN
            let _ = process::exec_command(&["sudo", "pkill", "-SIGINT", "openconnect"]).await;
            let _ = process::exec_command(&["sudo", "pkill", "-SIGINT", "openvpn"]).await;
        } else {
            if profile.id == "personal" {
                let _ = process::exec_command(&["nmcli", "connection", "down", "example-exit"]).await;
            }
        }
    }

    let mut state = VPN_STATE.write().await;
    state.state = VpnState::Disconnected;
    state.message = "Disconnected".to_string();
    state.active_profile_id = None;
    state.connection_log.clear();

    Ok(())
}

async fn check_connection_status() -> Result<()> {
    let state = VPN_STATE.read().await;
    let profile_id = state.active_profile_id.clone();
    let interface = profile_id
        .as_ref()
        .and_then(|id| VPN_PROFILES.iter().find(|p| p.id == *id))
        .map(|p| p.interface.clone());
    drop(state);

    if let Some(iface) = interface {
        // Check if interface exists
        let output = process::exec_command(&["ip", "-o", "addr", "show"]).await?;
        let is_connected =
            vpn_interface_connected(&output, &iface, is_vpn_process_running().await);

        let mut state = VPN_STATE.write().await;
        if is_connected && state.state != VpnState::Connected {
            state.state = VpnState::Connected;
            state.message = "Connected".to_string();
        } else if !is_connected && state.state == VpnState::Connected {
            state.state = VpnState::Disconnected;
            state.message = "Disconnected".to_string();
        }
    }

    Ok(())
}

/// Whether VPN appears up from `ip -o addr` output (unit-tested with fixtures).
pub(crate) fn vpn_interface_connected(
    ip_output: &str,
    iface: &str,
    vpn_process_running: bool,
) -> bool {
    ip_output.contains(iface)
        || (iface.starts_with("tun") && ip_output.contains("tun") && vpn_process_running)
}

async fn is_vpn_process_running() -> bool {
    process::exec_command(&["pgrep", "-x", "openconnect"])
        .await
        .is_ok()
        || process::exec_command(&["pgrep", "-x", "openvpn"])
            .await
            .is_ok()
}

#[cfg(test)]
mod tests {
    use super::vpn_interface_connected;

    #[test]
    fn vpn_connected_when_iface_present() {
        let out = "2: tun0: <POINTOPOINT> inet 10.0.0.2/32\n";
        assert!(vpn_interface_connected(out, "tun0", false));
    }

    #[test]
    fn vpn_tun_fallback_requires_process() {
        let out = "3: tun1: inet 10.0.0.3/32\n";
        assert!(!vpn_interface_connected(out, "tun9", false));
        assert!(vpn_interface_connected(out, "tun9", true));
    }

    #[test]
    fn vpn_nmcli_profile_interface_match() {
        let out = "4: example-exit: <BROADCAST> inet 192.168.1.5/24\n";
        assert!(vpn_interface_connected(out, "example-exit", false));
        assert!(!vpn_interface_connected(out, "eth0", false));
    }
}
