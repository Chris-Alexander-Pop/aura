use crate::services::ServiceRegistry;
use crate::utils::process;

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.DNS.ZoneTransfer", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let output = process::exec_command(&["dig", "axfr", &domain]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.DNS.ReverseLookup", |params| async move {
        let ip_range: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("ip_range").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing ip_range"))?,
        )?;

        let output = process::exec_command(&["nmap", "-sL", &ip_range]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.DNS.Query", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let record_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("record_type").cloned())
                .unwrap_or(serde_json::Value::String("A".to_string())),
        )?;

        let output = process::exec_command(&["dig", &record_type, &domain]).await?;
        Ok(serde_json::json!({ "output": output }))
    });
}
