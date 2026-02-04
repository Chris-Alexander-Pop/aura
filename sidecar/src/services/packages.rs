use crate::services::ServiceRegistry;
use crate::utils::process;
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
        let mut packages = Vec::new();

        // Try pacman (Arch Linux)
        if let Ok(output) = process::exec_command(&["pacman", "-Q"]).await {
            for line in output.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(Package {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
        } else if let Ok(output) = process::exec_command(&["dpkg", "-l"]).await {
            // Try dpkg (Debian/Ubuntu)
            for line in output.lines().skip(5) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    packages.push(Package {
                        name: parts[1].to_string(),
                        version: parts[2].to_string(),
                        description: parts.get(4).map(|s| s.to_string()).unwrap_or_default(),
                        installed: parts[0] == "ii",
                    });
                }
            }
        }

        Ok(serde_json::to_value(&packages)?)
    });

    registry.register("Packages.Search", |params| async move {
        let query: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("query").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?,
        )?;

        let mut packages = Vec::new();

        // Try pacman
        if let Ok(output) = process::exec_command(&["pacman", "-Ss", &query]).await {
            for line in output.lines() {
                if line.starts_with("core/") || line.starts_with("extra/") || line.starts_with("community/") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let name_version: Vec<&str> = parts[0].split('/').collect();
                        if name_version.len() >= 2 {
                            packages.push(Package {
                                name: name_version[1].to_string(),
                                version: String::new(),
                                description: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
                                installed: false,
                            });
                        }
                    }
                }
            }
        } else if let Ok(output) = process::exec_command(&["apt", "search", &query]).await {
            // Try apt
            for line in output.lines().skip(1) {
                if let Some(name_part) = line.split_whitespace().next() {
                    packages.push(Package {
                        name: name_part.to_string(),
                        version: String::new(),
                        description: line.to_string(),
                        installed: false,
                    });
                }
            }
        }

        Ok(serde_json::to_value(&packages)?)
    });

    registry.register("Packages.GetPackageInfo", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        // Try pacman
        if let Ok(output) = process::exec_command(&["pacman", "-Qi", &name]).await {
            Ok(serde_json::json!({ "info": output }))
        } else if let Ok(output) = process::exec_command(&["dpkg", "-s", &name]).await {
            Ok(serde_json::json!({ "info": output }))
        } else {
            anyhow::bail!("Package not found")
        }
    });

    registry.register("Packages.Install", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        // Try pacman
        if process::exec_command(&["sudo", "pacman", "-S", "--noconfirm", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["sudo", "apt", "install", "-y", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to install package")
        }
    });

    registry.register("Packages.Remove", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        if process::exec_command(&["sudo", "pacman", "-R", "--noconfirm", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["sudo", "apt", "remove", "-y", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to remove package")
        }
    });

    registry.register("Packages.Update", |_params| async move {
        if process::exec_command(&["sudo", "pacman", "-Sy"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["sudo", "apt", "update"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to update package lists")
        }
    });

    registry.register("Packages.Upgrade", |_params| async move {
        if process::exec_command(&["sudo", "pacman", "-Syu", "--noconfirm"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["sudo", "apt", "upgrade", "-y"]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to upgrade packages")
        }
    });

    registry.register("Packages.GetUpgradable", |_params| async move {
        let mut packages = Vec::new();

        if let Ok(output) = process::exec_command(&["pacman", "-Qu"]).await {
            for line in output.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(Package {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
        } else if let Ok(output) = process::exec_command(&["apt", "list", "--upgradable"]).await {
            for line in output.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    packages.push(Package {
                        name: parts[0].split('/').next().unwrap().to_string(),
                        version: String::new(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
        }

        Ok(serde_json::to_value(&packages)?)
    });

    registry.register("Packages.GetPackageFiles", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        if let Ok(output) = process::exec_command(&["pacman", "-Ql", &name]).await {
            Ok(serde_json::json!({ "files": output }))
        } else if let Ok(output) = process::exec_command(&["dpkg", "-L", &name]).await {
            Ok(serde_json::json!({ "files": output }))
        } else {
            anyhow::bail!("Package not found")
        }
    });

    registry.register("Packages.GetPackageDependencies", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        if let Ok(output) = process::exec_command(&["pacman", "-Qi", &name]).await {
            // Parse dependencies from pacman output
            Ok(serde_json::json!({ "dependencies": output }))
        } else if let Ok(output) = process::exec_command(&["apt-cache", "depends", &name]).await {
            Ok(serde_json::json!({ "dependencies": output }))
        } else {
            anyhow::bail!("Package not found")
        }
    });

    registry.register("Packages.GetAurPackages", |_params| async move {
        // Check if yay or paru is available
        if let Ok(output) = process::exec_command(&["yay", "-Qm"]).await {
            let mut packages = Vec::new();
            for line in output.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(Package {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
            Ok(serde_json::to_value(&packages)?)
        } else {
            Ok(serde_json::json!([]))
        }
    });

    registry.register("Packages.InstallAur", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        if process::exec_command(&["yay", "-S", "--noconfirm", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else if process::exec_command(&["paru", "-S", "--noconfirm", &name]).await.is_ok() {
            Ok(serde_json::json!({ "success": true }))
        } else {
            anyhow::bail!("Failed to install AUR package")
        }
    });

    registry.register("Packages.GetSnapPackages", |_params| async move {
        if let Ok(output) = process::exec_command(&["snap", "list"]).await {
            let mut packages = Vec::new();
            for line in output.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(Package {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
            Ok(serde_json::to_value(&packages)?)
        } else {
            Ok(serde_json::json!([]))
        }
    });

    registry.register("Packages.GetFlatpakPackages", |_params| async move {
        if let Ok(output) = process::exec_command(&["flatpak", "list"]).await {
            let mut packages = Vec::new();
            for line in output.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    packages.push(Package {
                        name: parts[0].to_string(),
                        version: String::new(),
                        description: String::new(),
                        installed: true,
                    });
                }
            }
            Ok(serde_json::to_value(&packages)?)
        } else {
            Ok(serde_json::json!([]))
        }
    });
}
