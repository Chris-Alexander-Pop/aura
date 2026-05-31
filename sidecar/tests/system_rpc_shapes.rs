//! Read-only `System.*` RPC response shapes.

mod common;

use common::{call_method, test_registry};

#[tokio::test]
async fn system_get_stats_matches_system_stats_contract() {
    let registry = test_registry();
    let value = call_method(&registry, "System.GetStats", None)
        .await
        .expect("System.GetStats");
    for key in ["cpu", "ram", "temp"] {
        let n = value
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("missing or non-numeric {key}"));
        assert!(
            (0.0..=1.0).contains(&n) || key == "temp",
            "{key} out of expected range: {n}"
        );
    }
    if let Some(gpu) = value.get("gpu") {
        assert!(gpu.is_null() || gpu.as_f64().is_some());
    }
}
