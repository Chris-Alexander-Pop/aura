use crate::services::ServiceRegistry;
use crate::types::{VpnProfile, VpnState, VpnStatus};
use crate::utils::{keyring, process};
use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

const VPN_BINARIES: &[&str] = &["openconnect", "openvpn", "wg-quick", "nmcli", "pkill"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VpnProtocol {
    Openconnect,
    Openvpn,
    Wireguard,
    Networkmanager,
}

impl Default for VpnProtocol {
    fn default() -> Self {
        Self::Openconnect
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct VpnProfileDef {
    id: String,
    name: String,
    icon: String,
    display_name: String,
    interface: String,
    requires_credentials: bool,
    #[serde(default)]
    protocol: VpnProtocol,
    server: Option<String>,
    authgroup: Option<String>,
    connection: Option<String>,
    config: Option<String>,
}

impl VpnProfileDef {
    fn to_public(&self) -> VpnProfile {
        VpnProfile {
            id: self.id.clone(),
            name: self.name.clone(),
            icon: self.icon.clone(),
            display_name: self.display_name.clone(),
            interface: self.interface.clone(),
            requires_credentials: self.requires_credentials,
        }
    }

    fn config_path(&self, config_dir: &Path) -> Option<PathBuf> {
        self.config
            .as_ref()
            .map(|name| config_dir.join(name))
    }
}

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

/// Host subprocess backend for VPN connect/disconnect (mockable in unit tests).
#[async_trait::async_trait]
pub trait VpnProcess: Send + Sync {
    async fn exec_command(&self, cmd: &[&str]) -> Result<String>;
    async fn exec_command_detached(&self, cmd: &[&str]) -> Result<()>;
    async fn spawn_with_stdin(&self, cmd: &[&str], stdin: &[u8]) -> Result<()>;
}

struct HostVpnProcess;

#[async_trait::async_trait]
impl VpnProcess for HostVpnProcess {
    async fn exec_command(&self, cmd: &[&str]) -> Result<String> {
        process::exec_command(cmd).await
    }

    async fn exec_command_detached(&self, cmd: &[&str]) -> Result<()> {
        assert_allowlisted_vpn_argv(cmd)?;
        process::exec_command_detached(cmd).await
    }

    async fn spawn_with_stdin(&self, cmd: &[&str], stdin: &[u8]) -> Result<()> {
        assert_allowlisted_vpn_argv(cmd)?;
        let mut child = Command::new(cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut child_stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            child_stdin.write_all(stdin).await?;
        }
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref VPN_STATE: RwLock<VpnServiceState> = RwLock::new(VpnServiceState::default());
}

fn vpn_dry_run() -> bool {
    std::env::var("AURA_VPN_DRY_RUN").ok().as_deref() == Some("1")
}

pub fn vpn_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("AURA_VPN_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    base.join("ags/vpn")
}

fn default_profile_defs() -> Vec<VpnProfileDef> {
    serde_json::from_str(include_str!("../../assets/vpn/profiles.json"))
        .expect("embedded default vpn profiles")
}

fn load_profile_defs_from_dir(config_dir: &Path) -> Vec<VpnProfileDef> {
    let manifest = config_dir.join("profiles.json");
    if !manifest.is_file() {
        return default_profile_defs();
    }
    match std::fs::read_to_string(&manifest) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            tracing::warn!("invalid vpn profiles.json in {}: {e}", config_dir.display());
            default_profile_defs()
        }),
        Err(e) => {
            tracing::warn!("failed to read vpn profiles.json: {e}");
            default_profile_defs()
        }
    }
}

pub(crate) fn load_profile_defs() -> Vec<VpnProfileDef> {
    load_profile_defs_from_dir(&vpn_config_dir())
}

pub fn load_public_profiles_from_dir(config_dir: &Path) -> Vec<VpnProfile> {
    load_profile_defs_from_dir(config_dir)
        .into_iter()
        .map(|p| p.to_public())
        .collect()
}

fn profile_id_from_params(params: &Option<serde_json::Value>) -> Result<String> {
    let p = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing params"))?;
    let value = p
        .get("profile_id")
        .or_else(|| p.get("profileId"))
        .cloned()
        .ok_or_else(|| anyhow!("Missing profile_id parameter"))?;
    serde_json::from_value(value).map_err(Into::into)
}

pub fn assert_allowlisted_vpn_argv(argv: &[&str]) -> Result<()> {
    if argv.is_empty() {
        bail!("vpn_binary_denied: empty argv");
    }
    let bin_idx = if argv[0] == "sudo" { 1 } else { 0 };
    let bin = argv
        .get(bin_idx)
        .ok_or_else(|| anyhow!("vpn_binary_denied: missing binary"))?;
    let name = Path::new(bin)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(bin);
    if !VPN_BINARIES.contains(&name) {
        bail!("vpn_binary_denied: {name}");
    }
    Ok(())
}

fn apply_connect_start(state: &mut VpnServiceState, profile_id: &str) {
    state.active_profile_id = Some(profile_id.to_string());
    state.state = VpnState::Connecting;
    state.message = "Connecting...".to_string();
    state.connection_log.clear();
}

fn apply_connected(state: &mut VpnServiceState, message: &str) {
    state.state = VpnState::Connected;
    state.message = message.to_string();
}

fn apply_disconnected(state: &mut VpnServiceState) {
    state.state = VpnState::Disconnected;
    state.message = "Disconnected".to_string();
    state.active_profile_id = None;
    state.connection_log.clear();
}

fn apply_error(state: &mut VpnServiceState, message: &str) {
    state.state = VpnState::Error;
    state.message = message.to_string();
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(2));
        loop {
            tick.tick().await;
            check_connection_status().await.ok();
        }
    });

    registry.register("Vpn.Connect", |params| async move {
        let profile_id = profile_id_from_params(&params)?;

        let auth: Option<HashMap<String, String>> = params
            .as_ref()
            .and_then(|p| p.get("auth").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        connect_vpn(&profile_id, auth, &HostVpnProcess).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Vpn.Disconnect", |_params| async move {
        disconnect_vpn(&HostVpnProcess).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Vpn.GetProfiles", |_params| async move {
        let profiles: Vec<VpnProfile> = load_profile_defs()
            .into_iter()
            .map(|p| p.to_public())
            .collect();
        Ok(serde_json::to_value(profiles)?)
    });

    registry.register("Vpn.GetStatus", |_params| async move {
        Ok(serde_json::to_value(snapshot_vpn_status().await?)?)
    });
}

async fn snapshot_vpn_status() -> Result<VpnStatus> {
    let state = VPN_STATE.read().await.clone();
    build_vpn_status(&state).await
}

async fn build_vpn_status(state: &VpnServiceState) -> Result<VpnStatus> {
    let profiles = load_profile_defs();
    let profile = state
        .active_profile_id
        .as_ref()
        .and_then(|id| profiles.iter().find(|p| p.id == *id));

    let mut interface = profile.map(|p| p.interface.clone());
    let mut local_ip = None;

    if state.state == VpnState::Connected {
        if let Some(iface) = interface.clone() {
            if let Ok(output) = process::exec_command(&["ip", "-o", "addr", "show"]).await {
                local_ip = parse_iface_ipv4(&output, &iface);
            }
        }
    } else if state.state == VpnState::Disconnected {
        interface = None;
    }

    Ok(VpnStatus {
        state: state.state.clone(),
        message: state.message.clone(),
        profile_id: state.active_profile_id.clone(),
        interface,
        local_ip,
    })
}

async fn connect_vpn(
    profile_id: &str,
    auth: Option<HashMap<String, String>>,
    backend: &dyn VpnProcess,
) -> Result<()> {
    let profiles = load_profile_defs();
    let profile = profiles
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| anyhow!("Profile not found: {profile_id}"))?
        .clone();
    let config_dir = vpn_config_dir();

    disconnect_vpn(backend).await.ok();

    {
        let mut state = VPN_STATE.write().await;
        apply_connect_start(&mut state, profile_id);
    }

    if vpn_dry_run() {
        let mut state = VPN_STATE.write().await;
        apply_connected(&mut state, "Connected (dry run)");
        return Ok(());
    }

    let result = match profile.protocol {
        VpnProtocol::Networkmanager => connect_networkmanager(&profile, backend).await,
        VpnProtocol::Wireguard => connect_wireguard(&profile, &config_dir, backend).await,
        VpnProtocol::Openvpn => connect_openvpn(&profile, &config_dir, profile_id, &auth, backend).await,
        VpnProtocol::Openconnect => connect_openconnect(profile_id, &profile, &auth, backend).await,
    };

    if let Err(e) = result {
        let mut state = VPN_STATE.write().await;
        apply_error(&mut state, &e.to_string());
        return Err(e);
    }

    Ok(())
}

async fn connect_networkmanager(profile: &VpnProfileDef, backend: &dyn VpnProcess) -> Result<()> {
    let connection = profile
        .connection
        .as_deref()
        .ok_or_else(|| anyhow!("Missing connection for NetworkManager profile"))?;
    let cmd = ["nmcli", "connection", "up", connection];
    assert_allowlisted_vpn_argv(&cmd)?;
    backend.exec_command_detached(&cmd).await
}

async fn connect_wireguard(
    profile: &VpnProfileDef,
    config_dir: &Path,
    backend: &dyn VpnProcess,
) -> Result<()> {
    let config_path = profile
        .config_path(config_dir)
        .ok_or_else(|| anyhow!("Missing config for WireGuard profile"))?;
    if !config_path.is_file() {
        bail!("WireGuard config not found: {}", config_path.display());
    }
    let path = config_path.to_string_lossy();
    let cmd = ["sudo", "wg-quick", "up", &path];
    assert_allowlisted_vpn_argv(&cmd)?;
    backend.exec_command_detached(&cmd).await
}

async fn connect_openvpn(
    profile: &VpnProfileDef,
    config_dir: &Path,
    profile_id: &str,
    auth: &Option<HashMap<String, String>>,
    backend: &dyn VpnProcess,
) -> Result<()> {
    let config_path = profile
        .config_path(config_dir)
        .filter(|p| p.is_file())
        .or_else(|| {
            std::env::var("HOME").ok().map(|home| {
                PathBuf::from(home).join(".config/ags/vpn/home.ovpn")
            })
        })
        .ok_or_else(|| anyhow!("OpenVPN config not found for profile {profile_id}"))?;

    let (username, password) = resolve_credentials(profile_id, profile.requires_credentials, auth).await?;
    let auth_file = format!("/tmp/aura-vpn-auth-{profile_id}.txt");
    tokio::fs::write(&auth_file, format!("{username}\n{password}")).await?;

    let config = config_path.to_string_lossy();
    let auth_path = auth_file.as_str();
    let cmd = [
        "sudo",
        "/usr/bin/openvpn",
        "--config",
        &config,
        "--auth-user-pass",
        auth_path,
        "--script-security",
        "2",
    ];
    assert_allowlisted_vpn_argv(&cmd)?;
    backend.exec_command_detached(&cmd).await
}

async fn connect_openconnect(
    profile_id: &str,
    profile: &VpnProfileDef,
    auth: &Option<HashMap<String, String>>,
    backend: &dyn VpnProcess,
) -> Result<()> {
    let server = profile
        .server
        .as_deref()
        .ok_or_else(|| anyhow!("Missing server for OpenConnect profile"))?;
    let (username, password) = resolve_credentials(profile_id, profile.requires_credentials, auth).await?;
    let mfa = resolve_mfa(profile_id, auth).await;

    let user = username;
    let pass = password;
    let mut args = vec![
        "sudo".to_string(),
        "/usr/bin/openconnect".to_string(),
        server.to_string(),
        "--user".to_string(),
        user.clone(),
        "--passwd-on-stdin".to_string(),
        "--disable-ipv6".to_string(),
        "--script=/etc/vpnc/vpnc-script".to_string(),
    ];
    if let Some(group) = profile.authgroup.as_deref() {
        args.push(format!("--authgroup={group}"));
    }

    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    assert_allowlisted_vpn_argv(&argv)?;
    backend
        .spawn_with_stdin(&argv, format!("{pass}\n{mfa}\n").as_bytes())
        .await
}

async fn resolve_credentials(
    profile_id: &str,
    requires_credentials: bool,
    auth: &Option<HashMap<String, String>>,
) -> Result<(String, String)> {
    if !requires_credentials {
        return Ok((String::new(), String::new()));
    }

    let mut username = auth.as_ref().and_then(|a| a.get("user").cloned());
    let mut password = auth.as_ref().and_then(|a| a.get("pass").cloned());

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

    match (username, password) {
        (Some(u), Some(p)) => Ok((u, p)),
        _ => {
            let mut state = VPN_STATE.write().await;
            apply_error(&mut state, "Missing credentials");
            Err(anyhow!("Missing credentials"))
        }
    }
}

async fn resolve_mfa(profile_id: &str, auth: &Option<HashMap<String, String>>) -> String {
    if let Some(m) = auth.as_ref().and_then(|a| a.get("mfa").cloned()) {
        return m;
    }
    if let Ok(Some(m)) = keyring::lookup_vpn_credential(profile_id, "vpn_mfa").await {
        return m;
    }
    "push".to_string()
}

async fn disconnect_vpn(backend: &dyn VpnProcess) -> Result<()> {
    if vpn_dry_run() {
        let mut state = VPN_STATE.write().await;
        apply_disconnected(&mut state);
        return Ok(());
    }

    let profile_id = {
        let state = VPN_STATE.read().await;
        state.active_profile_id.clone()
    };

    let profiles = load_profile_defs();
    if let Some(id) = profile_id.as_deref() {
        if let Some(profile) = profiles.iter().find(|p| p.id == id) {
            match profile.protocol {
                VpnProtocol::Networkmanager => {
                    if let Some(connection) = profile.connection.as_deref() {
                        let cmd = ["nmcli", "connection", "down", connection];
                        assert_allowlisted_vpn_argv(&cmd)?;
                        let _ = backend.exec_command(&cmd).await;
                    }
                }
                VpnProtocol::Wireguard => {
                    if let Some(config_path) = profile.config_path(&vpn_config_dir()) {
                        if config_path.is_file() {
                            let path = config_path.to_string_lossy();
                            let cmd = ["sudo", "wg-quick", "down", &path];
                            assert_allowlisted_vpn_argv(&cmd)?;
                            let _ = backend.exec_command(&cmd).await;
                        }
                    }
                }
                VpnProtocol::Openconnect | VpnProtocol::Openvpn => {
                    let oc = ["sudo", "pkill", "-SIGINT", "openconnect"];
                    assert_allowlisted_vpn_argv(&oc)?;
                    let _ = backend.exec_command(&oc).await;
                    let ov = ["sudo", "pkill", "-SIGINT", "openvpn"];
                    assert_allowlisted_vpn_argv(&ov)?;
                    let _ = backend.exec_command(&ov).await;
                }
            }
        }
    }

    let mut state = VPN_STATE.write().await;
    apply_disconnected(&mut state);
    Ok(())
}

async fn check_connection_status() -> Result<()> {
    let (profile_id, current_state) = {
        let state = VPN_STATE.read().await;
        (state.active_profile_id.clone(), state.state.clone())
    };

    let Some(id) = profile_id else {
        return Ok(());
    };

    let profiles = load_profile_defs();
    let Some(profile) = profiles.iter().find(|p| p.id == id) else {
        return Ok(());
    };

    let output = process::exec_command(&["ip", "-o", "addr", "show"]).await?;
    let process_running = is_vpn_process_running().await;
    let is_connected =
        vpn_interface_connected(&output, &profile.interface, process_running);

    let mut state = VPN_STATE.write().await;
    if is_connected && current_state != VpnState::Connected {
        apply_connected(&mut state, "Connected");
    } else if !is_connected && current_state == VpnState::Connected {
        apply_disconnected(&mut state);
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

/// Extract the first IPv4 address assigned to `iface` from `ip addr` / `ip -o addr` output.
pub fn parse_iface_ipv4(ip_output: &str, iface: &str) -> Option<String> {
    for line in ip_output.lines() {
        if line.contains(iface) && line.contains(" inet ") && !line.contains(" inet6 ") {
            let after = line.split(" inet ").nth(1)?;
            let addr = after.split_whitespace().next()?;
            return Some(addr.split('/').next()?.to_string());
        }
    }

    let mut current_iface: Option<String> = None;
    for line in ip_output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            if trimmed.starts_with("inet6 ") {
                continue;
            }
            if current_iface.as_deref() == Some(iface) {
                let addr = rest.split_whitespace().next()?;
                return Some(addr.split('/').next()?.to_string());
            }
        }
        if let Some((_, name_part)) = trimmed.split_once(':') {
            if let Some(name) = name_part.split(':').next() {
                let name = name.trim();
                if !name.is_empty() && !name.contains(' ') {
                    current_iface = Some(name.to_string());
                }
            }
        }
    }

    None
}

/// One row from `ip link` / `ip -o link show`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IpLinkLine {
    pub name: String,
    pub is_up: bool,
}

/// Parse a single `ip link` output line (`2: wg0: <POINTOPOINT,UP,LOWER_UP> ...`).
pub(crate) fn parse_ip_link_line(line: &str) -> Option<IpLinkLine> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let rest = line.split_once(':')?.1.trim();
    let (name, flags) = rest.split_once(':')?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return None;
    }
    let flags_upper = flags.to_ascii_uppercase();
    if flags_upper.contains("STATE DOWN") {
        return Some(IpLinkLine { name, is_up: false });
    }
    if flags_upper.contains("STATE UP") {
        return Some(IpLinkLine { name, is_up: true });
    }
    let is_up = flags
        .split('<')
        .nth(1)
        .and_then(|s| s.split('>').next())
        .is_some_and(|inner| {
            inner.split(',').any(|f| {
                matches!(
                    f.trim().to_ascii_uppercase().as_str(),
                    "UP" | "LOWER_UP"
                )
            })
        });
    Some(IpLinkLine { name, is_up })
}

/// True when `iface` appears UP in `ip link` output (WireGuard/NM profile checks).
pub(crate) fn ip_link_interface_up(output: &str, iface: &str) -> bool {
    output
        .lines()
        .filter_map(parse_ip_link_line)
        .any(|row| row.name == iface && row.is_up)
}

/// Summary fields from a WireGuard `.conf` (profile discovery / UI labels).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WireGuardConfigSummary {
    pub addresses: Vec<String>,
    pub dns: Vec<String>,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
}

/// Parse standard WireGuard config sections for future profile wiring.
pub(crate) fn parse_wireguard_conf(conf: &str) -> WireGuardConfigSummary {
    let mut summary = WireGuardConfigSummary::default();
    let mut section: Option<&str> = None;

    for raw in conf.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = Some(&line[1..line.len() - 1]);
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match section {
            Some("Interface") => match key {
                "Address" => summary
                    .addresses
                    .extend(value.split(',').map(|s| s.trim().to_string())),
                "DNS" => summary
                    .dns
                    .extend(value.split(',').map(|s| s.trim().to_string())),
                _ => {}
            },
            Some("Peer") => match key {
                "Endpoint" if summary.endpoint.is_none() => {
                    summary.endpoint = Some(value.to_string());
                }
                "AllowedIPs" => summary
                    .allowed_ips
                    .extend(value.split(',').map(|s| s.trim().to_string())),
                _ => {}
            },
            _ => {}
        }
    }

    summary
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
    use super::{
        apply_connect_start, apply_connected, apply_disconnected, apply_error,
        assert_allowlisted_vpn_argv, check_connection_status, connect_networkmanager,
        connect_vpn, connect_wireguard, default_profile_defs, disconnect_vpn,
        ip_link_interface_up, load_profile_defs_from_dir, parse_iface_ipv4, parse_ip_link_line,
        parse_wireguard_conf, snapshot_vpn_status, vpn_interface_connected, VpnProcess,
        VpnProfileDef, VpnProtocol, VpnServiceState, VpnState, WireGuardConfigSummary, VPN_STATE,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn vpn_fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/vpn")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    struct RecordingVpnProcess {
        calls: Mutex<Vec<Vec<String>>>,
        detached_errors: Mutex<HashMap<String, String>>,
        exec_errors: Mutex<HashMap<String, String>>,
        exec_responses: Mutex<HashMap<String, String>>,
    }

    impl RecordingVpnProcess {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                detached_errors: Mutex::new(HashMap::new()),
                exec_errors: Mutex::new(HashMap::new()),
                exec_responses: Mutex::new(HashMap::new()),
            }
        }

        fn with_detached_error(self, binary: &str, message: &str) -> Self {
            self.detached_errors
                .lock()
                .unwrap()
                .insert(binary.to_string(), message.to_string());
            self
        }

        fn with_exec_error(self, binary: &str, message: &str) -> Self {
            self.exec_errors
                .lock()
                .unwrap()
                .insert(binary.to_string(), message.to_string());
            self
        }

        fn with_exec_response(self, binary: &str, stdout: &str) -> Self {
            self.exec_responses
                .lock()
                .unwrap()
                .insert(binary.to_string(), stdout.to_string());
            self
        }

        fn recorded_calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }

        fn primary_binary(cmd: &[&str]) -> String {
            if cmd.first() == Some(&"sudo") {
                cmd.get(1).map(|s| s.to_string()).unwrap_or_default()
            } else {
                cmd.first().map(|s| s.to_string()).unwrap_or_default()
            }
        }
    }

    #[async_trait::async_trait]
    impl VpnProcess for RecordingVpnProcess {
        async fn exec_command(&self, cmd: &[&str]) -> Result<String, anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(cmd.iter().map(|s| s.to_string()).collect());
            let bin = Self::primary_binary(cmd);
            if let Some(msg) = self.exec_errors.lock().unwrap().get(&bin) {
                return Err(anyhow::anyhow!("{msg}"));
            }
            if let Some(out) = self.exec_responses.lock().unwrap().get(&bin) {
                return Ok(out.clone());
            }
            if bin.ends_with("nmcli") {
                return Ok(vpn_fixture("nmcli_connection_up_ok.txt"));
            }
            Ok(String::new())
        }

        async fn exec_command_detached(&self, cmd: &[&str]) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(cmd.iter().map(|s| s.to_string()).collect());
            let bin = Self::primary_binary(cmd);
            if let Some(msg) = self.detached_errors.lock().unwrap().get(&bin) {
                return Err(anyhow::anyhow!("{msg}"));
            }
            Ok(())
        }

        async fn spawn_with_stdin(&self, cmd: &[&str], _stdin: &[u8]) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(cmd.iter().map(|s| s.to_string()).collect());
            Ok(())
        }
    }

    #[test]
    fn vpn_connected_when_iface_present() {
        let out = vpn_fixture("ip_o_addr_show_connected.txt");
        assert!(vpn_interface_connected(&out, "tun0", false));
    }

    #[test]
    fn vpn_tun_fallback_requires_process() {
        let out = vpn_fixture("ip_o_addr_show_tun_only.txt");
        assert!(!vpn_interface_connected(&out, "tun9", false));
        assert!(vpn_interface_connected(&out, "tun9", true));
    }

    #[test]
    fn vpn_nmcli_profile_interface_match() {
        let out = vpn_fixture("ip_o_addr_show_nm_profile.txt");
        assert!(vpn_interface_connected(&out, "example-exit", false));
        assert!(!vpn_interface_connected(&out, "eth0", false));
    }

    #[test]
    fn parse_iface_ipv4_from_addr_fixture() {
        let out = vpn_fixture("ip_o_addr_show_connected.txt");
        assert_eq!(parse_iface_ipv4(&out, "tun0").as_deref(), Some("10.0.0.2"));
    }

    #[test]
    fn parse_iface_ipv4_from_ip_o_output() {
        let out = "3: tun0    inet 10.14.0.2/24 brd 10.14.0.2 scope global tun0";
        assert_eq!(parse_iface_ipv4(out, "tun0").as_deref(), Some("10.14.0.2"));
    }

    #[test]
    fn parse_ip_link_fixture_multi() {
        let out = vpn_fixture("ip_link_multi.txt");
        assert!(ip_link_interface_up(&out, "wg0"));
        assert!(ip_link_interface_up(&out, "example-exit"));
        assert!(!ip_link_interface_up(&out, "eth0"));
        assert!(!ip_link_interface_up(&out, "docker0"));
    }

    #[test]
    fn parse_ip_link_line_skips_malformed() {
        assert!(parse_ip_link_line("").is_none());
        assert!(parse_ip_link_line("not ip output").is_none());
        let row = parse_ip_link_line("2: wg0: <POINTOPOINT,UP,LOWER_UP> mtu 1420").expect("row");
        assert_eq!(row.name, "wg0");
        assert!(row.is_up);
        let down = parse_ip_link_line("3: eth0: <BROADCAST,MULTICAST> mtu 1500").expect("down");
        assert!(!down.is_up);
    }

    #[test]
    fn parse_wireguard_home_fixture() {
        let conf = vpn_fixture("wg_home.conf");
        let summary = parse_wireguard_conf(&conf);
        assert_eq!(
            summary,
            WireGuardConfigSummary {
                addresses: vec!["10.14.0.2/32".to_string()],
                dns: vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()],
                endpoint: Some("vpn.example.com:51820".to_string()),
                allowed_ips: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            }
        );
    }

    #[test]
    fn parse_wireguard_minimal_fixture_peer_only_fields() {
        let conf = vpn_fixture("wg_minimal.conf");
        let summary = parse_wireguard_conf(&conf);
        assert_eq!(summary.addresses, vec!["192.168.6.2/32"]);
        assert!(summary.dns.is_empty());
        assert_eq!(summary.endpoint.as_deref(), Some("vpn.example.com:443"));
        assert_eq!(summary.allowed_ips, vec!["10.0.0.0/8"]);
    }

    #[test]
    fn load_profiles_from_fixture_dir() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vpn");
        let profiles = load_profile_defs_from_dir(&dir);
        assert!(profiles.iter().any(|p| p.id == "education"));
        assert!(profiles.iter().any(|p| p.id == "lab-wg"));
    }

    #[test]
    fn default_profiles_include_expected_ids() {
        let profiles = default_profile_defs();
        let ids: Vec<_> = profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"education"));
        assert!(ids.contains(&"personal"));
        assert!(ids.contains(&"home"));
    }

    #[test]
    fn vpn_binary_allowlist_accepts_known_tools() {
        assert!(assert_allowlisted_vpn_argv(&["sudo", "/usr/bin/openconnect"]).is_ok());
        assert!(assert_allowlisted_vpn_argv(&["nmcli", "connection", "up", "example-exit"]).is_ok());
    }

    #[test]
    fn vpn_binary_allowlist_rejects_unknown() {
        let err = assert_allowlisted_vpn_argv(&["curl", "http://evil"]).unwrap_err();
        assert!(err.to_string().contains("vpn_binary_denied"));
    }

    #[test]
    fn state_machine_transitions() {
        let mut state = VpnServiceState::default();
        assert_eq!(state.state, VpnState::Disconnected);

        apply_connect_start(&mut state, "education");
        assert_eq!(state.state, VpnState::Connecting);
        assert_eq!(state.active_profile_id.as_deref(), Some("education"));

        apply_connected(&mut state, "Connected");
        assert_eq!(state.state, VpnState::Connected);

        apply_disconnected(&mut state);
        assert_eq!(state.state, VpnState::Disconnected);
        assert!(state.active_profile_id.is_none());

        apply_error(&mut state, "Missing credentials");
        assert_eq!(state.state, VpnState::Error);
    }

    #[tokio::test]
    async fn recording_backend_captures_disconnect_argv() {
        let backend = RecordingVpnProcess::new();
        assert_allowlisted_vpn_argv(&["sudo", "pkill", "-SIGINT", "openconnect"]).unwrap();
        backend
            .exec_command(&["sudo", "pkill", "-SIGINT", "openconnect"])
            .await
            .unwrap();
        let calls = backend.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0], "sudo");
    }

    #[test]
    fn vpn_disconnected_fixture_has_no_tun_iface() {
        let out = vpn_fixture("ip_o_addr_show_disconnected.txt");
        assert!(!vpn_interface_connected(&out, "tun0", false));
        assert!(!vpn_interface_connected(&out, "wg0", false));
        assert!(vpn_interface_connected(&out, "eth0", false));
    }

    #[test]
    fn nmcli_error_fixture_is_non_empty() {
        let err = vpn_fixture("nmcli_connection_error.txt");
        assert!(err.contains("Connection activation failed"));
    }

    #[test]
    fn wg_quick_error_fixture_mentions_existing_iface() {
        let err = vpn_fixture("wg_quick_error.txt");
        assert!(err.contains("already exists"));
    }

    #[tokio::test]
    async fn connect_networkmanager_records_nmcli_argv() {
        let backend = RecordingVpnProcess::new();
        let profile = VpnProfileDef {
            id: "personal".into(),
            name: "Personal".into(),
            icon: "person".into(),
            display_name: "Personal VPN".into(),
            interface: "example-exit".into(),
            requires_credentials: false,
            protocol: VpnProtocol::Networkmanager,
            server: None,
            authgroup: None,
            connection: Some("example-exit".into()),
            config: None,
        };
        connect_networkmanager(&profile, &backend).await.unwrap();
        let calls = backend.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], vec!["nmcli", "connection", "up", "example-exit"]);
    }

    #[tokio::test]
    async fn connect_networkmanager_propagates_nmcli_error_fixture() {
        let backend = RecordingVpnProcess::new().with_detached_error(
            "nmcli",
            &vpn_fixture("nmcli_connection_error.txt"),
        );
        let profile = VpnProfileDef {
            id: "personal".into(),
            name: "Personal".into(),
            icon: "person".into(),
            display_name: "Personal VPN".into(),
            interface: "example-exit".into(),
            requires_credentials: false,
            protocol: VpnProtocol::Networkmanager,
            server: None,
            authgroup: None,
            connection: Some("example-exit".into()),
            config: None,
        };
        let err = connect_networkmanager(&profile, &backend)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Connection activation failed"));
    }

    #[tokio::test]
    async fn connect_wireguard_records_wg_quick_argv() {
        let dir = tempfile::tempdir().expect("tempdir");
        let conf = dir.path().join("lab.conf");
        std::fs::write(&conf, vpn_fixture("wg_minimal.conf")).expect("write conf");

        let backend = RecordingVpnProcess::new();
        let profile = VpnProfileDef {
            id: "lab-wg".into(),
            name: "Lab".into(),
            icon: "shield".into(),
            display_name: "Lab WireGuard".into(),
            interface: "wg0".into(),
            requires_credentials: false,
            protocol: VpnProtocol::Wireguard,
            server: None,
            authgroup: None,
            connection: None,
            config: Some("lab.conf".into()),
        };
        connect_wireguard(&profile, dir.path(), &backend)
            .await
            .unwrap();
        let calls = backend.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0], "sudo");
        assert_eq!(calls[0][1], "wg-quick");
        assert_eq!(calls[0][2], "up");
    }

    #[tokio::test]
    async fn connect_wireguard_propagates_wg_quick_error_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let conf = dir.path().join("lab.conf");
        std::fs::write(&conf, vpn_fixture("wg_minimal.conf")).expect("write conf");

        let backend = RecordingVpnProcess::new().with_detached_error(
            "wg-quick",
            &vpn_fixture("wg_quick_error.txt"),
        );
        let profile = VpnProfileDef {
            id: "lab-wg".into(),
            name: "Lab".into(),
            icon: "shield".into(),
            display_name: "Lab WireGuard".into(),
            interface: "wg0".into(),
            requires_credentials: false,
            protocol: VpnProtocol::Wireguard,
            server: None,
            authgroup: None,
            connection: None,
            config: Some("lab.conf".into()),
        };
        let err = connect_wireguard(&profile, dir.path(), &backend)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn recording_exec_returns_nmcli_fixture_stdout() {
        let backend = RecordingVpnProcess::new();
        let out = backend
            .exec_command(&["nmcli", "connection", "down", "example-exit"])
            .await
            .unwrap();
        assert!(out.contains("successfully activated") || out.contains("Connection"));
    }

    fn set_exec_fixtures() {
        std::env::set_var(
            crate::utils::process::EXEC_FIXTURE_ENV,
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/exec")
                .to_string_lossy()
                .as_ref(),
        );
    }

    fn clear_exec_fixtures() {
        std::env::remove_var(crate::utils::process::EXEC_FIXTURE_ENV);
    }

    async fn reset_vpn_state() {
        let mut state = VPN_STATE.write().await;
        *state = VpnServiceState::default();
    }

    #[tokio::test]
    async fn connect_vpn_dry_run_transitions_and_local_ip() {
        reset_vpn_state().await;
        std::env::set_var("AURA_VPN_DRY_RUN", "1");
        set_exec_fixtures();
        connect_vpn("personal", None, &RecordingVpnProcess::new())
            .await
            .expect("connect dry run");
        let status = snapshot_vpn_status().await.expect("status");
        assert_eq!(status.state, VpnState::Connected);
        assert_eq!(status.local_ip.as_deref(), Some("192.168.1.5"));
        disconnect_vpn(&RecordingVpnProcess::new())
            .await
            .expect("disconnect dry run");
        let after = snapshot_vpn_status().await.expect("after");
        assert_eq!(after.state, VpnState::Disconnected);
        std::env::remove_var("AURA_VPN_DRY_RUN");
        clear_exec_fixtures();
    }

    #[tokio::test]
    async fn check_connection_status_keeps_connected_when_iface_up() {
        reset_vpn_state().await;
        set_exec_fixtures();
        {
            let mut state = VPN_STATE.write().await;
            apply_connected(&mut state, "Connected");
            state.active_profile_id = Some("personal".into());
        }
        check_connection_status().await.expect("check status");
        let state = VPN_STATE.read().await;
        assert_eq!(state.state, VpnState::Connected);
        clear_exec_fixtures();
    }

    #[tokio::test]
    async fn disconnect_vpn_nmcli_when_personal_active() {
        reset_vpn_state().await;
        set_exec_fixtures();
        {
            let mut state = VPN_STATE.write().await;
            apply_connected(&mut state, "Connected");
            state.active_profile_id = Some("personal".into());
        }
        disconnect_vpn(&RecordingVpnProcess::new())
            .await
            .expect("disconnect");
        let state = VPN_STATE.read().await;
        assert_eq!(state.state, VpnState::Disconnected);
        clear_exec_fixtures();
    }
}
