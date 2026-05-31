//! Mutating `Automation.*` workflow CRUD against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use common::{automation_test_lock, call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn automation_workflow_round_trip_and_trigger() {
    let _guard = automation_test_lock();
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Test flow",
            "actions": [{"type": "send_notification", "title": "t", "body": "b"}]
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    let list = call_rpc(&registry, "Automation.ListRules", None).await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|w| w.get("id") == Some(&json!(id))));

    let run = call_method_unchecked(
        &registry,
        "Automation.Trigger",
        Some(json!({ "workflow_id": id })),
    )
    .await;
    assert!(run.is_ok());

    let history = call_rpc(
        &registry,
        "Automation.GetWorkflowHistory",
        Some(json!({ "workflow_id": id, "limit": 5 })),
    )
    .await
    .unwrap();
    assert!(!history.as_array().unwrap().is_empty());

    let del = call_method_unchecked(
        &registry,
        "Automation.DeleteWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .unwrap();
    assert_eq!(del.get("deleted"), Some(&json!(true)));

    let list_after = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .unwrap();
    assert!(!list_after
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.get("id") == Some(&json!(id))));
}

#[tokio::test]
async fn automation_enable_flag_and_run_count_round_trip() {
    let _guard = automation_test_lock();
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Counter",
            "actions": [{"type": "send_notification", "title": "t", "body": "b"}]
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    call_method_unchecked(
        &registry,
        "Automation.DisableWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .unwrap();

    let list = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .unwrap();
    let row = list
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w.get("id") == Some(&json!(id)))
        .unwrap();
    assert_eq!(row.get("enabled"), Some(&json!(false)));

    call_method_unchecked(
        &registry,
        "Automation.EnableWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .unwrap();

    call_method_unchecked(
        &registry,
        "Automation.Trigger",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .unwrap();

    let list2 = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .unwrap();
    let row2 = list2
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w.get("id") == Some(&json!(id)))
        .unwrap();
    assert_eq!(row2.get("run_count").and_then(|v| v.as_u64()), Some(1));
}
