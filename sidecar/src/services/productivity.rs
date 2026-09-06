use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

static PRODUCTIVITY_EMIT_GEN: AtomicU64 = AtomicU64::new(0);

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
                        if std::env::var("AURA_SKIP_NOTIFY_SEND").is_err() {
                            let _ = process::exec_command(&[
                                "notify-send",
                                "Timer",
                                &format!("Timer '{}' completed!", timer.name),
                            ])
                            .await;
                        }
                        break;
                    } else {
                        timer.remaining_seconds = timer.duration_seconds - elapsed;
                    }
                    schedule_productivity_emit();
                } else {
                    break;
                }
            }
            schedule_productivity_emit();
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

        storage::init().await?;
        storage::set_kv("productivity", "pomodoro", &serde_json::to_value(&pomodoro)?).await?;

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
                                if std::env::var("AURA_SKIP_NOTIFY_SEND").is_err() {
                                    let _ = process::exec_command(&[
                                        "notify-send",
                                        "Pomodoro",
                                        "Work session complete! Take a break.",
                                    ])
                                    .await;
                                }
                                schedule_productivity_emit();
                            } else {
                                pomo.current_phase = "work".to_string();
                                pomo.remaining_seconds = (work_minutes * 60) as u64;
                                if std::env::var("AURA_SKIP_NOTIFY_SEND").is_err() {
                                    let _ = process::exec_command(&[
                                        "notify-send",
                                        "Pomodoro",
                                        "Break over! Back to work.",
                                    ])
                                    .await;
                                }
                                schedule_productivity_emit();
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                schedule_productivity_emit();
                drop(state);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(serde_json::to_value(&pomodoro)?)
    });

    registry.register("Productivity.GetPomodoroStatus", |_params| async move {
        let state = POMODORO.read().await;
        if state.is_some() {
            return Ok(serde_json::to_value(state.clone())?);
        }
        storage::init().await?;
        if let Some(v) = storage::get_kv("productivity", "pomodoro").await? {
            if let Ok(p) = serde_json::from_value::<PomodoroState>(v) {
                return Ok(serde_json::to_value(Some(p))?);
            }
        }
        Ok(serde_json::Value::Null)
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
        let enabled = focus_mode_enabled_from_value(
            storage::get_kv("productivity", "focus_mode").await?.as_ref(),
        );
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
        storage::init().await?;
        let items = storage::scan_namespace("productivity_tasks").await?;
        Ok(serde_json::to_value(&tasks_from_storage_values(items))?)
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
        let task_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("task_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?,
        )?;

        storage::init().await?;
        let deleted = storage::delete_kv("productivity_tasks", &task_id).await?;
        Ok(serde_json::json!({ "success": true, "deleted": deleted }))
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
        Ok(snapshot_productivity_stats().await?)
    });
}

pub(crate) async fn snapshot_productivity_stats() -> anyhow::Result<serde_json::Value> {
    let timers = TIMERS.read().await;
    let active_timers = timers.iter().filter(|t| t.active).count();
    let pomodoro = POMODORO.read().await.clone();

    storage::init().await?;
    let focus_mode_enabled = focus_mode_enabled_from_value(
        storage::get_kv("productivity", "focus_mode").await?.as_ref(),
    );
    let items = storage::scan_namespace("productivity_tasks").await?;
    let tasks = tasks_from_storage_values(items);
    let open_tasks = tasks.iter().filter(|t| !t.completed).count();

    Ok(build_productivity_stats(
        active_timers,
        timers.len(),
        pomodoro.as_ref(),
        focus_mode_enabled,
        open_tasks,
        tasks.len(),
    ))
}

pub async fn reset_productivity_state_for_tests() {
    TIMERS.write().await.clear();
    *POMODORO.write().await = None;
    PRODUCTIVITY_EMIT_GEN.store(0, Ordering::Relaxed);
}

pub(crate) fn tasks_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Task> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

fn schedule_productivity_emit() {
    let gen = PRODUCTIVITY_EMIT_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if PRODUCTIVITY_EMIT_GEN.load(Ordering::Relaxed) != gen {
            return;
        }
        notify::emit(
            "Productivity.TimerTick",
            serde_json::json!({ "ts": chrono::Utc::now().timestamp_millis() }),
        );
    });
}

pub(crate) fn focus_mode_enabled_from_value(value: Option<&serde_json::Value>) -> bool {
    value.and_then(|v| v.as_bool()).unwrap_or(false)
}

pub(crate) fn build_productivity_stats(
    active_timers: usize,
    total_timers: usize,
    pomodoro: Option<&PomodoroState>,
    focus_mode_enabled: bool,
    open_tasks: usize,
    total_tasks: usize,
) -> serde_json::Value {
    serde_json::json!({
        "active_timers": active_timers,
        "total_timers": total_timers,
        "pomodoro_active": pomodoro.map(|p| p.active).unwrap_or(false),
        "pomodoro_phase": pomodoro.map(|p| p.current_phase.clone()),
        "focus_mode_enabled": focus_mode_enabled,
        "screen_time_minutes": 0,
        "screen_time_note": "Phase 2: ActivityWatch integration",
        "open_task_count": open_tasks,
        "task_count": total_tasks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_mode_enabled_rejects_non_bool_storage() {
        assert!(!focus_mode_enabled_from_value(Some(&serde_json::json!("yes"))));
        assert!(!focus_mode_enabled_from_value(Some(&serde_json::json!(1))));
        assert!(focus_mode_enabled_from_value(Some(&serde_json::json!(true))));
        assert!(!focus_mode_enabled_from_value(None));
    }

    #[test]
    fn build_stats_includes_pomodoro_phase_when_active() {
        let pomo = PomodoroState {
            work_minutes: 25,
            break_minutes: 5,
            current_phase: "work".into(),
            remaining_seconds: 60,
            active: true,
        };
        let stats = build_productivity_stats(1, 2, Some(&pomo), false, 3, 5);
        assert_eq!(stats.get("active_timers").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(stats.get("open_task_count").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(stats.get("pomodoro_active"), Some(&serde_json::json!(true)));
        assert_eq!(
            stats.get("pomodoro_phase").and_then(|v| v.as_str()),
            Some("work")
        );
    }

    #[test]
    fn build_stats_null_pomodoro_phase_when_inactive() {
        let stats = build_productivity_stats(0, 0, None, true, 0, 0);
        assert_eq!(stats.get("focus_mode_enabled"), Some(&serde_json::json!(true)));
        assert!(stats.get("pomodoro_phase").map(|v| v.is_null()).unwrap_or(false));
        assert_eq!(stats.get("pomodoro_active"), Some(&serde_json::json!(false)));
    }

    #[test]
    fn tasks_from_storage_skips_corrupt_entries() {
        let valid = Task {
            id: "task_1".into(),
            title: "ok".into(),
            description: String::new(),
            due_date: None,
            completed: false,
            created_at: 1,
        };
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            serde_json::json!({ "title": "missing id" }),
            serde_json::json!("bare"),
        ];
        let tasks = tasks_from_storage_values(items);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "task_1");
    }

    #[test]
    fn build_stats_counts_open_tasks_only() {
        let stats = build_productivity_stats(0, 0, None, false, 2, 5);
        assert_eq!(stats.get("open_task_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(stats.get("task_count").and_then(|v| v.as_u64()), Some(5));
    }

    #[test]
    fn pomodoro_storage_json_rejects_corrupt_shapes() {
        use serde_json::json;
        assert!(serde_json::from_value::<PomodoroState>(json!("bad")).is_err());
        assert!(serde_json::from_value::<PomodoroState>(json!({ "work_minutes": 25 })).is_err());
    }

    #[tokio::test]
    async fn schedule_productivity_emit_debounces() {
        crate::notify::init_for_tests();
        schedule_productivity_emit();
        schedule_productivity_emit();
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}
