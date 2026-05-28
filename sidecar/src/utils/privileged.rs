use anyhow::Result;
use crate::utils::process;

/// Run a command with elevated privileges via `pkexec` when available, else `sudo`.
pub async fn run_privileged(args: &[&str]) -> Result<String> {
    if process::exec_command(&["which", "pkexec"]).await.is_ok() {
        let mut cmd = vec!["pkexec"];
        cmd.extend_from_slice(args);
        return process::exec_command(&cmd).await;
    }
    let mut cmd = vec!["sudo"];
    cmd.extend_from_slice(args);
    process::exec_command(&cmd).await
}
