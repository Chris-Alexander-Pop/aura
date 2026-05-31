//! Read-only `Settings.Get` / `Settings.GetSchema` RPC smoke.

mod common;

use common::{call_rpc, setup_temp_storage_db, test_registry};

#[tokio::test]
async fn settings_get_and_schema_resolve() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let got = call_rpc(&registry, "Settings.Get", None)
        .await
        .expect("Settings.Get");
    assert!(got.get("settings").is_some());
    assert_eq!(got.get("schema_version").and_then(|v| v.as_i64()), Some(1));

    let schema = call_rpc(&registry, "Settings.GetSchema", None)
        .await
        .expect("Settings.GetSchema");
    assert!(schema.get("fields").is_some());
}
