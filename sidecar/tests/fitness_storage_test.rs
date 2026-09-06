//! Fitness SQLite namespace persistence (goals and workouts).

mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn fitness_set_goal_round_trip_via_rpc() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let empty = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("GetGoals initial");
    assert!(empty.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let created = call_method_unchecked(
        &registry,
        "Fitness.SetGoal",
        Some(json!({ "type": "steps", "target": 10000.0 })),
    )
    .await
    .expect("Fitness.SetGoal");
    assert_eq!(created.get("goal_type").and_then(|v| v.as_str()), Some("steps"));
    assert_eq!(created.get("target").and_then(|v| v.as_f64()), Some(10000.0));
    assert_eq!(created.get("current").and_then(|v| v.as_f64()), Some(0.0));
    let goal_id = created.get("id").and_then(|v| v.as_str()).expect("goal id");

    let goals = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("GetGoals after SetGoal");
    let arr = goals.as_array().expect("goals array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some(goal_id));
    assert_eq!(arr[0].get("goal_type").and_then(|v| v.as_str()), Some("steps"));
    assert_eq!(arr[0].get("target").and_then(|v| v.as_f64()), Some(10000.0));
}

#[tokio::test]
async fn fitness_goals_persist_across_get_goals_calls() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

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
