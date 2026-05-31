mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn automation_workflow_round_trip_and_trigger() {
    setup_temp_storage_db().await;
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
}
