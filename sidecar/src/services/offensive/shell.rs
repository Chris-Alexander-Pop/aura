use crate::services::ServiceRegistry;
use crate::utils::process;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref ACTIVE_SHELLS: RwLock<HashMap<String, ShellSession>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShellSession {
    shell_id: String,
    host: String,
    port: u16,
    shell_type: String,
    active: bool,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.Shell.Listen", |params| async move {
        let port: u16 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("port").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing port"))?,
        )?;

        let shell_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .unwrap_or(serde_json::Value::String("nc".to_string())),
        )?;

        let shell_id = format!("shell_{}", chrono::Utc::now().timestamp_millis());

        match shell_type.as_str() {
            "nc" | "netcat" => {
                process::exec_command_detached(&["nc", "-lvp", &port.to_string()]).await.ok();
            }
            _ => {}
        }

        let session = ShellSession {
            shell_id: shell_id.clone(),
            host: "0.0.0.0".to_string(),
            port,
            shell_type,
            active: true,
        };

        let mut shells = ACTIVE_SHELLS.write().await;
        shells.insert(shell_id.clone(), session.clone());

        Ok(serde_json::to_value(&session)?)
    });
}
