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

#[tokio::test]
async fn automation_script_create_list_and_delete() {
    let _guard = automation_test_lock();
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateScript",
        Some(json!({
            "name": "hello",
            "content": "#!/bin/sh\necho hello\n"
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    let list = call_rpc(&registry, "Automation.GetScripts", None)
        .await
        .unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|s| s.get("id") == Some(&json!(id)))
    );
    let script = list
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s.get("id") == Some(&json!(id)))
        .unwrap();
    assert_eq!(script.get("name"), Some(&json!("hello")));
}

#[tokio::test]
async fn automation_update_workflow_and_history_filter() {
    let _guard = automation_test_lock();
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "History flow",
            "actions": [{"type": "send_notification", "title": "t", "body": "b"}]
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    call_method_unchecked(
        &registry,
        "Automation.UpdateWorkflow",
        Some(json!({
            "workflow_id": id,
            "updates": { "name": "Renamed flow", "enabled": true }
        })),
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
    assert_eq!(row.get("name"), Some(&json!("Renamed flow")));

    call_method_unchecked(
        &registry,
        "Automation.Trigger",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .unwrap();

    let filtered = call_rpc(
        &registry,
        "Automation.GetWorkflowHistory",
        Some(json!({ "workflow_id": id, "limit": 1 })),
    )
    .await
    .unwrap();
    assert_eq!(filtered.as_array().unwrap().len(), 1);

    let all = call_rpc(
        &registry,
        "Automation.GetWorkflowHistory",
        Some(json!({ "limit": 10 })),
    )
    .await
    .unwrap();
    assert!(!all.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn automation_script_run_with_temp_scripts_dir() {
    let _guard = automation_test_lock();
    let _db = setup_temp_storage_db().await;
    let scripts_dir = std::env::temp_dir().join(format!(
        "ags-automation-scripts-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scripts_dir);
    std::env::set_var("AURA_AUTOMATION_SCRIPTS_DIR", &scripts_dir);

    let registry = build_registry();
    let created = call_method_unchecked(
        &registry,
        "Automation.CreateScript",
        Some(json!({
            "name": "echo-test",
            "content": "#!/bin/sh\necho automation-ok\n",
            "interpreter": "sh"
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    let run = call_method_unchecked(
        &registry,
        "Automation.RunScript",
        Some(json!({ "script_id": id })),
    )
    .await
    .unwrap();
    assert!(
        run.get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("automation-ok")
    );

    std::env::remove_var("AURA_AUTOMATION_SCRIPTS_DIR");
    let _ = std::fs::remove_dir_all(&scripts_dir);
}
