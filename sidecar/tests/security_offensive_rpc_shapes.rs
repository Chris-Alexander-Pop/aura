//! Offensive-security feature RPC shapes (audit log, rate limit).

#![cfg(feature = "offensive-security")]

mod common;

use ags_sidecar::services::offensive_policy::{check_rate_limit, record_audit};
use common::{call_method_unchecked, test_registry};
use serde_json::json;

#[tokio::test]
async fn offensive_audit_log_round_trip() {
    let _db = common::setup_temp_storage_db().await;
    record_audit(
        "Security.Offensive.Nmap.Scan",
        &Some(json!({ "target": "127.0.0.1" })),
        "ok",
    )
    .await
    .expect("audit");

    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Security.Offensive.GetAuditLog",
        Some(json!({ "limit": 10 })),
    )
    .await
    .expect("GetAuditLog");
    let entries = value.as_array().expect("array");
    assert!(!entries.is_empty());
    assert!(entries[0].get("method").is_some());
    assert!(entries[0].get("arg_hash").is_some());
    assert!(!entries[0].get("password").is_some());
}

#[test]
fn offensive_rate_limit_blocks_rapid_repeat() {
    let method = "Security.Offensive.Nmap.Scan";
    check_rate_limit(method).expect("first");
    assert!(check_rate_limit(method).is_err());
}
