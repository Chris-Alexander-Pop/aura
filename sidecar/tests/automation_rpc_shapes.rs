//! Read-only `Automation.*` RPC shapes and storage-namespace edge cases (`AURA_STORAGE_DB`).

mod common;

use common::{assert_json_object_keys, call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn automation_get_workflows_empty_namespace() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let value = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .expect("Automation.GetWorkflows");
    assert_eq!(value.as_array().map(|a| a.len()), Some(0));
}

#[tokio::test]
async fn automation_get_workflows_skips_corrupt_storage_rows() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_workflows",
            "key": "good",
            "value": {
                "id": "good",
                "name": "Daily",
                "enabled": true,
                "triggers": [],
                "actions": [],
                "created_at": 1
            }
        })),
    )
    .await
    .expect("Storage.Set good");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_workflows",
            "key": "bad",
            "value": { "oops": true }
        })),
    )
    .await
    .expect("Storage.Set bad");

    let workflows = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .expect("Automation.GetWorkflows");
    let arr = workflows.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some("good"));
    assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Daily"));
}

#[tokio::test]
async fn automation_list_rules_matches_get_workflows() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_workflows",
            "key": "wf1",
            "value": {
                "id": "wf1",
                "name": "Sync",
                "enabled": true,
                "triggers": [],
                "actions": [],
                "created_at": 2,
                "run_count": 3
            }
        })),
    )
    .await
    .expect("Storage.Set");

    let workflows = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .expect("GetWorkflows");
    let rules = call_rpc(&registry, "Automation.ListRules", None)
        .await
        .expect("ListRules");
    assert_eq!(workflows, rules);
}

#[tokio::test]
async fn automation_get_triggers_and_actions_schema() {
    let registry = test_registry();
    for method in ["Automation.GetTriggers", "Automation.GetActions"] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        let arr = value.as_array().expect("string array");
        assert!(!arr.is_empty());
        assert!(arr.iter().all(|v| v.as_str().is_some()));
    }
}

#[tokio::test]
async fn automation_get_scripts_empty_and_corrupt_namespace() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Automation.GetScripts", None)
        .await
        .expect("GetScripts empty");
    assert_eq!(empty.as_array().map(|a| a.len()), Some(0));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_scripts",
            "key": "ok",
            "value": {
                "id": "ok",
                "name": "Hello",
                "content": "echo hi",
                "interpreter": "bash"
            }
        })),
    )
    .await
    .expect("Storage.Set script");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_scripts",
            "key": "bad",
            "value": { "oops": true }
        })),
    )
    .await
    .expect("Storage.Set bad script");

    let scripts = call_rpc(&registry, "Automation.GetScripts", None)
        .await
        .expect("GetScripts");
    let arr = scripts.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_json_object_keys(&arr[0], &["id", "name", "content", "interpreter"]);
    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some("ok"));
}
