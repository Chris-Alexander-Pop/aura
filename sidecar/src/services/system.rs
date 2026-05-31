use crate::services::ServiceRegistry;
use crate::types::SystemStats;
use crate::utils::process;
use anyhow::Result;
use serde_json;
use sysinfo::{CpuExt, System, SystemExt};
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

        let cpu_usage = calculate_cpu_usage().await;
        let cpu_usage = if cpu_usage > 0.0 {
            cpu_usage
        } else {
            cpu_usage_from_sysinfo(&sys)
        };

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

/// Average per-core usage from sysinfo (0.0–1.0) after `refresh_cpu`.
fn cpu_usage_from_sysinfo(sys: &System) -> f64 {
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return 0.0;
    }
    let sum: f64 = cpus.iter().map(|c| c.cpu_usage() as f64).sum();
    (sum / cpus.len() as f64 / 100.0).max(0.0).min(1.0)
}

async fn calculate_cpu_usage() -> f64 {
    if let Ok(content) = tokio::fs::read_to_string("/proc/stat").await {
        if let Some((total, idle)) = parse_proc_stat_cpu(&content) {
            let last_total = *LAST_CPU_TOTAL.read().await;
            let last_idle = *LAST_CPU_IDLE.read().await;

            if let Some(usage) = cpu_usage_from_samples(last_total, last_idle, total, idle) {
                *LAST_CPU_TOTAL.write().await = total;
                *LAST_CPU_IDLE.write().await = idle;
                return usage;
            }
            if last_total == 0 {
                *LAST_CPU_TOTAL.write().await = total;
                *LAST_CPU_IDLE.write().await = idle;
            }
        }
    }
    0.0
}

/// Parse aggregate `cpu` line from `/proc/stat` into (total jiffies, idle jiffies).
pub fn parse_proc_stat_cpu(content: &str) -> Option<(u64, u64)> {
    let line = content.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 8 {
        return None;
    }
    let stats: Vec<u64> = parts[1..8]
        .iter()
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let total: u64 = stats.iter().sum();
    let idle = stats[3] + stats.get(4).copied().unwrap_or(0);
    Some((total, idle))
}

/// CPU usage fraction from two `/proc/stat` samples; `None` on first sample or zero delta.
pub fn cpu_usage_from_samples(
    last_total: u64,
    last_idle: u64,
    total: u64,
    idle: u64,
) -> Option<f64> {
    if last_total == 0 {
        return None;
    }
    let total_diff = total.saturating_sub(last_total);
    let idle_diff = idle.saturating_sub(last_idle);
    if total_diff == 0 {
        return None;
    }
    let usage = 1.0 - (idle_diff as f64 / total_diff as f64);
    Some(usage.max(0.0).min(1.0))
}

async fn get_cpu_temp() -> Result<f64> {
    let output = process::exec_command(&["sensors"]).await?;
    if let Some(temp) = parse_sensors_cpu_temp(&output) {
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
    let nvidia_output = process::exec_command(&[
        "nvidia-smi",
        "--query-gpu=utilization.gpu",
        "--format=csv,noheader,nounits",
    ])
    .await;
    if let Ok(output) = nvidia_output {
        if let Some(usage) = parse_nvidia_gpu_utilization(&output) {
            return Ok(usage);
        }
    }

    if let Ok(content) = tokio::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent").await
    {
        if let Some(usage) = parse_sysfs_gpu_busy_percent(&content) {
            return Ok(usage);
        }
    }

    Ok(0.0)
}

async fn get_storage_usage() -> Result<f64> {
    let output = process::exec_command(&["df", "-B1"]).await?;
    Ok(parse_df_storage_usage(&output))
}

/// Extract CPU temperature (°C) from `sensors` text output.
pub fn parse_sensors_cpu_temp(output: &str) -> Option<f64> {
    let re = regex::Regex::new(r"(?:Package id \d+|Tdie|Tctl):\s*\+?([\d.]+)").ok()?;
    output.lines().find_map(|line| {
        re.captures(line)?
            .get(1)?
            .as_str()
            .parse::<f64>()
            .ok()
    })
}

/// Parse `nvidia-smi` utilization line as 0.0–1.0 fraction.
pub fn parse_nvidia_gpu_utilization(output: &str) -> Option<f64> {
    let line = output.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    let usage = line.parse::<f64>().ok()?;
    Some((usage / 100.0).max(0.0).min(1.0))
}

/// Parse `/sys/class/drm/.../gpu_busy_percent` as 0.0–1.0 fraction.
pub fn parse_sysfs_gpu_busy_percent(content: &str) -> Option<f64> {
    let usage = content.trim().parse::<f64>().ok()?;
    Some((usage / 100.0).max(0.0).min(1.0))
}

/// Aggregate block-device usage from `df -B1` output.
pub fn parse_df_storage_usage(output: &str) -> f64 {
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
        total_used as f64 / total as f64
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_proc_stat_cpu_fixture() {
        let text = include_str!("../../tests/fixtures/system/proc_stat.txt");
        let (total, idle) = parse_proc_stat_cpu(text).expect("cpu line");
        assert!(total > idle);
    }

    #[test]
    fn parse_proc_stat_cpu_rejects_short_line() {
        let text = include_str!("../../tests/fixtures/system/proc_stat_short.txt");
        assert!(parse_proc_stat_cpu(text).is_none());
    }

    #[test]
    fn cpu_usage_from_samples_computes_delta() {
        let usage = cpu_usage_from_samples(1000, 400, 1100, 420).expect("delta");
        assert!((usage - 0.8).abs() < f64::EPSILON);
        assert!(cpu_usage_from_samples(0, 0, 100, 50).is_none());
        assert!(cpu_usage_from_samples(100, 50, 100, 50).is_none());
    }

    #[test]
    fn parse_sensors_cpu_temp_fixture() {
        let text = include_str!("../../tests/fixtures/system/sensors_output.txt");
        let temp = parse_sensors_cpu_temp(text).expect("temp");
        assert!((temp - 52.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_df_storage_usage_fixture() {
        let text = include_str!("../../tests/fixtures/system/df_output.txt");
        let usage = parse_df_storage_usage(text);
        assert!((usage - 0.5).abs() < 0.01);
    }

    #[test]
    fn parse_nvidia_and_sysfs_gpu() {
        assert!((parse_nvidia_gpu_utilization("42\n").unwrap() - 0.42).abs() < f64::EPSILON);
        assert!(parse_nvidia_gpu_utilization("\n").is_none());
        assert!((parse_sysfs_gpu_busy_percent("80\n").unwrap() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn cpu_usage_from_sysinfo_empty_cpus() {
        let sys = System::new();
        assert_eq!(cpu_usage_from_sysinfo(&sys), 0.0);
    }
}
