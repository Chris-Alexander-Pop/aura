//! Read-only `Network.*` RPC response shapes and connect safety contracts.

mod common;

use ags_sidecar::contract_parsers::{map_nmcli_connect_error, parse_saved_connections};
use common::{
    assert_safe_rpc_method, call_method, call_method_unchecked, denied_rpc_reason, load_fixture,
    test_registry,
};
use serde_json::json;

#[tokio::test]
async fn network_get_status_matches_network_status_contract() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.GetStatus", None)
        .await
        .expect("Network.GetStatus");
    let obj = value.as_object().expect("object");
    for key in ["wifi_enabled", "connection_type", "ethernet_connected", "active_connection"] {
        assert!(obj.contains_key(key), "missing {key}");
    }
    let ctype = obj
        .get("connection_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        matches!(ctype, "none" | "wifi" | "ethernet"),
        "unexpected connection_type: {ctype}"
    );
}

#[tokio::test]
async fn network_list_saved_returns_saved_network_rows() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.ListSaved", None)
        .await
        .expect("Network.ListSaved");
    let rows = value.as_array().expect("array");
    for row in rows {
        let obj = row.as_object().expect("object");
        for key in ["name", "uuid", "autoconnect"] {
            assert!(obj.contains_key(key), "missing {key}");
        }
    }
}

#[test]
fn network_connect_is_denied_in_default_harness() {
    let err = std::panic::catch_unwind(|| assert_safe_rpc_method("Network.Connect"));
    assert!(err.is_err());
    assert_eq!(
        denied_rpc_reason("Network.Connect"),
        Some("mutates NetworkManager connections")
    );
}

#[test]
fn network_connect_error_fixtures_map_to_ui_messages() {
    let not_found = load_fixture("nmcli/connect_error_not_found.txt");
    assert_eq!(
        map_nmcli_connect_error(&not_found),
        "Wi‑Fi network not in range — scan again"
    );
    let secrets = load_fixture("nmcli/connect_error_secrets.txt");
    assert_eq!(
        map_nmcli_connect_error(&secrets),
        "Password required for this network"
    );
    assert!(ags_sidecar::contract_parsers::connect_error_needs_password(
        "Password required for this network"
    ));
    assert!(ags_sidecar::contract_parsers::connect_error_needs_password(
        "Incorrect Wi‑Fi password"
    ));
    assert!(!ags_sidecar::contract_parsers::connect_error_needs_password(
        "Wi‑Fi network not in range — scan again"
    ));
}

#[test]
fn network_saved_connections_parser_edge_cases() {
    let fixture = load_fixture("network/nmcli_saved_connections.txt");
    let saved = parse_saved_connections(&fixture);
    assert_eq!(saved.len(), 3);
    assert!(!saved.iter().any(|s| s.name.contains("Ethernet")));

    let edge = "short\nOffice:uuid-o:802-11-wireless:yes\n";
    let edge_saved = parse_saved_connections(edge);
    assert_eq!(edge_saved.len(), 1);
    assert_eq!(edge_saved[0].name, "Office");
    assert!(edge_saved[0].autoconnect);
}

/// Live join to a test AP — never run in CI/default `cargo test`.
///
/// `AURA_NETWORK_TEST_SSID=MyTestNet cargo test network_connect_test_ssid_live -- --ignored`
#[tokio::test]
#[ignore = "requires AURA_NETWORK_TEST_SSID and mutates NetworkManager"]
async fn network_connect_test_ssid_live() {
    let ssid = std::env::var("AURA_NETWORK_TEST_SSID")
        .expect("set AURA_NETWORK_TEST_SSID to a known test access point");
    let password = std::env::var("AURA_NETWORK_TEST_PASSWORD").ok();

    let registry = test_registry();
    let mut params = json!({ "ssid": ssid });
    if let Some(pass) = password {
        params["password"] = json!(pass);
    }

    let result = call_method_unchecked(&registry, "Network.Connect", Some(params))
        .await
        .expect("Network.Connect RPC");
    let success = result.get("success").and_then(|v| v.as_bool());
    if success == Some(true) {
        return;
    }
    let err = result
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown error");
    panic!("Network.Connect failed for {ssid}: {err}");
}
