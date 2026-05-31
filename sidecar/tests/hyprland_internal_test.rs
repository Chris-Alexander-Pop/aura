//! Hyprland helpers: dispatch allowlist, socket2 event parsing (no live compositor).

mod common;

use ags_sidecar::services::hyprland::{
    event_triggers_state_changed, note_hyprland_event_line, validate_dispatch,
};
use common::{call_method_unchecked, test_registry};
use serde_json::json;

#[test]
fn dispatch_validator_unit_matrix() {
    assert!(validate_dispatch("workspace 2").is_ok());
    assert!(validate_dispatch("movetoworkspace 3").is_ok());
    assert!(validate_dispatch("focuswindow address:0xdeadbeef").is_ok());
    assert!(validate_dispatch("killactive").is_ok());

    assert!(validate_dispatch("exec kitty")
        .unwrap_err()
        .to_string()
        .contains("dispatch_denied"));
    assert!(validate_dispatch("workspace 1; exec true")
        .unwrap_err()
        .to_string()
        .contains("dispatch_denied"));
}

#[tokio::test]
async fn dispatch_denied_via_rpc_without_hyprctl() {
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Hyprland.Dispatch",
        Some(json!({ "command": "exec kitty" })),
    )
    .await
    .unwrap_err();
    assert!(
        err.to_string().contains("dispatch_denied"),
        "expected dispatch_denied, got {err}"
    );
}

#[test]
fn event_names_cover_bar_invalidation() {
    assert!(event_triggers_state_changed("workspace"));
    assert!(event_triggers_state_changed("activewindow"));
    assert!(event_triggers_state_changed("closewindow"));
    assert!(!event_triggers_state_changed("bell"));
}

#[test]
fn event_line_parses_socket2_format() {
    note_hyprland_event_line("workspace>>3");
    note_hyprland_event_line("garbage without delimiter");
}
