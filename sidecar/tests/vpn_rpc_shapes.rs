//! Read-only `Vpn.*` RPC shapes and VPN CLI fixture parsers.

mod common;

use ags_sidecar::contract_parsers::{
    ip_link_interface_up, load_vpn_profile_defs_from_dir, parse_iface_ipv4, parse_wireguard_conf,
    vpn_interface_connected,
};
use common::{
    assert_json_object_keys, assert_safe_rpc_method, call_method, call_method_unchecked,
    is_denied_rpc_method, load_fixture, test_registry,
};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn vpn_ip_link_and_wireguard_fixture_parsers() {
    let link = load_fixture("vpn/ip_link_multi.txt");
    assert!(ip_link_interface_up(&link, "wg0"));
    assert!(!ip_link_interface_up(&link, "eth0"));
    assert!(!ip_link_interface_up(&link, "docker0"));

    let addr = load_fixture("vpn/ip_o_addr_show_connected.txt");
    assert!(vpn_interface_connected(&addr, "tun0", false));
    assert_eq!(parse_iface_ipv4(&addr, "tun0").as_deref(), Some("10.0.0.2"));

    let wg = load_fixture("vpn/wg_home.conf");
    let summary = parse_wireguard_conf(&wg);
    assert!(!summary.addresses.is_empty());
    assert!(summary.endpoint.is_some());
}

#[test]
fn vpn_tun_only_and_nm_profile_fixtures() {
    let tun = load_fixture("vpn/ip_o_addr_show_tun_only.txt");
    assert!(vpn_interface_connected(&tun, "tun1", false));
    assert_eq!(parse_iface_ipv4(&tun, "tun1").as_deref(), Some("10.0.0.3"));
    assert!(!vpn_interface_connected(&tun, "eth0", false));

    let nm = load_fixture("vpn/ip_o_addr_show_nm_profile.txt");
    assert_eq!(parse_iface_ipv4(&nm, "example-exit").as_deref(), Some("192.168.1.5"));
}

#[test]
fn vpn_wireguard_minimal_fixture() {
    let wg = load_fixture("vpn/wg_minimal.conf");
    let summary = parse_wireguard_conf(&wg);
    assert_eq!(summary.addresses, vec!["192.168.6.2/32"]);
    assert_eq!(summary.endpoint.as_deref(), Some("vpn.example.com:443"));
}

#[test]
fn vpn_profiles_load_from_fixture_config_dir() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vpn");
    let profiles = load_vpn_profile_defs_from_dir(&dir);
    assert!(profiles.iter().any(|p| p.id == "lab-wg"));
    let first = &profiles[0];
    for key in ["id", "name", "icon", "display_name", "interface", "requires_credentials"] {
        assert!(!first.id.is_empty(), "missing profile field {key}");
    }
}

#[test]
fn vpn_connect_and_disconnect_are_denied_in_default_harness() {
    assert!(is_denied_rpc_method("Vpn.Connect"));
    assert!(is_denied_rpc_method("Vpn.Disconnect"));
    let connect_panic =
        std::panic::catch_unwind(|| assert_safe_rpc_method("Vpn.Connect"));
    assert!(connect_panic.is_err());
}

#[tokio::test]
async fn vpn_get_status_and_profiles_schema() {
    let registry = test_registry();

    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    for key in ["state", "message"] {
        assert!(status.get(key).is_some(), "missing {key}");
    }
    assert!(status.get("profile_id").is_some());

    let profiles = call_method(&registry, "Vpn.GetProfiles", None)
        .await
        .expect("Vpn.GetProfiles");
    let arr = profiles.as_array().expect("profiles array");
    assert!(!arr.is_empty());
    let first = &arr[0];
    for key in ["id", "name", "icon", "display_name", "interface", "requires_credentials"] {
        assert!(first.get(key).is_some(), "missing profile field {key}");
    }
}

#[tokio::test]
async fn vpn_get_status_disconnected_safe_without_vpn() {
    let registry = test_registry();
    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    assert_json_object_keys(&status, &["state", "message", "profile_id"]);
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("disconnected")
    );
    assert!(
        status.get("profile_id").map(|v| v.is_null()).unwrap_or(false),
        "profile_id should be null when disconnected"
    );
    assert!(
        status.get("interface").is_none() || status.get("interface").unwrap().is_null(),
        "interface omitted when disconnected"
    );
    assert!(
        status.get("local_ip").is_none() || status.get("local_ip").unwrap().is_null(),
        "local_ip omitted when disconnected"
    );
}

#[tokio::test]
async fn vpn_status_state_enum_and_profiles_count() {
    let registry = test_registry();

    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    let state = status
        .get("state")
        .and_then(|v| v.as_str())
        .expect("state");
    assert!(
        matches!(
            state,
            "disconnected" | "connecting" | "connected" | "error"
        ),
        "unexpected state: {state}"
    );
    assert!(status.get("message").and_then(|v| v.as_str()).is_some());

    let profiles = call_method(&registry, "Vpn.GetProfiles", None)
        .await
        .expect("Vpn.GetProfiles");
    let arr = profiles.as_array().expect("profiles");
    assert!(arr.len() >= 3);
    let ids: Vec<_> = arr
        .iter()
        .filter_map(|p| p.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"education"));
    assert!(ids.contains(&"personal"));
}

/// `Vpn.Connect` / `Vpn.Disconnect` are deny-listed; dry-run avoids host mutation.
#[tokio::test]
#[ignore = "mutates VPN state; set AURA_VPN_DRY_RUN=1"]
async fn vpn_connect_dry_run_transitions_to_connected() {
    std::env::set_var("AURA_VPN_DRY_RUN", "1");
    let registry = test_registry();
    let connect = call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profile_id": "personal" })),
    )
    .await;
    std::env::remove_var("AURA_VPN_DRY_RUN");
    let value = connect.expect("Vpn.Connect dry-run");
    assert_eq!(value.get("success").and_then(|v| v.as_bool()), Some(true));

    std::env::set_var("AURA_VPN_DRY_RUN", "1");
    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus after connect");
    std::env::remove_var("AURA_VPN_DRY_RUN");
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("connected")
    );
}

#[tokio::test]
#[ignore = "requires temp vpn config dir with fake wireguard profile"]
async fn vpn_connect_wireguard_missing_config_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_dir = dir.path().to_path_buf();
    std::fs::write(
        config_dir.join("profiles.json"),
        r#"[
          {
            "id": "fake-wg",
            "name": "Fake",
            "icon": "shield",
            "display_name": "Fake WG",
            "interface": "wg-test",
            "requires_credentials": false,
            "protocol": "wireguard",
            "config": "missing.conf"
          }
        ]"#,
    )
    .expect("write profiles.json");

    std::env::set_var("AURA_VPN_CONFIG_DIR", config_dir.to_string_lossy().to_string());
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profileId": "fake-wg" })),
    )
    .await
    .expect_err("connect should fail without config file");
    std::env::remove_var("AURA_VPN_CONFIG_DIR");

    assert!(err.to_string().contains("WireGuard config not found"));
}

#[tokio::test]
#[ignore = "requires temp vpn config dir; exercises credential error path"]
async fn vpn_connect_openconnect_missing_credentials_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_dir = dir.path().to_path_buf();
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vpn/profiles.json"),
        config_dir.join("profiles.json"),
    )
    .expect("copy profiles.json");

    std::env::set_var("AURA_VPN_CONFIG_DIR", config_dir.to_string_lossy().to_string());
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profile_id": "education" })),
    )
    .await
    .expect_err("connect should fail without credentials");
    std::env::remove_var("AURA_VPN_CONFIG_DIR");

    assert!(err.to_string().contains("Missing credentials"));

    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("error")
    );
}
