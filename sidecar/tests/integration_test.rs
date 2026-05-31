//! P0 smoke, storage round-trips, and readonly gap sweeps (fast + `#[ignore]` slow host batch).
//!
//! Per-service JSON shape tests live in `tests/*_rpc_shapes.rs` and `tests/*_storage_test.rs`.

mod common;

use common::{call_method, call_rpc, test_registry};
use common::setup_temp_storage_db;
use serde_json::{json, Value};

const P0_METHODS: &[&str] = &[
    "Power.GetBatteryState",
    "Power.GetProfile",
    "Network.GetStatus",
    "Network.ScanNetworks",
    "Network.ListSaved",
    "Bluetooth.GetAdapters",
    "Bluetooth.GetDevices",
    "Audio.GetDevices",
    "Audio.GetStreams",
    "Packages.GetUpgradable",
    "Packages.GetTransactionHistory",
    "Logs.Get",
    "Security.GetStatus",
    "Performance.GetMetrics",
    "DevOps.GetStatus",
    "Productivity.GetStats",
    "Automation.GetWorkflows",
    "Communication.GetUnread",
    "GameMode.IsEnabled",
    "System.GetStats",
    "Sidecar.GetVersion",
];

#[tokio::test]
async fn p0_methods_resolve_without_panic() {
    let registry = test_registry();
    for method in P0_METHODS {
        let result = call_method(&registry, method, None).await;
        assert!(
            result.is_ok(),
            "method {} failed: {:?}",
            method,
            result.err()
        );
    }
}

#[tokio::test]
async fn logs_get_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Logs.Get", None).await.expect("Logs.Get");
    assert!(value.is_array(), "Logs.Get should return a JSON array");
}

#[tokio::test]
async fn devops_get_status_has_podman_field() {
    let registry = test_registry();
    let value = call_method(&registry, "DevOps.GetStatus", None)
        .await
        .expect("DevOps.GetStatus");
    assert!(value.get("podman_available").is_some());
}

#[tokio::test]
async fn communication_get_unread_returns_object() {
    let registry = test_registry();
    let value = call_method(&registry, "Communication.GetUnread", None)
        .await
        .expect("Communication.GetUnread");
    assert!(value.is_object());
}

#[tokio::test]
async fn storage_scan_namespace_round_trip() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let ns = format!("test_ns_{}", std::process::id());

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "a", "value": { "n": 1 } })),
    )
    .await
    .expect("Storage.Set a");

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "b", "value": { "n": 2 } })),
    )
    .await
    .expect("Storage.Set b");

    let scan = call_method(
        &registry,
        "Storage.ScanNamespace",
        Some(json!({ "namespace": ns })),
    )
    .await
    .expect("Storage.ScanNamespace");

    let items = scan.get("items").and_then(|v| v.as_array()).expect("items array");
    assert_eq!(items.len(), 2);

    call_method(
        &registry,
        "Storage.Delete",
        Some(json!({ "namespace": ns, "key": "a" })),
    )
    .await
    .expect("Storage.Delete");

    let keys = call_method(
        &registry,
        "Storage.ListKeys",
        Some(json!({ "namespace": ns })),
    )
    .await
    .expect("Storage.ListKeys");

    let key_list = keys.get("keys").and_then(|v| v.as_array()).expect("keys");
    assert_eq!(key_list.len(), 1);
}

#[tokio::test]
async fn network_get_status_has_extended_fields() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.GetStatus", None)
        .await
        .expect("Network.GetStatus");
    assert!(value.get("wifi_enabled").is_some());
    assert!(value.get("connection_type").is_some());
    assert!(value.get("ethernet_connected").is_some());
}

#[tokio::test]
async fn power_get_battery_state_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Power.GetBatteryState", None)
        .await
        .expect("Power.GetBatteryState");
    assert!(value.get("percent").is_some());
    assert!(value.get("charging").is_some());
    assert!(value.get("time_remaining").is_some());
}

#[tokio::test]
async fn bluetooth_get_adapters_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Bluetooth.GetAdapters", None)
        .await
        .expect("Bluetooth.GetAdapters");
    assert!(value.is_array());
}

#[tokio::test]
async fn audio_get_devices_has_sinks_and_sources() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.GetDevices", None)
        .await
        .expect("Audio.GetDevices");
    assert!(value.get("sinks").is_some());
    assert!(value.get("sources").is_some());
}

#[tokio::test]
async fn audio_get_streams_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Audio.GetStreams", None)
        .await
        .expect("Audio.GetStreams");
    assert!(value.is_array());
}

#[tokio::test]
async fn packages_get_upgradable_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Packages.GetUpgradable", None)
        .await
        .expect("Packages.GetUpgradable");
    assert!(value.is_array());
}

#[tokio::test]
async fn packages_get_installed_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Packages.GetInstalled", None)
        .await
        .expect("Packages.GetInstalled");
    assert!(value.is_array());
}

#[tokio::test]
async fn packages_search_readonly_with_query() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Packages.Search",
        Some(json!({ "query": "linux" })),
    )
    .await
    .expect("Packages.Search");
    assert!(value.is_array());
}

#[tokio::test]
async fn packages_get_transaction_history_returns_array() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Packages.GetTransactionHistory",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("Packages.GetTransactionHistory");
    assert!(value.is_array());
}

#[tokio::test]
async fn notify_emit_does_not_panic() {
    ags_sidecar::notify::init_for_tests();
    ags_sidecar::notify::emit("Test.Ping", serde_json::json!({ "ok": true }));
}

#[tokio::test]
async fn notifications_list_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Notifications.List", Some(json!({ "limit": 10 })))
        .await
        .expect("Notifications.List");
    assert!(value.is_array());
}

#[tokio::test]
async fn notifications_get_dnd_returns_object() {
    let registry = test_registry();
    let value = call_method(&registry, "Notifications.GetDnd", None)
        .await
        .expect("Notifications.GetDnd");
    assert!(value.get("schedule_enabled").is_some());
}

#[tokio::test]
async fn keybinds_list_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Keybinds.List", None)
        .await
        .expect("Keybinds.List");
    assert!(value.is_array());
}

#[tokio::test]
async fn security_get_status_extended_fields() {
    let registry = test_registry();
    let value = call_method(&registry, "Security.GetStatus", None)
        .await
        .expect("Security.GetStatus");
    assert!(value.get("fail2ban_active").is_some());
    assert!(value.get("clamav_installed").is_some());
    assert!(value.get("fprintd_available").is_some());
    assert!(value.get("firewall_enabled").is_some());
    assert!(value.get("ssh_enabled").is_some());
    assert!(value.get("encryption_enabled").is_some());
}

#[tokio::test]
async fn keybinds_validate_returns_validation_object() {
    let registry = test_registry();
    let value = call_method(&registry, "Keybinds.Validate", None)
        .await
        .expect("Keybinds.Validate");
    assert!(value.get("duplicates").and_then(|v| v.as_array()).is_some());
    assert!(value
        .get("unknown_dispatches")
        .and_then(|v| v.as_array())
        .is_some());
}

#[tokio::test]
async fn logs_get_respects_small_line_limit() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Logs.Get",
        Some(json!({ "lines": 3 })),
    )
    .await
    .expect("Logs.Get");
    let entries = value.as_array().expect("array");
    assert!(entries.len() <= 3);
    if let Some(first) = entries.first() {
        assert!(first.get("message").is_some());
        assert!(first.get("level").is_some());
        assert!(first.get("timestamp").is_some());
    }
}

#[tokio::test]
async fn logs_get_priority_and_grep_filters() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Logs.Get",
        Some(json!({ "lines": 5, "priority": "err", "grep": "kernel|systemd" })),
    )
    .await
    .expect("Logs.Get");
    assert!(value.is_array());
}

#[tokio::test]
async fn logs_get_unit_filter_small_limit() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Logs.Get",
        Some(json!({ "lines": 2, "unit": "systemd-journald.service", "priority": "info" })),
    )
    .await
    .expect("Logs.Get");
    let entries = value.as_array().expect("array");
    assert!(entries.len() <= 2);
}

#[tokio::test]
async fn logs_get_warn_priority_alias() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Logs.Get",
        Some(json!({ "lines": 4, "priority": "warning" })),
    )
    .await
    .expect("Logs.Get");
    assert!(value.is_array());
}

#[tokio::test]
async fn performance_get_metrics_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Performance.GetMetrics", None)
        .await
        .expect("Performance.GetMetrics");
    for key in [
        "cpu_percent",
        "memory_percent",
        "disk_percent",
        "gpu_percent",
        "temperature_c",
    ] {
        assert!(
            value.get(key).and_then(|v| v.as_f64()).is_some(),
            "missing {key}"
        );
    }
}

/// Bulk smoke: read-only RPCs across shell, compositor, media, and control-center services.
const READONLY_BULK_SMOKE_METHODS: &[&str] = &[
    "Sidecar.GetVersion",
    "Packages.GetInstalled",
    "Brightness.Get",
    "Vpn.GetStatus",
    "Vpn.GetProfiles",
    "GameMode.IsEnabled",
    "Calendar.GetEvents",
    "Calendar.GetCalendars",
    "Automation.GetWorkflows",
    "Automation.GetTriggers",
    "Automation.GetActions",
    "Productivity.GetStats",
    "Productivity.GetTimers",
    "Productivity.GetFocusModeStatus",
    "Fitness.GetGoals",
    "Fitness.GetActivity",
    "Hyprland.GetWorkspaces",
    "Hyprland.GetActiveWorkspace",
    "Hyprland.GetClients",
    "Hyprland.GetActiveWindow",
    "Hyprland.GetMonitors",
    "Media.GetNowPlaying",
    "Process.ListTop",
    "Communication.GetNotificationSettings",
    "Weather.GetLocations",
    "Weather.GetAlerts",
    "Settings.Get",
    "Settings.GetSchema",
    "Capture.ListDevices",
    "Automation.ListRules",
    "Notifications.List",
    "Notifications.GetDnd",
    "Audio.Media.GetPlayers",
    "Productivity.GetTasks",
    "DevOps.GetDockerContainers",
];

#[tokio::test]
async fn readonly_bulk_smoke_methods_resolve() {
    let registry = test_registry();
    for method in READONLY_BULK_SMOKE_METHODS {
        let result = call_method(&registry, method, None).await;
        assert!(
            result.is_ok(),
            "method {} failed: {:?}",
            method,
            result.err()
        );
    }
}

#[tokio::test]
async fn sidecar_get_version_has_version_field() {
    let registry = test_registry();
    let value = call_method(&registry, "Sidecar.GetVersion", None)
        .await
        .expect("Sidecar.GetVersion");
    assert!(value.get("version").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn brightness_get_returns_brightness_field() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Brightness.Get",
        Some(json!({ "monitor": "active" })),
    )
    .await
    .expect("Brightness.Get");
    assert!(value.get("brightness").is_some());
}

#[tokio::test]
async fn vpn_get_status_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    assert!(value.get("state").is_some());
    assert!(value.get("message").is_some());
}

#[tokio::test]
async fn vpn_get_profiles_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Vpn.GetProfiles", None)
        .await
        .expect("Vpn.GetProfiles");
    assert!(value.is_array());
    assert!(!value.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn gamemode_is_enabled_returns_bool() {
    let registry = test_registry();
    let value = call_method(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert!(value.get("enabled").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn process_list_top_returns_array_with_limit() {
    let registry = test_registry();
    let value = call_method(
        &registry,
        "Process.ListTop",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("Process.ListTop");
    let rows = value.as_array().expect("array");
    assert!(!rows.is_empty());
    assert!(rows.len() <= 5);
    assert!(rows[0].get("pid").is_some());
}

#[tokio::test]
async fn media_get_now_playing_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Media.GetNowPlaying", None)
        .await
        .expect("Media.GetNowPlaying");
    assert!(value.get("playing").is_some());
    assert!(value.get("title").is_some());
    assert!(value.get("artist").is_some());
}

#[tokio::test]
async fn productivity_get_stats_shape() {
    let registry = test_registry();
    let value = call_method(&registry, "Productivity.GetStats", None)
        .await
        .expect("Productivity.GetStats");
    assert!(value.get("active_timers").is_some());
    assert!(value.get("focus_mode_enabled").is_some());
}

#[tokio::test]
async fn automation_get_triggers_is_string_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Automation.GetTriggers", None)
        .await
        .expect("Automation.GetTriggers");
    let arr = value.as_array().expect("array");
    assert!(arr.iter().all(|v| v.as_str().is_some()));
}

#[tokio::test]
async fn calendar_get_events_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Calendar.GetEvents", None)
        .await
        .expect("Calendar.GetEvents");
    assert!(value.is_array());
}

#[tokio::test]
async fn hyprland_get_workspaces_does_not_panic() {
    let registry = test_registry();
    let _ = call_method(&registry, "Hyprland.GetWorkspaces", None)
        .await
        .expect("Hyprland.GetWorkspaces");
}

#[tokio::test]
async fn storage_get_round_trip_and_missing_null() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let ns = format!("it_get_{}", std::process::id());

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "prefs", "value": { "theme": "dark" } })),
    )
    .await
    .expect("Storage.Set");

    let got = call_method(
        &registry,
        "Storage.Get",
        Some(json!({ "namespace": ns, "key": "prefs" })),
    )
    .await
    .expect("Storage.Get");
    assert_eq!(got.get("value").and_then(|v| v.get("theme")), Some(&json!("dark")));

    let missing = call_method(
        &registry,
        "Storage.Get",
        Some(json!({ "namespace": ns, "key": "nope" })),
    )
    .await
    .expect("Storage.Get missing");
    assert!(missing.get("value").map(|v| v.is_null()).unwrap_or(false));
}

#[tokio::test]
async fn storage_delete_reports_deleted_flag() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let ns = format!("it_del_{}", std::process::id());

    call_method(
        &registry,
        "Storage.Set",
        Some(json!({ "namespace": ns, "key": "x", "value": 1 })),
    )
    .await
    .expect("Storage.Set");

    let deleted = call_method(
        &registry,
        "Storage.Delete",
        Some(json!({ "namespace": ns, "key": "x" })),
    )
    .await
    .expect("Storage.Delete");
    assert_eq!(deleted.get("deleted"), Some(&json!(true)));

    let again = call_method(
        &registry,
        "Storage.Delete",
        Some(json!({ "namespace": ns, "key": "x" })),
    )
    .await
    .expect("Storage.Delete again");
    assert_eq!(again.get("deleted"), Some(&json!(false)));
}

#[tokio::test]
async fn network_list_saved_returns_array() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.ListSaved", None)
        .await
        .expect("Network.ListSaved");
    let arr = value.as_array().expect("array of saved networks");
    for item in arr {
        let obj = item.as_object().expect("saved network object");
        assert!(obj.get("name").is_some());
        assert!(obj.get("uuid").is_some());
        assert!(obj.get("autoconnect").is_some());
    }
}

#[tokio::test]
async fn network_scan_networks_returns_access_points() {
    let registry = test_registry();
    let value = call_method(&registry, "Network.ScanNetworks", None)
        .await
        .expect("Network.ScanNetworks");
    let arr = value.as_array().expect("array of access points");
    for item in arr {
        let obj = item.as_object().expect("access point object");
        assert!(obj.get("ssid").is_some());
        assert!(obj.get("strength").is_some());
        assert!(obj.get("security").is_some());
    }
}

#[tokio::test]
async fn power_get_profile_returns_profile_string() {
    let registry = test_registry();
    let value = call_method(&registry, "Power.GetProfile", None)
        .await
        .expect("Power.GetProfile");
    let profile = value
        .get("profile")
        .and_then(|v| v.as_str())
        .expect("profile string");
    assert!(
        matches!(profile, "balanced" | "performance" | "saver"),
        "unexpected profile: {profile}"
    );
}

/// Default integration sweep: static params, no chained host discovery (see `readonly_gap_methods_resolve_slow_host`).
const READONLY_GAP_FAST: &[&str] = &[
    "Audio.Effects.GetPresets",
    "Audio.Effects.GetStatus",
    "Audio.Media.GetPlayers",
    "Audio.Profiles.GetCurrent",
    "Audio.Profiles.List",
    "Automation.GetScripts",
    "Automation.GetWorkflowHistory",
    "Calendar.ExportIcs",
    "Calendar.GetUpcomingEvents",
    "Keybinds.GetCategories",
    "Notifications.Get",
    "Notifications.GetRules",
    "Productivity.GetAppUsage",
    "Productivity.GetPomodoroStatus",
    "Productivity.GetScreenTime",
    "Productivity.GetTasks",
];

/// Optional host sweep (~5 min): subprocess-heavy CLI, HTTP, docker/k8s, journalctl, pacman, security probes.
const READONLY_GAP_SLOW_HOST: &[&str] = &[
    "Bluetooth.GetDeviceInfo",
    "Bluetooth.Scan",
    "Communication.GetActiveCalls",
    "Communication.GetCommunicationApps",
    "Communication.GetContacts",
    "Communication.GetConversations",
    "Communication.GetMessages",
    "DevOps.GetContainerLogs",
    "DevOps.GetCronJobs",
    "DevOps.GetDockerContainers",
    "DevOps.GetDockerImages",
    "DevOps.GetDockerStats",
    "DevOps.GetGitRepos",
    "DevOps.GetGitStatus",
    "DevOps.GetKubernetesPods",
    "DevOps.GetPodmanContainers",
    "DevOps.GetSystemdTimers",
    "Fitness.GetDevices",
    "Fitness.GetHeartRate",
    "Fitness.GetSleep",
    "Fitness.GetSteps",
    "Fitness.GetWorkoutHistory",
    "Logs.FilterLogs",
    "Logs.GetApplicationLogs",
    "Logs.GetLogLevels",
    "Logs.GetLogServices",
    "Logs.GetSystemLogs",
    "Logs.SearchLogs",
    "Packages.GetAurPackages",
    "Packages.GetFlatpakPackages",
    "Packages.GetInstalled",
    "Packages.GetPackageDependencies",
    "Packages.GetPackageFiles",
    "Packages.GetPackageInfo",
    "Packages.GetSnapPackages",
    "Packages.Search",
    "Performance.GetCpuStats",
    "Performance.GetDiskStats",
    "Performance.GetGpuStats",
    "Performance.GetMemoryStats",
    "Performance.GetNetworkStats",
    "Performance.GetProcesses",
    "Performance.GetSystemdServices",
    "Security.GetCertificates",
    "Security.GetEncryptionStatus",
    "Security.GetFailedLogins",
    "Security.GetFirewallRules",
    "Security.GetFirewallStatus",
    "Security.GetKeyringStatus",
    "Security.GetSshConnections",
    "Security.GetSshStatus",
    "Security.GetSudoLogs",
    "Security.GetVpnConnections",
    "Weather.Get",
    "Weather.GetForecast",
    "Weather.GetHourly",
];

fn readonly_rpc_params(method: &str) -> Option<Value> {
    match method {
        "Packages.Search" => Some(json!({ "query": "linux" })),
        "Packages.GetPackageInfo" | "Packages.GetPackageFiles" | "Packages.GetPackageDependencies" => {
            Some(json!({ "name": "pacman" }))
        }
        "Logs.GetApplicationLogs" => Some(json!({ "app_name": "systemd", "lines": 5 })),
        "Logs.SearchLogs" => Some(json!({ "query": "Started", "lines": 5 })),
        "Logs.FilterLogs" | "Logs.GetSystemLogs" => Some(json!({ "lines": 5 })),
        "Calendar.ExportIcs" => Some(json!({ "calendar_id": "default", "file_path": "/tmp/aura-test.ics" })),
        "Calendar.GetUpcomingEvents" => Some(json!({ "days": 7 })),
        "Automation.GetWorkflowHistory" => Some(json!({ "workflow_id": "fixture" })),
        "Notifications.Get" => Some(json!({ "id": 1 })),
        "DevOps.GetContainerLogs" => Some(json!({ "name": "__aura_test_no_container__", "lines": 1 })),
        "DevOps.GetGitStatus" => Some(json!({ "repo_path": env!("CARGO_MANIFEST_DIR") })),
        "DevOps.GetGitRepos" => Some(json!({ "path": env!("CARGO_MANIFEST_DIR") })),
        _ => None,
    }
}

async fn resolve_readonly_params(registry: &ags_sidecar::services::ServiceRegistry, method: &str) -> Option<Value> {
    if let Some(p) = readonly_rpc_params(method) {
        return Some(p);
    }
    match method {
        "Bluetooth.GetDeviceInfo" => {
            let devices = call_rpc(registry, "Bluetooth.GetDevices", None).await.ok()?;
            let addr = devices
                .as_array()?
                .first()?
                .get("address")?
                .as_str()?
                .to_string();
            Some(json!({ "device_address": addr }))
        }
        "DevOps.GetContainerLogs" => {
            let containers = call_rpc(registry, "DevOps.GetDockerContainers", None).await.ok()?;
            let name = containers
                .as_array()?
                .first()?
                .get("name")
                .or_else(|| containers.as_array()?.first()?.get("Names"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| "__aura_test_no_container__".into());
            Some(json!({ "name": name, "lines": 1 }))
        }
        _ => None,
    }
}

fn readonly_gap_optional_skip(method: &str, result: &Result<Value, anyhow::Error>) -> bool {
    matches!(
        method,
        "DevOps.GetContainerLogs"
            | "DevOps.GetKubernetesPods"
            | "DevOps.GetPodmanContainers"
            | "Bluetooth.GetDeviceInfo"
            | "Bluetooth.Scan"
            | "Weather.Get"
            | "Weather.GetForecast"
            | "Weather.GetHourly"
    ) || result
        .as_ref()
        .err()
        .map(|e| e.to_string().contains("Command failed"))
        .unwrap_or(false)
}

async fn assert_readonly_gap_methods_resolve(methods: &[&str]) {
    let registry = test_registry();
    for method in methods {
        let params = resolve_readonly_params(&registry, method).await;
        let result = call_rpc(&registry, method, params).await;
        if readonly_gap_optional_skip(method, &result) {
            continue;
        }
        assert!(
            result.is_ok(),
            "method {} failed: {:?}",
            method,
            result.err()
        );
    }
}

#[tokio::test]
async fn readonly_gap_methods_resolve() {
    assert_readonly_gap_methods_resolve(READONLY_GAP_FAST).await;
}

#[tokio::test]
#[ignore = "slow host sweep (~5 min); cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored"]
async fn readonly_gap_methods_resolve_slow_host() {
    assert_readonly_gap_methods_resolve(READONLY_GAP_SLOW_HOST).await;
}

