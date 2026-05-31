//! Read-only `Capture.ListDevices` shape smoke.

mod common;

use common::{call_rpc, test_registry};

#[tokio::test]
async fn capture_list_devices_returns_audio_array() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Capture.ListDevices", None)
        .await
        .expect("Capture.ListDevices");
    assert!(value.get("audio").and_then(|v| v.as_array()).is_some());
    assert!(value.get("video").and_then(|v| v.as_array()).is_some());
}
