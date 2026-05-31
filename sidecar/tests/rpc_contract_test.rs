//! Manifest ↔ UI ↔ integration-test contract checks (no live RPC calls).

mod common;

use common::{
    denied_rpc_reason, integration_test_methods_from_sources, is_denied_rpc_method,
    is_safe_readonly_rpc, load_api_ts_methods,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn load_manifest() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rpc-manifest.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("rpc-manifest.json array")
}

fn load_integration_sources() -> String {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut paths: Vec<PathBuf> = fs::read_dir(&tests_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", tests_dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
        .filter(|p| p.file_name().is_some_and(|n| n != "mod.rs"))
        .collect();
    paths.sort();
    let mut combined = String::new();
    for path in paths {
        combined.push_str(&fs::read_to_string(&path).expect("read integration source"));
        combined.push('\n');
    }
    combined
}

#[test]
fn api_ts_methods_in_manifest() {
    let manifest: BTreeSet<String> = load_manifest().into_iter().collect();
    let api = load_api_ts_methods();
    let mut missing = Vec::new();
    for method in &api {
        if !manifest.contains(method) {
            missing.push(method.clone());
        }
    }
    assert!(
        missing.is_empty(),
        "api.ts methods missing from rpc-manifest.json: {missing:?}"
    );
}

#[test]
fn api_ts_methods_covered_by_integration_or_denied() {
    let mut tested = integration_test_methods_from_sources(&load_integration_sources());
    let sources = load_integration_sources();
    if sources.contains("api_ts_readonly_methods_resolve") {
        tested.extend(
            load_api_ts_methods()
                .into_iter()
                .filter(|m| is_safe_readonly_rpc(m)),
        );
    }
    let api = load_api_ts_methods();
    let mut gaps = Vec::new();
    for method in api {
        if is_denied_rpc_method(&method) {
            let _ = denied_rpc_reason(&method);
            continue;
        }
        if !is_safe_readonly_rpc(&method) {
            continue;
        }
        if !tested.contains(&method) {
            gaps.push(method);
        }
    }
    assert!(
        gaps.is_empty(),
        "api.ts read-only methods without integration test: {gaps:?}"
    );
}

#[test]
fn readonly_manifest_methods_covered_or_denied() {
    let manifest = load_manifest();
    let tested = integration_test_methods_from_sources(&load_integration_sources());
    let mut gaps = Vec::new();
    for method in manifest {
        if !is_safe_readonly_rpc(&method) {
            continue;
        }
        if is_denied_rpc_method(&method) {
            continue;
        }
        if !tested.contains(&method) {
            gaps.push(method);
        }
    }
    assert!(
        gaps.is_empty(),
        "read-only rpc-manifest methods without integration test: {gaps:?}"
    );
}
