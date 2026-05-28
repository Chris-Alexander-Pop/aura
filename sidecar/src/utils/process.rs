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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn exec_command_success_trims_stdout() {
        let out = exec_command(&["echo", "  hello  "]).await.unwrap();
        assert_eq!(out, "hello");
    }

    #[tokio::test]
    async fn exec_command_empty_stdout_on_success() {
        let out = exec_command(&["true"]).await.unwrap();
        assert_eq!(out, "");
    }

    #[tokio::test]
    async fn exec_command_failure_reports_command() {
        let err = exec_command(&["sh", "-c", "echo fail >&2; exit 1"])
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Command failed: sh"));
        assert!(msg.contains("fail"));
    }

    #[tokio::test]
    async fn exec_command_with_env_applies_vars() {
        let out = exec_command_with_env(
            &["sh", "-c", "printf %s \"$AURA_TEST_ENV\""],
            &[("AURA_TEST_ENV", "ok")],
        )
        .await
        .unwrap();
        assert_eq!(out, "ok");
    }

    #[tokio::test]
    async fn exec_command_detached_spawns_without_waiting() {
        exec_command_detached(&["true"]).await.unwrap();
    }
}
