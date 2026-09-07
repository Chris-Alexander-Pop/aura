//! Read-only `Weather.*` RPC shapes (HTTP mocked via wiremock).

mod common;

use ags_sidecar::services::weather::reset_weather_cache_for_tests;
use common::{call_rpc, load_fixture, setup_temp_storage_db, test_registry, weather_test_lock};
use serde_json::json;
use std::time::{Duration, Instant};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

struct WeatherMockEnv {
    _server: MockServer,
}

impl Drop for WeatherMockEnv {
    fn drop(&mut self) {
        std::env::remove_var("AURA_WEATHER_WTTR_URL");
        std::env::remove_var("AURA_WEATHER_SKIP_CACHE");
        std::env::remove_var("AURA_WEATHER_API_KEY");
    }
}

async fn weather_mock_env_with_body(body: &str) -> WeatherMockEnv {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::set_var("AURA_WEATHER_SKIP_CACHE", "1");
    WeatherMockEnv { _server: server }
}

async fn weather_mock_env() -> WeatherMockEnv {
    weather_mock_env_with_body(&load_fixture("weather/wttr_current.json")).await
}

async fn weather_test_setup() -> std::sync::MutexGuard<'static, ()> {
    let lock = weather_test_lock();
    reset_weather_cache_for_tests().await;
    lock
}

#[tokio::test]
async fn weather_get_offline_mock_http() {
    let _lock = weather_test_setup().await;
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

#[test]
fn weather_wttr_json_fixture_shape() {
    let text = load_fixture("weather/wttr.json");
    let data: serde_json::Value = serde_json::from_str(&text).expect("wttr json");
    let cond = &data["current_condition"][0];
    assert_eq!(cond["weatherCode"], "116");
    assert_eq!(cond["humidity"], "72");
    assert_eq!(cond["temp_C"], "18");
    assert!(data["weather"].as_array().unwrap().is_empty());
}

#[test]
fn weather_wttr_forecast_fixture_shape() {
    let text = load_fixture("weather/wttr_forecast.json");
    let data: serde_json::Value = serde_json::from_str(&text).expect("forecast json");
    assert_eq!(data["weather"].as_array().unwrap().len(), 3);
    assert_eq!(data["current_condition"][0]["weatherCode"], "113");
}

#[tokio::test]
async fn weather_get_uses_in_memory_cache_when_not_skipped() {
    let _lock = weather_test_setup().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(load_fixture("weather/wttr_current.json")),
        )
        .expect(1)
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::remove_var("AURA_WEATHER_SKIP_CACHE");

    let registry = test_registry();
    let params = Some(json!({ "city": "CachedCity" }));
    let first = call_rpc(&registry, "Weather.Get", params.clone())
        .await
        .expect("first Weather.Get");
    let second = call_rpc(&registry, "Weather.Get", params)
        .await
        .expect("cached Weather.Get");
    assert_eq!(first.get("icon"), second.get("icon"));
    assert_eq!(first.get("humidity"), second.get("humidity"));

    std::env::remove_var("AURA_WEATHER_WTTR_URL");
}

#[tokio::test]
async fn weather_get_rate_limit_returns_cached_without_second_fetch() {
    let _lock = weather_test_setup().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(load_fixture("weather/wttr_current.json")),
        )
        .expect(1)
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::set_var("AURA_WEATHER_API_KEY", "test-key");
    std::env::remove_var("AURA_WEATHER_SKIP_CACHE");

    let registry = test_registry();
    let params = Some(json!({ "city": "RateLimited" }));
    call_rpc(&registry, "Weather.Get", params.clone())
        .await
        .expect("first fetch");
    let cached = call_rpc(&registry, "Weather.Get", params)
        .await
        .expect("rate-limited cache");
    assert_eq!(
        cached.get("icon").and_then(|v| v.as_str()),
        Some("partly_cloudy")
    );

    std::env::remove_var("AURA_WEATHER_WTTR_URL");
    std::env::remove_var("AURA_WEATHER_API_KEY");
}

#[tokio::test]
async fn weather_get_forecast_mock_http() {
    let _lock = weather_test_setup().await;
    let _mock = weather_mock_env_with_body(&load_fixture("weather/wttr_forecast.json")).await;
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Weather.GetForecast",
        Some(json!({ "city": "Kitchener", "days": 2 })),
    )
    .await
    .expect("Weather.GetForecast");
    let days = value.as_array().expect("forecast array");
    assert_eq!(days.len(), 2);
    assert!(days[0].get("date").and_then(|v| v.as_str()).is_some());
}

#[tokio::test]
async fn weather_get_sunny_icon_branch() {
    let _lock = weather_test_setup().await;
    let _mock = weather_mock_env_with_body(&load_fixture("weather/wttr_forecast.json")).await;
    let registry = test_registry();
    let value = call_rpc(
        &registry,
        "Weather.Get",
        Some(json!({ "city": "ClearSky" })),
    )
    .await
    .expect("Weather.Get sunny");
    assert_eq!(value.get("icon").and_then(|v| v.as_str()), Some("sunny"));
    assert_eq!(value.get("code").and_then(|v| v.as_str()), Some("113"));
}

#[tokio::test]
async fn weather_get_times_out_instead_of_hanging() {
    let _lock = weather_test_setup().await;
    std::env::set_var("AURA_HTTP_TIMEOUT_MS", "200");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(5))
                .set_body_string(load_fixture("weather/wttr_current.json")),
        )
        .mount(&server)
        .await;
    std::env::set_var("AURA_WEATHER_WTTR_URL", server.uri());
    std::env::set_var("AURA_WEATHER_SKIP_CACHE", "1");

    let registry = test_registry();
    let started = Instant::now();
    let result = call_rpc(
        &registry,
        "Weather.Get",
        Some(json!({ "city": "HangTown" })),
    )
    .await;

    std::env::remove_var("AURA_HTTP_TIMEOUT_MS");
    std::env::remove_var("AURA_WEATHER_WTTR_URL");
    std::env::remove_var("AURA_WEATHER_SKIP_CACHE");

    assert!(result.is_err(), "slow weather HTTP should fail, got {result:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "Weather.Get hung for {:?}",
        started.elapsed()
    );
}
