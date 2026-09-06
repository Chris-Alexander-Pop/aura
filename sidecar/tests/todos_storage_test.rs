//! Mutating `Todos.*` CRUD against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn todos_crud_project_and_due_text() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let project = call_method_unchecked(
        &registry,
        "Todos.CreateProject",
        Some(json!({ "name": "Work", "color": "#ff0000" })),
    )
    .await
    .unwrap();
    let project_id = project.get("id").and_then(|v| v.as_str()).unwrap();

    let created = call_method_unchecked(
        &registry,
        "Todos.Create",
        Some(json!({
            "title": "Ship ICS",
            "description": "calendar slice",
            "project_id": project_id,
            "due_text": "tomorrow",
            "reminder_minutes": 30
        })),
    )
    .await
    .unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();
    assert!(created.get("due_at").and_then(|v| v.as_i64()).is_some());

    let list = call_rpc(
        &registry,
        "Todos.List",
        Some(json!({ "project_id": project_id, "include_completed": false })),
    )
    .await
    .unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|t| t.get("id") == Some(&json!(id)))
    );

    call_method_unchecked(
        &registry,
        "Todos.Update",
        Some(json!({
            "id": id,
            "updates": { "completed": true }
        })),
    )
    .await
    .unwrap();

    let open = call_rpc(
        &registry,
        "Todos.List",
        Some(json!({ "include_completed": false })),
    )
    .await
    .unwrap();
    assert!(
        !open
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t.get("id") == Some(&json!(id)))
    );

    let del = call_method_unchecked(
        &registry,
        "Todos.Delete",
        Some(json!({ "id": id })),
    )
    .await
    .unwrap();
    assert_eq!(del.get("deleted"), Some(&json!(true)));
}
