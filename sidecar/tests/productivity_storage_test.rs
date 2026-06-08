//! Mutating `Productivity.*` task CRUD against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn productivity_task_round_trip_and_stats() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let task = call_method_unchecked(
        &registry,
        "Productivity.CreateTask",
        Some(json!({ "title": "Write tests", "description": "integration" })),
    )
    .await
    .unwrap();
    let id = task.get("id").and_then(|v| v.as_str()).unwrap();

    let list = call_rpc(&registry, "Productivity.GetTasks", None).await.unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|t| t.get("id") == Some(&json!(id))),
        "created task not in list"
    );

    let stats = call_rpc(&registry, "Productivity.GetStats", None).await.unwrap();
    assert!(stats.get("open_task_count").and_then(|v| v.as_u64()).unwrap_or(0) >= 1);

    let del = call_method_unchecked(
        &registry,
        "Productivity.DeleteTask",
        Some(json!({ "task_id": id })),
    )
    .await
    .unwrap();
    assert_eq!(del.get("deleted"), Some(&json!(true)));
}

#[tokio::test]
async fn productivity_update_task_and_focus_mode_round_trip() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let task = call_method_unchecked(
        &registry,
        "Productivity.CreateTask",
        Some(json!({ "title": "Ship", "description": "tests" })),
    )
    .await
    .unwrap();
    let id = task.get("id").and_then(|v| v.as_str()).unwrap();

    call_method_unchecked(
        &registry,
        "Productivity.UpdateTask",
        Some(json!({
            "task_id": id,
            "updates": { "completed": true, "title": "Shipped" }
        })),
    )
    .await
    .unwrap();

    let list = call_rpc(&registry, "Productivity.GetTasks", None).await.unwrap();
    let updated = list
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t.get("id") == Some(&json!(id)))
        .unwrap();
    assert_eq!(updated.get("completed"), Some(&json!(true)));
    assert_eq!(updated.get("title"), Some(&json!("Shipped")));

    call_method_unchecked(
        &registry,
        "Productivity.SetFocusMode",
        Some(json!({ "enabled": true })),
    )
    .await
    .unwrap();

    let focus = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .unwrap();
    assert_eq!(focus.get("enabled"), Some(&json!(true)));

    let stats = call_rpc(&registry, "Productivity.GetStats", None).await.unwrap();
    assert_eq!(stats.get("focus_mode_enabled"), Some(&json!(true)));
}

#[tokio::test]
async fn productivity_pomodoro_create_and_status_round_trip() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let created = call_method_unchecked(
        &registry,
        "Productivity.CreatePomodoro",
        Some(json!({ "work_minutes": 25, "break_minutes": 5 })),
    )
    .await
    .unwrap();
    assert_eq!(created.get("current_phase"), Some(&json!("work")));
    assert_eq!(created.get("active"), Some(&json!(true)));

    let status = call_rpc(&registry, "Productivity.GetPomodoroStatus", None)
        .await
        .unwrap();
    assert_eq!(status.get("work_minutes").and_then(|v| v.as_u64()), Some(25));
    assert_eq!(status.get("current_phase"), Some(&json!("work")));

    let stats = call_rpc(&registry, "Productivity.GetStats", None).await.unwrap();
    assert_eq!(stats.get("pomodoro_active"), Some(&json!(true)));
    assert_eq!(stats.get("pomodoro_phase"), Some(&json!("work")));
}

#[tokio::test]
async fn productivity_corrupt_storage_branches() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    call_method_unchecked(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity_tasks",
            "key": "task_bad",
            "value": { "oops": true }
        })),
    )
    .await
    .unwrap();

    let tasks = call_rpc(&registry, "Productivity.GetTasks", None).await.unwrap();
    assert!(tasks.as_array().unwrap().is_empty());

    call_method_unchecked(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity",
            "key": "focus_mode",
            "value": "yes"
        })),
    )
    .await
    .unwrap();

    let focus = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .unwrap();
    assert_eq!(focus.get("enabled"), Some(&json!(false)));

    let stats = call_rpc(&registry, "Productivity.GetStats", None).await.unwrap();
    assert_eq!(stats.get("focus_mode_enabled"), Some(&json!(false)));
    assert_eq!(stats.get("task_count").and_then(|v| v.as_u64()), Some(0));
}
