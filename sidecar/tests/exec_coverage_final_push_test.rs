//! Final 85% gate push: performance/keybinds/shell/productivity/automation exec fixtures.

mod common;

use ags_sidecar::services::productivity::reset_productivity_state_for_tests;
use ags_sidecar::services::weather::reset_weather_cache_for_tests;
use common::{
    automation_test_lock, call_method, call_method_unchecked, ExecFixtureGuard, performance_test_lock,
    setup_temp_storage_db, test_registry, weather_test_lock,
};
use serde_json::json;
use std::sync::Mutex;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

static PRODUCTIVITY_ASYNC_LOCK: Mutex<()> = Mutex::new(());

fn keybinds_temp_config() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let hypr_conf = dir.path().join("hyprland.conf");
    std::fs::write(
        &hypr_conf,
        r#"source = extra.conf
bind = SUPER, Q, killactive
"#,
    )
    .expect("hyprland.conf");
    std::fs::write(
        dir.path().join("extra.conf"),
        "bind = SUPER, X, exec foot\n",
    )
    .expect("extra.conf");
    let aura_binds = dir.path().join("aura-binds.conf");
    std::env::set_var("HYPRLAND_CONFIG", &hypr_conf);
    std::env::set_var("AURA_KEYBINDS_PATH", &aura_binds);
    (dir, hypr_conf, aura_binds)
}

#[tokio::test]
async fn performance_apply_preset_and_governor_with_fixtures() {
    let _lock = performance_test_lock();
    std::env::remove_var("AURA_PERFORMANCE_DRY_RUN");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    let preset = call_method_unchecked(
        &registry,
        "Performance.ApplyPreset",
        Some(json!({ "preset": "meeting" })),
    )
    .await
    .expect("ApplyPreset meeting");
    assert_eq!(preset.get("success").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        preset.get("governor").and_then(|v| v.as_str()),
        Some("powersave")
    );

    call_method_unchecked(
        &registry,
        "Performance.SetCpuGovernor",
        Some(json!({ "governor": "schedutil" })),
    )
    .await
    .expect("SetCpuGovernor");

    call_method_unchecked(
        &registry,
        "Performance.SetCpuFrequency",
        Some(json!({ "min_mhz": 800, "max_mhz": 3200 })),
    )
    .await
    .expect("SetCpuFrequency");
}

#[tokio::test]
async fn performance_process_and_systemd_mutators_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Performance.KillProcess",
        Some(json!({ "pid": 4242 })),
    )
    .await
    .expect("KillProcess");

    call_method_unchecked(
        &registry,
        "Performance.SetProcessPriority",
        Some(json!({ "pid": 4242, "priority": 5 })),
    )
    .await
    .expect("SetProcessPriority");

    for (method, name) in [
        ("Performance.StartService", "sshd.service"),
        ("Performance.StopService", "sshd.service"),
        ("Performance.RestartService", "nginx.service"),
    ] {
        call_method_unchecked(&registry, method, Some(json!({ "name": name })))
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
    }

    let services = call_method(&registry, "Performance.GetSystemdServices", None)
        .await
        .expect("GetSystemdServices");
    let rows = services.as_array().expect("services array");
    assert!(rows.iter().any(|r| r.get("name").and_then(|v| v.as_str()) == Some("sshd.service")));
}

#[tokio::test]
async fn keybinds_mutators_with_temp_config_and_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let (_dir, _hypr, aura_binds) = keybinds_temp_config();
    let registry = test_registry();

    let listed = call_method(&registry, "Keybinds.List", None)
        .await
        .expect("List");
    assert!(listed.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let workspace = call_method(
        &registry,
        "Keybinds.List",
        Some(json!({ "category": "window" })),
    )
    .await
    .expect("List window category");
    assert!(workspace.is_array());

    let categories = call_method(&registry, "Keybinds.GetCategories", None)
        .await
        .expect("GetCategories");
    assert!(categories.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let exported = call_method_unchecked(&registry, "Keybinds.Export", None)
        .await
        .expect("Export");
    assert!(exported.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    call_method_unchecked(
        &registry,
        "Keybinds.Set",
        Some(json!({
            "combo": "SUPER, T",
            "action": "dispatch togglefloating",
            "bind_type": "bind"
        })),
    )
    .await
    .expect("Set");

    call_method_unchecked(
        &registry,
        "Keybinds.Unset",
        Some(json!({ "combo": "SUPER, T" })),
    )
    .await
    .expect("Unset");

    let import_entries = json!([{
        "combo": "SUPER, I",
        "action": "exec wofi",
        "bind_type": "bind",
        "file": aura_binds.display().to_string(),
        "line": 1,
        "category": "launcher"
    }]);
    call_method_unchecked(
        &registry,
        "Keybinds.Import",
        Some(json!({ "entries": import_entries })),
    )
    .await
    .expect("Import");

    let text = tokio::fs::read_to_string(&aura_binds).await.expect("read binds");
    assert!(text.contains("SUPER, I"));

    call_method_unchecked(&registry, "Keybinds.Reload", None)
        .await
        .expect("Reload");

    std::env::remove_var("HYPRLAND_CONFIG");
    std::env::remove_var("AURA_KEYBINDS_PATH");
}

#[tokio::test]
async fn shell_lock_launch_and_toggle_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(&registry, "Session.Lock", None)
        .await
        .expect("Session.Lock");

    call_method_unchecked(
        &registry,
        "Aura.ToggleWindow",
        Some(json!({ "name": "control-center" })),
    )
    .await
    .expect("Aura.ToggleWindow");

    let err = call_method_unchecked(
        &registry,
        "Aura.ToggleWindow",
        Some(json!({ "name": "launcher" })),
    )
    .await
    .expect_err("disallowed window");
    assert!(err.to_string().contains("not allowed"));

    call_method_unchecked(
        &registry,
        "Apps.Launch",
        Some(json!({ "id": "terminal" })),
    )
    .await
    .expect("Apps.Launch");

    for app_id in ["browser", "code", "music", "discord", "firefox"] {
        call_method_unchecked(
            &registry,
            "Apps.Launch",
            Some(json!({ "id": app_id })),
        )
        .await
        .unwrap_or_else(|e| panic!("Apps.Launch {app_id}: {e}"));
    }

    std::env::set_var("XDG_SESSION_ID", "coverage-session");
    call_method_unchecked(&registry, "Session.Logout", None)
        .await
        .expect("Session.Logout");
    std::env::remove_var("XDG_SESSION_ID");
}

#[tokio::test]
async fn productivity_timer_notify_send_path() {
    let _guard = PRODUCTIVITY_ASYNC_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    reset_productivity_state_for_tests().await;
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    std::env::remove_var("AURA_SKIP_NOTIFY_SEND");
    let registry = test_registry();

    let timer = call_method_unchecked(
        &registry,
        "Productivity.CreateTimer",
        Some(json!({ "duration_seconds": 1, "name": "Notify path" })),
    )
    .await
    .expect("CreateTimer");
    let timer_id = timer.get("id").and_then(|v| v.as_str()).expect("id");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    call_method_unchecked(
        &registry,
        "Productivity.CancelTimer",
        Some(json!({ "timer_id": timer_id })),
    )
    .await
    .expect("CancelTimer");
    reset_productivity_state_for_tests().await;
}

#[tokio::test]
async fn automation_update_workflow_cron_triggers() {
    let _guard = automation_test_lock();
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Cron flow",
            "triggers": [{ "type": "time", "cron": "0 9 * * *" }],
            "actions": [{ "type": "send_notification", "title": "cron", "body": "ok" }]
        })),
    )
    .await
    .expect("CreateWorkflow");
    let id = created.get("id").and_then(|v| v.as_str()).expect("id");

    call_method_unchecked(
        &registry,
        "Automation.UpdateWorkflow",
        Some(json!({
            "workflow_id": id,
            "updates": {
                "triggers": [{ "type": "manual" }, { "type": "time", "cron": "*/15 * * * *" }]
            }
        })),
    )
    .await
    .expect("UpdateWorkflow triggers");

    let err = call_method_unchecked(
        &registry,
        "Automation.UpdateWorkflow",
        Some(json!({
            "workflow_id": id,
            "updates": {
                "actions": [{ "type": "execute_command", "command": "rm -rf /tmp/x" }]
            }
        })),
    )
    .await
    .expect_err("bad command");
    assert!(err.to_string().contains("not allowed"));
}

#[tokio::test]
async fn communication_mutators_and_app_discovery_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Communication.MarkRead",
        Some(json!({ "message_id": "msg-1" })),
    )
    .await
    .expect("MarkRead");

    call_method_unchecked(
        &registry,
        "Communication.SendMessage",
        Some(json!({
            "service": "telegram",
            "recipient": "alice",
            "message": "hello"
        })),
    )
    .await
    .expect("SendMessage");

    call_method_unchecked(
        &registry,
        "Communication.MuteNotifications",
        Some(json!({ "duration_minutes": 30 })),
    )
    .await
    .expect("MuteNotifications");

    call_method_unchecked(
        &registry,
        "Communication.LaunchApp",
        Some(json!({ "app_name": "telegram" })),
    )
    .await
    .expect("LaunchApp");

    let apps = call_method(&registry, "Communication.GetCommunicationApps", None)
        .await
        .expect("GetCommunicationApps");
    let ids = apps.as_array().expect("apps");
    assert!(ids.iter().any(|v| v.as_str() == Some("telegram")));
    assert!(ids.iter().any(|v| v.as_str() == Some("discord")));
}

#[tokio::test]
async fn packages_readonly_extended_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for method in [
        "Packages.GetSnapPackages",
        "Packages.GetAurPackages",
        "Packages.GetUpgradable",
    ] {
        let value = call_method(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(value.is_array(), "{method} shape");
    }

    let files = call_method(
        &registry,
        "Packages.GetPackageFiles",
        Some(json!({ "name": "pacman" })),
    )
    .await
    .expect("GetPackageFiles");
    assert!(files.get("files").and_then(|v| v.as_str()).is_some());

    let flatpaks = call_method(&registry, "Packages.GetFlatpakPackages", None)
        .await
        .expect("GetFlatpakPackages");
    assert!(flatpaks.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let history = call_method(
        &registry,
        "Packages.GetTransactionHistory",
        Some(json!({ "limit": 5 })),
    )
    .await
    .expect("GetTransactionHistory");
    assert!(history.is_array());
}

#[tokio::test]
async fn vault_list_remotes_exec_fixture_path() {
    std::env::remove_var("AURA_VAULT_LISTREMOTES_OUTPUT");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    let list = call_method(&registry, "Vault.List", None)
        .await
        .expect("Vault.List");
    assert_eq!(list.get("rclone_available"), Some(&json!(true)));
    let remotes = list.get("remotes").and_then(|v| v.as_array()).expect("remotes");
    assert!(remotes.iter().any(|r| r.get("name").and_then(|v| v.as_str()) == Some("gdrive")));
}

#[tokio::test]
async fn lock_test_fingerprint_mocked() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    let result = call_method_unchecked(&registry, "Lock.TestFingerprint", None)
        .await
        .expect("Lock.TestFingerprint");
    assert_eq!(result.get("ok").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn fitness_workout_and_goal_mutators() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let workout = call_method_unchecked(
        &registry,
        "Fitness.StartWorkout",
        Some(json!({ "type": "run" })),
    )
    .await
    .expect("StartWorkout");
    assert_eq!(workout.get("workout_type").and_then(|v| v.as_str()), Some("run"));

    call_method_unchecked(&registry, "Fitness.StopWorkout", None)
        .await
        .expect("StopWorkout");

    let goal = call_method_unchecked(
        &registry,
        "Fitness.SetGoal",
        Some(json!({ "type": "steps", "target": 8000.0 })),
    )
    .await
    .expect("SetGoal");
    assert_eq!(goal.get("target").and_then(|v| v.as_f64()), Some(8000.0));

    call_method_unchecked(
        &registry,
        "Fitness.SyncDevice",
        Some(json!({ "device_id": "watch-1" })),
    )
    .await
    .expect("SyncDevice");
}

#[tokio::test]
async fn weather_location_mutators_and_stubs() {
    let _lock = weather_test_lock();
    reset_weather_cache_for_tests().await;
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Weather.AddLocation",
        Some(json!({ "city": "Berlin" })),
    )
    .await
    .expect("AddLocation");

    call_method_unchecked(
        &registry,
        "Weather.SetLocation",
        Some(json!({ "city": "Berlin" })),
    )
    .await
    .expect("SetLocation");

    let locations = call_method(&registry, "Weather.GetLocations", None)
        .await
        .expect("GetLocations");
    assert!(locations.as_array().unwrap().iter().any(|c| c.as_str() == Some("Berlin")));

    call_method_unchecked(
        &registry,
        "Weather.SetUnits",
        Some(json!({ "units": "metric" })),
    )
    .await
    .expect("SetUnits");

    call_method_unchecked(
        &registry,
        "Weather.RemoveLocation",
        Some(json!({ "city": "Berlin" })),
    )
    .await
    .expect("RemoveLocation");

    let alerts = call_method(&registry, "Weather.GetAlerts", None)
        .await
        .expect("GetAlerts");
    assert!(alerts.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            common::load_fixture("weather/wttr_forecast.json"),
        ))
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::set_var("AURA_WEATHER_SKIP_CACHE", "1");

    let hourly = call_method(&registry, "Weather.GetHourly", None)
        .await
        .expect("GetHourly");
    assert!(hourly.is_array());

    std::env::remove_var("AURA_WEATHER_WTTR_URL");
    std::env::remove_var("AURA_WEATHER_SKIP_CACHE");
}

#[tokio::test]
async fn bluetooth_readonly_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for method in ["Bluetooth.GetAdapters", "Bluetooth.GetDevices"] {
        let value = call_method(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(value.is_array() || value.is_object(), "{method} shape");
    }

    let info = call_method(
        &registry,
        "Bluetooth.GetDeviceInfo",
        Some(json!({ "device_address": "AA:BB:CC:DD:EE:FF" })),
    )
    .await
    .expect("GetDeviceInfo fixture");
    assert!(info.get("name").is_some());
}

#[tokio::test]
async fn hyprland_readonly_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for method in [
        "Hyprland.GetWorkspaces",
        "Hyprland.GetActiveWorkspace",
        "Hyprland.GetClients",
        "Hyprland.GetActiveWindow",
        "Hyprland.GetMonitors",
    ] {
        let value = call_method(&registry, method, None)
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(
            value.is_array() || value.is_object() || value.is_null(),
            "{method} shape"
        );
    }
}

#[tokio::test]
async fn launcher_query_and_recent_with_fixture_desktop() {
    use ags_sidecar::services::launcher::invalidate_desktop_index_cache;
    use common::launcher_test_lock;

    let _guard = launcher_test_lock();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/desktop");
    std::env::set_var(
        "AURA_LAUNCHER_DESKTOP_DIRS",
        dir.to_string_lossy().to_string(),
    );
    invalidate_desktop_index_cache();
    let registry = test_registry();

    let query = call_method(
        &registry,
        "Launcher.Query",
        Some(json!({ "query": "fire" })),
    )
    .await
    .expect("Launcher.Query");
    assert!(query.get("results").and_then(|v| v.as_array()).is_some());

    let recent = call_method(&registry, "Launcher.Recent", None)
        .await
        .expect("Launcher.Recent");
    assert!(recent.get("items").and_then(|v| v.as_array()).is_some());

    std::env::remove_var("AURA_LAUNCHER_DESKTOP_DIRS");
}

#[tokio::test]
async fn productivity_timer_completion_and_pomodoro_phases() {
    let _guard = PRODUCTIVITY_ASYNC_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    reset_productivity_state_for_tests().await;
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    std::env::set_var("AURA_SKIP_NOTIFY_SEND", "1");
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Productivity.CreateTimer",
        Some(json!({ "duration_seconds": 1, "name": "Short" })),
    )
    .await
    .expect("CreateTimer");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let timers = call_method(&registry, "Productivity.GetTimers", None)
        .await
        .expect("GetTimers");
    let rows = timers.as_array().expect("timers");
    assert!(
        rows.iter()
            .any(|t| t.get("name").and_then(|v| v.as_str()) == Some("Short")
                && t.get("active").and_then(|v| v.as_bool()) == Some(false)),
        "timer should complete: {timers}"
    );

    call_method_unchecked(
        &registry,
        "Productivity.CreatePomodoro",
        Some(json!({ "work_minutes": 0, "break_minutes": 0 })),
    )
    .await
    .expect("CreatePomodoro");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let pomo = call_method(&registry, "Productivity.GetPomodoroStatus", None)
        .await
        .expect("GetPomodoroStatus");
    assert_eq!(pomo.get("active").and_then(|v| v.as_bool()), Some(true));
    assert!(
        pomo.get("current_phase").and_then(|v| v.as_str()).is_some(),
        "pomodoro should have transitioned phases"
    );

    let stats = call_method(&registry, "Productivity.GetStats", None)
        .await
        .expect("GetStats");
    assert!(stats.get("pomodoro_active").and_then(|v| v.as_bool()).unwrap_or(false));

    std::env::remove_var("AURA_SKIP_NOTIFY_SEND");
    reset_productivity_state_for_tests().await;
}

#[tokio::test]
async fn automation_execute_command_and_run_script_workflows() {
    let _guard = automation_test_lock();
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let scripts_dir = std::env::temp_dir().join(format!(
        "ags-automation-final-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scripts_dir);
    std::env::set_var("AURA_AUTOMATION_SCRIPTS_DIR", &scripts_dir);

    let registry = test_registry();

    let script = call_method_unchecked(
        &registry,
        "Automation.CreateScript",
        Some(json!({
            "name": "wf-script",
            "content": "#!/bin/sh\necho workflow-script-ok\n",
            "interpreter": "sh"
        })),
    )
    .await
    .expect("CreateScript");
    let script_id = script.get("id").and_then(|v| v.as_str()).expect("script id");

    let cmd_flow = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Exec flow",
            "actions": [{
                "type": "execute_command",
                "command": "notify-send Automation exec ok"
            }]
        })),
    )
    .await
    .expect("CreateWorkflow exec");
    let cmd_id = cmd_flow.get("id").and_then(|v| v.as_str()).expect("wf id");

    call_method_unchecked(
        &registry,
        "Automation.RunWorkflow",
        Some(json!({ "workflow_id": cmd_id })),
    )
    .await
    .expect("RunWorkflow exec");

    let script_flow = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Script flow",
            "actions": [{ "type": "run_script", "script_id": script_id }]
        })),
    )
    .await
    .expect("CreateWorkflow script");
    let script_wf_id = script_flow.get("id").and_then(|v| v.as_str()).expect("wf id");

    call_method_unchecked(
        &registry,
        "Automation.RunWorkflow",
        Some(json!({ "workflow_id": script_wf_id })),
    )
    .await
    .expect("RunWorkflow script");

    let triggers = call_method(&registry, "Automation.GetTriggers", None)
        .await
        .expect("GetTriggers");
    assert!(triggers.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let actions = call_method(&registry, "Automation.GetActions", None)
        .await
        .expect("GetActions");
    assert!(actions
        .as_array()
        .map(|a| a.iter().any(|v| v.as_str() == Some("execute_command")))
        .unwrap_or(false));

    std::env::remove_var("AURA_AUTOMATION_SCRIPTS_DIR");
    let _ = std::fs::remove_dir_all(&scripts_dir);
}

#[tokio::test]
async fn bluetooth_mutators_exec_fixture_sweep() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    call_method_unchecked(&registry, "Bluetooth.Scan", None)
        .await
        .expect("Scan");
    call_method_unchecked(&registry, "Bluetooth.StopScan", None)
        .await
        .expect("StopScan");

    let adapters = call_method(&registry, "Bluetooth.GetAdapters", None)
        .await
        .expect("GetAdapters");
    if let Some(adapter) = adapters.as_array().and_then(|a| a.first()) {
        let path = adapter
            .get("path")
            .and_then(|v| v.as_str())
            .expect("adapter path");
        call_method_unchecked(
            &registry,
            "Bluetooth.SetAdapterPower",
            Some(json!({ "adapter_path": path, "powered": true })),
        )
        .await
        .expect("SetAdapterPower");
        call_method_unchecked(
            &registry,
            "Bluetooth.SetAdapterDiscoverable",
            Some(json!({ "adapter_path": path, "discoverable": true })),
        )
        .await
        .expect("SetAdapterDiscoverable");
        call_method_unchecked(
            &registry,
            "Bluetooth.SetAdapterDiscoverable",
            Some(json!({ "adapter_path": path, "discoverable": false })),
        )
        .await
        .expect("SetAdapterDiscoverable off");
    }

    let addr = "AA:BB:CC:DD:EE:FF";
    for (method, params) in [
        (
            "Bluetooth.Pair",
            json!({ "device_address": addr }),
        ),
        (
            "Bluetooth.Connect",
            json!({ "device_address": addr }),
        ),
        (
            "Bluetooth.Disconnect",
            json!({ "device_address": addr }),
        ),
        (
            "Bluetooth.Remove",
            json!({ "device_address": addr }),
        ),
    ] {
        call_method_unchecked(&registry, method, Some(params))
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
    }
}

#[tokio::test]
async fn automation_workflow_lifecycle_with_fixtures() {
    let _guard = automation_test_lock();
    let _exec = ExecFixtureGuard::activate();
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let created = call_method_unchecked(
        &registry,
        "Automation.CreateWorkflow",
        Some(json!({
            "name": "Lifecycle",
            "triggers": [{ "type": "time", "cron": "0 9 * * *" }],
            "actions": [{ "type": "send_notification", "title": "Hi", "body": "there" }]
        })),
    )
    .await
    .expect("CreateWorkflow");
    let id = created.get("id").and_then(|v| v.as_str()).expect("id");

    call_method_unchecked(
        &registry,
        "Automation.UpdateWorkflow",
        Some(json!({
            "workflow_id": id,
            "updates": { "name": "Lifecycle updated" }
        })),
    )
    .await
    .expect("UpdateWorkflow");

    call_method_unchecked(
        &registry,
        "Automation.DisableWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .expect("DisableWorkflow");

    call_method_unchecked(
        &registry,
        "Automation.EnableWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .expect("EnableWorkflow");

    call_method_unchecked(
        &registry,
        "Automation.Trigger",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .expect("Trigger");

    let history = call_method(
        &registry,
        "Automation.GetWorkflowHistory",
        Some(json!({ "workflow_id": id, "limit": 5 })),
    )
    .await
    .expect("GetWorkflowHistory");
    assert!(history.is_array() || history.get("runs").is_some());

    call_method_unchecked(
        &registry,
        "Automation.DeleteWorkflow",
        Some(json!({ "workflow_id": id })),
    )
    .await
    .expect("DeleteWorkflow");
}

#[tokio::test]
async fn power_set_profile_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    for profile in ["balanced", "performance", "saver"] {
        call_method_unchecked(
            &registry,
            "Power.SetProfile",
            Some(json!({ "profile": profile })),
        )
        .await
        .unwrap_or_else(|e| panic!("SetProfile {profile}: {e}"));
    }

    let current = call_method(&registry, "Power.GetProfile", None)
        .await
        .expect("GetProfile");
    assert!(current.get("profile").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn security_mutators_with_exec_fixtures() {
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(&registry, "Security.EnableFirewall", None)
        .await
        .expect("EnableFirewall");

    call_method_unchecked(
        &registry,
        "Security.AddFirewallRule",
        Some(json!({
            "rule": {
                "id": "test-allow",
                "action": "allow",
                "direction": "in",
                "protocol": "tcp",
                "port": "8080",
                "source": null,
                "destination": null
            }
        })),
    )
    .await
    .expect("AddFirewallRule");

    call_method_unchecked(
        &registry,
        "Security.RemoveFirewallRule",
        Some(json!({ "rule_id": "1" })),
    )
    .await
    .expect("RemoveFirewallRule");

    call_method_unchecked(
        &registry,
        "Security.RunClamScan",
        Some(json!({ "path": "/tmp" })),
    )
    .await
    .expect("RunClamScan");

    call_method_unchecked(&registry, "Security.DisableFirewall", None)
        .await
        .expect("DisableFirewall");
}

#[tokio::test]
async fn vpn_education_openconnect_dry_run() {
    std::env::set_var("AURA_VPN_DRY_RUN", "1");
    let _exec = ExecFixtureGuard::activate();
    let registry = test_registry();

    call_method_unchecked(
        &registry,
        "Vpn.Connect",
        Some(json!({ "profile_id": "education" })),
    )
    .await
    .expect("Vpn.Connect education");

    let status = call_method(&registry, "Vpn.GetStatus", None)
        .await
        .expect("GetStatus");
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("connected")
    );

    call_method_unchecked(&registry, "Vpn.Disconnect", None)
        .await
        .expect("Vpn.Disconnect");

    std::env::remove_var("AURA_VPN_DRY_RUN");
}

#[tokio::test]
async fn calendar_update_and_sync_with_temp_storage() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let now = chrono::Utc::now().timestamp();

    let created = call_method_unchecked(
        &registry,
        "Calendar.CreateEvent",
        Some(json!({
            "title": "Coverage event",
            "start": now + 3600,
            "end": now + 7200
        })),
    )
    .await
    .expect("CreateEvent");
    let event_id = created.get("id").and_then(|v| v.as_str()).expect("id");

    call_method_unchecked(
        &registry,
        "Calendar.UpdateEvent",
        Some(json!({
            "event_id": event_id,
            "updates": { "title": "Updated title" }
        })),
    )
    .await
    .expect("UpdateEvent");

    call_method_unchecked(&registry, "Calendar.SyncCalendars", None)
        .await
        .expect("SyncCalendars");
}
