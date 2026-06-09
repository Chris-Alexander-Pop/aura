//! High-ROI exec-fixture sweeps for modules below the 85% coverage gate.
//!
//! Uses [`ExecFixtureGuard`] and `call_method_unchecked` for deny-listed mutators.

mod common;

use ags_sidecar::services::audio::audio_env_test_lock;
use ags_sidecar::services::notifications::{
    feed_dbus_monitor_fixture, parse_action_string_pairs, record_notification,
    reset_notifications_for_tests,
};
use common::{
    call_method, call_method_unchecked, ExecFixtureGuard, load_fixture, setup_temp_storage_db,
    test_registry,
};
use serde_json::json;
use std::sync::Mutex;

static NOTIFICATIONS_BOOST_LOCK: Mutex<()> = Mutex::new(());

fn notifications_lock() -> std::sync::MutexGuard<'static, ()> {
    NOTIFICATIONS_BOOST_LOCK.lock().unwrap()
}

#[tokio::test]
async fn audio_rpc_mutators_sweep_mocked() {
    let _lock = audio_env_test_lock();
    std::env::set_var("AURA_AUDIO_ADVANCED", "1");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(&registry, "Audio.Refresh", None)
        .await
        .expect("Refresh");

    for (method, params) in [
        (
            "Audio.SetSinkVolume",
            json!({ "device_id": 47, "volume": 0.42 }),
        ),
        (
            "Audio.SetSourceVolume",
            json!({ "device_id": 50, "volume": 0.8 }),
        ),
        (
            "Audio.SetSinkMute",
            json!({ "device_id": 47, "muted": true }),
        ),
        (
            "Audio.SetSourceMute",
            json!({ "device_id": 50, "muted": false }),
        ),
        (
            "Audio.SetStreamVolume",
            json!({ "stream_id": 8, "volume": 0.6 }),
        ),
        (
            "Audio.SetStreamMute",
            json!({ "stream_id": 8, "muted": false }),
        ),
        (
            "Audio.RouteStream",
            json!({ "stream_id": 8, "sink_id": 47 }),
        ),
        (
            "Audio.SetDefaultDevice",
            json!({ "device_id": 47, "type": "sink" }),
        ),
    ] {
        call_method_unchecked(&registry, method, Some(params))
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
    }

    call_method_unchecked(
        &registry,
        "Audio.CreateNullSink",
        Some(json!({ "name": "aura-test-null", "description": "Aura test" })),
    )
    .await
    .expect("CreateNullSink");

    call_method_unchecked(
        &registry,
        "Audio.CreateLoopback",
        Some(json!({ "source_name": "Built-in Microphone", "sink_name": "Headphones" })),
    )
    .await
    .expect("CreateLoopback");

    for method in [
        "Audio.Media.PlayPause",
        "Audio.Media.Next",
        "Audio.Media.Previous",
    ] {
        call_method_unchecked(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
    }

    call_method_unchecked(&registry, "Audio.Effects.Start", None)
        .await
        .expect("Effects.Start");
    call_method_unchecked(
        &registry,
        "Audio.Effects.LoadPreset",
        Some(json!({ "preset_name": "music" })),
    )
    .await
    .expect("LoadPreset");
    call_method_unchecked(
        &registry,
        "Audio.Effects.SavePreset",
        Some(json!({ "preset_name": "aura-test" })),
    )
    .await
    .expect("SavePreset");
    call_method_unchecked(
        &registry,
        "Audio.Effects.SetEqBand",
        Some(json!({ "band_index": 0, "gain_db": 1.5 })),
    )
    .await
    .expect("SetEqBand");
    call_method_unchecked(
        &registry,
        "Audio.Effects.SetNoiseSuppression",
        Some(json!({ "enabled": true })),
    )
    .await
    .expect("SetNoiseSuppression");
    call_method_unchecked(
        &registry,
        "Audio.Effects.SetMicGain",
        Some(json!({ "gain_db": 3.0 })),
    )
    .await
    .expect("SetMicGain");
    call_method_unchecked(
        &registry,
        "Audio.Effects.SetNoiseGate",
        Some(json!({ "enabled": true, "threshold_db": -40.0 })),
    )
    .await
    .expect("SetNoiseGate");
    call_method_unchecked(&registry, "Audio.Effects.Reset", None)
        .await
        .expect("Reset");
    call_method_unchecked(&registry, "Audio.Effects.Stop", None)
        .await
        .expect("Stop");

    let _db = setup_temp_storage_db().await;
    let profile_config = json!({
        "routing": { "defaultSink": 47 },
        "volumes": { "devices": [{ "id": 47, "volume": 0.5 }] },
        "effects": { "preset": "music" }
    });
    call_method_unchecked(
        &registry,
        "Audio.Profiles.Save",
        Some(json!({ "name": "coverage-profile", "config": profile_config })),
    )
    .await
    .expect("Profiles.Save");
    call_method_unchecked(
        &registry,
        "Audio.Profiles.Load",
        Some(json!({ "name": "coverage-profile" })),
    )
    .await
    .expect("Profiles.Load");
    call_method_unchecked(
        &registry,
        "Audio.Profiles.ApplyScenario",
        Some(json!({ "scenario": "streaming" })),
    )
    .await
    .expect("ApplyScenario");
    call_method_unchecked(
        &registry,
        "Audio.Profiles.Delete",
        Some(json!({ "name": "coverage-profile" })),
    )
    .await
    .expect("Profiles.Delete");

    std::env::remove_var("AURA_AUDIO_ADVANCED");
}

#[tokio::test]
async fn vpn_dry_run_connect_disconnect_mocked() {
    std::env::set_var("AURA_VPN_DRY_RUN", "1");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profile_id": "personal" })),
    )
    .await
    .expect("Vpn.Connect dry-run");

    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("GetStatus");
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("connected")
    );
    assert_eq!(
        status.get("local_ip").and_then(|v| v.as_str()),
        Some("192.168.1.5")
    );

    call_method_unchecked(&registry, "Vpn.Disconnect", None)
        .await
        .expect("Vpn.Disconnect dry-run");

    let after = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("GetStatus after disconnect");
    assert_eq!(
        after.get("state").and_then(|v| v.as_str()),
        Some("disconnected")
    );

    std::env::remove_var("AURA_VPN_DRY_RUN");
}

#[tokio::test]
async fn notifications_mutators_in_memory() {
    let _guard = notifications_lock();
    let _db = setup_temp_storage_db().await;
    reset_notifications_for_tests().await;

    record_notification(
        Some(100),
        "boost-app".into(),
        1,
        "Summary".into(),
        "Body".into(),
        None,
        1,
        parse_action_string_pairs(&["default".into(), "Open".into()]),
    )
    .await;

    let registry = test_registry();
    let listed = call_method(
        &registry,
        "Notifications.List",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("List");
    let id = listed[0]["id"].as_u64().expect("id");

    call_method_unchecked(
        &registry,
        "Notifications.InvokeAction",
        Some(json!({ "id": id, "action_key": "default" })),
    )
    .await
    .expect("InvokeAction");

    call_method_unchecked(
        &registry,
        "Notifications.Dismiss",
        Some(json!({ "id": id })),
    )
    .await
    .expect("Dismiss");

    call_method_unchecked(
        &registry,
        "Notifications.SetDnd",
        Some(json!({ "enabled": true, "until": null })),
    )
    .await
    .expect("SetDnd");

    let dnd = call_method(&registry, "Notifications.GetDnd", None)
        .await
        .expect("GetDnd");
    assert_eq!(dnd.get("enabled").and_then(|v| v.as_bool()), Some(true));

    call_method_unchecked(
        &registry,
        "Notifications.SetRules",
        Some(json!({ "muted_apps": ["Slack"] })),
    )
    .await
    .expect("SetRules");

    let rules = call_method(&registry, "Notifications.GetRules", None)
        .await
        .expect("GetRules");
    assert!(rules
        .get("muted_apps")
        .and_then(|v| v.as_array())
        .is_some());

    feed_dbus_monitor_fixture(&load_fixture("notifications/dbus_monitor_closed.txt")).await;

    call_method_unchecked(&registry, "Notifications.ClearAll", None)
        .await
        .expect("ClearAll");
}

#[tokio::test]
async fn devops_readonly_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for method in [
        "DevOps.GetDockerContainers",
        "DevOps.GetDockerImages",
        "DevOps.GetDockerStats",
        "DevOps.GetPodmanContainers",
        "DevOps.GetKubernetesPods",
        "DevOps.GetSystemdTimers",
        "DevOps.GetCronJobs",
    ] {
        let value = call_method(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(
            value.is_array()
                || value.get("pods").is_some()
                || value.get("stats").is_some(),
            "{method} unexpected shape"
        );
    }

    let repos = call_method(
        &registry,
        "DevOps.GetGitRepos",
        Some(json!({ "path": env!("CARGO_MANIFEST_DIR") })),
    )
    .await
    .expect("GetGitRepos");
    assert!(repos.is_array());
}

#[tokio::test]
async fn performance_gpu_and_network_stats_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    let gpu = call_method(&registry, "Performance.GetGpuStats", None)
        .await
        .expect("GetGpuStats");
    let gpu_rows = gpu.as_array().expect("gpu array");
    assert!(!gpu_rows.is_empty());
    assert!(gpu_rows[0].get("name").is_some());

    let net = call_method(&registry, "Performance.GetNetworkStats", None)
        .await
        .expect("GetNetworkStats");
    let net_rows = net.as_array().expect("net array");
    assert!(net_rows.iter().any(|r| r.get("interface").and_then(|v| v.as_str()) == Some("wlan0")));
}

#[tokio::test]
async fn security_readonly_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for method in [
        "Security.GetStatus",
        "Security.GetFirewallStatus",
        "Security.GetFirewallRules",
        "Security.GetSshStatus",
        "Security.GetSshConnections",
        "Security.GetFailedLogins",
        "Security.GetSudoLogs",
        "Security.GetEncryptionStatus",
        "Security.GetKeyringStatus",
        "Security.GetVpnConnections",
        "Security.ListFingerprints",
        "Security.GetPasswordPolicy",
    ] {
        let value = call_method(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(value.is_object() || value.is_array(), "{method} shape");
    }

    let ports = call_method_unchecked(&registry, "Security.ScanPorts", None)
        .await
        .expect("ScanPorts");
    assert!(ports.is_array());
}

#[tokio::test]
async fn automation_workflow_notify_send_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Notify flow",
            "actions": [{"type": "send_notification", "title": "Coverage", "body": "ok"}]
        })),
    )
    .await
    .expect("CreateWorkflow");
    let id = created.get("id").and_then(|v| v.as_str()).expect("id");

    call_method_unchecked(
        &registry,
        "Automation.RunWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .expect("RunWorkflow");
}

#[tokio::test]
async fn productivity_mutators_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let timer = call_method_unchecked(
        &registry,
        "Productivity.CreateTimer",
        Some(json!({ "duration_seconds": 7200, "name": "Focus block" })),
    )
    .await
    .expect("CreateTimer");
    let timer_id = timer.get("id").and_then(|v| v.as_str()).expect("timer id");

    call_method_unchecked(
        &registry,
        "Productivity.SetFocusMode",
        Some(json!({ "enabled": true })),
    )
    .await
    .expect("SetFocusMode");

    let task = call_method_unchecked(
        &registry,
        "Productivity.CreateTask",
        Some(json!({ "title": "Write tests", "description": "coverage" })),
    )
    .await
    .expect("CreateTask");
    let task_id = task.get("id").and_then(|v| v.as_str()).expect("task id");

    call_method_unchecked(
        &registry,
        "Productivity.UpdateTask",
        Some(json!({ "task_id": task_id, "updates": { "completed": true } })),
    )
    .await
    .expect("UpdateTask");

    call_method_unchecked(
        &registry,
        "Productivity.CreatePomodoro",
        Some(json!({ "work_minutes": 25, "break_minutes": 5 })),
    )
    .await
    .expect("CreatePomodoro");

    call_method_unchecked(
        &registry,
        "Productivity.CancelTimer",
        Some(json!({ "timer_id": timer_id })),
    )
    .await
    .expect("CancelTimer");

    call_method_unchecked(
        &registry,
        "Productivity.DeleteTask",
        Some(json!({ "task_id": task_id })),
    )
    .await
    .expect("DeleteTask");
}

#[tokio::test]
async fn packages_readonly_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for method in [
        "Packages.GetInstalled",
        "Packages.GetUpgradable",
        "Packages.Search",
        "Packages.GetPackageInfo",
        "Packages.GetPackageDependencies",
        "Packages.GetReverseDependencies",
    ] {
        let params = match method {
            "Packages.Search" => Some(json!({ "query": "linux" })),
            "Packages.GetPackageInfo" => Some(json!({ "name": "pacman" })),
            "Packages.GetPackageDependencies" | "Packages.GetReverseDependencies" => {
                Some(json!({ "name": "pacman" }))
            }
            _ => None,
        };
        let value = call_method(&registry, method, params)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(value.is_array() || value.is_object(), "{method} shape");
    }
}
