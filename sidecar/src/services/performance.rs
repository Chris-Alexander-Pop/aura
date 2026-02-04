use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use sysinfo::{System, SystemExt, CpuExt, PidExt, ProcessExt};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreStats {
    pub core_id: usize,
    pub frequency_mhz: u64,
    pub governor: String,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStats {
    pub name: String,
    pub utilization_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub cached_mb: u64,
    pub buffers_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    pub device: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interface: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdService {
    pub name: String,
    pub status: String,
    pub active: bool,
    pub enabled: bool,
}

lazy_static::lazy_static! {
    static ref SYSTEM: RwLock<System> = RwLock::new(System::new_all());
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Performance.GetCpuStats", |_params| async move {
        let mut sys = SYSTEM.write().await;
        sys.refresh_cpu();

        let mut cores = Vec::new();
        for (i, cpu) in sys.cpus().iter().enumerate() {
            let frequency = get_cpu_frequency(i).await.unwrap_or(0);
            let governor = get_cpu_governor(i).await.unwrap_or_else(|_| "unknown".to_string());

            cores.push(CpuCoreStats {
                core_id: i,
                frequency_mhz: frequency,
                governor,
                usage_percent: cpu.cpu_usage() as f64,
            });
        }

        Ok(serde_json::to_value(&cores)?)
    });

    registry.register("Performance.GetGpuStats", |_params| async move {
        let gpu_stats = get_gpu_stats().await?;
        Ok(serde_json::to_value(&gpu_stats)?)
    });

    registry.register("Performance.GetMemoryStats", |_params| async move {
        let mut sys = SYSTEM.write().await;
        sys.refresh_memory();

        let mem_total = sys.total_memory();
        let mem_used = sys.used_memory();
        let mem_free = sys.free_memory();
        // Note: cached_memory and buffered_memory may not be available in all sysinfo versions
        let mem_cached = 0; // sys.cached_memory() if available
        let mem_buffers = 0; // sys.buffered_memory() if available
        let swap_total = sys.total_swap();
        let swap_used = sys.used_swap();

        let stats = MemoryStats {
            total_mb: mem_total / 1024 / 1024,
            used_mb: mem_used / 1024 / 1024,
            free_mb: mem_free / 1024 / 1024,
            cached_mb: mem_cached / 1024 / 1024,
            buffers_mb: mem_buffers / 1024 / 1024,
            swap_total_mb: swap_total / 1024 / 1024,
            swap_used_mb: swap_used / 1024 / 1024,
        };

        Ok(serde_json::to_value(&stats)?)
    });

    registry.register("Performance.GetDiskStats", |_params| async move {
        let disk_stats = get_disk_stats().await?;
        Ok(serde_json::to_value(&disk_stats)?)
    });

    registry.register("Performance.GetNetworkStats", |_params| async move {
        let network_stats = get_network_stats().await?;
        Ok(serde_json::to_value(&network_stats)?)
    });

    registry.register("Performance.SetCpuGovernor", |params| async move {
        let governor: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("governor").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing governor"))?,
        )?;

        // Set governor for all cores
        let output = process::exec_command(&["nproc"]).await?;
        let num_cores: usize = output.trim().parse().unwrap_or(1);

        for i in 0..num_cores {
            let governor_path = format!(
                "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
                i
            );
            let _ = tokio::fs::write(&governor_path, &governor).await;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.SetCpuFrequency", |params| async move {
        let min_mhz: Option<u64> = params
            .as_ref()
            .and_then(|p| p.get("min_mhz").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let max_mhz: Option<u64> = params
            .as_ref()
            .and_then(|p| p.get("max_mhz").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let output = process::exec_command(&["nproc"]).await?;
        let num_cores: usize = output.trim().parse().unwrap_or(1);

        for i in 0..num_cores {
            if let Some(min) = min_mhz {
                let min_path = format!(
                    "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq",
                    i
                );
                let _ = tokio::fs::write(&min_path, &format!("{}000", min)).await;
            }

            if let Some(max) = max_mhz {
                let max_path = format!(
                    "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq",
                    i
                );
                let _ = tokio::fs::write(&max_path, &format!("{}000", max)).await;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.GetProcesses", |_params| async move {
        let mut sys = SYSTEM.write().await;
        sys.refresh_processes();

        let mut processes = Vec::new();
        for (pid, process) in sys.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_percent: process.cpu_usage() as f64,
                memory_mb: process.memory() / 1024,
                status: format!("{:?}", process.status()),
            });
        }

        // Sort by CPU usage
        processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());

        Ok(serde_json::to_value(&processes)?)
    });

    registry.register("Performance.KillProcess", |params| async move {
        let pid: u32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("pid").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing pid"))?,
        )?;

        process::exec_command(&["kill", &pid.to_string()]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.SetProcessPriority", |params| async move {
        let pid: u32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("pid").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing pid"))?,
        )?;

        let priority: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("priority").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing priority"))?,
        )?;

        process::exec_command(&["renice", &priority.to_string(), &pid.to_string()]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.GetSystemdServices", |_params| async move {
        let output = process::exec_command(&["systemctl", "list-units", "--type=service", "--no-pager", "--no-legend"]).await?;

        let mut services = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let status = parts[2].to_string();
                let active = status == "active";
                let enabled = parts.len() > 3 && parts[3] == "enabled";

                services.push(SystemdService {
                    name,
                    status,
                    active,
                    enabled,
                });
            }
        }

        Ok(serde_json::to_value(&services)?)
    });

    registry.register("Performance.StartService", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["systemctl", "start", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.StopService", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["systemctl", "stop", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Performance.RestartService", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        process::exec_command(&["systemctl", "restart", &name]).await?;
        Ok(serde_json::json!({ "success": true }))
    });
}

async fn get_cpu_frequency(core: usize) -> Result<u64> {
    let freq_path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
        core
    );
    let content = tokio::fs::read_to_string(&freq_path).await?;
    Ok(content.trim().parse::<u64>()? / 1000) // Convert kHz to MHz
}

async fn get_cpu_governor(core: usize) -> Result<String> {
    let gov_path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core
    );
    Ok(tokio::fs::read_to_string(&gov_path).await?.trim().to_string())
}

async fn get_gpu_stats() -> Result<Vec<GpuStats>> {
    let mut stats = Vec::new();

    // Try nvidia-smi first
    if let Ok(output) = process::exec_command(&["nvidia-smi", "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu", "--format=csv,noheader,nounits"]).await {
        for line in output.lines() {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                if let (Ok(util), Ok(mem_used), Ok(mem_total), Ok(temp)) = (
                    parts[1].parse::<f64>(),
                    parts[2].parse::<u64>(),
                    parts[3].parse::<u64>(),
                    parts[4].parse::<f64>(),
                ) {
                    stats.push(GpuStats {
                        name: parts[0].to_string(),
                        utilization_percent: util,
                        memory_used_mb: mem_used,
                        memory_total_mb: mem_total,
                        temperature_c: temp,
                    });
                }
            }
        }
    } else {
        // Fallback to /sys/class/drm
        let mut entries = tokio::fs::read_dir("/sys/class/drm").await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") && !name.contains("-") {
                stats.push(GpuStats {
                    name,
                    utilization_percent: 0.0,
                    memory_used_mb: 0,
                    memory_total_mb: 0,
                    temperature_c: 0.0,
                });
            }
        }
    }

    Ok(stats)
}

async fn get_disk_stats() -> Result<Vec<DiskStats>> {
    let output = process::exec_command(&["df", "-BG"]).await?;
    let mut stats = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let device = parts[0].to_string();
            let total_gb = parts[1]
                .trim_end_matches('G')
                .parse::<f64>()
                .unwrap_or(0.0);
            let used_gb = parts[2]
                .trim_end_matches('G')
                .parse::<f64>()
                .unwrap_or(0.0);
            let mount_point = parts[5].to_string();

            stats.push(DiskStats {
                device,
                mount_point,
                total_gb,
                used_gb,
                read_bytes_per_sec: 0, // Would need /proc/diskstats parsing
                write_bytes_per_sec: 0,
            });
        }
    }

    Ok(stats)
}

async fn get_network_stats() -> Result<Vec<NetworkStats>> {
    let output = process::exec_command(&["cat", "/proc/net/dev"]).await?;
    let mut stats = Vec::new();

    for line in output.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            let interface = parts[0].trim_end_matches(':').to_string();
            let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
            let tx_bytes: u64 = parts[9].parse().unwrap_or(0);

            stats.push(NetworkStats {
                interface,
                rx_bytes_per_sec: rx_bytes, // Would need to track previous values for per-second
                tx_bytes_per_sec: tx_bytes,
            });
        }
    }

    Ok(stats)
}
