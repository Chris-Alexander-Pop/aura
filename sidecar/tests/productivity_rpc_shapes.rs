//! Read-only `Productivity.*` RPC shapes and storage-namespace edge cases.

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn productivity_focus_mode_and_stats_schema() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let default = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus");
    assert_eq!(default.get("enabled"), Some(&json!(false)));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity",
            "key": "focus_mode",
            "value": true
        })),
    )
    .await
    .expect("Storage.Set focus_mode");

    let on = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus on");
    assert_eq!(on.get("enabled"), Some(&json!(true)));

    let stats = call_rpc(&registry, "Productivity.GetStats", None)
        .await
        .expect("Productivity.GetStats");
    for key in [
        "active_timers",
        "total_timers",
        "pomodoro_active",
        "focus_mode_enabled",
        "screen_time_minutes",
        "task_count",
    ] {
        assert!(stats.get(key).is_some(), "missing stats field {key}");
    }
    assert_eq!(stats.get("focus_mode_enabled"), Some(&json!(true)));
    assert!(stats.get("pomodoro_phase").map(|v| v.is_null()).unwrap_or(false));
}

#[tokio::test]
async fn productivity_readonly_stubs_and_corrupt_focus_mode() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let timers = call_rpc(&registry, "Productivity.GetTimers", None)
        .await
        .expect("GetTimers");
    assert!(timers.is_array());

    let pomo = call_rpc(&registry, "Productivity.GetPomodoroStatus", None)
        .await
        .expect("GetPomodoroStatus");
    assert!(pomo.is_null());

    let screen = call_rpc(&registry, "Productivity.GetScreenTime", None)
        .await
        .expect("GetScreenTime");
    assert_eq!(screen.get("minutes").and_then(|v| v.as_u64()), Some(0));

    let usage = call_rpc(&registry, "Productivity.GetAppUsage", None)
        .await
        .expect("GetAppUsage");
    assert!(usage.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let tasks = call_rpc(&registry, "Productivity.GetTasks", None)
        .await
        .expect("GetTasks");
    assert!(tasks.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity",
            "key": "focus_mode",
            "value": "not-a-bool"
        })),
    )
    .await
    .expect("Storage.Set corrupt focus_mode");

    let focus = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus");
    assert_eq!(focus.get("enabled"), Some(&json!(false)));
}
