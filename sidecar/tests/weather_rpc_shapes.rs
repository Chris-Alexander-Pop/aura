//! Read-only `Weather.*` RPC shapes (HTTP mocked via wiremock).

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
