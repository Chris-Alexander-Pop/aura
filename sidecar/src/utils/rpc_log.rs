//! RPC request logging with redaction for secrets and settings payloads.

use serde_json::Value;
use std::time::Duration;

const REDACTED: &str = "<redacted>";

/// Methods whose params must never appear in logs (even with `AURA_RPC_LOG_PARAMS=1`).
fn params_always_redacted(method: &str) -> bool {
    method.starts_with("Storage.")
        || method.starts_with("Settings.")
        || method.contains("Keyring")
        || method.contains("Vault.Store")
        || method.contains("Vpn.SaveCredentials")
}

fn redact_value(key: &str, value: &Value) -> Value {
    let sensitive = matches!(
        key,
        "password"
            | "secret"
            | "token"
            | "value"
            | "credentials"
            | "api_key"
            | "private_key"
    );
    if sensitive {
        Value::String(REDACTED.into())
    } else if value.is_object() {
        redact_params_object(value.as_object().unwrap())
    } else {
        value.clone()
    }
}

fn redact_params_object(map: &serde_json::Map<String, Value>) -> Value {
    let mut out = serde_json::Map::new();
    for (k, v) in map {
        out.insert(k.clone(), redact_value(k, v));
    }
    Value::Object(out)
}

pub fn redact_params_for_log(method: &str, params: &Value) -> String {
    if params_always_redacted(method) {
        return REDACTED.to_string();
    }
    match params {
        Value::Object(map) => redact_params_object(map).to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn storage_params_always_redacted() {
        let p = json!({ "namespace": "secrets", "key": "k", "value": { "token": "x" } });
        assert_eq!(
            redact_params_for_log("Storage.Set", &p),
            REDACTED
        );
    }

    #[test]
    fn dev_params_redact_sensitive_keys() {
        let p = json!({ "password": "secret", "ssid": "home" });
        let s = redact_params_for_log("Network.Connect", &p);
        assert!(s.contains(REDACTED));
        assert!(s.contains("home"));
    }
}

pub fn log_rpc_completed(method: &str, params: Option<&Value>, elapsed: Duration) {
    tracing::debug!(
        method = %method,
        elapsed_ms = elapsed.as_millis(),
        "rpc completed"
    );

    if std::env::var("AURA_RPC_LOG_PARAMS").ok().as_deref() == Some("1") {
        if let Some(p) = params {
            tracing::debug!(
                method = %method,
                params = %redact_params_for_log(method, p),
                "rpc params"
            );
        }
    }
}
