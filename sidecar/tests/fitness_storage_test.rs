//! Fitness SQLite namespace persistence (goals and workouts).

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn fitness_goals_persist_across_get_goals_calls() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("GetGoals initial");
    assert!(empty.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_goals",
            "key": "persist-g1",
            "value": {
                "id": "persist-g1",
                "goal_type": "distance",
                "target": 5.0,
                "current": 1.5
            }
        })),
    )
    .await
    .expect("Storage.Set goal");

    let goals = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("GetGoals loaded");
    let arr = goals.as_array().expect("goals array");
    assert_eq!(arr.len(), 1);
    assert_eq!(
        arr[0].get("goal_type").and_then(|v| v.as_str()),
        Some("distance")
    );
    assert_eq!(arr[0].get("current").and_then(|v| v.as_f64()), Some(1.5));
}
