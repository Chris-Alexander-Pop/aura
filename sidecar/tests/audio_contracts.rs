//! Golden CLI parse contracts for audio / playerctl fixtures (multi-branch, no RPC).

mod common;

use ags_sidecar::contract_parsers::{
    parse_device_line, parse_devices, parse_pactl_list_sinks, parse_playerctl_status,
    parse_streams, playing_from_status, PlayerctlPlayback,
};
use common::load_fixture;
use serde_json::Value;

fn assert_audio_device_contract(v: &Value) {
    let obj = v.as_object().expect("JSON object");
    for key in ["id", "name", "info", "volume", "is_default", "muted"] {
        assert!(obj.contains_key(key), "missing key {key}");
    }
}

fn assert_audio_stream_contract(v: &Value) {
    let obj = v.as_object().expect("JSON object");
    for key in ["id", "name", "app", "volume", "sink_id"] {
        assert!(obj.contains_key(key), "missing key {key}");
    }
}

#[test]
fn contract_wpctl_default_and_muted_sink() {
    let (sinks, sources) = parse_devices(&load_fixture("audio/wpctl_status.txt"));
    assert_eq!(sinks.len(), 2);
    assert!(sinks[0].muted);
    assert!(sinks[0].is_default);
    assert_eq!(sources.len(), 1);
    assert_audio_device_contract(&serde_json::to_value(&sinks[0]).unwrap());
}

#[test]
fn contract_wpctl_star_default_and_source_volume() {
    let (sinks, sources) = parse_devices(&load_fixture("audio/wpctl_status_star_default.txt"));
    assert!(sinks[0].is_default);
    assert!(sinks[1].muted);
    assert!(sources[0].is_default);
    assert!((sources[0].volume - 0.90).abs() < f64::EPSILON);
}

#[test]
fn contract_wpctl_multi_default_marker_on_second_sink() {
    let (sinks, _) = parse_devices(&load_fixture("audio/wpctl_status_multi.txt"));
    assert!(!sinks[0].is_default);
    assert!(sinks[1].is_default);
    assert!((sinks[1].volume - 0.75).abs() < f64::EPSILON);
}

#[test]
fn contract_wpctl_video_section_does_not_pollute_audio() {
    let (sinks, sources) = parse_devices(&load_fixture("audio/wpctl_status_video_boundary.txt"));
    assert_eq!(sinks.len(), 1);
    assert_eq!(sinks[0].id, 47);
    assert_eq!(sources.len(), 1);
    assert!(!sinks.iter().any(|s| s.id == 99));
}

#[test]
fn contract_wpctl_plain_name_without_brackets() {
    let (sinks, sources) = parse_devices(&load_fixture("audio/wpctl_status_plain_name.txt"));
    assert_eq!(sinks[0].name, "HDMI Output");
    assert_eq!(sinks[0].info, "");
    assert!(sources.is_empty());
}

#[test]
fn contract_pactl_list_sinks_descriptions() {
    let sinks = parse_pactl_list_sinks(&load_fixture("audio/pactl_list_sinks.txt"));
    assert_eq!(sinks.len(), 2);
    assert_eq!(sinks[0].description, "Headphones");
    assert_eq!(sinks[1].index, 55);
}

#[test]
fn contract_pactl_single_stream_shape() {
    let streams = parse_streams(&load_fixture("audio/pactl_sink_inputs.txt"));
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].app, "Firefox");
    assert_eq!(streams[0].sink_id, 47);
    assert_audio_stream_contract(&serde_json::to_value(&streams[0]).unwrap());
}

#[test]
fn contract_pactl_multi_streams_branch() {
    let streams = parse_streams(&load_fixture("audio/pactl_sink_inputs_multi.txt"));
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].app, "Spotify");
    assert_eq!(streams[0].volume, 50);
    assert_eq!(streams[1].app, "Discord");
    for stream in &streams {
        assert_audio_stream_contract(&serde_json::to_value(stream).unwrap());
    }
}

#[test]
fn contract_pactl_error_and_empty_streams() {
    assert!(parse_streams(&load_fixture("audio/pactl_error.txt")).is_empty());
    assert!(parse_streams(&load_fixture("audio/pactl_sink_inputs_empty.txt")).is_empty());
}

#[test]
fn contract_pactl_malformed_stream_partial_fields() {
    let streams = parse_streams(&load_fixture("audio/pactl_sink_inputs_malformed.txt"));
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, 12);
    assert_eq!(streams[0].sink_id, -1);
}

#[test]
fn contract_device_line_malformed_returns_none() {
    assert!(parse_device_line("│  broken row").is_none());
}

#[test]
fn contract_playerctl_status_playback_branches() {
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status.txt")),
        PlayerctlPlayback::Playing
    );
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status_paused.txt")),
        PlayerctlPlayback::Paused
    );
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status_stopped.txt")),
        PlayerctlPlayback::Stopped
    );
}

#[test]
fn contract_playerctl_status_stopped_branch() {
    assert_eq!(
        parse_playerctl_status(&load_fixture("audio/playerctl_status_stopped.txt")),
        PlayerctlPlayback::Stopped
    );
    assert!(!playing_from_status(PlayerctlPlayback::Stopped));
}

#[test]
fn contract_playerctl_unknown_status() {
    assert_eq!(parse_playerctl_status("Buffering"), PlayerctlPlayback::Unknown);
}
