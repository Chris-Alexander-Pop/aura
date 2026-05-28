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
}
