//! Read-only `Audio.*` and `Media.*` RPC response shapes (PipeWire / MPRIS).

mod common;

use common::{call_method, test_registry};

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
