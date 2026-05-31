//! Read-only `Vpn.*` RPC shapes and VPN CLI fixture parsers.

mod common;

use ags_sidecar::contract_parsers::{
    ip_link_interface_up, parse_wireguard_conf, vpn_interface_connected,
};
use common::{assert_json_object_keys, call_rpc, load_fixture, test_registry};

#[test]
fn vpn_ip_link_and_wireguard_fixture_parsers() {
    let link = load_fixture("vpn/ip_link_multi.txt");
    assert!(ip_link_interface_up(&link, "wg0"));
    assert!(!ip_link_interface_up(&link, "eth0"));
    assert!(!ip_link_interface_up(&link, "docker0"));

    let addr = load_fixture("vpn/ip_o_addr_show_connected.txt");
    assert!(vpn_interface_connected(&addr, "tun0", false));

    let wg = load_fixture("vpn/wg_home.conf");
    let summary = parse_wireguard_conf(&wg);
    assert!(!summary.addresses.is_empty());
    assert!(summary.endpoint.is_some());
}

#[tokio::test]
async fn vpn_get_status_and_profiles_schema() {
    let registry = test_registry();

    let status = call_rpc(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    for key in ["state", "message"] {
        assert!(status.get(key).is_some(), "missing {key}");
    }
    assert!(status.get("profile_id").is_some());

    let profiles = call_rpc(&registry, "Vpn.GetProfiles", None)
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
async fn vpn_status_state_enum_and_profiles_count() {
    let registry = test_registry();

    let status = call_rpc(&registry, "Vpn.GetStatus", None)
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

    let profiles = call_rpc(&registry, "Vpn.GetProfiles", None)
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
