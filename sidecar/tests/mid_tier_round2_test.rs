//! Round-2 mid-tier coverage: storage edge branches + read-only List/Get shape tests.
//! Complements `mid_tier_services_test.rs` without duplicating its scenarios.

mod common;

use ags_sidecar::contract_parsers::{
    ip_link_interface_up, parse_wireguard_conf, vpn_interface_connected,
};
use common::{call_rpc, load_fixture, setup_temp_storage_db, test_registry};
use serde_json::json;

fn assert_object_keys(value: &serde_json::Value, keys: &[&str]) {
    let obj = value.as_object().expect("object");
    for key in keys {
        assert!(obj.contains_key(*key), "missing key {key}");
    }
}

#[test]
fn vpn_ip_link_and_wireguard_fixture_parsers() {
    let link = load_fixture("vpn/ip_link_multi.txt");
    assert!(ip_link_interface_up(&link, "wg0"));
    assert!(!ip_link_interface_up(&link, "eth0"));
    assert!(!ip_link_interface_up(&link, "docker0"));

    let addr = load_fixture("vpn/ip_o_addr_show_connected.txt");
    assert!(vpn_interface_connected(&addr, "tun0", false));

    let wg = load_fixture("vpn/wg_home.conf");
    let summary = parse_wireguard_conf(&wg);
    assert!(!summary.addresses.is_empty());
    assert!(summary.endpoint.is_some());
}

#[tokio::test]
async fn productivity_readonly_stubs_and_corrupt_focus_mode() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let timers = call_rpc(&registry, "Productivity.GetTimers", None)
        .await
        .expect("GetTimers");
    assert!(timers.is_array());

    let pomo = call_rpc(&registry, "Productivity.GetPomodoroStatus", None)
        .await
        .expect("GetPomodoroStatus");
    assert!(pomo.is_null());

    let screen = call_rpc(&registry, "Productivity.GetScreenTime", None)
        .await
        .expect("GetScreenTime");
    assert_eq!(screen.get("minutes").and_then(|v| v.as_u64()), Some(0));

    let usage = call_rpc(&registry, "Productivity.GetAppUsage", None)
        .await
        .expect("GetAppUsage");
    assert!(usage.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let tasks = call_rpc(&registry, "Productivity.GetTasks", None)
        .await
        .expect("GetTasks");
    assert!(tasks.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity",
            "key": "focus_mode",
            "value": "not-a-bool"
        })),
    )
    .await
    .expect("Storage.Set corrupt focus_mode");

    let focus = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus");
    assert_eq!(focus.get("enabled"), Some(&json!(false)));
}

#[tokio::test]
async fn automation_get_scripts_empty_and_corrupt_namespace() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Automation.GetScripts", None)
        .await
        .expect("GetScripts empty");
    assert_eq!(empty.as_array().map(|a| a.len()), Some(0));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_scripts",
            "key": "ok",
            "value": {
                "id": "ok",
                "name": "Hello",
                "content": "echo hi",
                "interpreter": "bash"
            }
        })),
    )
    .await
    .expect("Storage.Set script");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_scripts",
            "key": "bad",
            "value": { "oops": true }
        })),
    )
    .await
    .expect("Storage.Set bad script");

    let scripts = call_rpc(&registry, "Automation.GetScripts", None)
        .await
        .expect("GetScripts");
    let arr = scripts.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_object_keys(&arr[0], &["id", "name", "content", "interpreter"]);
    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some("ok"));
}

#[tokio::test]
async fn communication_stub_list_methods_and_settings_branches() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let unread = call_rpc(&registry, "Communication.GetUnread", None)
        .await
        .expect("GetUnread");
    assert!(unread.is_object());
    assert!(unread.as_object().unwrap().is_empty());

    for method in [
        "Communication.GetMessages",
        "Communication.GetContacts",
        "Communication.GetConversations",
        "Communication.GetActiveCalls",
    ] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        assert!(
            value.as_array().map(|a| a.is_empty()).unwrap_or(false),
            "{method} should be empty array stub"
        );
    }

    let corrupt = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("settings default");
    assert!(corrupt.is_object());

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "communication",
            "key": "notification_settings",
            "value": "not-an-object"
        })),
    )
    .await
    .expect("Storage.Set corrupt settings");

    let normalized = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("settings normalized");
    assert_eq!(normalized, json!({}));

    let apps = call_rpc(&registry, "Communication.GetCommunicationApps", None)
        .await
        .expect("GetCommunicationApps");
    assert!(apps.is_array());
}

#[tokio::test]
async fn calendar_empty_namespace_and_event_shape() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let empty = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("GetEvents empty");
    assert!(empty.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let calendars = call_rpc(&registry, "Calendar.GetCalendars", None)
        .await
        .expect("GetCalendars");
    assert!(calendars.as_array().map(|a| a.is_empty()).unwrap_or(false));

    let upcoming = call_rpc(
        &registry,
        "Calendar.GetUpcomingEvents",
        Some(json!({ "days": 3 })),
    )
    .await
    .expect("GetUpcomingEvents");
    assert!(upcoming.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "calendar_events",
            "key": "meet",
            "value": {
                "id": "meet",
                "title": "Standup",
                "start": 1000,
                "end": 1100,
                "description": "daily",
                "calendar_id": "work",
                "reminder_minutes": 5
            }
        })),
    )
    .await
    .expect("Storage.Set event");

    let events = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("GetEvents");
    let first = events.as_array().and_then(|a| a.first()).expect("one event");
    for key in [
        "id",
        "title",
        "start",
        "end",
        "description",
        "calendar_id",
        "reminder_minutes",
    ] {
        assert!(first.get(key).is_some(), "missing {key}");
    }
}

#[tokio::test]
async fn fitness_workout_history_and_metric_stubs() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for (method, key) in [
        ("Fitness.GetSteps", "steps"),
        ("Fitness.GetHeartRate", "heart_rate"),
        ("Fitness.GetSleep", "sleep_hours"),
    ] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        assert!(value.get(key).is_some(), "missing {key} in {method}");
    }

    let history = call_rpc(&registry, "Fitness.GetWorkoutHistory", None)
        .await
        .expect("GetWorkoutHistory empty");
    assert!(history.as_array().map(|a| a.is_empty()).unwrap_or(false));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_workouts",
            "key": "w1",
            "value": {
                "id": "w1",
                "workout_type": "run",
                "start_time": 1,
                "end_time": 2,
                "duration_seconds": 60,
                "data": {}
            }
        })),
    )
    .await
    .expect("Storage.Set workout");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_workouts",
            "key": "bad",
            "value": { "nope": 1 }
        })),
    )
    .await
    .expect("Storage.Set bad workout");

    let workouts = call_rpc(&registry, "Fitness.GetWorkoutHistory", None)
        .await
        .expect("GetWorkoutHistory");
    let arr = workouts.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("workout_type").and_then(|v| v.as_str()), Some("run"));

    let devices = call_rpc(&registry, "Fitness.GetDevices", None)
        .await
        .expect("GetDevices");
    assert!(devices.as_array().map(|a| a.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn gamemode_is_enabled_defaults_false() {
    let registry = test_registry();
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert_object_keys(&value, &["enabled"]);
    assert_eq!(value.get("enabled"), Some(&json!(false)));
}

#[tokio::test]
async fn shell_get_version_semver_shape() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Sidecar.GetVersion", None)
        .await
        .expect("Sidecar.GetVersion");
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .expect("version string");
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "version should look like semver: {version}"
    );
}

#[tokio::test]
async fn vpn_status_state_enum_and_profiles_count() {
    let registry = test_registry();

    let status = call_rpc(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    let state = status
        .get("state")
        .and_then(|v| v.as_str())
        .expect("state");
    assert!(
        matches!(
            state,
            "disconnected" | "connecting" | "connected" | "error"
        ),
        "unexpected state: {state}"
    );
    assert!(status.get("message").and_then(|v| v.as_str()).is_some());

    let profiles = call_rpc(&registry, "Vpn.GetProfiles", None)
        .await
        .expect("Vpn.GetProfiles");
    let arr = profiles.as_array().expect("profiles");
    assert!(arr.len() >= 3);
    let ids: Vec<_> = arr
        .iter()
        .filter_map(|p| p.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"education"));
    assert!(ids.contains(&"personal"));
}
