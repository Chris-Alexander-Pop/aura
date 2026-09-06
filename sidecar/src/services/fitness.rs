use crate::services::ServiceRegistry;
use crate::utils::storage;
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
        storage::init().await?;
        let items = storage::scan_namespace("fitness_workouts").await?;
        let workouts = workouts_from_storage_values(items);
        let today = chrono::Utc::now().date_naive();
        let mut workout_minutes = 0u64;
        for w in workouts {
            if w.end_time.is_none() {
                continue;
            }
            let start_ms = w.start_time;
            let start_date = chrono::DateTime::from_timestamp_millis(start_ms)
                .map(|dt| dt.date_naive())
                .unwrap_or(today);
            if start_date != today {
                continue;
            }
            if let Some(secs) = w.duration_seconds {
                workout_minutes += secs / 60;
            }
        }
        Ok(serde_json::json!({
            "date": today.to_string(),
            "steps": 0,
            "calories": 0,
            "distance_km": 0.0,
            "workout_minutes": workout_minutes
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

    registry.register("Fitness.StopWorkout", |params| async move {
        let workout_id: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("id").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        storage::init().await?;
        let items = storage::scan_namespace("fitness_workouts").await?;
        let workouts = workouts_from_storage_values(items);

        let target_idx = if let Some(ref id) = workout_id {
            workouts.iter().position(|w| w.id == *id && w.end_time.is_none())
        } else {
            workouts.iter().position(|w| w.end_time.is_none())
        };

        let Some(idx) = target_idx else {
            return Ok(serde_json::json!({ "success": false, "message": "no active workout" }));
        };

        let end = chrono::Utc::now().timestamp_millis();
        let mut finished = workouts[idx].clone();
        finished.end_time = Some(end);
        finished.duration_seconds = Some(((end - finished.start_time).max(0) / 1000) as u64);
        storage::set_kv(
            "fitness_workouts",
            &finished.id,
            &serde_json::to_value(&finished)?,
        )
        .await?;
        Ok(serde_json::to_value(&finished)?)
    });

    registry.register("Fitness.GetWorkoutHistory", |_params| async move {
        storage::init().await?;
        let items = storage::scan_namespace("fitness_workouts").await?;
        Ok(serde_json::to_value(&workouts_from_storage_values(items))?)
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
        storage::init().await?;
        let items = storage::scan_namespace("fitness_goals").await?;
        Ok(serde_json::to_value(&goals_from_storage_values(items))?)
    });

    registry.register("Fitness.GetDevices", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Fitness.SyncDevice", |params| async move {
        let _device_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_id"))?,
        )?;

        // Would sync with fitness device
        Ok(serde_json::json!({ "success": true }))
    });
}

pub(crate) fn goals_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Goal> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub(crate) fn workouts_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Workout> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn goals_from_storage_skips_corrupt_entries() {
        let valid = Goal {
            id: "g1".into(),
            goal_type: "steps".into(),
            target: 10_000.0,
            current: 0.0,
        };
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            json!({ "bad": true }),
            json!("string"),
        ];
        let out = goals_from_storage_values(items);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].goal_type, "steps");
    }

    #[test]
    fn workouts_from_storage_empty_namespace() {
        assert!(workouts_from_storage_values(vec![]).is_empty());
    }

    #[test]
    fn goals_from_storage_preserves_multiple_valid_entries() {
        let g1 = Goal {
            id: "g1".into(),
            goal_type: "steps".into(),
            target: 10_000.0,
            current: 500.0,
        };
        let g2 = Goal {
            id: "g2".into(),
            goal_type: "calories".into(),
            target: 500.0,
            current: 120.0,
        };
        let items = vec![
            serde_json::to_value(&g1).unwrap(),
            serde_json::to_value(&g2).unwrap(),
        ];
        let out = goals_from_storage_values(items);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].goal_type, "steps");
        assert_eq!(out[1].goal_type, "calories");
    }

    #[test]
    fn workouts_from_storage_skips_corrupt_entries() {
        let valid = Workout {
            id: "w1".into(),
            workout_type: "run".into(),
            start_time: 100,
            end_time: Some(200),
            duration_seconds: Some(100),
            data: json!({}),
        };
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            json!({ "incomplete": true }),
        ];
        let out = workouts_from_storage_values(items);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].workout_type, "run");
    }
}
