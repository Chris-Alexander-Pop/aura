use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::Result;
use serde_json;

/// Generic key-value storage service backed by SQLite.
///
/// Methods:
/// - Storage.Set(namespace: string, key: string, value: any)
/// - Storage.Get(namespace: string, key: string) -> any | null
pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Storage.Set", |params| async move {
        let ns: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("namespace").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing namespace"))?,
        )?;

        let key: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("key").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
        )?;

        let value = params
            .as_ref()
            .and_then(|p| p.get("value").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing value"))?;

        storage::init().await?;
        storage::set_kv(&ns, &key, &value).await?;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Storage.Get", |params| async move {
        let ns: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("namespace").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing namespace"))?,
        )?;

        let key: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("key").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?,
        )?;

        storage::init().await?;
        let value = storage::get_kv(&ns, &key).await?;

        Ok(serde_json::json!({ "value": value }))
    });
}

