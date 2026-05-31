//! Read-only `Network.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};

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
