//! Golden CLI contracts for defensive security parsers (fixtures under `tests/fixtures/security/`).

mod common;

use ags_sidecar::contract_parsers::{
    filter_certificate_filenames, firewall_status_from_firewalld, firewall_status_from_ufw,
    firewall_status_none, keyring_status_json, parse_encryption_devices, parse_failed_login_lines,
    parse_nmap_xml_ports, parse_ssh_connections, parse_ss_listening_ports, parse_sudo_log_lines,
    parse_ufw_numbered_rules, ssh_status_json, vpn_connections_from_pgrep,
};
use common::load_fixture;
use serde_json::Value;

fn assert_firewall_rule_contract(v: &Value) {
    let obj = v.as_object().expect("firewall rule object");
    for key in ["id", "action", "direction", "protocol"] {
        assert!(obj.contains_key(key), "missing key {key}");
    }
}

fn assert_security_log_contract(v: &Value) {
    let obj = v.as_object().expect("security log object");
    for key in ["timestamp", "level", "message", "source"] {
        assert!(obj.contains_key(key), "missing key {key}");
    }
}

#[test]
fn contract_ufw_firewall_status_fixture() {
    let text = load_fixture("security/ufw_status_active.txt");
    let v = firewall_status_from_ufw(&text);
    assert_eq!(v["type"], "ufw");
    assert_eq!(v["enabled"], true);
}

#[test]
fn contract_ufw_numbered_rules_fixture() {
    let rules = parse_ufw_numbered_rules(&load_fixture("security/ufw_status_numbered.txt"));
    assert_eq!(rules.len(), 3);
    assert_eq!(rules[2].action, "DENY");
    for rule in &rules {
        assert_firewall_rule_contract(&serde_json::to_value(rule).unwrap());
    }
}

#[test]
fn contract_ssh_status_and_connections_fixtures() {
    let active = load_fixture("security/systemctl_sshd_active.txt");
    let status = ssh_status_json(&active, true);
    assert_eq!(status["active"], true);
    assert_eq!(status["enabled"], true);

    let conns = parse_ssh_connections(&load_fixture("security/ss_ssh_connections.txt"));
    assert_eq!(conns.len(), 1);
    let row = serde_json::to_value(&conns[0]).unwrap();
    assert!(row.get("host").and_then(|v| v.as_str()).is_some());
    assert!(row.get("port").and_then(|v| v.as_u64()).is_some());
}

#[test]
fn contract_auth_log_parsers_fixture() {
    let failed = parse_failed_login_lines(&load_fixture("security/auth_failed.txt"), "auth.log");
    assert_eq!(failed.len(), 1);
    assert_security_log_contract(&serde_json::to_value(&failed[0]).unwrap());
}

#[test]
fn contract_encryption_lsblk_fixture() {
    let (encrypted, devices) = parse_encryption_devices(&load_fixture("security/lsblk_luks.txt"));
    assert!(encrypted);
    assert!(!devices.is_empty());
    let (plain, devs) = parse_encryption_devices(&load_fixture("security/lsblk_plain.txt"));
    assert!(!plain);
    assert!(devs.is_empty());
}

#[test]
fn contract_nmap_xml_fixture() {
    let ports = parse_nmap_xml_ports(&load_fixture("security/nmap_minimal.xml")).expect("nmap xml");
    assert_eq!(ports.len(), 2);
    assert_eq!(ports[0]["port"], 22);
}

#[test]
fn contract_firewalld_inactive_fixture() {
    let fw = firewall_status_from_firewalld("stopped\n");
    assert_eq!(fw["type"], "firewalld");
    assert_eq!(fw["enabled"], false);
    let none = firewall_status_none();
    assert_eq!(none["type"], "none");
    assert_eq!(none["enabled"], false);
}

#[test]
fn contract_ufw_bracket_and_reject_fixtures() {
    let bracket = parse_ufw_numbered_rules(&load_fixture("security/ufw_status_brackets.txt"));
    assert_eq!(bracket.len(), 2);
    assert_eq!(bracket[0].port.as_deref(), Some("22"));

    let reject = parse_ufw_numbered_rules(&load_fixture("security/ufw_status_reject.txt"));
    assert_eq!(reject.len(), 1);
    assert_eq!(reject[0].action, "REJECT");
    for rule in reject {
        assert_firewall_rule_contract(&serde_json::to_value(rule).unwrap());
    }
}

#[test]
fn contract_sudo_log_fixture() {
    let sudo = parse_sudo_log_lines(&load_fixture("security/auth_sudo.txt"));
    assert_eq!(sudo.len(), 1);
    assert_security_log_contract(&serde_json::to_value(&sudo[0]).unwrap());
    assert_eq!(sudo[0].level, "info");
}

#[test]
fn contract_ss_listening_ports_fixture() {
    let ports = parse_ss_listening_ports(&load_fixture("security/ss_tuln_ports.txt"));
    assert!(ports.contains(&"22".to_string()));
    assert!(parse_ss_listening_ports("Netid State\nshort\n").is_empty());
}

#[test]
fn contract_encryption_empty_and_mixed_fixtures() {
    let (empty_enc, empty_devs) = parse_encryption_devices(&load_fixture("security/lsblk_empty.txt"));
    assert!(!empty_enc);
    assert!(empty_devs.is_empty());

    let (mixed_enc, mixed_devs) = parse_encryption_devices(&load_fixture("security/lsblk_mixed.txt"));
    assert!(mixed_enc);
    assert_eq!(mixed_devs.len(), 1);
}

#[test]
fn contract_certificate_filenames_fixture() {
    let listing = load_fixture("security/cert_dir_listing.txt");
    let names: Vec<&str> = listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let certs = filter_certificate_filenames(names);
    assert_eq!(certs.len(), 2);
    assert!(certs.iter().any(|name| name.ends_with(".crt")));
}

#[test]
fn contract_keyring_status_json() {
    assert_eq!(keyring_status_json(true)["available"], true);
    assert_eq!(keyring_status_json(false)["available"], false);
}

#[test]
fn contract_vpn_connections_from_pgrep() {
    assert!(vpn_connections_from_pgrep(false, false).is_empty());
    assert_eq!(
        vpn_connections_from_pgrep(true, true),
        vec!["OpenConnect".to_string(), "OpenVPN".to_string()]
    );
}

#[test]
fn contract_nmap_invalid_port_fixture() {
    let ports = parse_nmap_xml_ports(&load_fixture("security/nmap_invalid_port.xml")).expect("xml");
    assert!(ports.is_empty());
}
