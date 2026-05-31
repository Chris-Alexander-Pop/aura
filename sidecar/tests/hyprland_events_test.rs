use ags_sidecar::services::hyprland::{event_triggers_state_changed, note_hyprland_event_line};

#[test]
fn hyprland_event_names_covers_bar_invalidation() {
    assert!(event_triggers_state_changed("workspace"));
    assert!(event_triggers_state_changed("activewindow"));
    assert!(event_triggers_state_changed("closewindow"));
    assert!(!event_triggers_state_changed("bell"));
}

#[test]
fn hyprland_event_line_parses_socket2_format() {
    note_hyprland_event_line("workspace>>3");
    note_hyprland_event_line("garbage without delimiter");
}
