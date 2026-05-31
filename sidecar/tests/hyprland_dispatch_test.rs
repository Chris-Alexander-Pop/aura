mod common;

use ags_sidecar::build_registry;
use ags_sidecar::services::hyprland::validate_dispatch;
use common::call_method_unchecked;

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
    let registry = build_registry();
    let err = call_method_unchecked(
        &registry,
        "Hyprland.Dispatch",
        Some(serde_json::json!({ "command": "exec kitty" })),
    )
    .await
    .unwrap_err();
    assert!(
        err.to_string().contains("dispatch_denied"),
        "expected dispatch_denied, got {err}"
    );
}
