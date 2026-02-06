use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub service: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
    pub read: bool,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Communication.GetMessages", |_params| async move {
        // Would aggregate from various sources
        Ok(serde_json::json!([]))
    });

    registry.register("Communication.MarkRead", |params| async move {
        let _message_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("message_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing message_id"))?,
        )?;

        // Would mark message as read
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Communication.SendMessage", |params| async move {
        let _service: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("service").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing service"))?,
        )?;

        let _recipient: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("recipient").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing recipient"))?,
        )?;

        let _message: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("message").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing message"))?,
        )?;

        // Would send via appropriate service
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Communication.GetContacts", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Communication.GetConversations", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Communication.GetNotificationSettings", |_params| async move {
        storage::init().await?;
        let settings = storage::get_kv("communication", "notification_settings").await?
            .unwrap_or(serde_json::json!({}));
        Ok(settings)
    });

    registry.register("Communication.SetNotificationSettings", |params| async move {
        let settings: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("settings").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing settings"))?;

        storage::init().await?;
        storage::set_kv("communication", "notification_settings", &settings).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Communication.GetActiveCalls", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Communication.MuteNotifications", |params| async move {
        let _duration_minutes: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("duration_minutes").cloned())
                .unwrap_or(serde_json::Value::Number(serde_json::Number::from(60))),
        )?;

        // Would mute notifications
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Communication.GetCommunicationApps", |_params| async move {
        let mut apps = Vec::new();

        // Check for common messaging apps
        let app_names = vec!["telegram", "discord", "signal", "whatsapp", "slack"];
        for app in app_names {
            if process::exec_command(&["which", app]).await.is_ok() {
                apps.push(app.to_string());
            }
        }

        Ok(serde_json::to_value(&apps)?)
    });

    registry.register("Communication.LaunchApp", |params| async move {
        let app_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("app_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing app_name"))?,
        )?;

        process::exec_command_detached(&[&app_name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });
}
