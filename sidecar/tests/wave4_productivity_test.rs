mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn productivity_task_round_trip_and_stats() {
    setup_temp_storage_db().await;
    let registry = build_registry();

    let task = call_method_unchecked(
        &registry,
        "Productivity.CreateTask",
        Some(json!({ "title": "Write tests", "description": "wave4" })),
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
