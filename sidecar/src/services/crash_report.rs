//! Accept crash dumps from the React UI (WebKit) and write them under
//! `~/.local/share/aura/crashes/` via [`crate::utils::crash`].

use crate::services::ServiceRegistry;
use crate::utils::crash::{write_crash_dump, CrashDump};
use anyhow::bail;
use chrono::Utc;
use serde_json::json;

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".into())
}

fn clamp_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Crash.Report", |params| async move {
        let Some(p) = params else {
            bail!("Crash.Report requires params");
        };

        let message = p
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("react crash")
            .trim();
        if message.is_empty() {
            bail!("Crash.Report.message must be non-empty");
        }

        let kind = p
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("uncaught");
        let kind = match kind {
            "panic" | "uncaught" | "exit" | "web-process" => kind,
            _ => "uncaught",
        };

        let route = p
            .get("route")
            .and_then(|v| v.as_str())
            .map(|s| clamp_str(s, 200));
        let stack = p
            .get("stack")
            .and_then(|v| v.as_str())
            .map(|s| clamp_str(s, 16_384));

        let mut msg = clamp_str(message, 2_048);
        if let Some(r) = route.as_deref() {
            msg = format!("{msg} [route={r}]");
        }

        let dump = CrashDump {
            ts: Utc::now().to_rfc3339(),
            component: "react".into(),
            kind: kind.into(),
            message: msg,
            stack,
            host: hostname(),
            aura: format!("ags-sidecar {}", env!("CARGO_PKG_VERSION")),
            exit_status: None,
            signal: None,
        };

        let path = write_crash_dump(&dump)?;
        Ok(json!({
            "ok": true,
            "path": path.to_string_lossy(),
        }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::crash::crash_dir;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn report_writes_dump() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("AURA_CRASH_DIR", dir.path());

        let mut registry = ServiceRegistry::new();
        register(&mut registry);
        let handler = registry.handler_for("Crash.Report").expect("handler");

        let params = Some(json!({
            "message": "boom",
            "kind": "uncaught",
            "route": "#/calendar",
            "stack": "Error: boom\n    at foo",
        }));

        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(handler(params))
            .expect("report ok");

        assert_eq!(result["ok"], true);
        let count = std::fs::read_dir(crash_dir())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .count();
        assert!(count >= 1);

        std::env::remove_var("AURA_CRASH_DIR");
    }
}
