use crate::notify;
use crate::services::notifications;
use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;
use tokio::time::Duration;

static CALENDAR_EMIT_GEN: AtomicU64 = AtomicU64::new(0);
const CALENDAR_EMIT_DEBOUNCE_MS: u64 = 200;

static REMINDER_TICK: Once = Once::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub description: String,
    pub calendar_id: Option<String>,
    pub reminder_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Calendar.GetEvents", |params| async move {
        let start_date: Option<i64> = params
            .as_ref()
            .and_then(|p| p.get("start_date").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let end_date: Option<i64> = params
            .as_ref()
            .and_then(|p| p.get("end_date").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        storage::init().await?;
        let items = storage::scan_namespace("calendar_events").await?;
        let mut events = events_from_storage_values(items);

        filter_events_by_range(&mut events, start_date, end_date);

        Ok(serde_json::to_value(events)?)
    });

    registry.register("Calendar.CreateEvent", |params| async move {
        let title: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("title").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing title"))?,
        )?;

        let start: i64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("start").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing start"))?,
        )?;

        let end: i64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("end").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing end"))?,
        )?;

        let description: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("description").cloned())
                .unwrap_or(serde_json::Value::String(String::new())),
        )?;

        let reminder_minutes: Option<i32> = params
            .as_ref()
            .and_then(|p| p.get("reminder_minutes").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let event_id = format!("event_{}", chrono::Utc::now().timestamp_millis());
        let event = CalendarEvent {
            id: event_id.clone(),
            title,
            start,
            end,
            description,
            calendar_id: None,
            reminder_minutes,
        };

        storage::init().await?;
        storage::set_kv("calendar_events", &event_id, &serde_json::to_value(&event)?).await?;
        schedule_calendar_events_emit("create");
        Ok(serde_json::to_value(&event)?)
    });

    registry.register("Calendar.UpdateEvent", |params| async move {
        let event_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("event_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing event_id"))?,
        )?;

        let updates: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("updates").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing updates"))?;

        storage::init().await?;
        if let Some(mut event_value) = storage::get_kv("calendar_events", &event_id).await? {
            if let Some(event_obj) = event_value.as_object_mut() {
                for (key, value) in updates.as_object().unwrap() {
                    event_obj.insert(key.clone(), value.clone());
                }
                storage::set_kv("calendar_events", &event_id, &event_value).await?;
                schedule_calendar_events_emit("update");
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.DeleteEvent", |params| async move {
        let event_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("event_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing event_id"))?,
        )?;

        storage::init().await?;
        let deleted = storage::delete_kv("calendar_events", &event_id).await?;
        if deleted {
            schedule_calendar_events_emit("delete");
        }
        Ok(serde_json::json!({ "success": true, "deleted": deleted }))
    });

    registry.register("Calendar.GetCalendars", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Calendar.SyncCalendars", |_params| async move {
        // Would integrate with khal/vdirsyncer
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.GetUpcomingEvents", |params| async move {
        let limit: usize = params
            .as_ref()
            .and_then(|p| p.get("limit").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(10);

        let days: i64 = params
            .as_ref()
            .and_then(|p| p.get("days").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(7);

        storage::init().await?;
        let mut events = events_from_storage_values(storage::scan_namespace("calendar_events").await?);
        let now = chrono::Utc::now().timestamp();
        let horizon = now + days * 86_400;
        events.retain(|e| e.end >= now && e.start <= horizon);
        events.sort_by_key(|e| e.start);
        events.truncate(limit);
        Ok(serde_json::to_value(events)?)
    });

    registry.register("Calendar.SetReminder", |params| async move {
        let event_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("event_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing event_id"))?,
        )?;

        let minutes_before: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("minutes_before").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing minutes_before"))?,
        )?;

        storage::init().await?;
        if let Some(mut event_value) = storage::get_kv("calendar_events", &event_id).await? {
            if let Some(event_obj) = event_value.as_object_mut() {
                event_obj.insert("reminder_minutes".to_string(), serde_json::json!(minutes_before));
                storage::set_kv("calendar_events", &event_id, &event_value).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.ImportIcs", |params| async move {
        let _file_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("file_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?,
        )?;

        // Would parse .ics file
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.ExportIcs", |params| async move {
        let _calendar_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("calendar_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing calendar_id"))?,
        )?;

        let _file_path: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("file_path").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?,
        )?;

        // Would export .ics file
        Ok(serde_json::json!({ "success": true }))
    });

}

/// Start the 60s reminder poll (call once from the sidecar binary).
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
        if let Err(e) = check_calendar_reminders().await {
            tracing::debug!("calendar reminder tick: {}", e);
        }
    }
}

async fn check_calendar_reminders() -> Result<()> {
    storage::init().await?;
    let now = chrono::Utc::now().timestamp();
    let events = events_from_storage_values(storage::scan_namespace("calendar_events").await?);
    for event in events {
        let Some(mins) = event.reminder_minutes else {
            continue;
        };
        let trigger_at = event.start - i64::from(mins) * 60;
        if now < trigger_at || now >= event.start {
            continue;
        }
        let fired_key = format!("fired_{}", event.id);
        if storage::get_kv("calendar_reminders", &fired_key)
            .await?
            .is_some()
        {
            continue;
        }
        notifications::record_notification(
            None,
            "aura-calendar".to_string(),
            0,
            format!("Upcoming: {}", event.title),
            event.description.clone(),
            None,
            1,
            Vec::new(),
        )
        .await;
        storage::set_kv("calendar_reminders", &fired_key, &serde_json::json!(now)).await?;
        schedule_calendar_events_emit("reminder");
    }
    Ok(())
}

fn schedule_calendar_events_emit(reason: &str) {
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    let gen = CALENDAR_EMIT_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let reason = reason.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(CALENDAR_EMIT_DEBOUNCE_MS)).await;
        if CALENDAR_EMIT_GEN.load(Ordering::Relaxed) != gen {
            return;
        }
        notify::emit(
            "Calendar.EventsChanged",
            json!({ "reason": reason }),
        );
    });
}

/// Test hook — same debounce path as production CRUD/reminder handlers.
#[cfg(test)]
pub(crate) fn schedule_calendar_events_emit_for_tests(reason: &str) {
    schedule_calendar_events_emit(reason);
}


pub(crate) fn events_from_storage_values(items: Vec<serde_json::Value>) -> Vec<CalendarEvent> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub(crate) fn filter_events_by_range(
    events: &mut Vec<CalendarEvent>,
    start_date: Option<i64>,
    end_date: Option<i64>,
) {
    if let Some(start) = start_date {
        events.retain(|e| e.end >= start);
    }
    if let Some(end) = end_date {
        events.retain(|e| e.start <= end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    fn sample_event(id: &str, start: i64, end: i64) -> CalendarEvent {
        CalendarEvent {
            id: id.into(),
            title: "t".into(),
            start,
            end,
            description: String::new(),
            calendar_id: None,
            reminder_minutes: None,
        }
    }

    #[test]
    fn filter_events_by_date_range() {
        let mut events = vec![
            sample_event("a", 100, 200),
            sample_event("b", 500, 600),
            sample_event("c", 250, 350),
        ];
        filter_events_by_range(&mut events, Some(150), Some(400));
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.id == "a"));
        assert!(events.iter().any(|e| e.id == "c"));
    }

    #[test]
    fn filter_events_start_only_excludes_ending_before_window() {
        let mut events = vec![
            sample_event("early", 10, 50),
            sample_event("overlap", 100, 200),
        ];
        filter_events_by_range(&mut events, Some(80), None);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "overlap");
    }

    #[test]
    fn filter_events_end_only_excludes_starting_after_window() {
        let mut events = vec![
            sample_event("late", 500, 600),
            sample_event("overlap", 100, 200),
        ];
        filter_events_by_range(&mut events, None, Some(300));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "overlap");
    }

    #[test]
    fn filter_events_boundary_touching_edges_kept() {
        let mut events = vec![sample_event("touch", 100, 200)];
        filter_events_by_range(&mut events, Some(200), Some(100));
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn events_from_storage_skips_corrupt() {
        let valid = sample_event("ok", 1, 2);
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            serde_json::json!({ "nope": 1 }),
        ];
        assert_eq!(events_from_storage_values(items).len(), 1);
    }

    #[test]
    fn events_from_storage_empty_namespace() {
        assert!(events_from_storage_values(vec![]).is_empty());
    }

    #[tokio::test]
    async fn calendar_events_emit_debounce_coalesces() {
        static DEBOUNCE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = DEBOUNCE_LOCK.get_or_init(|| Mutex::new(())).lock().await;
        let tx = crate::notify::init_for_tests();
        let mut rx = tx.subscribe();

        schedule_calendar_events_emit_for_tests("create");
        schedule_calendar_events_emit_for_tests("update");
        tokio::time::sleep(Duration::from_millis(CALENDAR_EMIT_DEBOUNCE_MS + 80)).await;

        let mut count = 0;
        while let Ok(raw) = rx.try_recv() {
            let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
            assert_eq!(v["method"], "Calendar.EventsChanged");
            count += 1;
        }
        assert!(count <= 1, "expected at most one debounced emit, got {count}");
    }
}
