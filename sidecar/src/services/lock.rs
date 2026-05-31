//! Lock-screen preferences, fingerprint probe, and logind sleep inhibitors.

use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;

const LOCK_NS: &str = "lock";
const LOCK_KEY: &str = "screen_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockScreenConfig {
    pub show_clock: bool,
    pub show_notifications: bool,
    pub show_calendar: bool,
    pub show_media: bool,
}

pub fn default_lock_config() -> LockScreenConfig {
    LockScreenConfig {
        show_clock: true,
        show_notifications: false,
        show_calendar: false,
        show_media: false,
    }
}

fn validate_lock_config(config: &LockScreenConfig) -> Result<()> {
    let _ = config;
    Ok(())
}

pub fn merge_lock_partial(base: &LockScreenConfig, partial: &Map<String, Value>) -> Result<LockScreenConfig> {
    let mut out = base.clone();
    let mut value = serde_json::to_value(&out)?;
    let obj = value.as_object_mut().expect("lock config object");
    for (k, v) in partial {
        if !matches!(
            k.as_str(),
            "show_clock" | "show_notifications" | "show_calendar" | "show_media"
        ) {
            bail!("unknown lock config key: {k}");
        }
        obj.insert(k.clone(), v.clone());
    }
    out = serde_json::from_value(value)?;
    validate_lock_config(&out)?;
    Ok(out)
}

async fn load_lock_config() -> Result<LockScreenConfig> {
    storage::init().await?;
    if let Some(raw) = storage::get_kv(LOCK_NS, LOCK_KEY).await? {
        let config: LockScreenConfig = serde_json::from_value(raw)?;
        validate_lock_config(&config)?;
        return Ok(config);
    }
    Ok(default_lock_config())
}

async fn save_lock_config(config: &LockScreenConfig) -> Result<()> {
    validate_lock_config(config)?;
    storage::set_kv(LOCK_NS, LOCK_KEY, &serde_json::to_value(config)?).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct SleepInhibitor {
    pub what: String,
    pub who: String,
    pub why: String,
    pub mode: String,
    pub uid: u32,
    pub pid: u32,
}

pub async fn list_sleep_inhibitors() -> Result<Vec<SleepInhibitor>> {
    use zbus::{Connection, Proxy};

    let conn = Connection::system().await?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let rows: Vec<(String, String, String, String, u32, u32)> =
        proxy.call("ListInhibitors", &()).await?;
    Ok(rows
        .into_iter()
        .map(|(what, who, why, mode, uid, pid)| SleepInhibitor {
            what,
            who,
            why,
            mode,
            uid,
            pid,
        })
        .collect())
}

static INHIBITOR_COOKIE: AtomicU32 = AtomicU32::new(1);

lazy_static::lazy_static! {
    static ref INHIBITOR_HANDLES: Mutex<HashMap<u32, zbus::zvariant::OwnedFd>> =
        Mutex::new(HashMap::new());
}

async fn inhibit_sleep(what: &str, who: &str, why: &str, mode: &str) -> Result<u32> {
    use zbus::{Connection, Proxy};

    if !matches!(mode, "block" | "delay") {
        bail!("mode must be block or delay");
    }
    let conn = Connection::system().await?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let (fd, cookie): (zbus::zvariant::OwnedFd, u32) =
        proxy.call("Inhibit", &(what, who, why, mode)).await?;
    let local_cookie = INHIBITOR_COOKIE.fetch_add(1, Ordering::Relaxed);
    INHIBITOR_HANDLES
        .lock()
        .await
        .insert(local_cookie, fd);
    let _ = cookie;
    Ok(local_cookie)
}

pub async fn test_fingerprint() -> Result<Value> {
    if !crate::services::security::probe_fprintd_available().await {
        return Ok(json!({
            "ok": false,
            "reason": "unavailable",
        }));
    }
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        crate::utils::process::exec_command(&["fprintd-verify"]),
    )
    .await;
    match output {
        Ok(Ok(_)) => Ok(json!({ "ok": true })),
        Ok(Err(e)) => {
            let code = super::shell::classify_session_stderr(&e.to_string());
            Ok(json!({
                "ok": false,
                "reason": code,
            }))
        }
        Err(_) => Ok(json!({
            "ok": false,
            "reason": "timeout",
        })),
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Lock.GetConfig", |_params| async move {
        let config = load_lock_config().await?;
        Ok(json!({ "config": config }))
    });

    registry.register("Lock.SetConfig", |params| async move {
        let partial = params
            .as_ref()
            .and_then(|p| p.get("partial"))
            .and_then(|v| v.as_object())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("missing partial object"))?;
        let current = load_lock_config().await?;
        let merged = merge_lock_partial(&current, &partial)?;
        save_lock_config(&merged).await?;
        Ok(json!({ "config": merged }))
    });

    registry.register("Lock.TestFingerprint", |_params| async move {
        test_fingerprint().await
    });

    registry.register("Sleep.GetInhibitors", |_params| async move {
        let inhibitors = list_sleep_inhibitors().await?;
        Ok(json!({ "inhibitors": inhibitors }))
    });

    registry.register("Sleep.Inhibit", |params| async move {
        let p = params.as_ref().ok_or_else(|| anyhow::anyhow!("missing params"))?;
        let what = p
            .get("what")
            .and_then(|v| v.as_str())
            .unwrap_or("sleep");
        let who = p
            .get("who")
            .and_then(|v| v.as_str())
            .unwrap_or("ags-sidecar");
        let why = p
            .get("why")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing why"))?;
        let mode = p
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("delay");
        let cookie = inhibit_sleep(what, who, why, mode).await?;
        Ok(json!({ "ok": true, "cookie": cookie }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_lock_config_hides_sensitive_widgets() {
        let d = default_lock_config();
        assert!(d.show_clock);
        assert!(!d.show_notifications);
        assert!(!d.show_calendar);
        assert!(!d.show_media);
    }

    #[test]
    fn merge_lock_partial_updates_flags() {
        let base = default_lock_config();
        let mut partial = Map::new();
        partial.insert("show_media".into(), json!(true));
        let merged = merge_lock_partial(&base, &partial).unwrap();
        assert!(merged.show_media);
        assert!(!merged.show_notifications);
    }

    #[test]
    fn merge_lock_rejects_unknown_key() {
        let base = default_lock_config();
        let mut partial = Map::new();
        partial.insert("evil".into(), json!(true));
        assert!(merge_lock_partial(&base, &partial).is_err());
    }
}
