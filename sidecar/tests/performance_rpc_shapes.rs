//! Read-only `Performance.*` RPC response shapes.

mod common;

use ags_sidecar::contract_parsers::parse_meminfo_cached_buffers_kb;
use common::{call_method, call_method_unchecked, load_fixture, test_registry};
use serde_json::json;

#[test]
fn contract_meminfo_cached_buffers_fixture() {
    let text = load_fixture("performance/meminfo.txt");
    let (cached, buffers) = parse_meminfo_cached_buffers_kb(&text);
    assert_eq!(cached, 4096000);
    assert_eq!(buffers, 512000);
}

#[tokio::test]
async fn performance_get_metrics_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Performance.GetMetrics", None)
        .await
        .expect("Performance.GetMetrics");
    for key in ["cpu_percent", "memory_percent", "disk_percent", "temperature_c"] {
        assert!(
            value.get(key).and_then(|v| v.as_f64()).is_some(),
            "missing {key}"
        );
    }
}

#[tokio::test]
async fn performance_get_memory_stats_includes_cached_buffers() {
    let registry = test_registry();
    let value = call_method(&registry, "Performance.GetMemoryStats", None)
        .await
        .expect("Performance.GetMemoryStats");
    assert!(value.get("cached_mb").and_then(|v| v.as_u64()).is_some());
    assert!(value.get("buffers_mb").and_then(|v| v.as_u64()).is_some());
}

#[tokio::test]
async fn performance_get_processes_aligns_with_list_top() {
    let registry = test_registry();
    let perf = call_method(
        &registry,
        "Performance.GetProcesses",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("Performance.GetProcesses");
    let top = call_method(
        &registry,
        "Process.ListTop",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("Process.ListTop");
    let perf_rows = perf.as_array().expect("perf array");
    let top_rows = top.as_array().expect("top array");
    assert!(!perf_rows.is_empty());
    assert_eq!(perf_rows.len(), top_rows.len());
    for row in perf_rows {
        assert!(row.get("pid").and_then(|v| v.as_u64()).is_some());
        assert!(row.get("cpu_percent").or_else(|| row.get("cpu")).is_some());
        assert!(row.get("memory_mb").is_some());
        assert!(row.get("status").and_then(|v| v.as_str()).is_some());
    }
    let mut last_cpu = f64::MAX;
    for row in perf_rows {
        let cpu = row
            .get("cpu_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        assert!(cpu <= last_cpu + f64::EPSILON, "GetProcesses should be CPU-sorted");
        last_cpu = cpu;
    }
}

#[tokio::test]
async fn performance_apply_preset_dry_run() {
    std::env::set_var("AURA_PERFORMANCE_DRY_RUN", "1");
    let registry = test_registry();
    let value = call_method_unchecked(
        &registry,
        "Performance.ApplyPreset",
        Some(json!({ "preset": "meeting" })),
    )
    .await
    .expect("Performance.ApplyPreset");
    assert_eq!(value.get("dry_run").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        value.get("profile").and_then(|v| v.as_str()),
        Some("balanced")
    );
    std::env::remove_var("AURA_PERFORMANCE_DRY_RUN");
}
