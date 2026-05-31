mod common;

use ags_sidecar::build_registry;
use common::call_method;

#[tokio::test]
async fn media_get_now_playing_shape() {
    let registry = build_registry();
    let v = call_method(&registry, "Media.GetNowPlaying", None)
        .await
        .unwrap();
    assert!(v.get("playing").and_then(|x| x.as_bool()).is_some());
    assert!(v.get("title").and_then(|x| x.as_str()).is_some());
    assert!(v.get("artist").and_then(|x| x.as_str()).is_some());
}

#[tokio::test]
async fn audio_media_get_players_shape() {
    let registry = build_registry();
    let v = call_method(&registry, "Audio.Media.GetPlayers", None)
        .await
        .unwrap();
    assert!(v.is_array());
}
