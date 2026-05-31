//! Read-only `Packages.*` dependency and policy RPC shapes.

mod common;

use ags_sidecar::contract_parsers::{
    package_dependency_graph_from_qi, parse_pactree_reverse,
};
use common::{call_method, load_fixture, setup_temp_storage_db, test_registry};
use serde_json::json;

#[test]
fn contract_package_dependency_graph_fixture() {
    let qi = load_fixture("packages/pacman_qi_pacman.txt");
    let graph = package_dependency_graph_from_qi("pacman", &qi);
    assert_eq!(graph.required_by, vec!["aura", "yay"]);
    assert!(!graph.depends.is_empty());
}

#[test]
fn contract_pactree_reverse_fixture() {
    let tree = load_fixture("packages/pactree_r_firefox.txt");
    let names = parse_pactree_reverse(&tree);
    assert!(names.contains(&"firefox".to_string()));
}

#[tokio::test]
async fn packages_get_dependencies_shape() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Packages.GetPackageDependencies",
        Some(json!({ "name": "pacman" })),
    )
    .await
    .expect("Packages.GetPackageDependencies");
    let obj = value.as_object().expect("object");
    assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("pacman"));
    assert!(obj.get("depends").and_then(|v| v.as_array()).is_some());
    assert!(obj.get("required_by").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn packages_get_reverse_dependencies_shape() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Packages.GetReverseDependencies",
        Some(json!({ "name": "pacman" })),
    )
    .await
    .expect("Packages.GetReverseDependencies");
    assert!(value.get("reverse_dependencies").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn packages_get_auto_update_policy_defaults() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let value = call_method(&registry, "Packages.GetAutoUpdatePolicy", None)
        .await
        .expect("Packages.GetAutoUpdatePolicy");
    assert_eq!(value.get("mode").and_then(|v| v.as_str()), Some("manual"));
    assert_eq!(
        value.get("reboot_hint").and_then(|v| v.as_bool()),
        Some(false)
    );
}
