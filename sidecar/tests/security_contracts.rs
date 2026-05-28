//! Golden CLI contracts for defensive security parsers (fixtures under `tests/fixtures/security/`).

mod common;

use ags_sidecar::contract_parsers::{
    firewall_status_from_ufw, parse_encryption_devices, parse_failed_login_lines,
    parse_nmap_xml_ports, parse_ssh_connections, parse_ufw_numbered_rules, ssh_status_json,
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
