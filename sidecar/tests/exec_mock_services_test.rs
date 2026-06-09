//! Subprocess-mocked integration tests for priority services (audio, vpn, network, capture).
//!
//! Requires `AURA_EXEC_FIXTURE_DIR` via [`common::ExecFixtureGuard`].

mod common;

use ags_sidecar::services::audio::audio_env_test_lock;
use common::{
    call_method, call_method_unchecked, ExecFixtureGuard, load_fixture, test_registry,
};
use serde_json::json;

#[tokio::test]
async fn audio_get_devices_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(&registry, "Audio.Refresh", None)
        .await
        .expect("Audio.Refresh");
    let value = call_method(&registry, "Audio.GetDevices", None)
        .await
        .expect("Audio.GetDevices");
    let sinks = value.get("sinks").and_then(|v| v.as_array()).expect("sinks");
    assert!(!sinks.is_empty());
}

#[tokio::test]
async fn audio_get_streams_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(&registry, "Audio.Refresh", None)
        .await
        .expect("Audio.Refresh");
    let value = call_method(&registry, "Audio.GetStreams", None)
        .await
        .expect("Audio.GetStreams");
    assert!(value.is_array());
}

#[tokio::test]
async fn audio_media_now_playing_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_method(&registry, "Media.GetNowPlaying", None)
        .await
        .expect("Media.GetNowPlaying");
    assert!(value.is_object());
}

#[tokio::test]
async fn vpn_connect_disconnect_mocked() {
    std::env::set_var("AURA_VPN_DRY_RUN", "1");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profile_id": "personal" })),
    )
    .await
    .expect("Vpn.Connect");
    call_method_unchecked(&registry, "Vpn.Disconnect", None)
        .await
        .expect("Vpn.Disconnect");
    std::env::remove_var("AURA_VPN_DRY_RUN");
}

#[tokio::test]
async fn audio_set_sink_volume_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(
        &registry,
        "Audio.SetSinkVolume",
        Some(json!({ "device_id": 47, "volume": 0.5 })),
    )
    .await
    .expect("SetSinkVolume");
}

#[tokio::test]
async fn audio_set_default_device_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(
        &registry,
        "Audio.SetDefaultDevice",
        Some(json!({ "device_id": 47, "type": "sink" })),
    )
    .await
    .expect("SetDefaultDevice");
}

#[tokio::test]
async fn audio_effects_load_preset_mocked() {
    let _lock = audio_env_test_lock();
    std::env::set_var("AURA_AUDIO_ADVANCED", "1");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(
        &registry,
        "Audio.Effects.LoadPreset",
        Some(json!({ "preset_name": "music" })),
    )
    .await
    .expect("LoadPreset");
    std::env::remove_var("AURA_AUDIO_ADVANCED");
}

#[tokio::test]
async fn network_scan_and_status_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let scan = call_method(&registry, "Network.ScanNetworks", None)
        .await
        .expect("ScanNetworks");
    assert!(scan.is_array());
    let status = call_method(&registry, "Network.GetStatus", None)
        .await
        .expect("GetStatus");
    assert!(status.get("wifi_enabled").is_some());
}

#[tokio::test]
async fn network_list_saved_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let saved = call_method(&registry, "Network.ListSaved", None)
        .await
        .expect("ListSaved");
    assert!(saved.is_array());
}

#[tokio::test]
async fn network_connect_disconnect_forget_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method(&registry, "Network.ScanNetworks", None)
        .await
        .expect("ScanNetworks before Connect");
    let connect = call_method_unchecked(
        &registry,
        "Network.Connect",
        Some(json!({ "ssid": "TestNet", "password": "secret" })),
    )
    .await
    .expect("Connect");
    assert_eq!(connect.get("success"), Some(&json!(true)));
    call_method_unchecked(&registry, "Network.Disconnect", None)
        .await
        .expect("Disconnect");
    call_method_unchecked(
        &registry,
        "Network.Forget",
        Some(json!({ "name": "TestNet" })),
    )
    .await
    .expect("Forget");
}

#[tokio::test]
async fn vpn_get_status_mocked_ip() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    assert!(status.get("state").is_some());
}

#[tokio::test]
async fn capture_screenshot_full_file_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let path = format!(
        "{}/aura-capture-test.png",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
    );
    let value = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({
            "mode": "full",
            "output": "file",
            "path": path
        })),
    )
    .await
    .expect("Screenshot");
    assert_eq!(value.get("ok"), Some(&json!(true)));
}

#[tokio::test]
async fn capture_screenshot_region_clipboard_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Capture.Screenshot",
        Some(json!({ "mode": "region", "output": "clipboard" })),
    )
    .await
    .expect("Screenshot clipboard");
    assert_eq!(value.get("ok"), Some(&json!(true)));
}

#[tokio::test]
async fn capture_record_start_stop_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    call_method_unchecked(&registry, "Capture.RecordStart", None)
        .await
        .expect("RecordStart");
    call_method_unchecked(&registry, "Capture.RecordStop", None)
        .await
        .expect("RecordStop");
}

#[test]
fn exec_manifest_wpctl_fixture_loads() {
    let text = load_fixture("exec/wpctl/status.stdout");
    assert!(text.contains("Sinks"));
}
