//! Offensive-security feature RPC shapes (fixture nmap, list scans, audit log).

#![cfg(feature = "offensive-security")]

mod common;

use ags_sidecar::services::offensive_policy::record_audit;
use common::{call_method_unchecked, setup_temp_storage_db, test_registry};
use serde_json::json;
use std::sync::Once;

static OFFENSIVE_TEST_HARNESS: Once = Once::new();

fn offensive_test_harness() {
    OFFENSIVE_TEST_HARNESS.call_once(|| {
        std::env::set_var("AURA_OFFENSIVE_SKIP_RATE_LIMIT", "1");
    });
}

fn nmap_fixture_path() -> String {
    format!(
        "{}/tests/fixtures/security/nmap_minimal.xml",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[tokio::test]
async fn offensive_audit_log_round_trip() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
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
    let entry = &entries[0];
    assert_eq!(entry["method"], "Security.Offensive.Nmap.Scan");
    assert_eq!(entry["status"], "ok");
    assert!(entry["arg_hash"].as_str().is_some_and(|h| !h.is_empty()));
    assert!(entry["user"].as_str().is_some());
    assert!(entry["timestamp"].as_i64().is_some());
    assert!(entry.get("password").is_none());
}

#[tokio::test]
async fn offensive_audit_log_pagination() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    for i in 0..5 {
        record_audit(
            &format!("Security.Offensive.Test.Paginate{i}"),
            &Some(json!({ "n": i })),
            "ok",
        )
        .await
        .expect("audit");
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }

    let registry = test_registry();
    let page0 = call_method_unchecked(
        &registry,
        "Security.Offensive.GetAuditLog",
        Some(json!({ "limit": 2, "offset": 0 })),
    )
    .await
    .expect("page0");
    let page1 = call_method_unchecked(
        &registry,
        "Security.Offensive.GetAuditLog",
        Some(json!({ "limit": 2, "offset": 2 })),
    )
    .await
    .expect("page1");
    let tail = call_method_unchecked(
        &registry,
        "Security.Offensive.GetAuditLog",
        Some(json!({ "limit": 2, "offset": 4 })),
    )
    .await
    .expect("tail");
    let past_end = call_method_unchecked(
        &registry,
        "Security.Offensive.GetAuditLog",
        Some(json!({ "limit": 10, "offset": 10 })),
    )
    .await
    .expect("past_end");

    fn paginate_only<'a>(entries: &'a [serde_json::Value]) -> Vec<&'a str> {
        entries
            .iter()
            .filter_map(|e| {
                e["method"]
                    .as_str()
                    .filter(|m| m.starts_with("Security.Offensive.Test.Paginate"))
            })
            .collect()
    }

    let p0 = paginate_only(page0.as_array().expect("page0 array"));
    let p1 = paginate_only(page1.as_array().expect("page1 array"));
    let t = paginate_only(tail.as_array().expect("tail array"));
    assert_eq!(p0.len(), 2);
    assert_eq!(p1.len(), 2);
    assert_eq!(t.len(), 1);
    assert!(past_end.as_array().expect("past_end array").is_empty());

    for a in p0.iter() {
        assert!(!p1.contains(a), "pages must not overlap: {a}");
    }
    assert_eq!(p0[0], "Security.Offensive.Test.Paginate4");
    assert_eq!(p0[1], "Security.Offensive.Test.Paginate3");
    assert_eq!(t[0], "Security.Offensive.Test.Paginate0");
}

#[tokio::test]
async fn offensive_nmap_scan_fixture_shape() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.Scan",
        Some(json!({ "target": "127.0.0.1", "scan_type": "syn" })),
    )
    .await
    .expect("Nmap.Scan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    assert_eq!(value["status"], "completed");
    assert_eq!(value["target"], "127.0.0.1");
    let scan_id = value["scan_id"]
        .as_str()
        .expect("scan_id")
        .to_string();
    assert!(scan_id.starts_with("scan_"));
    assert!(
        value["output"]
            .as_str()
            .is_some_and(|o| o.contains("AURA_NMAP_XML_FIXTURE"))
    );

    let ports = value["ports"].as_array().expect("ports");
    assert_eq!(ports.len(), 2);
    assert_eq!(ports[0]["port"], 22);
    assert_eq!(ports[0]["protocol"], "tcp");
    assert!(
        ports[0]["state"]
            .as_str()
            .is_some_and(|s| s.contains("open"))
    );
    assert!(
        ports[0]["service"]
            .as_str()
            .is_some_and(|s| s.contains("ssh"))
    );
    assert_eq!(ports[1]["port"], 53);
    assert_eq!(ports[1]["protocol"], "udp");
    assert!(value["hosts"].as_array().expect("hosts").is_empty());
}

#[tokio::test]
async fn offensive_nmap_list_scans_after_fixture_scan() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let scan = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.Scan",
        Some(json!({ "target": "10.0.0.5" })),
    )
    .await
    .expect("scan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    let scan_id = scan["scan_id"].as_str().expect("scan_id");
    let list = call_method_unchecked(&registry, "Security.Offensive.Nmap.ListScans", None)
        .await
        .expect("ListScans");
    let rows = list.as_array().expect("list array");
    assert!(!rows.is_empty());
    let row = rows
        .iter()
        .find(|r| r["scan_id"] == scan_id)
        .expect("scan in list");
    assert_eq!(row["target"], "10.0.0.5");
    assert_eq!(row["status"], "completed");
    assert_eq!(row["port_count"], 2);
}

#[tokio::test]
async fn offensive_nmap_quick_scan_fixture_shape() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.QuickScan",
        Some(json!({ "target": "192.168.1.1" })),
    )
    .await
    .expect("QuickScan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    assert_eq!(value["status"], "completed");
    assert_eq!(value["target"], "192.168.1.1");
    assert!(value["ports"].as_array().expect("ports").len() >= 1);
}

#[tokio::test]
async fn offensive_nmap_scan_results_round_trip() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let scan = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.Scan",
        Some(json!({ "target": "172.16.0.1", "scan_type": "udp" })),
    )
    .await
    .expect("scan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    let scan_id = scan["scan_id"].as_str().expect("scan_id");
    let results = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.ScanResults",
        Some(json!({ "scan_id": scan_id })),
    )
    .await
    .expect("ScanResults");
    assert_eq!(results["scan_id"], scan_id);
    assert_eq!(results["target"], "172.16.0.1");

    let missing = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.ScanResults",
        Some(json!({ "scan_id": "scan_nonexistent" })),
    )
    .await
    .expect_err("missing scan");
    assert!(missing.to_string().contains("Scan not found"));
}

#[tokio::test]
async fn offensive_nmap_save_results_json_and_errors() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let scan = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.Scan",
        Some(json!({ "target": "10.10.10.10" })),
    )
    .await
    .expect("scan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    let scan_id = scan["scan_id"].as_str().expect("scan_id");
    let saved = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.SaveResults",
        Some(json!({ "scan_id": scan_id, "format": "json" })),
    )
    .await
    .expect("SaveResults json");
    assert_eq!(saved["scan_id"], scan_id);

    let bad_format = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.SaveResults",
        Some(json!({ "scan_id": scan_id, "format": "pdf" })),
    )
    .await
    .expect_err("unsupported format");
    assert!(bad_format.to_string().contains("Unsupported format"));
}

#[tokio::test]
async fn offensive_nmap_scan_missing_target_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.Nmap.Scan", None)
        .await
        .expect_err("missing target");
    assert!(err.to_string().contains("Missing target"));
}

#[tokio::test]
async fn offensive_nmap_full_scan_fixture_shape() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.FullScan",
        Some(json!({ "target": "10.0.0.99" })),
    )
    .await
    .expect("FullScan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    assert_eq!(value["status"], "completed");
    assert_eq!(value["target"], "10.0.0.99");
    assert!(value["ports"].as_array().expect("ports").len() >= 1);
}

#[tokio::test]
async fn offensive_nmap_save_results_xml_fixture_round_trip() {
    offensive_test_harness();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_NMAP_XML_FIXTURE", nmap_fixture_path());
    let registry = test_registry();
    let scan = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.Scan",
        Some(json!({ "target": "203.0.113.1", "scan_type": "full" })),
    )
    .await
    .expect("scan");
    std::env::remove_var("AURA_NMAP_XML_FIXTURE");

    let scan_id = scan["scan_id"].as_str().expect("scan_id");
    let xml = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.SaveResults",
        Some(json!({ "scan_id": scan_id, "format": "xml" })),
    )
    .await
    .expect("SaveResults xml");
    let content = xml["xml"].as_str().expect("xml body");
    assert!(content.contains("<nmaprun>"));
}

#[tokio::test]
async fn offensive_nmap_save_results_missing_scan_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Security.Offensive.Nmap.SaveResults",
        Some(json!({ "scan_id": "scan_missing", "format": "json" })),
    )
    .await
    .expect_err("missing scan");
    assert!(err.to_string().contains("Scan not found"));
}