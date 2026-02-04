use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

pub async fn exec_command(cmd: &[&str]) -> Result<String> {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {} - {}", cmd[0], stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn exec_command_detached(cmd: &[&str]) -> Result<()> {
    Command::new(cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub async fn exec_command_with_env(
    cmd: &[&str],
    env: &[(&str, &str)],
) -> Result<String> {
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    for (key, value) in env {
        command.env(key, value);
    }

    let output = command.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {} - {}", cmd[0], stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
