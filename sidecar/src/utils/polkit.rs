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

/// Run a command with elevated privileges via `pkexec` when available, else `sudo`.
///
/// The wrapper is prepended to `args` (e.g. `["ufw", "enable"]` → `pkexec ufw enable`).
pub async fn run_privileged(args: &[&str]) -> Result<String> {
    let wrapper = if process::run_allowlisted(&["which", "pkexec"])
        .await
        .is_ok()
    {
        privilege_wrapper(true)
    } else {
        privilege_wrapper(false)
    };
    let mut cmd = vec![wrapper];
    cmd.extend_from_slice(args);
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
}
