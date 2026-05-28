//! Focused integration tests for mid-tier services (storage branches, schema shapes).
//! Avoids duplicating the broad `readonly_gap_methods_resolve` sweep.

mod common;

use common::{call_rpc, load_fixture, setup_temp_storage_db, test_registry};
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

struct WeatherMockEnv {
    _server: MockServer,
}

impl Drop for WeatherMockEnv {
    fn drop(&mut self) {
        std::env::remove_var("AURA_WEATHER_WTTR_URL");
        std::env::remove_var("AURA_WEATHER_SKIP_CACHE");
    }
}

async fn weather_mock_env() -> WeatherMockEnv {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_fixture("weather/wttr_current.json")))
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::set_var("AURA_WEATHER_SKIP_CACHE", "1");
    WeatherMockEnv { _server: server }
}

#[tokio::test]
async fn automation_get_workflows_empty_namespace() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let value = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .expect("Automation.GetWorkflows");
    assert_eq!(value.as_array().map(|a| a.len()), Some(0));
}

#[tokio::test]
async fn automation_get_workflows_skips_corrupt_storage_rows() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_workflows",
            "key": "good",
            "value": {
                "id": "good",
                "name": "Daily",
                "enabled": true,
                "triggers": [],
                "actions": [],
                "created_at": 1
            }
        })),
    )
    .await
    .expect("Storage.Set good");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "automation_workflows",
            "key": "bad",
            "value": { "oops": true }
        })),
    )
    .await
    .expect("Storage.Set bad");

    let workflows = call_rpc(&registry, "Automation.GetWorkflows", None)
        .await
        .expect("Automation.GetWorkflows");
    let arr = workflows.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("id").and_then(|v| v.as_str()), Some("good"));
    assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Daily"));
}

#[tokio::test]
async fn automation_get_triggers_and_actions_schema() {
    let registry = test_registry();
    for method in ["Automation.GetTriggers", "Automation.GetActions"] {
        let value = call_rpc(&registry, method, None).await.expect(method);
        let arr = value.as_array().expect("string array");
        assert!(!arr.is_empty());
        assert!(arr.iter().all(|v| v.as_str().is_some()));
    }
}

#[tokio::test]
async fn productivity_focus_mode_and_stats_schema() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let default = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus");
    assert_eq!(default.get("enabled"), Some(&json!(false)));

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "productivity",
            "key": "focus_mode",
            "value": true
        })),
    )
    .await
    .expect("Storage.Set focus_mode");

    let on = call_rpc(&registry, "Productivity.GetFocusModeStatus", None)
        .await
        .expect("GetFocusModeStatus on");
    assert_eq!(on.get("enabled"), Some(&json!(true)));

    let stats = call_rpc(&registry, "Productivity.GetStats", None)
        .await
        .expect("Productivity.GetStats");
    for key in [
        "active_timers",
        "total_timers",
        "pomodoro_active",
        "focus_mode_enabled",
        "screen_time_minutes",
        "task_count",
    ] {
        assert!(stats.get(key).is_some(), "missing stats field {key}");
    }
    assert_eq!(stats.get("focus_mode_enabled"), Some(&json!(true)));
    assert!(stats.get("pomodoro_phase").map(|v| v.is_null()).unwrap_or(false));
}

#[tokio::test]
async fn communication_unread_and_notification_settings_schema() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let unread = call_rpc(&registry, "Communication.GetUnread", None)
        .await
        .expect("Communication.GetUnread");
    assert!(unread.is_object());
    assert!(unread.as_object().unwrap().is_empty());

    let default_settings = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("GetNotificationSettings");
    assert!(default_settings.is_object());

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "communication",
            "key": "notification_settings",
            "value": { "mute_all": false, "per_app": {} }
        })),
    )
    .await
    .expect("Storage.Set notification_settings");

    let settings = call_rpc(&registry, "Communication.GetNotificationSettings", None)
        .await
        .expect("GetNotificationSettings loaded");
    assert_eq!(settings.get("mute_all"), Some(&json!(false)));
    assert!(settings.get("per_app").and_then(|v| v.as_object()).is_some());
}

#[tokio::test]
async fn calendar_get_events_filters_and_skips_corrupt() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    for (key, start, end) in [
        ("inside", 200, 300),
        ("before", 10, 50),
        ("after", 900, 1000),
    ] {
        call_rpc(
            &registry,
            "Storage.Set",
            Some(json!({
                "namespace": "calendar_events",
                "key": key,
                "value": {
                    "id": key,
                    "title": key,
                    "start": start,
                    "end": end,
                    "description": "",
                    "calendar_id": null,
                    "reminder_minutes": null
                }
            })),
        )
        .await
        .expect("Storage.Set event");
    }

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "calendar_events",
            "key": "corrupt",
            "value": { "not_an_event": 1 }
        })),
    )
    .await
    .expect("Storage.Set corrupt");

    let filtered = call_rpc(
        &registry,
        "Calendar.GetEvents",
        Some(json!({ "start_date": 150, "end_date": 400 })),
    )
    .await
    .expect("Calendar.GetEvents");
    let ids: Vec<_> = filtered
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["inside"]);

    let all = call_rpc(&registry, "Calendar.GetEvents", None)
        .await
        .expect("Calendar.GetEvents all");
    assert_eq!(all.as_array().map(|a| a.len()), Some(3));
}

#[tokio::test]
async fn vpn_get_status_and_profiles_schema() {
    let registry = test_registry();

    let status = call_rpc(&registry, "Vpn.GetStatus", None)
        .await
        .expect("Vpn.GetStatus");
    for key in ["state", "message"] {
        assert!(status.get(key).is_some(), "missing {key}");
    }
    assert!(status.get("profile_id").is_some());

    let profiles = call_rpc(&registry, "Vpn.GetProfiles", None)
        .await
        .expect("Vpn.GetProfiles");
    let arr = profiles.as_array().expect("profiles array");
    assert!(!arr.is_empty());
    let first = &arr[0];
    for key in ["id", "name", "icon", "display_name", "interface", "requires_credentials"] {
        assert!(first.get(key).is_some(), "missing profile field {key}");
    }
}

#[tokio::test]
async fn gamemode_is_enabled_schema() {
    let registry = test_registry();
    let value = call_rpc(&registry, "GameMode.IsEnabled", None)
        .await
        .expect("GameMode.IsEnabled");
    assert!(value.get("enabled").and_then(|v| v.as_bool()).is_some());
}

#[tokio::test]
async fn fitness_get_activity_and_goals_from_storage() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();

    let activity = call_rpc(&registry, "Fitness.GetActivity", None)
        .await
        .expect("Fitness.GetActivity");
    for key in ["date", "steps", "calories", "distance_km"] {
        assert!(activity.get(key).is_some(), "missing {key}");
    }

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_goals",
            "key": "g1",
            "value": {
                "id": "g1",
                "goal_type": "steps",
                "target": 10000.0,
                "current": 1200.0
            }
        })),
    )
    .await
    .expect("Storage.Set goal");

    call_rpc(
        &registry,
        "Storage.Set",
        Some(json!({
            "namespace": "fitness_goals",
            "key": "bad",
            "value": "not json goal"
        })),
    )
    .await
    .expect("Storage.Set bad goal");

    let goals = call_rpc(&registry, "Fitness.GetGoals", None)
        .await
        .expect("Fitness.GetGoals");
    let arr = goals.as_array().expect("goals");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("goal_type").and_then(|v| v.as_str()), Some("steps"));
}

#[tokio::test]
async fn shell_get_version_readonly_schema() {
    let registry = test_registry();
    let value = call_rpc(&registry, "Sidecar.GetVersion", None)
        .await
        .expect("Sidecar.GetVersion");
    assert!(value
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false));
}

#[tokio::test]
async fn weather_get_offline_mock_http() {
    let _mock = weather_mock_env().await;
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Weather.Get",
        Some(json!({ "city": "Kitchener" })),
    )
    .await
    .expect("Weather.Get");
    assert_eq!(value.get("icon").and_then(|v| v.as_str()), Some("partly_cloudy"));
    assert_eq!(value.get("humidity").and_then(|v| v.as_i64()), Some(72));
    assert!(value.get("temp").and_then(|v| v.as_str()).is_some());
    assert!(value.get("description").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn weather_get_locations_default_empty_array() {
    let _db = setup_temp_storage_db().await;
    let registry = test_registry();
    let value = call_rpc(&registry, "Weather.GetLocations", None)
        .await
        .expect("Weather.GetLocations");
    assert!(value.is_array());
    assert!(value.as_array().unwrap().is_empty());
}
