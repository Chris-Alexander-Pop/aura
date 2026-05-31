//! Read-only shell / lock / sleep RPC response shapes.

mod common;

use common::{call_rpc, test_registry};

#[tokio::test]
async fn sidecar_get_version_nonempty_string() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Sidecar.GetVersion", None)
        .await
        .expect("Sidecar.GetVersion");
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .expect("version string");
    assert!(!version.is_empty());
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "version should look like semver: {version}"
    );
}

#[tokio::test]
async fn sleep_get_inhibitors_returns_array() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Sleep.GetInhibitors", None)
        .await
        .expect("Sleep.GetInhibitors");
    let inhibitors = value
        .get("inhibitors")
        .and_then(|v| v.as_array())
        .expect("inhibitors array");
    for entry in inhibitors {
        assert!(entry.get("what").and_then(|v| v.as_str()).is_some());
        assert!(entry.get("who").and_then(|v| v.as_str()).is_some());
        assert!(entry.get("why").and_then(|v| v.as_str()).is_some());
        assert!(entry.get("mode").and_then(|v| v.as_str()).is_some());
        assert!(entry.get("uid").and_then(|v| v.as_u64()).is_some());
        assert!(entry.get("pid").and_then(|v| v.as_u64()).is_some());
    }
}

#[tokio::test]
async fn lock_get_config_has_visibility_fields() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Lock.GetConfig", None)
        .await
        .expect("Lock.GetConfig");
    let config = value.get("config").expect("config object");
    for key in [
        "show_clock",
        "show_notifications",
        "show_calendar",
        "show_media",
    ] {
        assert!(
            config.get(key).and_then(|v| v.as_bool()).is_some(),
            "missing bool {key}"
        );
    }
}
