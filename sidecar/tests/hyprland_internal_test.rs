//! Hyprland helpers: dispatch allowlist, socket2 event parsing (no live compositor).

mod common;

use ags_sidecar::services::hyprland::{
    event_triggers_state_changed, note_hyprland_event_line, parse_active_window,
    parse_active_workspace, parse_clients, parse_monitors, parse_workspaces,
    parse_workspace_event_id, parse_workspace_numeric_id, validate_dispatch,
};
use common::{call_method_unchecked, load_fixture, test_registry};
use serde_json::json;

#[test]
fn dispatch_validator_unit_matrix() {
    assert!(validate_dispatch("workspace 2").is_ok());
    assert!(validate_dispatch("movetoworkspace 3").is_ok());
    assert!(validate_dispatch("focuswindow address:0xdeadbeef").is_ok());
    assert!(validate_dispatch("killactive").is_ok());
    assert!(validate_dispatch("movefocus r").is_ok());
    assert!(validate_dispatch("swapwindow d").is_ok());
    assert!(validate_dispatch("togglespecialworkspace communication").is_ok());
    assert!(validate_dispatch("togglespecialworkspace special").is_ok());

    assert!(validate_dispatch("exec kitty")
        .unwrap_err()
        .to_string()
        .contains("dispatch_denied"));
    assert!(validate_dispatch("workspace 1; exec true")
        .unwrap_err()
        .to_string()
        .contains("dispatch_denied"));
    assert!(validate_dispatch("movefocus x")
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
    assert!(event_triggers_state_changed("workspacev2"));
    assert!(event_triggers_state_changed("activewindowv2"));
    assert!(event_triggers_state_changed("closewindow"));
    assert!(event_triggers_state_changed("float"));
    assert!(event_triggers_state_changed("activespecial"));
    assert!(event_triggers_state_changed("activespecialv2"));
    assert!(event_triggers_state_changed("monitoradded"));
    assert!(event_triggers_state_changed("monitorremoved"));
    assert!(event_triggers_state_changed("monitoraddedv2"));
    assert!(event_triggers_state_changed("monitorremovedv2"));
    assert!(!event_triggers_state_changed("bell"));
    // Animated titles (Cursor agent spinners, etc.) must not full-refresh.
    assert!(!event_triggers_state_changed("windowtitle"));
    assert!(!event_triggers_state_changed("windowtitlev2"));
    // Legacy activewindow includes the title and re-fires on rename — ignore.
    assert!(!event_triggers_state_changed("activewindow"));
}

#[test]
fn event_line_parses_socket2_format() {
    note_hyprland_event_line("workspace>>3");
    note_hyprland_event_line("workspacev2>>7,7");
    note_hyprland_event_line("garbage without delimiter");
}

#[test]
fn workspace_event_id_parses_numeric_payload() {
    assert_eq!(parse_workspace_event_id("3"), Some(3));
    assert_eq!(parse_workspace_event_id(" 10 "), Some(10));
    assert_eq!(parse_workspace_event_id("7,desktop"), Some(7));
    assert_eq!(parse_workspace_event_id("name:special"), None);
    assert_eq!(parse_workspace_event_id("0"), None);
}

#[test]
fn json_fixture_variants_parse_clients_and_monitors() {
    let clients_raw: serde_json::Value =
        serde_json::from_str(&load_fixture("hyprland/clients_skip_empty_address.json")).unwrap();
    let clients = parse_clients(&clients_raw);
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].address, "0xbeef");

    let ws_raw: serde_json::Value =
        serde_json::from_str(&load_fixture("hyprland/workspaces_not_array.json")).unwrap();
    assert!(parse_workspaces(&ws_raw).is_empty());

    let null_win: serde_json::Value =
        serde_json::from_str(&load_fixture("hyprland/activewindow_null.json")).unwrap();
    assert!(parse_active_window(&null_win).is_none());

    let mon_raw: serde_json::Value =
        serde_json::from_str(&load_fixture("hyprland/monitors_snake_case.json")).unwrap();
    let mon = parse_monitors(&mon_raw);
    assert_eq!(mon[0].active_workspace.id, 3);
}

#[test]
fn lua_config_hyprctl_uses_address_instead_of_id() {
    let ws = parse_workspaces(
        &serde_json::from_str(&load_fixture("hyprland/workspaces_address.json")).unwrap(),
    );
    assert_eq!(ws.iter().map(|w| w.id).collect::<Vec<_>>(), vec![1, 3]);

    let clients = parse_clients(
        &serde_json::from_str(&load_fixture("hyprland/clients_address_workspace.json")).unwrap(),
    );
    assert_eq!(clients.len(), 2);
    assert_eq!(clients[0].workspace.id, 2);
    assert_eq!(clients[1].workspace.id, -1);
    assert_eq!(
        clients[1].workspace.name.as_deref(),
        Some("special:communication")
    );

    let active = parse_active_workspace(
        &serde_json::from_str(&load_fixture("hyprland/activeworkspace_address.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(active.id, 2);

    let monitors = parse_monitors(
        &serde_json::from_str(&load_fixture("hyprland/monitors_address_workspace.json")).unwrap(),
    );
    assert_eq!(monitors[0].active_workspace.id, 1);
    assert_eq!(monitors[0].special_workspace.id, -1);
    assert_eq!(
        monitors[0].special_workspace.name.as_deref(),
        Some("special:special")
    );
    assert_eq!(monitors[1].special_workspace.id, 0);

    assert_eq!(
        parse_workspace_numeric_id(&serde_json::json!({
            "address": "4",
            "type": "numbered",
            "name": "4"
        })),
        Some(4)
    );
}
