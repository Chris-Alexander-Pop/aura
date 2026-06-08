use anyhow::Result;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Human-readable secret-tool label for a VPN credential type.
pub(crate) fn vpn_credential_label(credential_type: &str) -> &str {
    match credential_type {
        "vpn_user" => "Caelestia VPN User",
        "vpn_password" => "Caelestia VPN Password",
        "vpn_mfa" => "Caelestia VPN MFA",
        other => other,
    }
}

/// Human-readable secret-tool label for a stored Wi-Fi password.
pub(crate) fn wifi_credential_label(ssid: &str) -> String {
    format!("Aura Wi-Fi: {ssid}")
}

/// Store a VPN credential in the desktop keyring using `secret-tool`, matching the
/// scheme used by the legacy Caelestia QML implementation.
///
/// - `credential_type` is one of: "vpn_user", "vpn_password", "vpn_mfa"
pub async fn store_vpn_credential(vpn_id: &str, credential_type: &str, value: &str) -> Result<()> {
    let label = vpn_credential_label(credential_type);

    // `secret-tool store` reads the secret from stdin.
    let mut cmd = Command::new("secret-tool");
    cmd.arg("store")
        .arg("--label")
        .arg(label)
        .arg("application")
        .arg("caelestia")
        .arg("type")
        .arg(credential_type)
        .arg("vpn_id")
        .arg(vpn_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(value.as_bytes()).await?;
    }

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("secret-tool store failed with status {}", status);
    }

    Ok(())
}

/// Look up a VPN credential by vpn_id and credential_type from the keyring.
pub async fn lookup_vpn_credential(vpn_id: &str, credential_type: &str) -> Result<Option<String>> {
    let output = Command::new("secret-tool")
        .arg("lookup")
        .arg("application")
        .arg("caelestia")
        .arg("type")
        .arg(credential_type)
        .arg("vpn_id")
        .arg(vpn_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    if !output.status.success() {
        // Treat missing secret or error as "not found"
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Store a Wi-Fi password in GNOME Keyring (Aura application namespace).
pub async fn store_wifi_password(ssid: &str, password: &str) -> Result<()> {
    let mut cmd = Command::new("secret-tool");
    cmd.arg("store")
        .arg("--label")
        .arg(wifi_credential_label(ssid))
        .arg("application")
        .arg("aura")
        .arg("type")
        .arg("wifi_password")
        .arg("ssid")
        .arg(ssid)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(password.as_bytes()).await?;
    }

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("secret-tool store failed with status {}", status);
    }
    Ok(())
}

/// Store CalDAV basic-auth password (never logged by RPC layer).
pub async fn store_caldav_password(username: &str, password: &str) -> Result<()> {
    let mut cmd = Command::new("secret-tool");
    cmd.arg("store")
        .arg("--label")
        .arg(format!("Aura CalDAV: {username}"))
        .arg("application")
        .arg("aura")
        .arg("type")
        .arg("caldav_password")
        .arg("username")
        .arg(username)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(password.as_bytes()).await?;
    }
    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("secret-tool store failed with status {}", status);
    }
    Ok(())
}

/// Look up CalDAV password for a username.
pub async fn lookup_caldav_password(username: &str) -> Result<Option<String>> {
    let output = Command::new("secret-tool")
        .arg("lookup")
        .arg("application")
        .arg("aura")
        .arg("type")
        .arg("caldav_password")
        .arg("username")
        .arg(username)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Store an Aura Vault entry (opaque secret; never log values).
pub async fn store_vault_entry(key: &str, value: &str) -> Result<()> {
    let mut cmd = Command::new("secret-tool");
    cmd.arg("store")
        .arg("--label")
        .arg(format!("Aura Vault: {key}"))
        .arg("application")
        .arg("aura-vault")
        .arg("type")
        .arg("vault_entry")
        .arg("key")
        .arg(key)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(value.as_bytes()).await?;
    }
    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("secret-tool store failed with status {}", status);
    }
    Ok(())
}

/// Look up a Vault entry by key.
pub async fn lookup_vault_entry(key: &str) -> Result<Option<String>> {
    let output = Command::new("secret-tool")
        .arg("lookup")
        .arg("application")
        .arg("aura-vault")
        .arg("type")
        .arg("vault_entry")
        .arg("key")
        .arg(key)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Look up a stored Wi-Fi password by SSID.
pub async fn lookup_wifi_password(ssid: &str) -> Result<Option<String>> {
    let output = Command::new("secret-tool")
        .arg("lookup")
        .arg("application")
        .arg("aura")
        .arg("type")
        .arg("wifi_password")
        .arg("ssid")
        .arg(ssid)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    static KEYRING_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn keyring_test_lock() -> &'static Mutex<()> {
        KEYRING_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn vpn_credential_labels_match_caelestia() {
        assert_eq!(vpn_credential_label("vpn_user"), "Caelestia VPN User");
        assert_eq!(vpn_credential_label("vpn_password"), "Caelestia VPN Password");
        assert_eq!(vpn_credential_label("vpn_mfa"), "Caelestia VPN MFA");
    }

    #[test]
    fn vpn_credential_label_passthrough_unknown_type() {
        assert_eq!(vpn_credential_label("custom_token"), "custom_token");
    }

    #[test]
    fn wifi_credential_label_includes_ssid() {
        assert_eq!(wifi_credential_label("Cafe-Guest"), "Aura Wi-Fi: Cafe-Guest");
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

    fn prepend_path(dir: &std::path::Path) {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.display(), path));
    }

    struct MockKeyring {
        _dir: tempfile::TempDir,
        store: std::path::PathBuf,
    }

    impl MockKeyring {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let store = dir.path().join("store.txt");
            write_mock_secret_tool(dir.path(), &store);
            prepend_path(dir.path());
            Self {
                _dir: dir,
                store,
            }
        }
    }

    #[tokio::test]
    async fn store_and_lookup_vpn_credential_via_mock_secret_tool() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        store_vpn_credential("corp-vpn", "vpn_user", "alice").await.expect("store");
        let value = lookup_vpn_credential("corp-vpn", "vpn_user")
            .await
            .expect("lookup")
            .expect("some");
        assert_eq!(value, "alice");
    }

    #[tokio::test]
    async fn lookup_vpn_credential_missing_returns_none() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        let value = lookup_vpn_credential("missing", "vpn_password")
            .await
            .expect("lookup");
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn store_and_lookup_wifi_password_via_mock_secret_tool() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        store_wifi_password("HomeNet", "s3cret").await.expect("store");
        let value = lookup_wifi_password("HomeNet")
            .await
            .expect("lookup")
            .expect("some");
        assert_eq!(value, "s3cret");
    }

    #[tokio::test]
    async fn lookup_wifi_empty_stdout_returns_none() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        store_wifi_password("EmptySSID", "").await.expect("store");
        let value = lookup_wifi_password("EmptySSID").await.expect("lookup");
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn store_and_lookup_caldav_password_via_mock_secret_tool() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        store_caldav_password("alice@example.com", "cal-secret")
            .await
            .expect("store");
        let value = lookup_caldav_password("alice@example.com")
            .await
            .expect("lookup")
            .expect("some");
        assert_eq!(value, "cal-secret");
    }

    #[tokio::test]
    async fn store_and_lookup_vault_entry_via_mock_secret_tool() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        store_vault_entry("api-token", "vault-value")
            .await
            .expect("store");
        let value = lookup_vault_entry("api-token")
            .await
            .expect("lookup")
            .expect("some");
        assert_eq!(value, "vault-value");
    }

    #[tokio::test]
    async fn lookup_vault_entry_missing_returns_none() {
        let _guard = keyring_test_lock().lock().await;
        let _mock = MockKeyring::new();
        let value = lookup_vault_entry("missing-key").await.expect("lookup");
        assert!(value.is_none());
    }
}
