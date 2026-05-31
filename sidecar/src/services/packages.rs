use crate::services::ServiceRegistry;
use crate::utils::{process, storage, transactions};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

const AUTO_UPDATE_NS: &str = "packages";
const AUTO_UPDATE_KEY: &str = "auto_update_policy";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependencyRef {
    pub name: String,
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependencyGraph {
    pub name: String,
    pub depends: Vec<PackageDependencyRef>,
    pub optional_depends: Vec<PackageDependencyRef>,
    pub required_by: Vec<String>,
    pub optional_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdatePolicy {
    /// `manual` | `security_only` | `all`
    pub mode: String,
    pub schedule_cron: Option<String>,
    pub reboot_hint: bool,
}

impl Default for AutoUpdatePolicy {
    fn default() -> Self {
        Self {
            mode: "manual".to_string(),
            schedule_cron: None,
            reboot_hint: false,
        }
    }
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
        let result = crate::utils::polkit::run_privileged(&["pacman", "-S", "--noconfirm", &name]).await;
        log_result("install", vec![name.clone()], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Remove", |params| async move {
        let name: String = package_name_from_params(params)?;
        let result = crate::utils::polkit::run_privileged(&["pacman", "-R", "--noconfirm", &name]).await;
        log_result("remove", vec![name.clone()], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Update", |_params| async move {
        let result = crate::utils::polkit::run_privileged(&["pacman", "-Sy"]).await;
        log_result("update", vec![], &result).await;
        result?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Packages.Upgrade", |_params| async move {
        let result = crate::utils::polkit::run_privileged(&["pacman", "-Syu", "--noconfirm"]).await;
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
        let graph = package_dependency_graph_from_qi(&name, &output);
        Ok(serde_json::to_value(graph)?)
    });

    registry.register("Packages.GetReverseDependencies", |params| async move {
        let name: String = package_name_from_params(params)?;
        let mut reverse = Vec::new();

        if let Ok(output) = process::exec_command(&["pacman", "-Qi", &name]).await {
            reverse.extend(parse_pacman_qi_name_list(
                &parse_pacman_qi_field(&output, "Required By"),
            ));
            reverse.extend(parse_pacman_qi_name_list(
                &parse_pacman_qi_field(&output, "Optional For"),
            ));
        }

        if let Ok(tree) = process::exec_command(&["pactree", "-r", &name]).await {
            for pkg in parse_pactree_reverse(&tree) {
                if pkg != name && !reverse.contains(&pkg) {
                    reverse.push(pkg);
                }
            }
        }

        reverse.sort();
        Ok(json!({ "name": name, "reverse_dependencies": reverse }))
    });

    registry.register("Packages.GetAutoUpdatePolicy", |_params| async move {
        let policy = load_auto_update_policy().await?;
        Ok(serde_json::to_value(policy)?)
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

async fn load_auto_update_policy() -> Result<AutoUpdatePolicy> {
    storage::init().await?;
    if let Some(raw) = storage::get_kv(AUTO_UPDATE_NS, AUTO_UPDATE_KEY).await? {
        if let Ok(policy) = serde_json::from_value::<AutoUpdatePolicy>(raw) {
            return Ok(policy);
        }
    }
    Ok(AutoUpdatePolicy::default())
}

/// Parse `pacman -Qi` field value lines into package dependency refs.
pub fn parse_pacman_dep_tokens(value: &str) -> Vec<PackageDependencyRef> {
    if value.eq_ignore_ascii_case("none") || value.trim().is_empty() {
        return Vec::new();
    }
    value
        .split_whitespace()
        .map(parse_dep_token)
        .collect()
}

fn parse_dep_token(token: &str) -> PackageDependencyRef {
    for sep in [">=", "<=", ">", "<", "="] {
        if let Some((name, constraint)) = token.split_once(sep) {
            return PackageDependencyRef {
                name: name.to_string(),
                constraint: Some(format!("{sep}{constraint}")),
            };
        }
    }
    PackageDependencyRef {
        name: token.to_string(),
        constraint: None,
    }
}

pub fn parse_pacman_qi_field(output: &str, field: &str) -> String {
    let mut value = String::new();
    let mut in_field = false;

    for line in output.lines() {
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim();
            if key == field {
                in_field = true;
                value = line[idx + 1..].trim().to_string();
                continue;
            }
        }
        if in_field {
            if line.starts_with(' ') || line.starts_with('\t') {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
            } else {
                break;
            }
        }
    }
    value
}

pub fn parse_pacman_qi_name_list(value: &str) -> Vec<String> {
    if value.eq_ignore_ascii_case("none") || value.trim().is_empty() {
        return Vec::new();
    }
    value
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

pub fn package_dependency_graph_from_qi(name: &str, qi_output: &str) -> PackageDependencyGraph {
    PackageDependencyGraph {
        name: name.to_string(),
        depends: parse_pacman_dep_tokens(&parse_pacman_qi_field(qi_output, "Depends On")),
        optional_depends: parse_pacman_dep_tokens(&parse_pacman_qi_field(qi_output, "Optional Deps")),
        required_by: parse_pacman_qi_name_list(&parse_pacman_qi_field(qi_output, "Required By")),
        optional_for: parse_pacman_qi_name_list(&parse_pacman_qi_field(qi_output, "Optional For")),
    }
}

/// Unique package names from `pactree -r` output (excludes tree-drawing characters).
pub fn parse_pactree_reverse(output: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let without_tree = trimmed
            .trim_start_matches(|c: char| {
                c.is_whitespace() || "│├└─".contains(c)
            })
            .split_whitespace()
            .next()
            .unwrap_or("");
        if without_tree.is_empty() {
            continue;
        }
        if without_tree
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
        {
            names.push(without_tree.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
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

    #[test]
    fn parse_pacman_qi_dependency_graph_fixture() {
        let fixture = include_str!("../../tests/fixtures/packages/pacman_qi_pacman.txt");
        let graph = package_dependency_graph_from_qi("pacman", fixture);
        assert_eq!(graph.name, "pacman");
        assert_eq!(graph.depends.len(), 4);
        assert_eq!(graph.depends[0].name, "libarchive");
        assert_eq!(graph.optional_depends[0].name, "git");
        assert_eq!(graph.required_by, vec!["aura", "yay"]);
        assert!(graph.optional_for.is_empty());
    }

    #[test]
    fn parse_pactree_reverse_fixture() {
        let fixture = include_str!("../../tests/fixtures/packages/pactree_r_firefox.txt");
        let names = parse_pactree_reverse(fixture);
        assert!(names.contains(&"firefox".to_string()));
        assert!(names.contains(&"gtk3".to_string()));
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn parse_pacman_qi_field_none_is_empty() {
        let text = "Required By     : None\n";
        assert!(parse_pacman_qi_name_list(&parse_pacman_qi_field(text, "Required By")).is_empty());
    }
}
