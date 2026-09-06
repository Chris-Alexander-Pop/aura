use crate::services::ServiceRegistry;
use crate::utils::keyring;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub action: String,
    pub direction: String,
    pub protocol: String,
    pub port: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub user: String,
    pub host: String,
    pub port: u16,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLog {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

pub(crate) fn parse_ufw_active(output: &str) -> bool {
    output.contains("Status: active")
}

pub(crate) fn parse_firewalld_running(output: &str) -> bool {
    output.trim() == "running"
}

pub(crate) fn parse_systemd_active(output: &str) -> bool {
    output.trim() == "active"
}

pub(crate) fn parse_lsblk_luks(output: &str) -> bool {
    output.lines().any(|l| l.contains("crypto_LUKS"))
}

pub(crate) fn parse_command_found(output: &str) -> bool {
    !output.trim().is_empty()
}

pub fn parse_systemd_enabled(output: &str) -> bool {
    output.trim() == "enabled"
}

pub fn firewall_status_from_ufw(output: &str) -> serde_json::Value {
    serde_json::json!({
        "enabled": parse_ufw_active(output),
        "type": "ufw"
    })
}

pub fn firewall_status_from_firewalld(output: &str) -> serde_json::Value {
    serde_json::json!({
        "enabled": parse_firewalld_running(output),
        "type": "firewalld"
    })
}

pub fn firewall_status_none() -> serde_json::Value {
    serde_json::json!({
        "enabled": false,
        "type": "none"
    })
}

pub fn keyring_status_json(available: bool, unlocked: bool, message: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "available": available,
        "unlocked": unlocked,
        "message": message
    })
}

pub(crate) fn is_certificate_filename(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".crt") || lower.ends_with(".pem")
}

pub fn filter_certificate_filenames(names: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    names
        .into_iter()
        .filter(|name| is_certificate_filename(name.as_ref()))
        .map(|name| name.as_ref().to_string())
        .collect()
}

pub fn vpn_connections_from_pgrep(openconnect_running: bool, openvpn_running: bool) -> Vec<String> {
    let mut connections = Vec::new();
    if openconnect_running {
        connections.push("OpenConnect".to_string());
    }
    if openvpn_running {
        connections.push("OpenVPN".to_string());
    }
    connections
}

pub(crate) fn parse_ufw_rule_line(line: &str, id: usize) -> Option<FirewallRule> {
    let action = ["ALLOW", "DENY", "REJECT"]
        .into_iter()
        .find(|candidate| line.contains(candidate))?;
    let idx = line.find(action)?;
    let before = line[..idx].trim();
    let after = line[idx + action.len()..].trim();
    let after_parts: Vec<&str> = after.split_whitespace().collect();
    if after_parts.is_empty() {
        return None;
    }

    let direction = after_parts[0].to_string();
    let mut protocol = "any".to_string();
    let mut port = None;

    for token in before.split_whitespace().rev() {
        let clean = token.trim_start_matches('[').trim_end_matches(']');
        if let Some((p, proto)) = clean.split_once('/') {
            port = Some(p.to_string());
            protocol = proto.to_string();
            break;
        }
    }

    if after_parts.len() >= 2 {
        let token = after_parts[1];
        if let Some((p, proto)) = token.split_once('/') {
            let looks_like_port_proto = proto.eq_ignore_ascii_case("tcp")
                || proto.eq_ignore_ascii_case("udp")
                || p.chars().all(|c| c.is_ascii_digit());
            if looks_like_port_proto && port.is_none() {
                port = Some(p.to_string());
                protocol = proto.to_string();
            }
        } else if token.eq_ignore_ascii_case("tcp") || token.eq_ignore_ascii_case("udp") {
            protocol = token.to_string();
            if after_parts.len() >= 3 {
                port = Some(after_parts[2].to_string());
            }
        }
    }

    if port.is_none() && after_parts.len() < 2 {
        return None;
    }

    Some(FirewallRule {
        id: id.to_string(),
        action: action.to_string(),
        direction,
        protocol,
        port,
        source: None,
        destination: None,
    })
}

pub fn parse_ufw_numbered_rules(output: &str) -> Vec<FirewallRule> {
    let mut rules = Vec::new();
    for line in output.lines() {
        if let Some(rule) = parse_ufw_rule_line(line, rules.len()) {
            rules.push(rule);
        }
    }
    rules
}

pub fn ssh_status_json(active_output: &str, enabled: bool) -> serde_json::Value {
    serde_json::json!({
        "active": parse_systemd_active(active_output),
        "enabled": enabled
    })
}

pub fn parse_ssh_connections(output: &str) -> Vec<SshConnection> {
    let mut connections = Vec::new();
    for line in output.lines() {
        if line.contains(":22") || line.contains("ssh") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Some(addr) = parts.get(4) {
                    if let Some((host, port)) = addr.split_once(':') {
                        if let Ok(port_num) = port.parse::<u16>() {
                            connections.push(SshConnection {
                                user: "unknown".to_string(),
                                host: host.to_string(),
                                port: port_num,
                                pid: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    connections
}

fn security_log_line(line: &str, level: &str, source: &str) -> SecurityLog {
    SecurityLog {
        timestamp: "unknown".to_string(),
        level: level.to_string(),
        message: line.to_string(),
        source: source.to_string(),
    }
}

pub fn parse_failed_login_lines(output: &str, source: &str) -> Vec<SecurityLog> {
    output
        .lines()
        .map(|line| security_log_line(line, "warning", source))
        .collect()
}

pub fn parse_sudo_log_lines(output: &str) -> Vec<SecurityLog> {
    output
        .lines()
        .map(|line| security_log_line(line, "info", "auth.log"))
        .collect()
}

pub fn parse_encryption_devices(output: &str) -> (bool, Vec<String>) {
    let mut encrypted = false;
    let mut devices = Vec::new();
    for line in output.lines() {
        if line.contains("crypto_LUKS") {
            encrypted = true;
            if let Some(device) = line.split_whitespace().next() {
                devices.push(device.to_string());
            }
        }
    }
    (encrypted, devices)
}

pub fn parse_ss_listening_ports(output: &str) -> Vec<String> {
    let mut ports = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            if let Some(addr) = parts.get(4) {
                if let Some((_, port)) = addr.rsplit_once(':') {
                    ports.push(port.to_string());
                }
            }
        }
    }
    ports
}

async fn probe_firewall_enabled() -> bool {
    if let Ok(output) = process::exec_command(&["ufw", "status"]).await {
        return parse_ufw_active(&output);
    }
    if let Ok(output) = process::exec_command(&["firewall-cmd", "--state"]).await {
        return parse_firewalld_running(&output);
    }
    false
}

async fn probe_ssh_enabled() -> bool {
    if let Ok(output) = process::exec_command(&["systemctl", "is-active", "sshd"]).await {
        return parse_systemd_active(&output);
    }
    false
}

async fn probe_encryption_enabled() -> bool {
    if let Ok(output) = process::exec_command(&["lsblk", "-f"]).await {
        return parse_lsblk_luks(&output);
    }
    false
}

pub(crate) async fn probe_fail2ban_active() -> bool {
    if let Ok(output) = process::exec_command(&["systemctl", "is-active", "fail2ban"]).await {
        return parse_systemd_active(&output);
    }
    false
}

pub(crate) async fn probe_clamav_installed() -> bool {
    process::exec_command(&["command", "-v", "clamscan"])
        .await
        .map(|o| parse_command_found(&o))
        .unwrap_or(false)
}

pub(crate) async fn probe_fprintd_available() -> bool {
    process::exec_command(&["command", "-v", "fprintd-list"])
        .await
        .map(|o| parse_command_found(&o))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintEntry {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyStatus {
    pub max_days_between_change: Option<u32>,
    pub min_days_between_change: Option<u32>,
    pub warn_days_before_expiry: Option<u32>,
    pub password_expired: bool,
    pub account_locked: bool,
}

/// Parse `fprintd-list` fingerprint lines (`#N: name`).
pub fn parse_fprintd_list(output: &str) -> Vec<FingerprintEntry> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix('#') else {
            continue;
        };
        let Some((id_part, name)) = rest.split_once(':') else {
            continue;
        };
        if let Ok(id) = id_part.trim().parse::<u32>() {
            entries.push(FingerprintEntry {
                id,
                name: name.trim().to_string(),
            });
        }
    }
    entries
}

/// Parse `chage -l` password aging fields.
pub fn parse_chage_l(output: &str) -> PasswordPolicyStatus {
    let mut max_days = None;
    let mut min_days = None;
    let mut warn_days = None;
    let mut password_expired = false;

    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("maximum number of days between password change") {
            max_days = line
                .split(':')
                .nth(1)
                .and_then(|v| v.trim().parse().ok());
        } else if lower.contains("minimum number of days between password change") {
            min_days = line
                .split(':')
                .nth(1)
                .and_then(|v| v.trim().parse().ok());
        } else if lower.contains("number of days of warning before password expires") {
            warn_days = line
                .split(':')
                .nth(1)
                .and_then(|v| v.trim().parse().ok());
        } else if lower.contains("password expires") {
            if let Some(v) = line.split(':').nth(1) {
                password_expired = v.trim().eq_ignore_ascii_case("password must be changed");
            }
        }
    }

    PasswordPolicyStatus {
        max_days_between_change: max_days,
        min_days_between_change: min_days,
        warn_days_before_expiry: warn_days,
        password_expired,
        account_locked: false,
    }
}

/// Parse `passwd -S` status line (`user STATUS ...`).
pub fn parse_passwd_s(output: &str) -> PasswordPolicyStatus {
    let mut policy = PasswordPolicyStatus {
        max_days_between_change: None,
        min_days_between_change: None,
        warn_days_before_expiry: None,
        password_expired: false,
        account_locked: false,
    };
    let line = output.lines().next().unwrap_or("");
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        match parts[1] {
            "P" => {}
            "L" => policy.account_locked = true,
            "NP" => {}
            "LK" => policy.account_locked = true,
            _ => {}
        }
    }
    if parts.len() >= 6 {
        policy.min_days_between_change = parts[3].parse().ok();
        policy.max_days_between_change = parts[4].parse().ok();
        policy.warn_days_before_expiry = parts[5].parse().ok();
    }
    policy
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.GetFirewallStatus", |_params| async move {
        if let Ok(output) = process::exec_command(&["ufw", "status"]).await {
            Ok(firewall_status_from_ufw(&output))
        } else if let Ok(output) = process::exec_command(&["firewall-cmd", "--state"]).await {
            Ok(firewall_status_from_firewalld(&output))
        } else {
            Ok(firewall_status_none())
        }
    });

    registry.register("Security.EnableFirewall", |_params| async move {
        let enabled = if process::exec_command(&["command", "-v", "ufw"]).await.is_ok() {
            crate::utils::polkit::run_privileged(&["ufw", "enable"]).await?;
            probe_firewall_enabled().await
        } else if process::exec_command(&["command", "-v", "firewall-cmd"]).await.is_ok() {
            crate::utils::polkit::run_privileged(&["systemctl", "start", "firewalld"]).await?;
            probe_firewall_enabled().await
        } else {
            anyhow::bail!("no supported firewall tool found")
        };
        Ok(serde_json::json!({ "success": enabled, "enabled": enabled }))
    });

    registry.register("Security.DisableFirewall", |_params| async move {
        let disabled = if process::exec_command(&["command", "-v", "ufw"]).await.is_ok() {
            crate::utils::polkit::run_privileged(&["ufw", "disable"]).await?;
            !probe_firewall_enabled().await
        } else if process::exec_command(&["command", "-v", "firewall-cmd"]).await.is_ok() {
            crate::utils::polkit::run_privileged(&["systemctl", "stop", "firewalld"]).await?;
            !probe_firewall_enabled().await
        } else {
            anyhow::bail!("no supported firewall tool found")
        };
        Ok(serde_json::json!({ "success": disabled, "enabled": !disabled }))
    });

    registry.register("Security.GetFirewallRules", |_params| async move {
        let rules = if let Ok(output) = process::exec_command(&["ufw", "status", "numbered"]).await {
            parse_ufw_numbered_rules(&output)
        } else {
            Vec::new()
        };
        Ok(serde_json::to_value(&rules)?)
    });

    registry.register("Security.AddFirewallRule", |params| async move {
        let rule: FirewallRule = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("rule").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing rule"))?,
        )?;

        let mut cmd = vec!["ufw"];
        cmd.push(&rule.action);
        cmd.push(&rule.direction);
        if let Some(ref port) = rule.port {
            cmd.push(port);
        }
        if let Some(ref source) = rule.source {
            cmd.push("from");
            cmd.push(source);
        }

        process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Security.RemoveFirewallRule", |params| async move {
        let rule_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("rule_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing rule_id"))?,
        )?;

        process::exec_command(&["ufw", "delete", &rule_id]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Security.GetSshStatus", |_params| async move {
        let output = process::exec_command(&["systemctl", "is-active", "sshd"]).await?;
        let enabled = process::exec_command(&["systemctl", "is-enabled", "sshd"])
            .await
            .map(|o| parse_systemd_enabled(&o))
            .unwrap_or(false);
        Ok(ssh_status_json(&output, enabled))
    });

    registry.register("Security.GetSshConnections", |_params| async move {
        let output = process::exec_command(&["ss", "-tnp"]).await?;
        Ok(serde_json::to_value(&parse_ssh_connections(&output))?)
    });

    registry.register("Security.GetFailedLogins", |_params| async move {
        let mut logs = Vec::new();

        if let Ok(output) = process::exec_command(&["grep", "Failed password", "/var/log/auth.log"]).await {
            logs.extend(parse_failed_login_lines(&output, "auth.log"));
        }

        if let Ok(output) = process::exec_command(&["grep", "Failed password", "/var/log/secure"]).await {
            logs.extend(parse_failed_login_lines(&output, "secure"));
        }

        Ok(serde_json::to_value(&logs)?)
    });

    registry.register("Security.GetSudoLogs", |_params| async move {
        let output = process::exec_command(&["grep", "sudo", "/var/log/auth.log"]).await?;
        Ok(serde_json::to_value(&parse_sudo_log_lines(&output))?)
    });

    registry.register("Security.ScanPorts", |_params| async move {
        let output = process::exec_command(&["ss", "-tuln"]).await?;
        Ok(serde_json::to_value(&parse_ss_listening_ports(&output))?)
    });

    registry.register("Security.GetEncryptionStatus", |_params| async move {
        let (encrypted, devices) = if let Ok(output) = process::exec_command(&["lsblk", "-f"]).await {
            parse_encryption_devices(&output)
        } else {
            (false, Vec::new())
        };
        Ok(serde_json::json!({
            "encrypted": encrypted,
            "devices": devices
        }))
    });

    registry.register("Security.GetKeyringStatus", |_params| async move {
        let probe = keyring::probe_keyring().await;
        Ok(keyring_status_json(
            probe.available,
            probe.unlocked,
            probe.message.as_deref(),
        ))
    });

    registry.register("Security.GetCertificates", |_params| async move {
        let mut names = Vec::new();

        // Check common certificate locations
        let home = std::env::var("HOME").unwrap_or_default();
        let cert_dirs: Vec<String> = vec![
            "/etc/ssl/certs".to_string(),
            "/usr/share/ca-certificates".to_string(),
            format!("{}/.local/share/certificates", home),
        ];

        for dir in &cert_dirs {
            if let Ok(entries) = tokio::fs::read_dir(&dir).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        names.push(file_name);
                    }
                }
            }
        }

        Ok(serde_json::to_value(&filter_certificate_filenames(names))?)
    });

    registry.register("Security.GetVpnConnections", |_params| async move {
        let openconnect_running = process::exec_command(&["pgrep", "openconnect"])
            .await
            .is_ok();
        let openvpn_running = process::exec_command(&["pgrep", "openvpn"]).await.is_ok();
        Ok(serde_json::to_value(&vpn_connections_from_pgrep(
            openconnect_running,
            openvpn_running,
        ))?)
    });

    registry.register("Security.GetStatus", |_params| async move {
        Ok(serde_json::json!({
            "firewall_enabled": probe_firewall_enabled().await,
            "ssh_enabled": probe_ssh_enabled().await,
            "encryption_enabled": probe_encryption_enabled().await,
            "fail2ban_active": probe_fail2ban_active().await,
            "clamav_installed": probe_clamav_installed().await,
            "fprintd_available": probe_fprintd_available().await,
        }))
    });

    registry.register("Security.ListFingerprints", |_params| async move {
        if !probe_fprintd_available().await {
            return Ok(serde_json::json!([]));
        }
        let output = process::exec_command(&["fprintd-list"]).await?;
        Ok(serde_json::to_value(parse_fprintd_list(&output))?)
    });

    registry.register("Security.GetPasswordPolicy", |_params| async move {
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        if let Ok(output) = process::exec_command(&["chage", "-l", &user]).await {
            return Ok(serde_json::to_value(parse_chage_l(&output))?);
        }
        let output = process::exec_command(&["passwd", "-S", &user]).await?;
        Ok(serde_json::to_value(parse_passwd_s(&output))?)
    });

    registry.register("Security.RunClamScan", |params| async move {
        let path: String = params
            .as_ref()
            .and_then(|p| p.get("path").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_else(|| ".".to_string());
        if !probe_clamav_installed().await {
            anyhow::bail!("clamscan not installed");
        }
        let output = crate::utils::polkit::run_privileged(&["clamscan", "-r", &path]).await?;
        Ok(serde_json::json!({ "success": true, "output": output }))
    });
}

#[cfg(test)]
mod probe_tests {
    use super::*;

    #[test]
    fn parse_ufw_status_active() {
        assert!(parse_ufw_active("Status: active\n"));
        assert!(!parse_ufw_active("Status: inactive\n"));
    }

    #[test]
    fn parse_firewalld_and_systemd() {
        assert!(parse_firewalld_running("running"));
        assert!(!parse_firewalld_running("stopped"));
        assert!(parse_systemd_active("active\n"));
        assert!(!parse_systemd_active("inactive"));
    }

    #[test]
    fn parse_lsblk_luks_line() {
        let sample = "sda1  crypto_LUKS  ext4  /\n";
        assert!(parse_lsblk_luks(sample));
        assert!(!parse_lsblk_luks("sda1  ext4  /\n"));
    }

    #[test]
    fn parse_command_found_nonempty() {
        assert!(parse_command_found("/usr/bin/clamscan\n"));
        assert!(!parse_command_found("   \n"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NmapPort {
    port: u16,
    protocol: String,
    state: String,
    service: String,
    version: Option<String>,
}

pub(crate) fn parse_nmap_xml(xml_content: &str) -> Result<Vec<NmapPort>> {
    let mut ports = Vec::new();
    
    // Simple XML parsing - extract port information
    // In production, use proper XML parser
    let port_re = regex::Regex::new(r#"<port protocol="(\w+)" portid="(\d+)">"#)?;
    let state_re = regex::Regex::new(r#"<state state="(\w+)""#)?;
    let service_re = regex::Regex::new(r#"<service name="([^"]+)""#)?;
    
    for cap in port_re.captures_iter(xml_content) {
        if let (Some(protocol), Some(port_str)) = (cap.get(1), cap.get(2)) {
            if let Ok(port) = port_str.as_str().parse::<u16>() {
                let mut state = "unknown".to_string();
                let mut service = "unknown".to_string();
                
                // Find state and service in nearby text
                if let Some(state_cap) = state_re.find(xml_content) {
                    state = state_cap.as_str().to_string();
                }
                if let Some(service_cap) = service_re.find(xml_content) {
                    service = service_cap.as_str().to_string();
                }
                
                ports.push(NmapPort {
                    port,
                    protocol: protocol.as_str().to_string(),
                    state,
                    service,
                    version: None,
                });
            }
        }
    }
    
    Ok(ports)
}

pub fn nmap_ports_as_json(xml_content: &str) -> Result<Vec<serde_json::Value>> {
    parse_nmap_xml(xml_content)?
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/security/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    #[test]
    fn parse_ufw_status_active_inactive() {
        assert!(parse_ufw_active(&fixture("ufw_status_active.txt")));
        assert!(!parse_ufw_active(&fixture("ufw_status_inactive.txt")));
    }

    #[test]
    fn parse_firewalld_and_systemd() {
        assert!(parse_firewalld_running(&fixture("firewalld_running.txt")));
        assert!(!parse_firewalld_running("stopped"));
        assert!(parse_systemd_active(&fixture("systemctl_sshd_active.txt")));
        assert!(!parse_systemd_active("inactive"));
        assert!(parse_systemd_enabled(&fixture("systemctl_sshd_enabled.txt")));
        assert!(!parse_systemd_enabled("disabled"));
    }

    #[test]
    fn firewall_status_json_from_fixtures() {
        let ufw = firewall_status_from_ufw(&fixture("ufw_status_active.txt"));
        assert_eq!(ufw["type"], "ufw");
        assert_eq!(ufw["enabled"], true);
        let fw = firewall_status_from_firewalld(&fixture("firewalld_running.txt"));
        assert_eq!(fw["type"], "firewalld");
        assert_eq!(fw["enabled"], true);
    }

    #[test]
    fn parse_ufw_rules_skips_short_and_non_rule_lines() {
        let rules = parse_ufw_numbered_rules(&fixture("ufw_status_numbered.txt"));
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].action, "ALLOW");
        assert_eq!(rules[0].protocol, "tcp");
        assert_eq!(rules[2].action, "DENY");
    }

    #[test]
    fn parse_lsblk_and_encryption_devices() {
        let luks = fixture("lsblk_luks.txt");
        assert!(parse_lsblk_luks(&luks));
        let (encrypted, devices) = parse_encryption_devices(&luks);
        assert!(encrypted);
        assert!(devices.iter().any(|d| d.contains("nvme")));
        let (plain, devs) = parse_encryption_devices(&fixture("lsblk_plain.txt"));
        assert!(!plain);
        assert!(devs.is_empty());
    }

    #[test]
    fn parse_command_found_nonempty() {
        assert!(parse_command_found("/usr/bin/clamscan\n"));
        assert!(!parse_command_found("   \n"));
    }

    #[test]
    fn parse_ssh_status_json_fixture() {
        let active = fixture("systemctl_sshd_active.txt");
        let v = ssh_status_json(&active, true);
        assert_eq!(v["active"], true);
        assert_eq!(v["enabled"], true);
        let off = ssh_status_json("inactive", false);
        assert_eq!(off["active"], false);
        assert_eq!(off["enabled"], false);
    }

    #[test]
    fn parse_ssh_connections_fixture_and_edge_cases() {
        let conns = parse_ssh_connections(&fixture("ss_ssh_connections.txt"));
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].host, "203.0.113.5");
        assert_eq!(conns[0].port, 54321);
        assert!(parse_ssh_connections("no ssh here\n").is_empty());
    }

    #[test]
    fn parse_failed_and_sudo_log_fixtures() {
        let failed = parse_failed_login_lines(&fixture("auth_failed.txt"), "auth.log");
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].level, "warning");
        assert_eq!(failed[0].source, "auth.log");
        assert!(parse_failed_login_lines("", "auth.log").is_empty());
        let sudo = parse_sudo_log_lines(&fixture("auth_sudo.txt"));
        assert_eq!(sudo.len(), 1);
        assert_eq!(sudo[0].level, "info");
    }

    #[test]
    fn parse_ss_listening_ports_fixture() {
        let ports = parse_ss_listening_ports(&fixture("ss_tuln_ports.txt"));
        assert!(ports.contains(&"22".to_string()));
        assert!(ports.contains(&"443".to_string()));
        assert!(parse_ss_listening_ports("Netid State\nshort\n").is_empty());
    }

    #[test]
    fn parse_nmap_xml_fixture_and_empty() {
        let ports = parse_nmap_xml(&fixture("nmap_minimal.xml")).expect("xml");
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 22);
        assert_eq!(ports[0].protocol, "tcp");
        assert!(parse_nmap_xml("<nmaprun></nmaprun>").unwrap().is_empty());
        assert!(parse_nmap_xml("not xml at all").unwrap().is_empty());
        assert!(
            parse_nmap_xml(&fixture("nmap_invalid_port.xml"))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn firewall_status_none_json() {
        let none = firewall_status_none();
        assert_eq!(none["enabled"], false);
        assert_eq!(none["type"], "none");
    }

    #[test]
    fn keyring_status_json_branches() {
        assert_eq!(keyring_status_json(true, true, None)["available"], true);
        assert_eq!(keyring_status_json(true, true, None)["unlocked"], true);
        assert_eq!(keyring_status_json(false, false, Some("x"))["available"], false);
        assert_eq!(keyring_status_json(true, false, Some("locked"))["unlocked"], false);
        assert!(!parse_command_found(&fixture("which_not_found.txt")));
        assert!(parse_command_found(&fixture("which_secret_tool.txt")));
    }

    #[test]
    fn filter_certificate_filenames_fixture() {
        let listing = fixture("cert_dir_listing.txt");
        let names: Vec<&str> = listing
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        let certs = filter_certificate_filenames(names);
        assert_eq!(certs, vec!["CA.crt".to_string(), "user.PEM".to_string()]);
        assert!(!is_certificate_filename("readme.txt"));
        assert!(!is_certificate_filename("bundle.cer"));
    }

    #[test]
    fn vpn_connections_from_pgrep_branches() {
        assert!(vpn_connections_from_pgrep(false, false).is_empty());
        assert_eq!(
            vpn_connections_from_pgrep(true, false),
            vec!["OpenConnect".to_string()]
        );
        assert_eq!(
            vpn_connections_from_pgrep(false, true),
            vec!["OpenVPN".to_string()]
        );
        assert_eq!(
            vpn_connections_from_pgrep(true, true),
            vec!["OpenConnect".to_string(), "OpenVPN".to_string()]
        );
    }

    #[test]
    fn parse_ufw_bracket_and_reject_fixtures() {
        let rules = parse_ufw_numbered_rules(&fixture("ufw_status_brackets.txt"));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].action, "ALLOW");
        assert_eq!(rules[0].port.as_deref(), Some("22"));
        assert_eq!(rules[0].protocol, "tcp");

        let reject = parse_ufw_numbered_rules(&fixture("ufw_status_reject.txt"));
        assert_eq!(reject.len(), 1);
        assert_eq!(reject[0].action, "REJECT");
        assert_eq!(reject[0].direction, "IN");
    }

    #[test]
    fn parse_encryption_empty_and_mixed_fixtures() {
        let (encrypted, devices) = parse_encryption_devices(&fixture("lsblk_empty.txt"));
        assert!(!encrypted);
        assert!(devices.is_empty());

        let (mixed_enc, mixed_devs) = parse_encryption_devices(&fixture("lsblk_mixed.txt"));
        assert!(mixed_enc);
        assert_eq!(mixed_devs.len(), 1);
        assert!(mixed_devs[0].contains("nvme"));
    }

    #[test]
    fn parse_fprintd_list_fixture() {
        let entries = parse_fprintd_list(&fixture("fprintd_list.txt"));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "right-index-finger");
        assert_eq!(entries[1].id, 1);
    }

    #[test]
    fn parse_chage_l_fixture() {
        let policy = parse_chage_l(&fixture("chage_l.txt"));
        assert_eq!(policy.max_days_between_change, Some(99999));
        assert_eq!(policy.warn_days_before_expiry, Some(7));
        assert!(!policy.password_expired);
    }
}
