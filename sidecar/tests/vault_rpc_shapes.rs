//! Read-only `Vault.List` / `Vault.Backup.Status` RPC shapes; transfer blocked in harness.

mod common;

use ags_sidecar::contract_parsers::parse_rclone_listremotes;
use common::{
    assert_json_object_keys, call_rpc, is_denied_rpc_method, load_fixture, test_registry,
};
use serde_json::json;

#[test]
fn vault_listremotes_fixture_parser() {
    let raw = load_fixture("vault/listremotes.txt");
    let remotes = parse_rclone_listremotes(&raw);
    assert_eq!(remotes.len(), 3);
    assert_eq!(remotes[0].name, "gdrive");
    assert_eq!(remotes[2].name, "s3-remote");
}

#[test]
fn vault_transfer_and_backup_start_denied_in_harness() {
    for method in ["Vault.Transfer", "Vault.Backup.Start"] {
        assert!(
            is_denied_rpc_method(method),
            "{method} should be deny-listed before host integration"
        );
    }
}

#[tokio::test]
async fn vault_list_and_backup_status_schemas() {
    std::env::set_var(
        "AURA_VAULT_LISTREMOTES_OUTPUT",
        load_fixture("vault/listremotes.txt"),
    );
    let registry = test_registry();

    let list = call_rpc(&registry, "Vault.List", None)
        .await
        .expect("Vault.List");
    assert_json_object_keys(&list, &["remotes", "rclone_available"]);
    assert_eq!(list.get("rclone_available"), Some(&json!(true)));
    let remotes = list.get("remotes").and_then(|v| v.as_array()).expect("remotes");
    assert_eq!(remotes.len(), 3);
    let first = remotes.first().and_then(|v| v.as_object()).expect("remote object");
    assert!(first.contains_key("name"));

    let status = call_rpc(&registry, "Vault.Backup.Status", None)
        .await
        .expect("Vault.Backup.Status");
    assert_json_object_keys(
        &status,
        &["state", "engine", "last_success_at", "last_error", "in_progress"],
    );
    assert_eq!(status.get("state").and_then(|v| v.as_str()), Some("idle"));
    assert_eq!(status.get("in_progress"), Some(&json!(false)));

    std::env::remove_var("AURA_VAULT_LISTREMOTES_OUTPUT");
}
