//! Polkit / elevated command helper (`pkexec` when available, else `sudo`).

use anyhow::Result;
use crate::utils::process;

/// Wrapper binary for elevated commands (`pkexec` when present, else `sudo`).
pub(crate) fn privilege_wrapper(has_pkexec: bool) -> &'static str {
    if has_pkexec {
        "pkexec"
    } else {
        "sudo"
    }
}

/// Build argv for an elevated command (wrapper + args).
pub(crate) fn build_privileged_argv(has_pkexec: bool, args: &[&str]) -> Vec<String> {
    let mut cmd = vec![privilege_wrapper(has_pkexec).to_string()];
    cmd.extend(args.iter().map(|s| (*s).to_string()));
    cmd
}

/// Run a command with elevated privileges via `pkexec` when available, else `sudo`.
///
/// The wrapper is prepended to `args` (e.g. `["ufw", "enable"]` → `pkexec ufw enable`).
pub async fn run_privileged(args: &[&str]) -> Result<String> {
    let has_pkexec = process::run_allowlisted(&["which", "pkexec"])
        .await
        .is_ok();
    let cmd_owned = build_privileged_argv(has_pkexec, args);
    let cmd: Vec<&str> = cmd_owned.iter().map(String::as_str).collect();
    process::exec_command(&cmd).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privilege_wrapper_prefers_pkexec_when_available() {
        assert_eq!(privilege_wrapper(true), "pkexec");
        assert_eq!(privilege_wrapper(false), "sudo");
    }

    #[test]
    fn build_privileged_argv_prepends_wrapper() {
        assert_eq!(
            build_privileged_argv(true, &["ufw", "enable"]),
            vec!["pkexec".to_string(), "ufw".to_string(), "enable".to_string()]
        );
        assert_eq!(
            build_privileged_argv(false, &["pacman", "-Sy"]),
            vec!["sudo".to_string(), "pacman".to_string(), "-Sy".to_string()]
        );
        assert!(build_privileged_argv(true, &[]).len() == 1);
    }
}
