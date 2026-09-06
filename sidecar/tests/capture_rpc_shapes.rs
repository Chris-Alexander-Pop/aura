//! `Capture.*` RPC shapes and param validation (no live grim/wf-recorder when idle).

mod common;

use common::{call_method_unchecked, call_rpc, test_registry};
use serde_json::json;

#[tokio::test]
async fn capture_list_devices_returns_audio_array() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Capture.ListDevices", None)
        .await
        .expect("Capture.ListDevices");
    assert!(value.get("audio").and_then(|v| v.as_array()).is_some());
    assert!(value.get("video").and_then(|v| v.as_array()).is_some());
    assert!(value.get("tool_missing").and_then(|v| v.as_object()).is_some());
}

#[tokio::test]
async fn capture_screenshot_missing_mode_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "output": "file" })),
    )
    .await
    .expect_err("missing mode");
    assert!(err.to_string().contains("missing mode"));
}

#[tokio::test]
async fn capture_screenshot_missing_output_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "mode": "full" })),
    )
    .await
    .expect_err("missing output");
    assert!(err.to_string().contains("missing output"));
}

#[tokio::test]
async fn capture_screenshot_invalid_mode_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "mode": "bogus", "output": "file" })),
    )
    .await
    .expect_err("invalid mode");
    assert!(err.to_string().contains("invalid mode"));
}

#[tokio::test]
async fn capture_screenshot_invalid_output_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "mode": "full", "output": "ftp" })),
    )
    .await
    .expect_err("invalid output");
    assert!(err.to_string().contains("invalid output"));
}

#[tokio::test]
async fn capture_screenshot_window_mode_not_implemented() {
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "mode": "window", "output": "clipboard" })),
    )
    .await
    .expect("window mode response");
    assert_eq!(value["ok"], false);
    assert!(
        value["error"]
            .as_str()
            .is_some_and(|e| e.contains("window mode not implemented"))
    );
}

#[tokio::test]
async fn capture_record_stop_when_not_recording() {
    let registry = test_registry();
    let value = call_method_unchecked(&registry, "Capture.RecordStop", None)
        .await
        .expect("RecordStop idle");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"], "not recording");
}
