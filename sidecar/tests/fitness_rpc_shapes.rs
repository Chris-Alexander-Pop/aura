//! Read-only `Fitness.*` RPC shapes and storage-namespace edge cases.

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;

#[tokio::test]
async fn fitness_get_activity_and_goals_from_storage() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let activity = call_rpc(&registry, "Fitness.GetActivity", None)
        .await
        .expect("Fitness.GetActivity");
    for key in ["date", "steps", "calories", "distance_km"] {
        assert!(activity.get(key).is_some(), "missing {key}");
    }

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_goals",
            "key": "g1",
            "value": {
                "id": "g1",
                "goal_type": "steps",
                "target": 10000.0,
                "current": 1200.0
            }
        })),
    )
    .await
    .expect("Storage.Set goal");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_goals",
            "key": "bad",
            "value": "not json goal"
        })),
    )
    .await
    .expect("Storage.Set bad goal");

    let goals = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("Fitness.GetGoals");
    let arr = goals.as_array().expect("goals");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("goal_type").and_then(|v| v.as_str()), Some("steps"));
}

#[tokio::test]
async fn fitness_workout_history_and_metric_stubs() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for (method, key) in [
        ("Fitness.GetSteps", "steps"),
        ("Fitness.GetHeartRate", "heart_rate"),
        ("Fitness.GetSleep", "sleep_hours"),
    ] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        assert!(value.get(key).is_some(), "missing {key} in {method}");
    }

    let history = call_rpc(&registry, "Fitness.GetWorkoutHistory", None)
        .await
        .expect("GetWorkoutHistory empty");
    assert!(history.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_workouts",
            "key": "w1",
            "value": {
                "id": "w1",
                "workout_type": "run",
                "start_time": 1,
                "end_time": 2,
                "duration_seconds": 60,
                "data": {}
            }
        })),
    )
    .await
    .expect("Storage.Set workout");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_workouts",
            "key": "bad",
            "value": { "nope": 1 }
        })),
    )
    .await
    .expect("Storage.Set bad workout");

    let workouts = call_rpc(&registry, "Fitness.GetWorkoutHistory", None)
        .await
        .expect("GetWorkoutHistory");
    let arr = workouts.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("workout_type").and_then(|v| v.as_str()), Some("run"));

    let devices = call_rpc(&registry, "Fitness.GetDevices", None)
        .await
        .expect("GetDevices");
    assert!(devices.as_array().map(|a| a.is_empty()).unwrap_or(false));
}
