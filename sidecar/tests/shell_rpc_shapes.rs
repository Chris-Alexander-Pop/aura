//! Read-only `Sidecar.*` RPC response shapes.

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
