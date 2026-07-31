//! Read-only Appearance / Tray RPC shapes (in-memory; no host side effects).

mod common;

use common::{call_method, ExecFixtureGuard, test_registry};

#[tokio::test]
async fn appearance_get_night_light_and_theme_resolve() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    let night = call_method(&registry, "Appearance.GetNightLight", None)
        .await
        .expect("Appearance.GetNightLight");
    assert!(night.get("enabled").and_then(|v| v.as_bool()).is_some());
    assert!(night.get("temperature").and_then(|v| v.as_u64()).is_some());

    let theme = call_method(&registry, "Appearance.GetTheme", None)
        .await
        .expect("Appearance.GetTheme");
    assert!(theme.get("theme").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn tray_list_returns_items_array() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    let value = call_method(&registry, "Tray.List", None)
        .await
        .expect("Tray.List");
    assert!(value.get("items").and_then(|v| v.as_array()).is_some());
}
