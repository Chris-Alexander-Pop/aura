//! Audit log + rate limits for `Security.Offensive.*` (feature `offensive-security`).

use crate::utils::storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const RATE_LIMITED_CODE: i32 = -32099;

#[derive(Debug)]
pub struct RateLimited(pub String);

impl std::fmt::Display for RateLimited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RateLimited {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffensiveAuditEntry {
    pub id: i64,
    pub timestamp: i64,
    pub method: String,
    pub arg_hash: String,
    pub user: String,
    pub status: String,
}

lazy_static::lazy_static! {
    static ref LAST_CALL: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn hash_params(params: &Option<Value>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    if let Some(p) = params {
        p.to_string().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub fn offensive_method_is_mutating(method: &str) -> bool {
    if !method.starts_with("Security.Offensive.") {
        return false;
    }
    let suffix = method.strip_prefix("Security.Offensive.").unwrap_or("");
    !suffix.contains("Get")
        && !suffix.ends_with(".List")
        && method != "Security.Offensive.GetAuditLog"
}

pub fn check_rate_limit(method: &str) -> Result<()> {
    if !method.starts_with("Security.Offensive.") {
        return Ok(());
    }
    let cooldown = if method.contains("Nmap") {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(5)
    };
    let mut map = LAST_CALL.lock().expect("rate map");
    if let Some(last) = map.get(method) {
        if last.elapsed() < cooldown {
            return Err(anyhow::Error::new(RateLimited(format!(
                "rate limited: retry after {}s",
                (cooldown - last.elapsed()).as_secs().max(1)
            ))));
        }
    }
    map.insert(method.to_string(), Instant::now());
    Ok(())
}

pub async fn record_audit(method: &str, params: &Option<Value>, status: &str) -> Result<()> {
    storage::init().await?;
    let entry = OffensiveAuditEntry {
        id: chrono::Utc::now().timestamp_millis(),
        timestamp: chrono::Utc::now().timestamp(),
        method: method.to_string(),
        arg_hash: hash_params(params),
        user: current_user(),
        status: status.to_string(),
    };
    let key = format!("audit_{}", entry.id);
    storage::set_kv("offensive_audit", &key, &serde_json::to_value(&entry)?).await?;
    Ok(())
}

pub async fn get_audit_log(limit: usize, offset: usize) -> Result<Vec<OffensiveAuditEntry>> {
    storage::init().await?;
    let items = storage::scan_namespace("offensive_audit").await?;
    let mut entries: Vec<OffensiveAuditEntry> = items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let end = (offset + limit).min(entries.len());
    if offset >= entries.len() {
        return Ok(Vec::new());
    }
    Ok(entries[offset..end].to_vec())
}

pub fn offensive_feature_enabled() -> bool {
    cfg!(feature = "offensive-security")
}
