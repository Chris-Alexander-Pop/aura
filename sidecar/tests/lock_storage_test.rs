//! Mutating `Lock.SetConfig` against a temporary storage DB (`call_method_unchecked`).

mod common;

use ags_sidecar::build_registry;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db};
use serde_json::json;

#[tokio::test]
async fn lock_config_defaults_merge_and_reject_unknown_key() {
    let _db = setup_temp_storage_db().await;
    let registry = build_registry();

    let default = call_rpc(&registry, "Lock.GetConfig", None)
        .await
        .unwrap();
    let config = default.get("config").expect("config");
    assert_eq!(config.get("show_clock"), Some(&json!(true)));
    assert_eq!(config.get("show_media"), Some(&json!(false)));

    let set = call_method_unchecked(
        &registry,
        "Lock.SetConfig",
        Some(json!({
            "partial": {
                "show_media": true,
                "show_notifications": true
            }
        })),
    )
    .await
    .unwrap();
    assert_eq!(set.get("config").and_then(|c| c.get("show_media")), Some(&json!(true)));
    assert_eq!(
        set.get("config").and_then(|c| c.get("show_notifications")),
        Some(&json!(true))
    );
    assert_eq!(set.get("config").and_then(|c| c.get("show_clock")), Some(&json!(true)));

    let got = call_rpc(&registry, "Lock.GetConfig", None).await.unwrap();
    assert_eq!(
        got.get("config").and_then(|c| c.get("show_media")),
        Some(&json!(true))
    );

    let err = call_method_unchecked(
        &registry,
        "Lock.SetConfig",
        Some(json!({ "partial": { "evil": true } })),
    )
    .await;
    assert!(err.is_err());

    let err = call_method_unchecked(&registry, "Lock.SetConfig", None)
        .await
        .expect_err("missing partial");
    assert!(err.to_string().to_lowercase().contains("partial"));
}
