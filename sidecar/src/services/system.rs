use crate::services::ServiceRegistry;
use crate::types::SystemStats;
use crate::utils::process;
use anyhow::Result;
use serde_json;
use sysinfo::{System, SystemExt};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

lazy_static::lazy_static! {
    static ref SYSTEM: RwLock<System> = RwLock::new(System::new_all());
    static ref LAST_CPU_IDLE: RwLock<u64> = RwLock::new(0);
    static ref LAST_CPU_TOTAL: RwLock<u64> = RwLock::new(0);
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start stats polling task
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let mut sys = SYSTEM.write().await;
            sys.refresh_cpu();
            sys.refresh_memory();
        }
    });

    registry.register("System.GetStats", |_params| async move {
        let mut sys = SYSTEM.write().await;
        sys.refresh_cpu();
        sys.refresh_memory();

        // Calculate CPU usage
        let cpu_usage = calculate_cpu_usage().await;

        // Get RAM usage
        let mem_total = sys.total_memory();
        let mem_used = sys.used_memory();
        let ram_usage = if mem_total > 0 {
            mem_used as f64 / mem_total as f64
        } else {
            0.0
        };

        // Get CPU temperature
        let cpu_temp = get_cpu_temp().await.unwrap_or(0.0);

        // Get GPU stats (optional)
        let gpu_usage = get_gpu_usage().await.ok();

        // Get storage stats (optional)
        let storage_usage = get_storage_usage().await.ok();

        let stats = SystemStats {
            cpu: cpu_usage,
            ram: ram_usage,
            temp: cpu_temp,
            gpu: gpu_usage,
            storage: storage_usage,
        };

        Ok(serde_json::to_value(stats)?)
    });
}

async fn calculate_cpu_usage() -> f64 {
    // Read /proc/stat for more accurate CPU calculation
    if let Ok(content) = tokio::fs::read_to_string("/proc/stat").await {
        if let Some(line) = content.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                let stats: Vec<u64> = parts[1..8]
                    .iter()
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();
                let total: u64 = stats.iter().sum();
                let idle = stats[3] + stats.get(4).copied().unwrap_or(0);

                let last_total = *LAST_CPU_TOTAL.read().await;
                let last_idle = *LAST_CPU_IDLE.read().await;

                if last_total > 0 {
                    let total_diff = total - last_total;
                    let idle_diff = idle - last_idle;
                    if total_diff > 0 {
                        let usage = 1.0 - (idle_diff as f64 / total_diff as f64);
                        *LAST_CPU_TOTAL.write().await = total;
                        *LAST_CPU_IDLE.write().await = idle;
                        return usage.max(0.0).min(1.0);
                    }
                } else {
                    *LAST_CPU_TOTAL.write().await = total;
                    *LAST_CPU_IDLE.write().await = idle;
                }
            }
        }
    }
    0.0
}

async fn get_cpu_temp() -> Result<f64> {
    // Try sensors command first
    let output = process::exec_command(&["sensors"]).await?;
    
    // Look for Package id or Tdie (AMD) or Tctl
    let temp_match = output
        .lines()
        .find_map(|line| {
            if let Some(cap) = regex::Regex::new(r"(?:Package id \d+|Tdie|Tctl):\s*\+?([\d.]+)")
                .ok()
                .and_then(|re| re.captures(line))
            {
                cap.get(1)?.as_str().parse::<f64>().ok()
            } else {
                None
            }
        });

    if let Some(temp) = temp_match {
        return Ok(temp);
    }

    // Fallback: try /sys/class/thermal
    if let Ok(content) = tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await {
        if let Ok(temp_millidegrees) = content.trim().parse::<f64>() {
            return Ok(temp_millidegrees / 1000.0);
        }
    }

    Ok(0.0)
}

async fn get_gpu_usage() -> Result<f64> {
    // Try NVIDIA first
    let nvidia_output = process::exec_command(&["nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"]).await;
    if let Ok(output) = nvidia_output {
        if let Ok(usage) = output.trim().parse::<f64>() {
            return Ok(usage / 100.0);
        }
    }

    // Try generic GPU
    if let Ok(content) = tokio::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent").await {
        if let Ok(usage) = content.trim().parse::<f64>() {
            return Ok(usage / 100.0);
        }
    }

    Ok(0.0)
}

async fn get_storage_usage() -> Result<f64> {
    let output = process::exec_command(&["df", "-B1"]).await?;
    
    let mut total_used = 0u64;
    let mut total_avail = 0u64;

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[0].starts_with("/dev/") {
            if let (Ok(used), Ok(avail)) = (parts[2].parse::<u64>(), parts[3].parse::<u64>()) {
                total_used += used;
                total_avail += avail;
            }
        }
    }

    let total = total_used + total_avail;
    if total > 0 {
        Ok(total_used as f64 / total as f64)
    } else {
        Ok(0.0)
    }
}
