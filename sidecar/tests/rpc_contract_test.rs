//! Manifest ↔ UI ↔ integration-test contract checks (no live RPC calls).

mod common;

use common::{denied_rpc_reason, integration_test_methods_from_sources, is_denied_rpc_method};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn load_manifest() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rpc-manifest.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("rpc-manifest.json array")
}

fn load_api_ts_methods() -> BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/lib/api.ts");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let re = regex::Regex::new(
        r#""((?:Power|Network|Bluetooth|Audio|System|Weather|Vpn|Calendar|Packages|Notifications|Keybinds|Logs|Security|Performance|DevOps|Productivity|Automation|Communication|Fitness|Brightness|Hyprland|Session|Aura|Apps|Media|Process)\.[A-Za-z]+)""#,
    )
    .expect("api method regex");
    re.captures_iter(&text)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn load_integration_sources() -> String {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut combined = String::new();
    for name in ["integration_test.rs", "integration_contracts.rs", "rpc_contract_test.rs"] {
        let path = tests_dir.join(name);
        if path.exists() {
            combined.push_str(&fs::read_to_string(&path).expect("read integration source"));
            combined.push('\n');
        }
    }
    combined
}

fn is_safe_readonly_rpc(method: &str) -> bool {
    if is_denied_rpc_method(method) {
        return false;
    }
    let verb = method.rsplit('.').next().unwrap_or("");
    const MUTATING: &[&str] = &[
        "Set", "Create", "Delete", "Update", "Install", "Remove", "Upgrade", "Connect",
        "Disconnect", "Forget", "Toggle", "Enable", "Disable", "Launch", "Kill", "Start",
        "Stop", "Restart", "Apply", "Load", "Save", "Import", "Clear", "Dismiss", "Invoke",
        "Run", "Pull", "Pair", "Sync", "Mark", "Send", "Mute", "Capture", "Add", "Reload",
        "Refresh", "Route", "Follow", "Export", "Import", "Unset", "PowerOff", "Logout", "Lock",
        "Reboot", "Suspend", "ScanPorts",
    ];
    if MUTATING.iter().any(|v| verb == *v || verb.starts_with(v)) {
        return false;
    }
    verb.starts_with("Get")
        || verb.starts_with("List")
        || verb.starts_with("Validate")
        || verb.starts_with("Scan")
        || verb.starts_with("Search")
        || verb.starts_with("Filter")
        || verb == "IsEnabled"
        || verb == "ExportIcs"
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
    let tested = integration_test_methods_from_sources(&load_integration_sources());
    let api = load_api_ts_methods();
    let mut gaps = Vec::new();
    for method in api {
        if is_denied_rpc_method(&method) {
            let _ = denied_rpc_reason(&method);
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
