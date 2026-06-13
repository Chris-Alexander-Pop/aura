//! StatusNotifier (system tray) bridge for the React bar.
use crate::notify;
use crate::services::ServiceRegistry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use zbus::zvariant::OwnedValue;

const REFRESH_SECS: u64 = 2;
const PROPS_IFACE: &str = "org.freedesktop.DBus.Properties";
const TRAY_IFACE: &str = "org.kde.StatusNotifierItem";
const WATCHER_IFACE: &str = "org.kde.StatusNotifierWatcher";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_name: Option<String>,
    pub status: String,
    pub category: String,
    pub service: String,
    pub path: String,
}

lazy_static::lazy_static! {
    static ref ITEMS: RwLock<Vec<TrayItem>> = RwLock::new(Vec::new());
    static ref SERVICE_PATHS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

fn fixture_items() -> Option<Vec<TrayItem>> {
    let raw = std::env::var("AURA_TRAY_FIXTURE").ok()?;
    serde_json::from_str(&raw).ok()
}

fn value_to_string(v: OwnedValue) -> Result<String> {
    String::try_from(v).context("expected string")
}

async fn dbus_get_string(
    conn: &zbus::Connection,
    service: &str,
    path: &str,
    iface: &str,
    prop: &str,
) -> Result<String> {
    use zbus::Proxy;
    let props = Proxy::new(conn, service, path, PROPS_IFACE).await?;
    let v: OwnedValue = props.call("Get", &(iface, prop)).await?;
    value_to_string(v)
}

async fn fetch_tray_items_zbus() -> Result<Vec<TrayItem>> {
    use zbus::{Connection, Proxy};

    let conn = Connection::session().await.context("session bus")?;
    let watcher = Proxy::new(
        &conn,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        PROPS_IFACE,
    )
    .await?;

    let raw: OwnedValue = watcher
        .call("Get", &(WATCHER_IFACE, "RegisteredStatusNotifierItems"))
        .await?;
    let services: Vec<String> = Vec::try_from(raw).context("RegisteredStatusNotifierItems")?;

    let mut items = Vec::new();
    for service in services {
        let path = SERVICE_PATHS
            .read()
            .await
            .get(&service)
            .cloned()
            .unwrap_or_else(|| "/StatusNotifierItem".to_string());

        if let Ok(item) = read_tray_item(&conn, &service, &path).await {
            items.push(item);
            continue;
        }
        let alt = format!("/org/kde/StatusNotifierItem/{service}");
        if let Ok(item) = read_tray_item(&conn, &service, &alt).await {
            SERVICE_PATHS
                .write()
                .await
                .insert(service.clone(), alt.clone());
            items.push(item);
        }
    }
    Ok(items)
}

async fn read_tray_item(
    conn: &zbus::Connection,
    service: &str,
    path: &str,
) -> Result<TrayItem> {
    let id = dbus_get_string(conn, service, path, TRAY_IFACE, "Id").await?;
    let title = dbus_get_string(conn, service, path, TRAY_IFACE, "Title")
        .await
        .unwrap_or_else(|_| id.clone());
    let icon_name = dbus_get_string(conn, service, path, TRAY_IFACE, "IconName")
        .await
        .unwrap_or_default();
    let status = dbus_get_string(conn, service, path, TRAY_IFACE, "Status")
        .await
        .unwrap_or_else(|_| "Active".to_string());
    let category = dbus_get_string(conn, service, path, TRAY_IFACE, "Category")
        .await
        .unwrap_or_else(|_| "ApplicationStatus".to_string());

    Ok(TrayItem {
        id: id.clone(),
        title,
        icon_name: if icon_name.is_empty() {
            None
        } else {
            Some(icon_name)
        },
        status,
        category,
        service: service.to_string(),
        path: path.to_string(),
    })
}

async fn activate_item(item: &TrayItem, secondary: bool) -> Result<()> {
    use zbus::{Connection, Proxy};

    let conn = Connection::session().await?;
    let proxy = Proxy::new(
        &conn,
        item.service.as_str(),
        item.path.as_str(),
        TRAY_IFACE,
    )
    .await?;

    if secondary {
        let _: () = proxy.call("SecondaryActivate", &(0i32, 0i32)).await?;
    } else {
        let _: () = proxy.call("Activate", &(0i32, 0i32)).await?;
    }
    Ok(())
}

async fn refresh_once() {
    let next = if let Some(fix) = fixture_items() {
        fix
    } else {
        fetch_tray_items_zbus().await.unwrap_or_default()
    };
    let changed = {
        let guard = ITEMS.read().await;
        guard.as_slice() != next.as_slice()
    };
    if changed {
        *ITEMS.write().await = next;
        notify::emit("Tray.Changed", json!({}));
    }
}

async fn monitor_loop() {
    loop {
        refresh_once().await;
        sleep(Duration::from_secs(REFRESH_SECS)).await;
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(monitor_loop());

    registry.register("Tray.List", |_params| async move {
        let items = ITEMS.read().await.clone();
        Ok(json!({ "items": items }))
    });

    registry.register("Tray.Activate", |params| async move {
        let id = params
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing id"))?;
        let item = ITEMS
            .read()
            .await
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown tray id: {id}"))?;
        activate_item(&item, false).await?;
        Ok(json!({ "ok": true }))
    });

    registry.register("Tray.SecondaryActivate", |params| async move {
        let id = params
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing id"))?;
        let item = ITEMS
            .read()
            .await
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown tray id: {id}"))?;
        activate_item(&item, true).await?;
        Ok(json!({ "ok": true }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_item_serializes() {
        let item = TrayItem {
            id: "nm".into(),
            title: "Network".into(),
            icon_name: Some("nm-signal-strong".into()),
            status: "Active".into(),
            category: "SystemServices".into(),
            service: ":1.1".into(),
            path: "/StatusNotifierItem".into(),
        };
        let v = serde_json::to_value(&item).unwrap();
        assert_eq!(v["id"], "nm");
    }
}
