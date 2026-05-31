//! `Settings.*` round-trip against a temporary SQLite DB.

mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn settings_get_defaults_without_row() {
    setup_temp_storage_db().await;
    let registry = build_registry();

    let got = call_rpc(&registry, "Settings.Get", None).await.unwrap();
    assert_eq!(got.get("schema_version").and_then(|v| v.as_i64()), Some(1));
    let settings = got.get("settings").expect("settings");
    assert!(settings.get("bar_section_order").and_then(|v| v.as_array()).is_some());
    assert_eq!(settings.get("theme").and_then(|v| v.as_str()), Some("dark"));
}

#[tokio::test]
async fn settings_set_merge_and_reset() {
    setup_temp_storage_db().await;
    let registry = build_registry();

    let set = call_method_unchecked(
        &registry,
        "Settings.Set",
        Some(json!({
            "partial": {
                "theme": "catppuccin-mocha",
                "bar_section_order": ["workspaces", "clock", "power"]
            }
        })),
    )
    .await
    .unwrap();
    assert_eq!(
        set.get("settings")
            .and_then(|s| s.get("theme"))
            .and_then(|v| v.as_str()),
        Some("catppuccin-mocha")
    );

    let got = call_rpc(&registry, "Settings.Get", None).await.unwrap();
    let order = got
        .get("settings")
        .and_then(|s| s.get("bar_section_order"))
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(order.len(), 3);

    call_method_unchecked(&registry, "Settings.Reset", None)
        .await
        .unwrap();
    let after = call_rpc(&registry, "Settings.Get", None).await.unwrap();
    assert_eq!(
        after
            .get("settings")
            .and_then(|s| s.get("theme"))
            .and_then(|v| v.as_str()),
        Some("dark")
    );
}

#[tokio::test]
async fn settings_set_rejects_unknown_key() {
    setup_temp_storage_db().await;
    let registry = build_registry();
    let err = call_method_unchecked(
        &registry,
        "Settings.Set",
        Some(json!({ "partial": { "evil": true } })),
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn settings_get_schema_lists_fields() {
    setup_temp_storage_db().await;
    let registry = build_registry();
    let schema = call_rpc(&registry, "Settings.GetSchema", None)
        .await
        .unwrap();
    assert_eq!(schema.get("schema_version").and_then(|v| v.as_i64()), Some(1));
    let fields = schema.get("fields").and_then(|v| v.as_object()).expect("fields");
    let theme = fields.get("theme").and_then(|v| v.as_object()).expect("theme field");
    let examples = theme
        .get("examples")
        .and_then(|v| v.as_array())
        .expect("theme examples");
    assert!(examples.iter().any(|v| v.as_str() == Some("dark")));
    assert!(examples.iter().any(|v| v.as_str() == Some("catppuccin-mocha")));
}
