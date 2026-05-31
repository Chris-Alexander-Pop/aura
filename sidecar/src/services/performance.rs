use crate::notify;
use crate::services::processes::{self, ProcessRow};
use crate::services::ServiceRegistry;
use crate::utils::{privileged, process};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::{CpuExt, PidExt, ProcessExt, System, SystemExt};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

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
    static ref LAST_METRICS: RwLock<Option<Value>> = RwLock::new(None);
}

static METRICS_EMIT_GEN: AtomicU64 = AtomicU64::new(0);

pub fn register(registry: &mut ServiceRegistry) {
    tokio::spawn(async {
        let mut tick = interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
            if let Ok(m) = collect_metrics_json().await {
                if metrics_changed(&m).await {
                    schedule_metrics_emit().await;
                }
                *LAST_METRICS.write().await = Some(m);
            }
        }
    });

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
        let (mem_cached_kb, mem_buffers_kb) = read_meminfo_cached_buffers_kb().await.unwrap_or((0, 0));
        let mem_cached = mem_cached_kb;
        let mem_buffers = mem_buffers_kb;
        let swap_total = sys.total_swap();
        let swap_used = sys.used_swap();

        let stats = MemoryStats {
            total_mb: mem_total / 1024 / 1024,
            used_mb: mem_used / 1024 / 1024,
            free_mb: mem_free / 1024 / 1024,
            cached_mb: mem_cached / 1024,
            buffers_mb: mem_buffers / 1024,
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
            let script = format!("echo '{}' > '{}'", governor.replace('\'', ""), governor_path);
            privileged::run_privileged(&["sh", "-c", &script]).await?;
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

    registry.register("Performance.GetProcesses", |params| async move {
        let limit = processes::list_top_limit_from_params(params.as_ref());
        let rows: Vec<ProcessInfo> = processes::collect_top_process_rows(limit)
            .into_iter()
            .map(process_row_to_info)
            .collect();
        Ok(serde_json::to_value(&rows)?)
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
        Ok(serde_json::to_value(&parse_systemd_service_lines(&output))?)
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

    registry.register("Performance.GetMetrics", |_params| async move {
        Ok(collect_metrics_json().await?)
    });
}

async fn collect_metrics_json() -> Result<Value> {
    let mut sys = SYSTEM.write().await;
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpu_count = sys.cpus().len().max(1);
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count as f32;

    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let memory_percent = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let disk_percent = get_disk_stats()
        .await
        .ok()
        .and_then(|disks| {
            disks.first().map(|d| {
                if d.total_gb > 0.0 {
                    (d.used_gb / d.total_gb) * 100.0
                } else {
                    0.0
                }
            })
        })
        .unwrap_or(0.0);

    let gpu_percent = get_gpu_stats()
        .await
        .ok()
        .and_then(|gpus| gpus.first().map(|g| g.utilization_percent))
        .unwrap_or(0.0);

    let temperature_c = read_cpu_temp_c().await.unwrap_or(0.0);

    Ok(json!({
        "cpu_percent": cpu_usage as f64,
        "memory_percent": memory_percent,
        "disk_percent": disk_percent,
        "gpu_percent": gpu_percent,
        "temperature_c": temperature_c,
    }))
}

/// Returns true when any tracked metric moved by at least `threshold` points.
pub(crate) fn metrics_delta_significant(prev: &Value, current: &Value, threshold: f64) -> bool {
    for key in ["cpu_percent", "memory_percent", "disk_percent"] {
        let prev_v = prev.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cur_v = current.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0);
        if (prev_v - cur_v).abs() >= threshold {
            return true;
        }
    }
    false
}

async fn metrics_changed(current: &Value) -> bool {
    let prev = LAST_METRICS.read().await;
    let Some(p) = prev.as_ref() else {
        return true;
    };
    metrics_delta_significant(p, current, 2.0)
}

#[cfg(test)]
pub(crate) async fn set_last_metrics_for_tests(value: Option<Value>) {
    *LAST_METRICS.write().await = value;
}

#[cfg(test)]
pub(crate) async fn metrics_changed_for_tests(current: &Value) -> bool {
    metrics_changed(current).await
}

#[cfg(test)]
pub(crate) async fn schedule_metrics_emit_for_tests() {
    schedule_metrics_emit().await;
}

async fn schedule_metrics_emit() {
    let gen = METRICS_EMIT_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        if METRICS_EMIT_GEN.load(Ordering::SeqCst) != gen {
            return;
        }
        notify::emit("Performance.MetricsChanged", json!({}));
    });
}

fn process_row_to_info(row: ProcessRow) -> ProcessInfo {
    ProcessInfo {
        pid: row.pid,
        name: row.name,
        cpu_percent: row.cpu_percent,
        memory_mb: row.memory_mb,
        status: row.status,
    }
}

/// Read `Cached` and `Buffers` from `/proc/meminfo` (kB).
pub(crate) fn parse_meminfo_cached_buffers_kb(output: &str) -> (u64, u64) {
    let mut cached_kb = 0u64;
    let mut buffers_kb = 0u64;
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("Cached:") {
            cached_kb = rest.trim().trim_end_matches(" kB").parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("Buffers:") {
            buffers_kb = rest.trim().trim_end_matches(" kB").parse().unwrap_or(0);
        }
    }
    (cached_kb, buffers_kb)
}

async fn read_meminfo_cached_buffers_kb() -> Result<(u64, u64)> {
    let content = tokio::fs::read_to_string("/proc/meminfo").await?;
    Ok(parse_meminfo_cached_buffers_kb(&content))
}

async fn read_cpu_temp_c() -> Result<f64> {
    if let Ok(content) = tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await {
        if let Ok(milli) = content.trim().parse::<f64>() {
            return Ok(milli / 1000.0);
        }
    }
    Ok(0.0)
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
    if let Ok(output) = process::exec_command(&[
        "nvidia-smi",
        "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu",
        "--format=csv,noheader,nounits",
    ])
    .await
    {
        let stats = parse_nvidia_smi_output(&output);
        if !stats.is_empty() {
            return Ok(stats);
        }
    }

    // Fallback to /sys/class/drm
    let mut stats = Vec::new();
    let mut entries = tokio::fs::read_dir("/sys/class/drm").await?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("card") && !name.contains('-') {
            stats.push(GpuStats {
                name,
                utilization_percent: 0.0,
                memory_used_mb: 0,
                memory_total_mb: 0,
                temperature_c: 0.0,
            });
        }
    }

    Ok(stats)
}

/// Parse `nvidia-smi` CSV output (`--format=csv,noheader,nounits`).
pub(crate) fn parse_nvidia_smi_output(output: &str) -> Vec<GpuStats> {
    let mut stats = Vec::new();
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
    stats
}

async fn get_disk_stats() -> Result<Vec<DiskStats>> {
    let output = process::exec_command(&["df", "-BG"]).await?;
    Ok(parse_df_bg_output(&output))
}

/// Parse `df -BG` output into disk usage rows.
pub(crate) fn parse_df_bg_output(output: &str) -> Vec<DiskStats> {
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
                read_bytes_per_sec: 0,
                write_bytes_per_sec: 0,
            });
        }
    }

    stats
}

async fn get_network_stats() -> Result<Vec<NetworkStats>> {
    let output = process::exec_command(&["cat", "/proc/net/dev"]).await?;
    Ok(parse_proc_net_dev(&output))
}

/// Parse `/proc/net/dev` lines into interface byte counters.
pub(crate) fn parse_proc_net_dev(output: &str) -> Vec<NetworkStats> {
    let mut stats = Vec::new();

    for line in output.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            let interface = parts[0].trim_end_matches(':').to_string();
            let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
            let tx_bytes: u64 = parts[9].parse().unwrap_or(0);

            stats.push(NetworkStats {
                interface,
                rx_bytes_per_sec: rx_bytes,
                tx_bytes_per_sec: tx_bytes,
            });
        }
    }

    stats
}

/// Parse `systemctl list-units --type=service --no-legend` rows.
pub(crate) fn parse_systemd_service_lines(output: &str) -> Vec<SystemdService> {
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
    services
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    #[test]
    fn metrics_delta_significant_detects_cpu_change() {
        let prev = json!({ "cpu_percent": 10.0, "memory_percent": 50.0, "disk_percent": 30.0 });
        let cur = json!({ "cpu_percent": 13.0, "memory_percent": 50.5, "disk_percent": 30.0 });
        assert!(metrics_delta_significant(&prev, &cur, 2.0));
    }

    #[test]
    fn metrics_delta_significant_ignores_small_drift() {
        let prev = json!({ "cpu_percent": 10.0, "memory_percent": 50.0, "disk_percent": 30.0 });
        let cur = json!({ "cpu_percent": 11.0, "memory_percent": 51.0, "disk_percent": 31.0 });
        assert!(!metrics_delta_significant(&prev, &cur, 2.0));
    }

    #[test]
    fn metrics_delta_significant_missing_keys_treated_as_zero() {
        let cur = json!({ "cpu_percent": 5.0, "memory_percent": 40.0, "disk_percent": 20.0 });
        assert!(metrics_delta_significant(&json!({}), &cur, 2.0));
    }

    #[test]
    fn metrics_delta_significant_detects_disk_change() {
        let prev = json!({ "cpu_percent": 10.0, "memory_percent": 50.0, "disk_percent": 30.0 });
        let cur = json!({ "cpu_percent": 10.5, "memory_percent": 50.5, "disk_percent": 33.0 });
        assert!(metrics_delta_significant(&prev, &cur, 2.0));
    }

    #[test]
    fn parse_nvidia_smi_fixture() {
        let text = include_str!("../../tests/fixtures/performance/nvidia_smi.csv");
        let gpus = parse_nvidia_smi_output(text);
        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].name, "NVIDIA GeForce RTX 3080");
        assert!((gpus[0].utilization_percent - 45.0).abs() < f64::EPSILON);
        assert_eq!(gpus[0].memory_used_mb, 2048);
        assert_eq!(gpus[0].memory_total_mb, 10240);
        assert!((gpus[0].temperature_c - 62.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_nvidia_smi_skips_malformed_lines() {
        assert!(parse_nvidia_smi_output("bad,row\n").is_empty());
    }

    #[test]
    fn parse_df_bg_fixture() {
        let text = include_str!("../../tests/fixtures/performance/df_bg.txt");
        let disks = parse_df_bg_output(text);
        assert_eq!(disks.len(), 3);
        assert_eq!(disks[0].mount_point, "/");
        assert!((disks[0].used_gb - 200.0).abs() < f64::EPSILON);
        assert_eq!(disks[1].device, "/dev/sda1");
    }

    #[tokio::test]
    async fn metrics_changed_first_sample_is_true() {
        set_last_metrics_for_tests(None).await;
        let cur = json!({ "cpu_percent": 1.0, "memory_percent": 1.0, "disk_percent": 1.0 });
        assert!(metrics_changed_for_tests(&cur).await);
    }

    #[tokio::test]
    async fn metrics_changed_small_drift_is_false() {
        let prev = json!({ "cpu_percent": 10.0, "memory_percent": 50.0, "disk_percent": 30.0 });
        set_last_metrics_for_tests(Some(prev)).await;
        let cur = json!({ "cpu_percent": 11.0, "memory_percent": 51.0, "disk_percent": 31.0 });
        assert!(!metrics_changed_for_tests(&cur).await);
    }

    #[test]
    fn parse_systemd_service_lines_fixture() {
        let text = include_str!("../../tests/fixtures/performance/systemctl_services.txt");
        let services = parse_systemd_service_lines(text);
        assert_eq!(services.len(), 3);
        assert!(services[0].active);
        assert!(!services[2].active);
    }

    #[test]
    fn parse_meminfo_cached_buffers_fixture() {
        let text = include_str!("../../tests/fixtures/performance/meminfo.txt");
        let (cached, buffers) = parse_meminfo_cached_buffers_kb(text);
        assert_eq!(cached, 4096000);
        assert_eq!(buffers, 512000);
    }

    #[test]
    fn parse_proc_net_dev_fixture() {
        let text = include_str!("../../tests/fixtures/performance/proc_net_dev.txt");
        let stats = parse_proc_net_dev(text);
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[1].interface, "wlan0");
        assert_eq!(stats[1].rx_bytes_per_sec, 987654321);
    }

    #[test]
    fn metrics_delta_significant_detects_memory_change() {
        let prev = json!({ "cpu_percent": 10.0, "memory_percent": 50.0, "disk_percent": 30.0 });
        let cur = json!({ "cpu_percent": 10.1, "memory_percent": 53.0, "disk_percent": 30.1 });
        assert!(metrics_delta_significant(&prev, &cur, 2.0));
    }

    #[tokio::test]
    async fn metrics_emit_debounce_coalesces() {
        static DEBOUNCE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = DEBOUNCE_LOCK.get_or_init(|| Mutex::new(())).lock().await;

        let tx = crate::notify::init_for_tests();
        let mut rx = tx.subscribe();
        while rx.try_recv().is_ok() {}

        schedule_metrics_emit_for_tests().await;
        schedule_metrics_emit_for_tests().await;
        tokio::time::sleep(Duration::from_millis(350)).await;

        let mut count = 0;
        while let Ok(raw) = rx.try_recv() {
            let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
            if v.get("method").and_then(|m| m.as_str()) == Some("Performance.MetricsChanged") {
                count += 1;
            }
        }
        assert!(count <= 1, "expected debounced metrics emit, got {count}");
    }

    #[tokio::test]
    async fn collect_metrics_json_shape() {
        let m = collect_metrics_json().await.expect("metrics");
        assert!(m.get("cpu_percent").and_then(|v| v.as_f64()).is_some());
        assert!(m.get("memory_percent").and_then(|v| v.as_f64()).is_some());
        assert!(m.get("disk_percent").and_then(|v| v.as_f64()).is_some());
        assert!(m.get("temperature_c").and_then(|v| v.as_f64()).is_some());
    }
}
