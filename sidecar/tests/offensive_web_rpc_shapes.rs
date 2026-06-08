//! `Security.Offensive.Web.*` RPC shapes (payload generation + validation errors; no live HTTP).

#![cfg(feature = "offensive-security")]

mod common;

use common::{call_method_unchecked, test_registry};
use serde_json::json;
use std::sync::Once;

static OFFENSIVE_TEST_HARNESS: Once = Once::new();

fn offensive_test_harness() {
    OFFENSIVE_TEST_HARNESS.call_once(|| {
        std::env::set_var("AURA_OFFENSIVE_SKIP_RATE_LIMIT", "1");
    });
}

#[tokio::test]
async fn offensive_web_xss_payload_generate_variants() {
    offensive_test_harness();
    let registry = test_registry();

    for (kind, needle) in [
        ("basic", "<script>"),
        ("img", "<img"),
        ("svg", "<svg"),
        ("unknown", "<script>"),
    ] {
        let value = call_method_unchecked(
            &registry,
            "Security.Offensive.Web.XSS.Payload.Generate",
            Some(json!({ "type": kind })),
        )
        .await
        .expect("XSS.Payload.Generate");
        let payload = value["payload"].as_str().expect("payload");
        assert!(payload.contains(needle), "type={kind} payload={payload}");
    }

    let default_type = call_method_unchecked(
        &registry,
        "Security.Offensive.Web.XSS.Payload.Generate",
        None,
    )
    .await
    .expect("default type");
    assert!(default_type["payload"]
        .as_str()
        .unwrap()
        .contains("<script>"));
}

#[tokio::test]
async fn offensive_web_sqli_missing_params_errors() {
    offensive_test_harness();
    let registry = test_registry();

    let no_url = call_method_unchecked(
        &registry,
        "Security.Offensive.Web.SQLi.Test",
        Some(json!({ "parameter": "id" })),
    )
    .await
    .expect_err("missing url");
    assert!(no_url.to_string().contains("Missing url"));

    let no_param = call_method_unchecked(
        &registry,
        "Security.Offensive.Web.SQLi.Test",
        Some(json!({ "url": "http://127.0.0.1" })),
    )
    .await
    .expect_err("missing parameter");
    assert!(no_param.to_string().contains("Missing parameter"));
}

#[tokio::test]
async fn offensive_web_directory_bruteforce_missing_wordlist() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Security.Offensive.Web.DirectoryBruteForce",
        Some(json!({ "url": "http://127.0.0.1" })),
    )
    .await
    .expect_err("missing wordlist");
    assert!(err.to_string().contains("Missing wordlist"));
}
