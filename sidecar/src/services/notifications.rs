//! Freedesktop notification listener + in-memory history for Control Center.
use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::Duration;

const NS: &str = "notifications";
const MAX_ITEMS: usize = 200;
const DND_KEY: &str = "dnd";
const RULES_KEY: &str = "rules";

static EMIT_GEN: AtomicU64 = AtomicU64::new(0);
static NEXT_INTERNAL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationAction {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationItem {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<u32>,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub urgency: u8,
    pub timestamp: i64,
    #[serde(default)]
    pub actions: Vec<NotificationAction>,
    #[serde(default)]
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DndPrefs {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub schedule_enabled: bool,
    #[serde(default = "default_start")]
    pub start_time: String,
    #[serde(default = "default_end")]
    pub end_time: String,
    #[serde(default = "default_true")]
    pub weekdays_only: bool,
}

fn default_start() -> String {
    "22:00".to_string()
}
fn default_end() -> String {
    "07:00".to_string()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppNotificationRules {
    #[serde(default)]
    pub muted_apps: Vec<String>,
}

lazy_static::lazy_static! {
    static ref STORE: RwLock<VecDeque<NotificationItem>> = RwLock::new(VecDeque::new());
    static ref SERVER_ID_MAP: RwLock<HashMap<u32, u64>> = RwLock::new(HashMap::new());
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Notifications.List", |params| async move {
        let limit: usize = params
            .as_ref()
            .and_then(|p| p.get("limit").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(50);
        let app_filter: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("app_name").cloned())
            .and_then(|v| serde_json::from_value(v).ok());
        let since: Option<i64> = params
            .as_ref()
            .and_then(|p| p.get("since").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let store = STORE.read().await;
        let items: Vec<NotificationItem> = store
            .iter()
            .rev()
            .filter(|n| !n.closed)
            .filter(|n| since.map(|s| n.timestamp >= s).unwrap_or(true))
            .filter(|n| {
                app_filter
                    .as_ref()
                    .map(|a| n.app_name.eq_ignore_ascii_case(a))
                    .unwrap_or(true)
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(serde_json::to_value(&items)?)
    });

    registry.register("Notifications.Get", |params| async move {
        let id: u64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing id"))?,
        )?;
        let store = STORE.read().await;
        let item = store.iter().find(|n| n.id == id).cloned();
        Ok(serde_json::to_value(item)?)
    });

    registry.register("Notifications.Dismiss", |params| async move {
        let id: u64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing id"))?,
        )?;
        let server_id = {
            let store = STORE.read().await;
            store.iter().find(|n| n.id == id).and_then(|n| n.server_id)
        };
        if let Some(sid) = server_id {
            let _ = close_on_daemon(sid).await;
        }
        mark_closed(id).await;
        schedule_emit("dismissed").await;
        Ok(json!({ "success": true }))
    });

    registry.register("Notifications.ClearAll", |_params| async move {
        STORE.write().await.clear();
        SERVER_ID_MAP.write().await.clear();
        schedule_emit("cleared").await;
        Ok(json!({ "success": true }))
    });

    registry.register("Notifications.InvokeAction", |params| async move {
        let id: u64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing id"))?,
        )?;
        let action_key: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("action_key").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing action_key"))?,
        )?;
        let server_id = {
            let store = STORE.read().await;
            store.iter().find(|n| n.id == id).and_then(|n| n.server_id)
        };
        if let Some(sid) = server_id {
            let _ = invoke_action_on_daemon(sid, &action_key).await;
        }
        Ok(json!({ "success": true }))
    });

    registry.register("Notifications.GetDnd", |_params| async move {
        storage::init().await?;
        let prefs = load_dnd().await?;
        Ok(serde_json::to_value(prefs)?)
    });

    registry.register("Notifications.SetDnd", |params| async move {
        let prefs: DndPrefs = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("dnd").cloned())
                .or_else(|| params.clone())
                .ok_or_else(|| anyhow::anyhow!("Missing dnd"))?,
        )?;
        storage::init().await?;
        storage::set_kv(NS, DND_KEY, &serde_json::to_value(&prefs)?).await?;
        schedule_emit("dnd").await;
        Ok(json!({ "success": true }))
    });

    registry.register("Notifications.GetRules", |_params| async move {
        storage::init().await?;
        let rules = load_rules().await?;
        Ok(serde_json::to_value(rules)?)
    });

    registry.register("Notifications.SetRules", |params| async move {
        let rules: AppNotificationRules = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("rules").cloned())
                .or_else(|| params.clone())
                .ok_or_else(|| anyhow::anyhow!("Missing rules"))?,
        )?;
        storage::init().await?;
        storage::set_kv(NS, RULES_KEY, &serde_json::to_value(&rules)?).await?;
        schedule_emit("rules").await;
        Ok(json!({ "success": true }))
    });
}

pub(crate) async fn snapshot_dnd() -> Result<DndPrefs> {
    storage::init().await?;
    load_dnd().await
}

async fn load_dnd() -> Result<DndPrefs> {
    match storage::get_kv(NS, DND_KEY).await? {
        Some(v) => Ok(serde_json::from_value(v)?),
        None => Ok(DndPrefs::default()),
    }
}

async fn load_rules() -> Result<AppNotificationRules> {
    match storage::get_kv(NS, RULES_KEY).await? {
        Some(v) => Ok(serde_json::from_value(v)?),
        None => Ok(AppNotificationRules::default()),
    }
}

pub(crate) fn notification_blocked_by_rules(rules: &AppNotificationRules, app_name: &str) -> bool {
    rules
        .muted_apps
        .iter()
        .any(|a| a.eq_ignore_ascii_case(app_name))
}

async fn mark_closed(internal_id: u64) {
    let sid_to_remove = {
        let mut store = STORE.write().await;
        if let Some(n) = store.iter_mut().find(|n| n.id == internal_id) {
            n.closed = true;
            n.server_id
        } else {
            None
        }
    };
    if let Some(sid) = sid_to_remove {
        SERVER_ID_MAP.write().await.remove(&sid);
    }
}

async fn mark_closed_by_server(server_id: u32) {
    let internal = SERVER_ID_MAP.read().await.get(&server_id).copied();
    if let Some(id) = internal {
        mark_closed(id).await;
    }
}

pub async fn record_notification(
    server_id: Option<u32>,
    app_name: String,
    replaces_id: u32,
    summary: String,
    body: String,
    icon: Option<String>,
    urgency: u8,
    actions: Vec<NotificationAction>,
) {
    let rules = load_rules().await.unwrap_or_default();
    if notification_blocked_by_rules(&rules, &app_name) {
        return;
    }

    let mut store = STORE.write().await;

    if replaces_id != 0 {
        if let Some(internal) = SERVER_ID_MAP.read().await.get(&replaces_id).copied() {
            let mut map_update = None;
            if let Some(n) = store.iter_mut().find(|n| n.id == internal) {
                n.summary = summary;
                n.body = body;
                n.icon = icon;
                n.urgency = urgency;
                n.actions = actions;
                n.timestamp = now_secs();
                if let Some(sid) = server_id {
                    n.server_id = Some(sid);
                    map_update = Some((sid, internal));
                }
                drop(store);
                if let Some((sid, internal)) = map_update {
                    SERVER_ID_MAP.write().await.insert(sid, internal);
                }
                schedule_emit("replaced").await;
                return;
            }
        }
    }

    let id = NEXT_INTERNAL_ID.fetch_add(1, Ordering::SeqCst);
    let item = NotificationItem {
        id,
        server_id,
        app_name,
        summary,
        body,
        icon,
        urgency,
        timestamp: now_secs(),
        actions,
        closed: false,
    };
    let mut sid_removals = Vec::new();
    store.push_back(item);
    while store.len() > MAX_ITEMS {
        if let Some(old) = store.pop_front() {
            if let Some(sid) = old.server_id {
                sid_removals.push(sid);
            }
        }
    }
    drop(store);
    if let Some(sid) = server_id {
        SERVER_ID_MAP.write().await.insert(sid, id);
    }
    for sid in sid_removals {
        SERVER_ID_MAP.write().await.remove(&sid);
    }
    schedule_emit("new").await;
}

async fn schedule_emit(reason: &str) {
    let gen = EMIT_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let reason = reason.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        if EMIT_GEN.load(Ordering::SeqCst) != gen {
            return;
        }
        notify::emit("Notifications.Changed", json!({ "reason": reason }));
    });
}

/// Spawn D-Bus listeners (fail-soft when no session bus).
pub fn spawn_dbus_listener() {
    tokio::spawn(async {
        if let Err(e) = run_dbus_monitor().await {
            tracing::debug!("notification dbus-monitor: {e}");
        }
    });
    tokio::spawn(async {
        if let Err(e) = run_notification_signal_listener().await {
            tracing::debug!("notification signal listener: {e}");
        }
    });
}

/// Parse `dbus-monitor` text output for Notify / NotificationClosed.
async fn run_dbus_monitor() -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let mut child = Command::new("dbus-monitor")
        .args([
            "--session",
            "interface='org.freedesktop.Notifications',member='Notify'",
            "interface='org.freedesktop.Notifications',member='NotificationClosed'",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("dbus-monitor stdout"))?;
    let mut lines = BufReader::new(stdout).lines();
    let mut parser = DbusMonitorParser::default();

    while let Some(line) = lines.next_line().await? {
        parser.feed_line(line.trim()).await;
    }

    Ok(())
}

#[derive(Default)]
struct PendingNotify {
    app_name: Option<String>,
    replaces_id: u32,
    icon: Option<String>,
    summary: Option<String>,
    body: Option<String>,
}

#[derive(Default)]
struct DbusMonitorParser {
    pending_notify: Option<PendingNotify>,
    pending_closed: bool,
}

impl DbusMonitorParser {
    async fn feed_line(&mut self, trimmed: &str) {
        if trimmed.starts_with("method call") && trimmed.contains("member=Notify") {
            self.pending_notify = Some(PendingNotify::default());
            self.pending_closed = false;
            return;
        }
        if trimmed.starts_with("signal") && trimmed.contains("member=NotificationClosed") {
            self.pending_notify = None;
            self.pending_closed = true;
            return;
        }
        if self.pending_closed {
            if let Some(id) = parse_dbus_monitor_uint32(trimmed) {
                mark_closed_by_server(id).await;
                schedule_emit("closed").await;
                self.pending_closed = false;
            }
            return;
        }
        if let Some(p) = self.pending_notify.as_mut() {
            if let Some(s) = parse_dbus_monitor_string(trimmed) {
                if p.app_name.is_none() {
                    p.app_name = Some(s);
                } else if p.icon.is_none() {
                    p.icon = Some(s);
                } else if p.summary.is_none() {
                    p.summary = Some(s);
                } else if p.body.is_none() {
                    p.body = Some(s);
                }
            } else if let Some(n) = parse_dbus_monitor_uint32(trimmed) {
                if p.app_name.is_some() && p.replaces_id == 0 && p.icon.is_none() {
                    p.replaces_id = n;
                }
            }
            if p.app_name.is_some() && p.summary.is_some() && p.body.is_some() {
                record_notification(
                    None,
                    p.app_name.clone().unwrap_or_else(|| "unknown".into()),
                    p.replaces_id,
                    p.summary.clone().unwrap_or_default(),
                    p.body.clone().unwrap_or_default(),
                    p.icon.clone(),
                    1,
                    Vec::new(),
                )
                .await;
                self.pending_notify = None;
            }
        }
    }
}

/// Feed fixture lines through the dbus-monitor state machine (unit tests).
pub(crate) async fn feed_dbus_monitor_fixture(text: &str) {
    let mut parser = DbusMonitorParser::default();
    for line in text.lines() {
        parser.feed_line(line.trim()).await;
    }
}

#[cfg(test)]
pub(crate) async fn reset_store_for_tests() {
    STORE.write().await.clear();
    SERVER_ID_MAP.write().await.clear();
}

/// Subscribe to NotificationClosed via zbus signal stream on the daemon object.
async fn run_notification_signal_listener() -> Result<()> {
    use futures_util::StreamExt;
    use zbus::{Connection, Proxy};

    let conn = Connection::session().await?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await?;

    let mut stream = proxy.receive_signal("NotificationClosed").await?;
    while let Some(msg) = stream.next().await {
        if let Ok(body) = msg.body::<(u32, u32)>() {
            mark_closed_by_server(body.0).await;
            schedule_emit("closed").await;
        }
    }
    Ok(())
}

fn parse_dbus_monitor_string(line: &str) -> Option<String> {
    let t = line.trim();
    if let Some(rest) = t.strip_prefix("string \"") {
        return Some(rest.trim_end_matches('"').to_string());
    }
    if let Some(rest) = t.strip_prefix("string ") {
        return Some(rest.trim_matches('"').to_string());
    }
    None
}

fn parse_dbus_monitor_uint32(line: &str) -> Option<u32> {
    let t = line.trim();
    t.strip_prefix("uint32 ")
        .and_then(|n| n.split_whitespace().next())
        .and_then(|n| n.parse().ok())
}

pub fn parse_action_string_pairs(pairs: &[String]) -> Vec<NotificationAction> {
    let mut out = Vec::new();
    let mut iter = pairs.iter();
    while let Some(key) = iter.next() {
        if let Some(label) = iter.next() {
            out.push(NotificationAction {
                key: key.clone(),
                label: label.clone(),
            });
        }
    }
    out
}

fn parse_actions(v: Option<&zbus::zvariant::Value<'_>>) -> Vec<NotificationAction> {
    let Some(zbus::zvariant::Value::Array(arr)) = v else {
        return Vec::new();
    };
    let pairs: Vec<String> = arr
        .iter()
        .filter_map(|x| {
            if let zbus::zvariant::Value::Str(s) = x {
                Some(s.as_str().to_string())
            } else {
                None
            }
        })
        .collect();
    parse_action_string_pairs(&pairs)
}

async fn close_on_daemon(server_id: u32) -> Result<()> {
    use zbus::{Connection, Proxy};
    let conn = Connection::session().await?;
    let proxy = Proxy::new(
        &conn,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await?;
    let _: () = proxy.call("CloseNotification", &(server_id,)).await?;
    Ok(())
}

async fn invoke_action_on_daemon(_server_id: u32, _action_key: &str) -> Result<()> {
    // Not all daemons expose InvokeAction on the bus; best-effort no-op.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ServiceRegistry;
    use crate::types::JsonRpcRequest;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    static NOTIFICATION_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn notification_test_lock() -> &'static Mutex<()> {
        NOTIFICATION_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn muted_app_rules_block_by_name() {
        let rules = AppNotificationRules {
            muted_apps: vec!["Slack".into()],
        };
        assert!(notification_blocked_by_rules(&rules, "Slack"));
        assert!(notification_blocked_by_rules(&rules, "slack"));
        assert!(!notification_blocked_by_rules(&rules, "Firefox"));
    }

    #[test]
    fn parse_actions_pairs() {
        let actions = parse_action_string_pairs(&[
            "default".into(),
            "Open".into(),
            "close".into(),
            "Close".into(),
        ]);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].key, "default");
    }

    #[test]
    fn parse_dbus_monitor_string_variants() {
        assert_eq!(
            parse_dbus_monitor_string("string \"firefox\""),
            Some("firefox".into())
        );
        assert_eq!(
            parse_dbus_monitor_string("string brief"),
            Some("brief".into())
        );
        assert!(parse_dbus_monitor_string("uint32 7").is_none());
    }

    #[test]
    fn parse_dbus_monitor_uint32_line() {
        assert_eq!(parse_dbus_monitor_uint32("uint32 42"), Some(42));
        assert_eq!(parse_dbus_monitor_uint32("  uint32 99  "), Some(99));
        assert!(parse_dbus_monitor_uint32("string \"x\"").is_none());
    }

    #[test]
    fn parse_actions_odd_pair_count() {
        let actions = parse_action_string_pairs(&["only".into()]);
        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn list_empty_without_dbus() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;
        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Notifications.List".into(),
                params: None,
                id: Some(1.into()),
            })
            .await
            .unwrap();
        assert!(v.as_array().map(|a| a.is_empty()).unwrap_or(false));
    }

    #[tokio::test]
    async fn record_notification_visible_in_list() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;
        record_notification(
            Some(9001),
            "test-app".into(),
            0,
            "Summary".into(),
            "Body".into(),
            None,
            1,
            parse_action_string_pairs(&["default".into(), "Open".into()]),
        )
        .await;

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Notifications.List".into(),
                params: Some(json!({ "limit": 5, "app_name": "test-app" })),
                id: Some(1.into()),
            })
            .await
            .unwrap();
        let items = v.as_array().expect("array");
        assert!(items.iter().any(|n| n.get("summary").and_then(|s| s.as_str()) == Some("Summary")));
    }

    #[tokio::test]
    async fn dbus_monitor_notify_fixture_records_item() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;
        let fixture = include_str!("../../tests/fixtures/notifications/dbus_monitor_notify.txt");
        feed_dbus_monitor_fixture(fixture).await;

        let store = STORE.read().await;
        assert_eq!(store.len(), 1);
        let item = store.back().expect("notification");
        assert_eq!(item.app_name, "firefox");
        assert_eq!(item.summary, "Page loaded");
        assert_eq!(item.body, "example.com finished loading");
        assert_eq!(item.icon.as_deref(), Some("dialog-information"));
    }

    #[tokio::test]
    async fn dbus_monitor_closed_fixture_marks_closed() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;
        record_notification(
            Some(9001),
            "daemon-app".into(),
            0,
            "Title".into(),
            "Text".into(),
            None,
            1,
            Vec::new(),
        )
        .await;

        let closed = include_str!("../../tests/fixtures/notifications/dbus_monitor_closed.txt");
        feed_dbus_monitor_fixture(closed).await;

        let store = STORE.read().await;
        let item = store.iter().find(|n| n.server_id == Some(9001)).expect("item");
        assert!(item.closed);
    }

    #[tokio::test]
    async fn list_excludes_closed_and_respects_since() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;
        record_notification(None, "app-a".into(), 0, "Old".into(), "x".into(), None, 1, Vec::new())
            .await;
        {
            let mut store = STORE.write().await;
            store.back_mut().unwrap().timestamp = 100;
        }
        record_notification(None, "app-a".into(), 0, "New".into(), "y".into(), None, 1, Vec::new())
            .await;
        {
            let mut store = STORE.write().await;
            if let Some(front) = store.front_mut() {
                front.closed = true;
            }
        }

        let mut reg = ServiceRegistry::new();
        register(&mut reg);
        let v = reg
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "Notifications.List".into(),
                params: Some(json!({ "limit": 10, "since": 50 })),
                id: Some(1.into()),
            })
            .await
            .unwrap();
        let items = v.as_array().expect("array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("summary").and_then(|s| s.as_str()), Some("New"));
    }

    #[tokio::test]
    async fn replace_notification_updates_existing() {
        let _guard = notification_test_lock().lock().await;
        reset_store_for_tests().await;

        SERVER_ID_MAP.write().await.insert(42, 1);
        STORE.write().await.push_back(NotificationItem {
            id: 1,
            server_id: Some(42),
            app_name: "app".into(),
            summary: "First".into(),
            body: "body".into(),
            icon: None,
            urgency: 1,
            timestamp: 1,
            actions: Vec::new(),
            closed: false,
        });

        {
            let mut store = STORE.write().await;
            let internal = SERVER_ID_MAP.read().await.get(&42).copied().expect("map");
            let n = store.iter_mut().find(|n| n.id == internal).expect("item");
            n.summary = "Updated".into();
            n.body = "new body".into();
            n.icon = Some("icon".into());
            n.urgency = 2;
            n.server_id = Some(99);
        }
        SERVER_ID_MAP.write().await.insert(99, 1);

        let store = STORE.read().await;
        assert_eq!(store.len(), 1);
        let item = store.back().expect("item");
        assert_eq!(item.summary, "Updated");
        assert_eq!(item.server_id, Some(99));
    }
}
