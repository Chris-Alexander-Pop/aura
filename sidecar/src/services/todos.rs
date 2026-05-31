use crate::notify;
use crate::services::notifications;
use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;
use tokio::time::Duration;

static TODOS_EMIT_GEN: AtomicU64 = AtomicU64::new(0);
const TODOS_EMIT_DEBOUNCE_MS: u64 = 200;
static REMINDER_TICK: Once = Once::new();

const NS_ITEMS: &str = "todos";
const NS_PROJECTS: &str = "todo_projects";
const NS_REMINDERS_FIRED: &str = "todo_reminders";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub project_id: Option<String>,
    pub due_at: Option<i64>,
    pub completed: bool,
    pub reminder_minutes: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoProject {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Todos.List", |params| async move {
        let project_id: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("project_id").cloned())
            .and_then(|v| serde_json::from_value(v).ok());
        let include_completed: bool = params
            .as_ref()
            .and_then(|p| p.get("include_completed").cloned())
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        storage::init().await?;
        let mut items = todos_from_storage_values(storage::scan_namespace(NS_ITEMS).await?);
        if let Some(pid) = project_id {
            items.retain(|t| t.project_id.as_deref() == Some(pid.as_str()));
        }
        if !include_completed {
            items.retain(|t| !t.completed);
        }
        items.sort_by_key(|t| (t.completed, t.due_at.unwrap_or(i64::MAX), t.created_at));
        Ok(serde_json::to_value(items)?)
    });

    registry.register("Todos.Create", |params| async move {
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

        let project_id: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("project_id").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut due_at: Option<i64> = params
            .as_ref()
            .and_then(|p| p.get("due_at").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        if due_at.is_none() {
            if let Some(text) = params
                .as_ref()
                .and_then(|p| p.get("due_text").cloned())
                .and_then(|v| v.as_str().map(str::to_string))
            {
                due_at = parse_due_text(&text);
            }
        }

        let reminder_minutes: Option<i32> = params
            .as_ref()
            .and_then(|p| p.get("reminder_minutes").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let now = Utc::now().timestamp_millis();
        let id = format!("todo_{now}");
        let item = TodoItem {
            id: id.clone(),
            title,
            description,
            project_id,
            due_at,
            completed: false,
            reminder_minutes,
            created_at: now,
            updated_at: now,
        };

        storage::init().await?;
        storage::set_kv(NS_ITEMS, &id, &serde_json::to_value(&item)?).await?;
        schedule_todos_changed("create");
        Ok(serde_json::to_value(&item)?)
    });

    registry.register("Todos.Update", |params| async move {
        let id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing id"))?,
        )?;

        let updates: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("updates").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing updates"))?;

        storage::init().await?;
        if let Some(mut value) = storage::get_kv(NS_ITEMS, &id).await? {
            if let Some(obj) = value.as_object_mut() {
                for (key, val) in updates.as_object().unwrap() {
                    if key == "due_text" {
                        if let Some(text) = val.as_str() {
                            obj.insert(
                                "due_at".to_string(),
                                json!(parse_due_text(text)),
                            );
                        }
                    } else {
                        obj.insert(key.clone(), val.clone());
                    }
                }
                obj.insert(
                    "updated_at".to_string(),
                    json!(Utc::now().timestamp_millis()),
                );
                storage::set_kv(NS_ITEMS, &id, &value).await?;
                schedule_todos_changed("update");
            }
        }

        Ok(json!({ "success": true }))
    });

    registry.register("Todos.Delete", |params| async move {
        let id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing id"))?,
        )?;

        storage::init().await?;
        let deleted = storage::delete_kv(NS_ITEMS, &id).await?;
        if deleted {
            schedule_todos_changed("delete");
        }
        Ok(json!({ "success": true, "deleted": deleted }))
    });

    registry.register("Todos.ListProjects", |_params| async move {
        storage::init().await?;
        let mut projects =
            projects_from_storage_values(storage::scan_namespace(NS_PROJECTS).await?);
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(serde_json::to_value(projects)?)
    });

    registry.register("Todos.CreateProject", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let color: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("color").cloned())
                .unwrap_or(serde_json::Value::String("#7aa2f7".to_string())),
        )?;

        let now = Utc::now().timestamp_millis();
        let id = format!("project_{now}");
        let project = TodoProject {
            id: id.clone(),
            name,
            color,
        };

        storage::init().await?;
        storage::set_kv(NS_PROJECTS, &id, &serde_json::to_value(&project)?).await?;
        schedule_todos_changed("project_create");
        Ok(serde_json::to_value(&project)?)
    });

    registry.register("Todos.ParseDueDate", |params| async move {
        let text: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("text").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing text"))?,
        )?;
        Ok(json!({
            "due_at": parse_due_text(&text),
            "parsed": parse_due_text(&text).is_some()
        }))
    });
}

/// Start the 60s todo reminder poll (call once from the sidecar binary).
pub fn spawn_reminder_tick() {
    REMINDER_TICK.call_once(|| {
        tokio::spawn(async {
            reminder_tick_loop().await;
        });
    });
}

async fn reminder_tick_loop() {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        if let Err(e) = check_todo_reminders().await {
            tracing::debug!("todo reminder tick: {}", e);
        }
    }
}

async fn check_todo_reminders() -> Result<()> {
    storage::init().await?;
    let now = Utc::now().timestamp();
    let items = todos_from_storage_values(storage::scan_namespace(NS_ITEMS).await?);
    for item in items {
        if item.completed {
            continue;
        }
        let Some(due) = item.due_at else {
            continue;
        };
        let Some(mins) = item.reminder_minutes else {
            continue;
        };
        let trigger_at = due - i64::from(mins) * 60;
        if now < trigger_at || now >= due {
            continue;
        }
        let fired_key = format!("fired_{}", item.id);
        if storage::get_kv(NS_REMINDERS_FIRED, &fired_key)
            .await?
            .is_some()
        {
            continue;
        }
        notifications::record_notification(
            None,
            "aura-todos".to_string(),
            0,
            format!("Due soon: {}", item.title),
            item.description.clone(),
            None,
            1,
            Vec::new(),
        )
        .await;
        storage::set_kv(NS_REMINDERS_FIRED, &fired_key, &json!(now)).await?;
        schedule_todos_changed("reminder");
    }
    Ok(())
}

fn schedule_todos_changed(reason: &str) {
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    let gen = TODOS_EMIT_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let reason = reason.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(TODOS_EMIT_DEBOUNCE_MS)).await;
        if TODOS_EMIT_GEN.load(Ordering::Relaxed) != gen {
            return;
        }
        notify::emit("Todos.Changed", json!({ "reason": reason }));
    });
}

/// Natural-language due date parsing (v1 heuristics; no external NLP crate).
pub fn parse_due_text(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp());
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return local_date_at_hour(date, 9);
    }

    let lower = trimmed.to_ascii_lowercase();
    let today = Local::now().date_naive();
    match lower.as_str() {
        "today" => return local_date_at_hour(today, 17),
        "tomorrow" => return local_date_at_hour(today.succ_opt()?, 9),
        "next week" => return local_date_at_hour(today + chrono::Duration::days(7), 9),
        _ => {}
    }

    if let Some(rest) = lower.strip_prefix("in ") {
        let parts: Vec<_> = rest.split_whitespace().collect();
        if parts.len() == 2 {
            if let Ok(n) = parts[0].parse::<i64>() {
                let unit = parts[1].trim_end_matches('s');
                let days = match unit {
                    "day" => n,
                    "week" => n * 7,
                    "hour" => return Some(Utc::now().timestamp() + n * 3600),
                    _ => return None,
                };
                return local_date_at_hour(today + chrono::Duration::days(days), 9);
            }
        }
    }

    None
}

fn local_date_at_hour(date: NaiveDate, hour: u32) -> Option<i64> {
    let naive = date.and_hms_opt(hour, 0, 0)?;
    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.timestamp())
}

pub(crate) fn todos_from_storage_values(items: Vec<serde_json::Value>) -> Vec<TodoItem> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub(crate) fn projects_from_storage_values(items: Vec<serde_json::Value>) -> Vec<TodoProject> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_due_text_tomorrow_and_rfc3339() {
        assert!(parse_due_text("tomorrow").is_some());
        assert!(parse_due_text("2026-06-15").is_some());
        assert!(parse_due_text("not-a-date").is_none());
    }

    #[test]
    fn parse_due_text_in_days() {
        let ts = parse_due_text("in 3 days").expect("in 3 days");
        let today = Local::now().date_naive();
        let expected = today + chrono::Duration::days(3);
        let got = Local.timestamp_opt(ts, 0).single().unwrap().date_naive();
        assert_eq!(got, expected);
    }

    #[test]
    fn todos_from_storage_skips_corrupt() {
        let valid = TodoItem {
            id: "t1".into(),
            title: "x".into(),
            description: String::new(),
            project_id: None,
            due_at: None,
            completed: false,
            reminder_minutes: None,
            created_at: 1,
            updated_at: 1,
        };
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            json!({ "bad": true }),
        ];
        assert_eq!(todos_from_storage_values(items).len(), 1);
    }
}
