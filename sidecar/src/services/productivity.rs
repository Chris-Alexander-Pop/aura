use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub id: String,
    pub name: String,
    pub duration_seconds: u64,
    pub remaining_seconds: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroState {
    pub work_minutes: u32,
    pub break_minutes: u32,
    pub current_phase: String, // "work" or "break"
    pub remaining_seconds: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub due_date: Option<i64>,
    pub completed: bool,
    pub created_at: i64,
}

lazy_static::lazy_static! {
    static ref TIMERS: RwLock<Vec<Timer>> = RwLock::new(Vec::new());
    static ref POMODORO: RwLock<Option<PomodoroState>> = RwLock::new(None);
}

pub fn register(registry: &mut ServiceRegistry) {
    // Timer management
    registry.register("Productivity.CreateTimer", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let duration_seconds: u64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("duration_seconds").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing duration_seconds"))?,
        )?;

        let timer_id = format!("timer_{}", chrono::Utc::now().timestamp_millis());
        let timer = Timer {
            id: timer_id.clone(),
            name,
            duration_seconds,
            remaining_seconds: duration_seconds,
            active: true,
        };

        let mut timers = TIMERS.write().await;
        timers.push(timer.clone());

        // Start timer task
        let timer_id_clone = timer_id.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let mut timers = TIMERS.write().await;
                if let Some(timer) = timers.iter_mut().find(|t| t.id == timer_id_clone) {
                    let elapsed = start.elapsed().as_secs();
                    if elapsed >= timer.duration_seconds {
                        timer.active = false;
                        timer.remaining_seconds = 0;
                        // Send notification
                        let _ = process::exec_command(&["notify-send", "Timer", &format!("Timer '{}' completed!", timer.name)]).await;
                        break;
                    } else {
                        timer.remaining_seconds = timer.duration_seconds - elapsed;
                    }
                } else {
                    break;
                }
            }
        });

        Ok(serde_json::to_value(&timer)?)
    });

    registry.register("Productivity.GetTimers", |_params| async move {
        let timers = TIMERS.read().await;
        Ok(serde_json::to_value(timers.clone())?)
    });

    registry.register("Productivity.CancelTimer", |params| async move {
        let timer_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("timer_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing timer_id"))?,
        )?;

        let mut timers = TIMERS.write().await;
        timers.retain(|t| t.id != timer_id);
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Productivity.CreatePomodoro", |params| async move {
        let work_minutes: u32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("work_minutes").cloned())
                .unwrap_or(serde_json::Value::Number(serde_json::Number::from(25))),
        )?;

        let break_minutes: u32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("break_minutes").cloned())
                .unwrap_or(serde_json::Value::Number(serde_json::Number::from(5))),
        )?;

        let pomodoro = PomodoroState {
            work_minutes,
            break_minutes,
            current_phase: "work".to_string(),
            remaining_seconds: (work_minutes * 60) as u64,
            active: true,
        };

        let mut state = POMODORO.write().await;
        *state = Some(pomodoro.clone());

        // Start pomodoro task
        tokio::spawn(async move {
            let work_minutes = work_minutes;
            let break_minutes = break_minutes;
            loop {
                let mut state = POMODORO.write().await;
                if let Some(ref mut pomo) = *state {
                    if pomo.active {
                        if pomo.remaining_seconds > 0 {
                            pomo.remaining_seconds -= 1;
                        } else {
                            if pomo.current_phase == "work" {
                                pomo.current_phase = "break".to_string();
                                pomo.remaining_seconds = (break_minutes * 60) as u64;
                                let _ = process::exec_command(&["notify-send", "Pomodoro", "Work session complete! Take a break."]).await;
                            } else {
                                pomo.current_phase = "work".to_string();
                                pomo.remaining_seconds = (work_minutes * 60) as u64;
                                let _ = process::exec_command(&["notify-send", "Pomodoro", "Break over! Back to work."]).await;
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                drop(state);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(serde_json::to_value(&pomodoro)?)
    });

    registry.register("Productivity.GetPomodoroStatus", |_params| async move {
        let state = POMODORO.read().await;
        Ok(serde_json::to_value(state.clone())?)
    });

    registry.register("Productivity.SetFocusMode", |params| async move {
        let enabled: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("enabled").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing enabled"))?,
        )?;

        // Store focus mode state
        storage::init().await?;
        storage::set_kv("productivity", "focus_mode", &serde_json::json!(enabled)).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Productivity.GetFocusModeStatus", |_params| async move {
        storage::init().await?;
        let enabled = storage::get_kv("productivity", "focus_mode").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(serde_json::json!({ "enabled": enabled }))
    });

    // Task management
    registry.register("Productivity.CreateTask", |params| async move {
        let title: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("title").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing title"))?,
        )?;

        let description: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("description").cloned())
                .unwrap_or(serde_json::Value::String(String::new())),
        )?;

        let due_date: Option<i64> = params
            .as_ref()
            .and_then(|p| p.get("due_date").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());
        let task = Task {
            id: task_id.clone(),
            title,
            description,
            due_date,
            completed: false,
            created_at: chrono::Utc::now().timestamp_millis(),
        };

        storage::init().await?;
        storage::set_kv("productivity_tasks", &task_id, &serde_json::to_value(&task)?).await?;
        Ok(serde_json::to_value(&task)?)
    });

    registry.register("Productivity.GetTasks", |_params| async move {
        // Note: Would need storage list method to get all tasks
        // For now, return empty list
        Ok(serde_json::json!([]))
    });

    registry.register("Productivity.UpdateTask", |params| async move {
        let task_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("task_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?,
        )?;

        let updates: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("updates").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing updates"))?;

        storage::init().await?;
        if let Some(mut task_value) = storage::get_kv("productivity_tasks", &task_id).await? {
            if let Some(task_obj) = task_value.as_object_mut() {
                for (key, value) in updates.as_object().unwrap() {
                    task_obj.insert(key.clone(), value.clone());
                }
                storage::set_kv("productivity_tasks", &task_id, &task_value).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Productivity.DeleteTask", |params| async move {
        let _task_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("task_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?,
        )?;

        // Would need storage delete method
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Productivity.GetScreenTime", |_params| async move {
        // Screen time tracking would require additional tooling
        Ok(serde_json::json!({ "minutes": 0 }))
    });

    registry.register("Productivity.GetAppUsage", |_params| async move {
        // App usage tracking would require additional tooling
        Ok(serde_json::json!([]))
    });

    registry.register("Productivity.GetStats", |_params| async move {
        let timers = TIMERS.read().await;
        let active_timers = timers.iter().filter(|t| t.active).count();
        let pomodoro = POMODORO.read().await.clone();

        storage::init().await?;
        let focus_mode_enabled = storage::get_kv("productivity", "focus_mode")
            .await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(serde_json::json!({
            "active_timers": active_timers,
            "total_timers": timers.len(),
            "pomodoro_active": pomodoro.as_ref().map(|p| p.active).unwrap_or(false),
            "pomodoro_phase": pomodoro.as_ref().map(|p| p.current_phase.clone()),
            "focus_mode_enabled": focus_mode_enabled,
            "screen_time_minutes": 0,
            "task_count": 0,
        }))
    });
}
