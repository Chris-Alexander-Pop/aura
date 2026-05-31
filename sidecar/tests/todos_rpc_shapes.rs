//! Read-only `Todos.*` RPC shapes and storage-namespace edge cases.

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn todos_list_empty_and_parse_due_date_shape() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Todos.List", None)
        .await
        .expect("Todos.List");
    assert!(empty.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let projects = call_rpc(&registry, "Todos.ListProjects", None)
        .await
        .expect("Todos.ListProjects");
    assert!(projects.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let parsed = call_rpc(
        &registry,
        "Todos.ParseDueDate",
        Some(json!({ "text": "tomorrow" })),
    )
    .await
    .expect("Todos.ParseDueDate");
    assert_eq!(parsed.get("parsed"), Some(&json!(true)));
    assert!(parsed.get("due_at").and_then(|v| v.as_i64()).is_some());

    let bad = call_rpc(
        &registry,
        "Todos.ParseDueDate",
        Some(json!({ "text": "not a date" })),
    )
    .await
    .expect("Todos.ParseDueDate bad");
    assert_eq!(bad.get("parsed"), Some(&json!(false)));
    assert!(bad.get("due_at").map(|v| v.is_null()).unwrap_or(false));
}

#[tokio::test]
async fn todos_list_skips_corrupt_storage_rows() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "todos",
            "key": "good",
            "value": {
                "id": "good",
                "title": "OK",
                "description": "",
                "project_id": null,
                "due_at": null,
                "completed": false,
                "reminder_minutes": null,
                "created_at": 1,
                "updated_at": 1
            }
        })),
    )
    .await
    .expect("Storage.Set todo");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "todos",
            "key": "bad",
            "value": { "nope": 1 }
        })),
    )
    .await
    .expect("Storage.Set corrupt");

    let list = call_rpc(&registry, "Todos.List", None)
        .await
        .expect("Todos.List");
    let ids: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["good"]);
}

#[tokio::test]
async fn todos_project_shape_via_storage() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "todo_projects",
            "key": "p1",
            "value": {
                "id": "p1",
                "name": "Personal",
                "color": "#aabbcc"
            }
        })),
    )
    .await
    .expect("Storage.Set project");

    let projects = call_rpc(&registry, "Todos.ListProjects", None)
        .await
        .expect("Todos.ListProjects");
    let first = projects.as_array().and_then(|a| a.first()).expect("project");
    for key in ["id", "name", "color"] {
        assert!(first.get(key).is_some(), "missing {key}");
    }
}
