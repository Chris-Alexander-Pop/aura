//! Read-only `System.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};

#[tokio::test]
async fn system_get_stats_matches_system_stats_contract() {
    let registry = test_registry();
    let value = call_method(&registry, "System.GetStats", None)
        .await
        .expect("System.GetStats");
    for key in ["cpu", "ram"] {
        let n = value
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("missing or non-numeric {key}"));
        assert!(
            (0.0..=1.0).contains(&n),
            "{key} out of expected range: {n}"
        );
    }
    let temp = value
        .get("temp")
        .and_then(|v| v.as_f64())
        .expect("missing or non-numeric temp");
    assert!(temp >= 0.0, "temp out of range: {temp}");
    if let Some(gpu) = value.get("gpu") {
        if let Some(g) = gpu.as_f64() {
            assert!((0.0..=1.0).contains(&g), "gpu out of range: {g}");
        } else {
            assert!(gpu.is_null(), "gpu should be null or fraction");
        }
    }
    if let Some(storage) = value.get("storage") {
        if let Some(s) = storage.as_f64() {
            assert!((0.0..=1.0).contains(&s), "storage out of range: {s}");
        } else {
            assert!(storage.is_null(), "storage should be null or fraction");
        }
    }
}
