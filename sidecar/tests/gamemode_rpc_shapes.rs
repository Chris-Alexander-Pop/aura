//! Read-only `GameMode.*` RPC response shapes.

mod common;

use ags_sidecar::services::gamemode::reset_gamemode_state_for_tests;
use common::{assert_json_object_keys, call_method_unchecked, call_rpc, gamemode_test_lock, test_registry};
use serde_json::json;

async fn reset_gamemode(registry: &ags_sidecar::services::ServiceRegistry) {
    reset_gamemode_state_for_tests().await;
    std::env::set_var("AURA_GAMEMODE_DRY_RUN", "1");
    let _ = call_method_unchecked(registry, "GameMode.Disable", None).await;
    std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
}

#[tokio::test]
async fn gamemode_is_enabled_schema() {
    let _lock = gamemode_test_lock();
    let registry = test_registry();
    reset_gamemode(&registry).await;
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert!(value.get("enabled").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn gamemode_is_enabled_defaults_false() {
    let _lock = gamemode_test_lock();
    let registry = test_registry();
    reset_gamemode(&registry).await;
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert_json_object_keys(&value, &["enabled"]);
    assert_eq!(value.get("enabled"), Some(&json!(false)));
}

#[tokio::test]
async fn gamemode_enable_dry_run_skips_hyprctl() {
    let _lock = gamemode_test_lock();
    std::env::set_var("AURA_GAMEMODE_DRY_RUN", "1");
    let registry = test_registry();
    reset_gamemode_state_for_tests().await;
    let enabled = call_method_unchecked(&registry, "GameMode.Enable", None)
        .await
        .expect("GameMode.Enable dry-run");
    assert_eq!(enabled.get("enabled"), Some(&json!(true)));

    let status = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert_eq!(status.get("enabled"), Some(&json!(true)));

    let disabled = call_method_unchecked(&registry, "GameMode.Disable", None)
        .await
        .expect("GameMode.Disable dry-run");
    assert_eq!(disabled.get("enabled"), Some(&json!(false)));
    std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
}

#[tokio::test]
async fn gamemode_toggle_dry_run_flips_state() {
    let _lock = gamemode_test_lock();
    std::env::set_var("AURA_GAMEMODE_DRY_RUN", "1");
    let registry = test_registry();
    reset_gamemode_state_for_tests().await;
    let first = call_method_unchecked(&registry, "GameMode.Toggle", None)
        .await
        .expect("GameMode.Toggle");
    assert_eq!(first.get("enabled"), Some(&json!(true)));
    let second = call_method_unchecked(&registry, "GameMode.Toggle", None)
        .await
        .expect("GameMode.Toggle again");
    assert_eq!(second.get("enabled"), Some(&json!(false)));
    std::env::remove_var("AURA_GAMEMODE_DRY_RUN");
}
