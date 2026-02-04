use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workout {
    pub id: String,
    pub workout_type: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: Option<u64>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub goal_type: String,
    pub target: f64,
    pub current: f64,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Fitness.GetActivity", |_params| async move {
        // Get today's activity
        let today = chrono::Utc::now().date_naive();
        Ok(serde_json::json!({
            "date": today.to_string(),
            "steps": 0,
            "calories": 0,
            "distance_km": 0.0
        }))
    });

    registry.register("Fitness.GetSteps", |_params| async move {
        Ok(serde_json::json!({ "steps": 0 }))
    });

    registry.register("Fitness.GetHeartRate", |_params| async move {
        Ok(serde_json::json!({ "heart_rate": null }))
    });

    registry.register("Fitness.GetSleep", |_params| async move {
        Ok(serde_json::json!({ "sleep_hours": null }))
    });

    registry.register("Fitness.StartWorkout", |params| async move {
        let workout_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing type"))?,
        )?;

        let workout_id = format!("workout_{}", chrono::Utc::now().timestamp_millis());
        let workout = Workout {
            id: workout_id.clone(),
            workout_type,
            start_time: chrono::Utc::now().timestamp_millis(),
            end_time: None,
            duration_seconds: None,
            data: serde_json::json!({}),
        };

        storage::init().await?;
        storage::set_kv("fitness_workouts", &workout_id, &serde_json::to_value(&workout)?).await?;
        Ok(serde_json::to_value(&workout)?)
    });

    registry.register("Fitness.StopWorkout", |_params| async move {
        // Would find active workout and stop it
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Fitness.GetWorkoutHistory", |_params| async move {
        // Would get all workouts from storage
        Ok(serde_json::json!([]))
    });

    registry.register("Fitness.SetGoal", |params| async move {
        let goal_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing type"))?,
        )?;

        let target: f64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let goal_id = format!("goal_{}", chrono::Utc::now().timestamp_millis());
        let goal = Goal {
            id: goal_id.clone(),
            goal_type,
            target,
            current: 0.0,
        };

        storage::init().await?;
        storage::set_kv("fitness_goals", &goal_id, &serde_json::to_value(&goal)?).await?;
        Ok(serde_json::to_value(&goal)?)
    });

    registry.register("Fitness.GetGoals", |_params| async move {
        // Would get all goals from storage
        Ok(serde_json::json!([]))
    });

    registry.register("Fitness.GetDevices", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Fitness.SyncDevice", |params| async move {
        let device_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_id"))?,
        )?;

        // Would sync with fitness device
        Ok(serde_json::json!({ "success": true }))
    });
}
