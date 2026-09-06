//! Read-only Security RPC shape tests (no host mutation).

mod common;

use common::{call_method, test_registry};
use serde_json::Value;

fn assert_json_object_keys(value: &Value, keys: &[&str]) {
    let obj = value.as_object().expect("JSON object");
    for key in keys {
        assert!(obj.contains_key(*key), "missing key {key}");
    }
}

fn assert_security_log_array(value: &Value) {
    let arr = value.as_array().expect("array");
    for row in arr {
        assert_json_object_keys(row, &["timestamp", "level", "message", "source"]);
    }
}

fn host_tool_missing(err: &anyhow::Error) -> bool {
    err.to_string().contains("Command failed")
}

#[tokio::test]
async fn security_get_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetStatus", None)
        .await
        .expect("Security.GetStatus");
    assert_json_object_keys(
        &value,
        &[
            "firewall_enabled",
            "ssh_enabled",
            "encryption_enabled",
            "fail2ban_active",
            "clamav_installed",
            "fprintd_available",
        ],
    );
}

#[tokio::test]
async fn security_get_firewall_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetFirewallStatus", None)
        .await
        .expect("Security.GetFirewallStatus");
    assert_json_object_keys(&value, &["enabled", "type"]);
}

#[tokio::test]
async fn security_get_firewall_rules_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetFirewallRules", None)
        .await
        .expect("Security.GetFirewallRules");
    assert!(value.is_array());
}

#[tokio::test]
async fn security_get_encryption_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetEncryptionStatus", None)
        .await
        .expect("Security.GetEncryptionStatus");
    assert_json_object_keys(&value, &["encrypted", "devices"]);
    assert!(value.get("devices").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn security_get_failed_logins_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetFailedLogins", None)
        .await
        .expect("Security.GetFailedLogins");
    assert_security_log_array(&value);
}

#[tokio::test]
async fn security_get_ssh_status_or_missing_tool() {
    let registry = test_registry();
    match call_method(&registry, "Security.GetSshStatus", None).await {
        Ok(value) => assert_json_object_keys(&value, &["active", "enabled"]),
        Err(e) if host_tool_missing(&e) => {}
        Err(e) => panic!("Security.GetSshStatus: {e}"),
    }
}

#[tokio::test]
async fn security_get_sudo_logs_or_missing_auth_log() {
    let registry = test_registry();
    match call_method(&registry, "Security.GetSudoLogs", None).await {
        Ok(value) => assert_security_log_array(&value),
        Err(e) if host_tool_missing(&e) => {}
        Err(e) => panic!("Security.GetSudoLogs: {e}"),
    }
}

#[tokio::test]
async fn security_get_keyring_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetKeyringStatus", None)
        .await
        .expect("Security.GetKeyringStatus");
    assert_json_object_keys(&value, &["available"]);
    assert!(value.get("available").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn security_get_certificates_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetCertificates", None)
        .await
        .expect("Security.GetCertificates");
    let arr = value.as_array().expect("certificate array");
    for name in arr {
        let s = name.as_str().expect("certificate filename");
        assert!(
            s.ends_with(".crt") || s.ends_with(".pem") || s.ends_with(".CRT") || s.ends_with(".PEM"),
            "unexpected cert extension: {s}"
        );
    }
}

#[tokio::test]
async fn security_get_vpn_connections_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetVpnConnections", None)
        .await
        .expect("Security.GetVpnConnections");
    assert!(value.is_array());
}

#[tokio::test]
async fn security_list_fingerprints_array_or_empty() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.ListFingerprints", None)
        .await
        .expect("Security.ListFingerprints");
    let arr = value.as_array().expect("array");
    for row in arr {
        assert_json_object_keys(row, &["id", "name"]);
    }
}

#[tokio::test]
async fn security_get_password_policy_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetPasswordPolicy", None)
        .await
        .expect("Security.GetPasswordPolicy");
    assert!(value.get("max_days_between_change").is_some());
    assert!(value.get("password_expired").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn security_get_ssh_connections_or_missing_ss() {
    let registry = test_registry();
    match call_method(&registry, "Security.GetSshConnections", None).await {
        Ok(value) => {
            let arr = value.as_array().expect("ssh connections array");
            for row in arr {
                assert_json_object_keys(row, &["user", "host", "port", "pid"]);
            }
        }
        Err(e) if host_tool_missing(&e) => {}
        Err(e) => panic!("Security.GetSshConnections: {e}"),
    }
}
