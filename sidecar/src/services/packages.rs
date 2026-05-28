use crate::services::ServiceRegistry;
use crate::utils::{privileged, process, transactions};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Packages.GetInstalled", |_params| async move {
        let output = process::exec_command(&["pacman", "-Q"]).await?;
        Ok(serde_json::to_value(parse_pacman_q(&output))?)
    });

    registry.register("Packages.Search", |params| async move {
        let query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("query").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?,
        )?;

        let output = process::exec_command(&["pacman", "-Ss", &query]).await?;
        Ok(serde_json::to_value(parse_pacman_search(&output))?)
    });

    registry.register("Packages.GetPackageInfo", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let output = process::exec_command(&["pacman", "-Qi", &name]).await?;
        Ok(serde_json::json!({ "info": output }))
    });

    registry.register("Packages.Install", |params| async move {
        let name: String = package_name_from_params(params)?;
        let result = privileged::run_privileged(&["pacman", "-S", "--noconfirm", &name]).await;
        log_result("install", vec![name.clone()], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Remove", |params| async move {
        let name: String = package_name_from_params(params)?;
        let result = privileged::run_privileged(&["pacman", "-R", "--noconfirm", &name]).await;
        log_result("remove", vec![name.clone()], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Update", |_params| async move {
        let result = privileged::run_privileged(&["pacman", "-Sy"]).await;
        log_result("update", vec![], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Upgrade", |_params| async move {
        let result = privileged::run_privileged(&["pacman", "-Syu", "--noconfirm"]).await;
        log_result("upgrade", vec![], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.GetUpgradable", |_params| async move {
        let output = process::exec_command(&["pacman", "-Qu"])
            .await
            .unwrap_or_default();
        Ok(serde_json::to_value(parse_pacman_qu(&output))?)
    });

    registry.register("Packages.GetTransactionHistory", |params| async move {
        let limit: usize = params
            .as_ref()
            .and_then(|p| p.get("limit").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(50);
        let rows = transactions::read_package_transactions(limit).await?;
        Ok(serde_json::to_value(rows)?)
    });

    registry.register("Packages.GetPackageFiles", |params| async move {
        let name: String = package_name_from_params(params)?;
        let output = process::exec_command(&["pacman", "-Ql", &name]).await?;
        Ok(serde_json::json!({ "files": output }))
    });

    registry.register("Packages.GetPackageDependencies", |params| async move {
        let name: String = package_name_from_params(params)?;
        let output = process::exec_command(&["pacman", "-Qi", &name]).await?;
        Ok(serde_json::json!({ "dependencies": output }))
    });

    registry.register("Packages.GetAurPackages", |_params| async move {
        if let Ok(output) = process::exec_command(&["yay", "-Qm"]).await {
            Ok(serde_json::to_value(parse_pacman_q(&output))?)
        } else if let Ok(output) = process::exec_command(&["paru", "-Qm"]).await {
            Ok(serde_json::to_value(parse_pacman_q(&output))?)
        } else {
            Ok(serde_json::json!([]))
        }
    });

    registry.register("Packages.InstallAur", |params| async move {
        let name: String = package_name_from_params(params)?;
        let result = if process::exec_command(&["which", "yay"]).await.is_ok() {
            process::exec_command(&["yay", "-S", "--noconfirm", &name]).await
        } else {
            process::exec_command(&["paru", "-S", "--noconfirm", &name]).await
        };
        log_result("install_aur", vec![name.clone()], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.GetSnapPackages", |_params| async move {
        Ok(serde_json::json!([]))
    });

    registry.register("Packages.GetFlatpakPackages", |_params| async move {
        if let Ok(output) = process::exec_command(&["flatpak", "list", "--columns=application"]).await {
            let apps: Vec<Package> = output
                .lines()
                .skip(1)
                .filter(|l| !l.trim().is_empty())
                .map(|l| Package {
                    name: l.trim().to_string(),
                    version: String::new(),
                    description: String::new(),
                    installed: true,
                })
                .collect();
            Ok(serde_json::to_value(apps)?)
        } else {
            Ok(serde_json::json!([]))
        }
    });
}

fn package_name_from_params(params: Option<serde_json::Value>) -> Result<String> {
    Ok(serde_json::from_value(
        params
            .and_then(|p| p.get("name").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
    )?)
}

async fn log_result(action: &str, packages: Vec<String>, result: &Result<String>) {
    match result {
        Ok(_) => {
            transactions::log_package_transaction(action, packages, true, None).await;
        }
        Err(e) => {
            transactions::log_package_transaction(
                action,
                packages,
                false,
                Some(e.to_string()),
            )
            .await;
        }
    }
}

fn looks_like_pacman_version(version: &str) -> bool {
    version.contains('-') && version.chars().any(|c| c.is_ascii_digit())
}

pub fn parse_pacman_qu(output: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && looks_like_pacman_version(parts[1]) {
            packages.push(Package {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                description: String::new(),
                installed: true,
            });
        }
    }
    packages
}

pub fn parse_pacman_q(output: &str) -> Vec<Package> {
    parse_pacman_qu(output)
}

pub fn parse_pacman_search(output: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for line in output.lines() {
        if line.starts_with("core/")
            || line.starts_with("extra/")
            || line.starts_with("community/")
            || line.contains('/')
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let name_version: Vec<&str> = parts[0].split('/').collect();
            if name_version.len() >= 2 {
                packages.push(Package {
                    name: name_version[1].to_string(),
                    version: parts.get(1).unwrap_or(&"").to_string(),
                    description: parts.get(2).map(|s| s.to_string()).unwrap_or_default(),
                    installed: false,
                });
            }
        }
    }
    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pacman_qu_fixture() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_qu.txt");
        let pkgs = parse_pacman_qu(fixture);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "linux");
        assert_eq!(pkgs[1].name, "firefox");
    }

    #[test]
    fn parse_pacman_q_installed_fixture() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_q.txt");
        let pkgs = parse_pacman_q(fixture);
        assert_eq!(pkgs.len(), 3);
        assert_eq!(pkgs[2].name, "vim");
        assert_eq!(pkgs[2].version, "9.1-1");
    }

    #[test]
    fn parse_pacman_search_fixture() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_ss.txt");
        let pkgs = parse_pacman_search(fixture);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "firefox");
        assert!(!pkgs[0].installed);
        assert_eq!(pkgs[1].name, "firefox-developer-edition");
    }

    #[test]
    fn parse_pacman_qu_skips_malformed_lines() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_qu_malformed.txt");
        let pkgs = parse_pacman_qu(fixture);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "linux");
    }

    #[test]
    fn parse_pacman_search_ignores_non_repo_lines() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_ss_malformed.txt");
        assert!(parse_pacman_search(fixture).is_empty());
    }
}
