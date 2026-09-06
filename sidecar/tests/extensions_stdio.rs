//! Runtime JSON-RPC stdio extras (core registry stays unchanged without them).

mod common;

use ags_sidecar::build_registry;
use ags_sidecar::extensions;
use ags_sidecar::services::MethodNotFound;
use common::call_method;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static EXT_ENV_LOCK: Mutex<()> = Mutex::new(());

fn mock_ext_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extensions/mock_ext.py")
}

#[tokio::test]
async fn attach_proxies_advertised_methods_from_manifest_dir() {
    let _lock = EXT_ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("mock.json");
    let payload = json!({
        "command": ["python3", mock_ext_path().to_string_lossy()]
    });
    fs::write(&manifest, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    std::env::set_var("AURA_EXTENSIONS_DIR", dir.path());
    std::env::remove_var("AURA_EXTENSIONS");

    let mut registry = build_registry();
    extensions::attach(&mut registry).await;

    let ping = call_method(&registry, "Ext.Ping", Some(json!({ "n": 1 })))
        .await
        .expect("Ext.Ping via mock extra");
    assert_eq!(ping["ok"], true);
    assert_eq!(
        registry.loaded_extensions().len(),
        1,
        "expected one attached extra"
    );

    std::env::remove_var("AURA_EXTENSIONS_DIR");
}

#[tokio::test]
async fn empty_aura_extensions_disables_discovery() {
    let _lock = EXT_ENV_LOCK.lock().unwrap();
    std::env::set_var("AURA_EXTENSIONS", "");
    let mut registry = build_registry();
    extensions::attach(&mut registry).await;
    assert!(registry.loaded_extensions().is_empty());
    std::env::remove_var("AURA_EXTENSIONS");
}

#[tokio::test]
async fn unknown_method_without_extras_is_not_found() {
    let registry = common::test_registry();
    let err = common::call_method_unchecked(&registry, "Ext.DoesNotExist", None)
        .await
        .expect_err("missing extra method");
    assert!(err.downcast_ref::<MethodNotFound>().is_some());
}
