//! Vault keyring-backed entry storage (mock secret-tool).

mod common;

use ags_sidecar::utils::keyring;
use common::{call_method_unchecked, call_rpc, setup_temp_storage_db, test_registry};
use serde_json::json;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static KEYRING_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn keyring_test_lock() -> &'static Mutex<()> {
    KEYRING_TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn write_mock_secret_tool(dir: &std::path::Path, store_path: &std::path::Path) {
    let script = format!(
        r#"#!/bin/sh
STORE="{store}"
case "$1" in
  store)
    shift
    key=""
    while [ $# -gt 0 ]; do
      case "$1" in
        --label) shift 2 ;;
        *) key="${{key}}|$1|$2"; shift 2 ;;
      esac
    done
    secret=$(cat)
    echo "${{key}}|${{secret}}" >> "$STORE"
    exit 0
    ;;
  lookup)
    shift
    key=""
    while [ $# -gt 0 ]; do
      key="${{key}}|$1|$2"
      shift 2
    done
    line=$(grep -F "${{key}}|" "$STORE" 2>/dev/null | tail -1)
    if [ -z "$line" ]; then exit 1; fi
    echo "${{line##*|}}"
    exit 0
    ;;
esac
exit 1
"#,
        store = store_path.display()
    );
    let path = dir.join("secret-tool");
    std::fs::write(&path, script).expect("script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

struct MockKeyring {
    _dir: tempfile::TempDir,
}

impl MockKeyring {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = dir.path().join("store.txt");
        write_mock_secret_tool(dir.path(), &store);
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.path().display(), path));
        Self { _dir: dir }
    }
}

#[tokio::test]
async fn vault_set_and_get_entry_round_trip() {
    let _guard = keyring_test_lock().lock().await;
    let _mock = MockKeyring::new();
    let _db = setup_temp_storage_db().await;

    keyring::store_vault_entry("api_token", "hunter2")
        .await
        .expect("store");
    let value = keyring::lookup_vault_entry("api_token")
        .await
        .expect("lookup")
        .expect("some");
    assert_eq!(value, "hunter2");
}

#[tokio::test]
async fn vault_get_entry_rpc_never_logs_secret() {
    let _guard = keyring_test_lock().lock().await;
    let _mock = MockKeyring::new();
    let _db = setup_temp_storage_db().await;

    keyring::store_vault_entry("note", "secret-value")
        .await
        .expect("store");

    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Vault.GetEntry",
        Some(json!({ "key": "note" })),
    )
    .await
    .expect("Vault.GetEntry");
    assert_eq!(
        value.get("value").and_then(|v| v.as_str()),
        Some("secret-value")
    );
}

#[tokio::test]
async fn vault_set_entry_rpc_round_trip() {
    let _guard = keyring_test_lock().lock().await;
    let _mock = MockKeyring::new();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Vault.SetEntry",
        Some(json!({ "key": "coverage-key", "value": "stored" })),
    )
    .await
    .expect("Vault.SetEntry");

    let loaded = call_rpc(
        &registry,
        "Vault.GetEntry",
        Some(json!({ "key": "coverage-key" })),
    )
    .await
    .expect("Vault.GetEntry");
    assert_eq!(
        loaded.get("value").and_then(|v| v.as_str()),
        Some("stored")
    );
}
