//! Read-only `Audio.*` and `Media.*` RPC response shapes (PipeWire / MPRIS).

mod common;

use ags_sidecar::contract_parsers::{
    parse_now_playing_line, parse_playerctl_list, parse_playerctl_status, parse_streams,
    playing_from_status, PlayerctlPlayback,
};
use ags_sidecar::services::audio::audio_env_test_lock;
use common::{
    assert_safe_rpc_method, call_method, call_method_unchecked, load_fixture, setup_temp_storage_db,
    test_registry,
};
use serde_json::json;

fn assert_audio_device_row(obj: &serde_json::Map<String, serde_json::Value>) {
    for key in ["id", "name", "info", "volume", "is_default", "muted"] {
        assert!(obj.contains_key(key), "missing device key {key}");
    }
}

fn assert_audio_stream_row(obj: &serde_json::Map<String, serde_json::Value>) {
    for key in ["id", "name", "app", "volume", "sink_id"] {
        assert!(obj.contains_key(key), "missing stream key {key}");
    }
}

#[tokio::test]
async fn audio_get_devices_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.GetDevices", None)
        .await
        .expect("Audio.GetDevices");
    let obj = value.as_object().expect("object");
    let sinks = obj.get("sinks").and_then(|v| v.as_array()).expect("sinks array");
    let sources = obj
        .get("sources")
        .and_then(|v| v.as_array())
        .expect("sources array");
    for row in sinks.iter().chain(sources.iter()) {
        assert_audio_device_row(row.as_object().expect("device object"));
    }
}

#[tokio::test]
async fn audio_get_streams_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.GetStreams", None)
        .await
        .expect("Audio.GetStreams");
    let streams = value.as_array().expect("streams array");
    for row in streams {
        assert_audio_stream_row(row.as_object().expect("stream object"));
    }
}

#[tokio::test]
async fn audio_effects_get_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Effects.GetStatus", None)
        .await
        .expect("Audio.Effects.GetStatus");
    for key in ["available", "running", "current_preset"] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
}

#[tokio::test]
async fn audio_effects_get_presets_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Effects.GetPresets", None)
        .await
        .expect("Audio.Effects.GetPresets");
    assert!(value.is_array());
}

#[tokio::test]
async fn audio_profiles_get_current_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Profiles.GetCurrent", None)
        .await
        .expect("Audio.Profiles.GetCurrent");
    let obj = value.as_object().expect("object");
    for key in ["routing", "volumes", "effects"] {
        assert!(obj.contains_key(key), "missing profile section {key}");
    }
}

#[tokio::test]
async fn audio_profiles_list_shape() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Profiles.List", None)
        .await
        .expect("Audio.Profiles.List");
    assert!(value.is_array());
}

#[tokio::test]
async fn media_get_now_playing_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Media.GetNowPlaying", None)
        .await
        .expect("Media.GetNowPlaying");
    assert!(value.get("playing").and_then(|v| v.as_bool()).is_some());
    assert!(value.get("title").and_then(|v| v.as_str()).is_some());
    assert!(value.get("artist").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn audio_media_get_players_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.Media.GetPlayers", None)
        .await
        .expect("Audio.Media.GetPlayers");
    assert!(value.is_array());
}

#[test]
fn audio_set_sink_volume_is_denied_in_default_harness() {
    let err = std::panic::catch_unwind(|| assert_safe_rpc_method("Audio.SetSinkVolume"));
    assert!(err.is_err());
}

#[test]
fn audio_media_play_pause_is_denied_in_default_harness() {
    let err = std::panic::catch_unwind(|| assert_safe_rpc_method("Audio.Media.PlayPause"));
    assert!(err.is_err());
}

#[test]
fn audio_playerctl_fixture_contracts() {
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status.txt")),
        PlayerctlPlayback::Playing
    );
    assert!(playing_from_status(PlayerctlPlayback::Playing));
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status_paused.txt")),
        PlayerctlPlayback::Paused
    );

    let (title, artist) = parse_now_playing_line(&load_fixture("audio/playerctl_metadata.txt"));
    assert_eq!(title, "Creep");
    assert_eq!(artist, "Radiohead");

    let players = parse_playerctl_list(&load_fixture("audio/playerctl_list.txt"));
    assert_eq!(players, vec!["spotify", "firefox"]);
}

#[test]
fn audio_pactl_streams_fixture_matches_contract() {
    let streams = parse_streams(&load_fixture("audio/pactl_sink_inputs_multi.txt"));
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].app, "Spotify");
    assert_eq!(streams[1].sink_id, 55);
}

#[tokio::test]
async fn audio_effects_start_blocked_without_advanced_flag() {
    let _lock = audio_env_test_lock();
    let registry = test_registry();
    std::env::remove_var("AURA_AUDIO_ADVANCED");
    let err = call_method_unchecked(&registry, "Audio.Effects.Start", None)
        .await
        .expect_err("Audio.Effects.Start without AURA_AUDIO_ADVANCED");
    assert!(
        err.to_string().contains("AURA_AUDIO_ADVANCED"),
        "unexpected error: {err}"
    );
}

/// Round-trip sink volume via wpctl (mutates host audio).
///
/// `AURA_AUDIO_VOLUME_TEST=1 cargo test audio_set_sink_volume_round_trip -- --ignored`
#[tokio::test]
#[ignore = "mutates PipeWire volume; set AURA_AUDIO_VOLUME_TEST=1"]
async fn audio_set_sink_volume_round_trip() {
    if std::env::var("AURA_AUDIO_VOLUME_TEST").ok().as_deref() != Some("1") {
        panic!("Set AURA_AUDIO_VOLUME_TEST=1 to run volume round-trip test");
    }

    let registry = test_registry();
    let devices = call_method(&registry, "Audio.GetDevices", None)
        .await
        .expect("Audio.GetDevices");
    let sinks = devices.get("sinks").and_then(|v| v.as_array()).expect("sinks");
    let sink = sinks
        .first()
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_i64())
        .expect("default sink id") as i32;

    let target = 0.42;
    call_method_unchecked(
        &registry,
        "Audio.SetSinkVolume",
        Some(json!({ "id": sink, "volume": target })),
    )
    .await
    .expect("set volume");

    let after = call_method(&registry, "Audio.GetDevices", None)
        .await
        .expect("refresh devices");
    let vol = after
        .get("sinks")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("volume"))
        .and_then(|v| v.as_f64())
        .expect("sink volume");
    assert!(
        (vol - target).abs() < 0.05,
        "expected ~{target}, got {vol}"
    );
}

#[tokio::test]
async fn audio_set_sink_volume_missing_device_id_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.SetSinkVolume",
        Some(json!({ "volume": 0.5 })),
    )
    .await
    .expect_err("missing device_id");
    assert!(err.to_string().contains("Missing device_id"));
}

#[tokio::test]
async fn audio_set_sink_volume_missing_volume_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.SetSinkVolume",
        Some(json!({ "device_id": 1 })),
    )
    .await
    .expect_err("missing volume");
    assert!(err.to_string().contains("Missing volume"));
}

#[tokio::test]
async fn audio_set_default_device_missing_id_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Audio.SetDefaultDevice", None)
        .await
        .expect_err("missing device_id");
    assert!(err.to_string().contains("Missing device_id"));
}

#[tokio::test]
async fn audio_set_sink_mute_missing_muted_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.SetSinkMute",
        Some(json!({ "device_id": 1 })),
    )
    .await
    .expect_err("missing muted");
    assert!(err.to_string().contains("Missing muted"));
}

#[tokio::test]
async fn audio_route_stream_missing_stream_id_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.RouteStream",
        Some(json!({ "sink_id": 1 })),
    )
    .await
    .expect_err("missing stream_id");
    assert!(err.to_string().contains("Missing stream_id"));
}

#[tokio::test]
async fn audio_create_loopback_missing_source_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.CreateLoopback",
        Some(json!({ "sink_name": "null-sink" })),
    )
    .await
    .expect_err("missing source_name");
    assert!(err.to_string().contains("Missing source_name"));
}

#[tokio::test]
async fn audio_create_null_sink_missing_name_errors() {
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Audio.CreateNullSink", None)
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("Missing name"));
}

#[tokio::test]
async fn audio_effects_load_preset_missing_name_errors() {
    let _lock = audio_env_test_lock();
    std::env::set_var("AURA_AUDIO_ADVANCED", "1");
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Audio.Effects.LoadPreset", None)
        .await
        .expect_err("missing preset_name");
    assert!(err.to_string().contains("Missing preset_name"));
    std::env::remove_var("AURA_AUDIO_ADVANCED");
}

#[tokio::test]
async fn audio_profiles_save_missing_config_errors() {
    let _lock = audio_env_test_lock();
    std::env::set_var("AURA_AUDIO_ADVANCED", "1");
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Audio.Profiles.Save",
        Some(json!({ "name": "test-profile" })),
    )
    .await
    .expect_err("missing config");
    assert!(err.to_string().contains("Missing config"));
    std::env::remove_var("AURA_AUDIO_ADVANCED");
}

#[tokio::test]
async fn audio_profiles_apply_scenario_missing_scenario_errors() {
    let _lock = audio_env_test_lock();
    std::env::set_var("AURA_AUDIO_ADVANCED", "1");
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Audio.Profiles.ApplyScenario", None)
        .await
        .expect_err("missing scenario");
    assert!(err.to_string().contains("Missing scenario"));
    std::env::remove_var("AURA_AUDIO_ADVANCED");
}
