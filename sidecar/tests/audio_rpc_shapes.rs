//! Read-only `Audio.*` and `Media.*` RPC response shapes (PipeWire / MPRIS).

mod common;

use common::{call_method, call_method_unchecked, test_registry};
use serde_json::json;

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

#[tokio::test]
async fn audio_effects_start_blocked_without_advanced_flag() {
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
