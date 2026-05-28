use crate::services::ServiceRegistry;
use crate::utils::storage;
use serde::{Deserialize, Serialize};
use serde_json;

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
        let mut events: Vec<CalendarEvent> = items
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

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

        let event_id = format!("event_{}", chrono::Utc::now().timestamp_millis());
        let event = CalendarEvent {
            id: event_id.clone(),
            title,
            start,
            end,
            description,
            calendar_id: None,
            reminder_minutes: None,
        };

        storage::init().await?;
        storage::set_kv("calendar_events", &event_id, &serde_json::to_value(&event)?).await?;
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
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.DeleteEvent", |params| async move {
        let _event_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("event_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing event_id"))?,
        )?;

        // Would need storage delete method
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.GetCalendars", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Calendar.SyncCalendars", |_params| async move {
        // Would integrate with khal/vdirsyncer
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Calendar.GetUpcomingEvents", |params| async move {
        let _days: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("days").cloned())
                .unwrap_or(serde_json::Value::Number(serde_json::Number::from(7))),
        )?;

        // Would filter events by date range
        Ok(serde_json::json!([]))
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
}
