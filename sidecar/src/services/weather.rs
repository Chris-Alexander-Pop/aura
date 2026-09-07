use crate::services::ServiceRegistry;
use crate::utils::{http, storage};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Deserialize)]
struct WttrResponse {
    current_condition: Vec<CurrentCondition>,
    weather: Vec<serde_json::Value>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct CurrentCondition {
    temp_C: String,
    temp_F: String,
    FeelsLikeC: String,
    #[allow(dead_code)]
    FeelsLikeF: String,
    humidity: String,
    weatherDesc: Vec<WeatherDesc>,
    weatherCode: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct WeatherDesc {
    value: String,
}

lazy_static::lazy_static! {
    static ref CACHED_WEATHER: Arc<RwLock<Option<serde_json::Value>>> = Arc::new(RwLock::new(None));
    static ref LAST_UPDATE: Arc<RwLock<Option<std::time::Instant>>> = Arc::new(RwLock::new(None));
    static ref LAST_FETCH: Arc<RwLock<Option<std::time::Instant>>> = Arc::new(RwLock::new(None));
}

const WEATHER_CACHE_TTL_SECS: u64 = 900;
const WEATHER_RATE_LIMIT_SECS: u64 = 60;

/// Test-only: clear process-global weather cache between integration tests.
pub async fn reset_weather_cache_for_tests() {
    *CACHED_WEATHER.write().await = None;
    *LAST_UPDATE.write().await = None;
    *LAST_FETCH.write().await = None;
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Weather.Get", |params| async move {
        let city: Option<String> = params
            .and_then(|p| p.get("city").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        // In-memory cache + rate limit; tests set AURA_WEATHER_SKIP_CACHE=1
        if !weather_skip_cache() {
            let last_update = LAST_UPDATE.read().await;
            if let Some(instant) = *last_update {
                if instant.elapsed().as_secs() < WEATHER_CACHE_TTL_SECS {
                    if let Some(cached) = CACHED_WEATHER.read().await.as_ref() {
                        return Ok(cached.clone());
                    }
                }
            }
            drop(last_update);

            let min_interval = weather_rate_limit_secs();
            let last_fetch = LAST_FETCH.read().await;
            if let Some(instant) = *last_fetch {
                if instant.elapsed().as_secs() < min_interval {
                    if let Some(cached) = CACHED_WEATHER.read().await.as_ref() {
                        return Ok(cached.clone());
                    }
                }
            }
            drop(last_fetch);
        }

        let city_name = if let Some(c) = city {
            c
        } else if let Some(c) = get_default_city().await {
            c
        } else if let Some(cached) = CACHED_WEATHER.read().await.as_ref() {
            return Ok(cached.clone());
        } else {
            anyhow::bail!("Weather location unavailable (offline or unset)");
        };

        match fetch_and_cache_current(&city_name).await {
            Ok(json) => Ok(json),
            Err(e) => {
                if let Some(cached) = CACHED_WEATHER.read().await.as_ref() {
                    tracing::debug!("weather fetch failed, using cache: {e}");
                    Ok(cached.clone())
                } else {
                    Err(e)
                }
            }
        }
    });

    registry.register("Weather.GetForecast", |params| async move {
        let days: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("days").cloned())
                .unwrap_or(serde_json::Value::Number(serde_json::Number::from(3))),
        )?;

        let city: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("city").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let city_name = resolve_city(city).await?;
        let weather_data = fetch_wttr_json(&city_name).await?;

        // Extract forecast data
        let forecast = weather_data.weather.iter().take(days as usize).collect::<Vec<_>>();
        Ok(serde_json::to_value(&forecast)?)
    });

    registry.register("Weather.GetHourly", |_params| async move {
        let city = resolve_city(None).await?;
        let weather_data = fetch_wttr_json(&city).await?;

        // Extract hourly data from weather array
        Ok(serde_json::to_value(&weather_data.weather)?)
    });

    registry.register("Weather.SetLocation", |params| async move {
        let city: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("city").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing city"))?,
        )?;

        storage::init().await?;
        storage::set_kv("weather", "location", &serde_json::json!(city)).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Weather.GetLocations", |_params| async move {
        storage::init().await?;
        let locations = storage::get_kv("weather", "saved_locations").await?
            .unwrap_or(serde_json::json!([]));
        Ok(locations)
    });

    registry.register("Weather.AddLocation", |params| async move {
        let city: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("city").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing city"))?,
        )?;

        storage::init().await?;
        let mut locations: Vec<String> = storage::get_kv("weather", "saved_locations").await?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        if !locations.contains(&city) {
            locations.push(city);
            storage::set_kv("weather", "saved_locations", &serde_json::to_value(&locations)?).await?;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Weather.RemoveLocation", |params| async move {
        let city: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("city").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing city"))?,
        )?;

        storage::init().await?;
        let mut locations: Vec<String> = storage::get_kv("weather", "saved_locations").await?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        locations.retain(|c| c != &city);
        storage::set_kv("weather", "saved_locations", &serde_json::to_value(&locations)?).await?;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Weather.GetAlerts", |_params| async move {
        // Weather alerts would require a different API
        Ok(serde_json::json!([]))
    });

    registry.register("Weather.SetUnits", |params| async move {
        let units: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("units").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing units"))?,
        )?;

        if units != "metric" && units != "imperial" {
            anyhow::bail!("Units must be 'metric' or 'imperial'");
        }

        storage::init().await?;
        storage::set_kv("weather", "units", &serde_json::json!(units)).await?;
        Ok(serde_json::json!({ "success": true }))
    });
}

fn weather_skip_cache() -> bool {
    std::env::var("AURA_WEATHER_SKIP_CACHE").ok().as_deref() == Some("1")
}

fn weather_rate_limit_secs() -> u64 {
    if std::env::var("AURA_WEATHER_API_KEY").is_ok() {
        30
    } else {
        WEATHER_RATE_LIMIT_SECS
    }
}

#[cfg(test)]
pub(crate) fn weather_cache_ttl_secs() -> u64 {
    WEATHER_CACHE_TTL_SECS
}

async fn fetch_and_cache_current(city_name: &str) -> anyhow::Result<serde_json::Value> {
    let weather_data = fetch_wttr_json(city_name).await?;
    let weather_json = current_weather_json(&weather_data.current_condition[0]);
    *LAST_FETCH.write().await = Some(std::time::Instant::now());
    *CACHED_WEATHER.write().await = Some(weather_json.clone());
    *LAST_UPDATE.write().await = Some(std::time::Instant::now());
    Ok(weather_json)
}

async fn resolve_city(explicit: Option<String>) -> anyhow::Result<String> {
    if let Some(c) = explicit {
        return Ok(c);
    }
    get_default_city()
        .await
        .ok_or_else(|| anyhow::anyhow!("Weather location unavailable (offline or unset)"))
}

/// wttr JSON fetch; integration tests set `AURA_WEATHER_WTTR_URL` to a mock HTTP endpoint.
async fn fetch_wttr_json(city_name: &str) -> anyhow::Result<WttrResponse> {
    let url = std::env::var("AURA_WEATHER_WTTR_URL")
        .unwrap_or_else(|_| format!("https://wttr.in/{}?format=j1", city_name));
    let mut req = http::client_with_timeout(http::request_timeout()).get(&url);
    if let Ok(key) = std::env::var("AURA_WEATHER_API_KEY") {
        req = req.header("Authorization", format!("Bearer {key}"));
    }
    let response = req.send().await?;
    Ok(response.json().await?)
}

async fn get_default_city() -> Option<String> {
    storage::init().await.ok();
    if let Ok(Some(city)) = storage::get_kv("weather", "location").await {
        if let Some(city_str) = city.as_str() {
            let trimmed = city_str.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    lookup_city_from_ip().await
}

async fn lookup_city_from_ip() -> Option<String> {
    let timeout = Duration::from_secs(2).min(http::request_timeout());
    let response = http::client_with_timeout(timeout)
        .get("https://ipinfo.io/json")
        .send()
        .await
        .ok()?;
    let json = response.json::<serde_json::Value>().await.ok()?;
    json.get("city")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

pub(crate) fn current_weather_json(cond: &CurrentCondition) -> serde_json::Value {
    let description = cond
        .weatherDesc
        .first()
        .map(|d| d.value.as_str())
        .unwrap_or("");
    let humidity = cond.humidity.parse::<i64>().unwrap_or(0);
    serde_json::json!({
        "temp": cond.temp_C,
        "temp_f": cond.temp_F,
        "feels_like": cond.FeelsLikeC,
        "description": description,
        "icon": get_weather_icon(&cond.weatherCode),
        "humidity": humidity,
        "code": cond.weatherCode,
    })
}

pub(crate) fn get_weather_icon(code: &str) -> String {
    // Simple mapping - can be enhanced
    match code {
        "113" => "sunny".to_string(),
        "116" => "partly_cloudy".to_string(),
        "119" | "122" => "cloudy".to_string(),
        "143" | "248" | "260" => "foggy".to_string(),
        "176" | "263" | "266" | "281" | "284" | "293" | "296" | "299" | "302" | "305" | "308" | "311" | "314" | "317" | "320" | "323" | "326" | "329" | "332" | "335" | "338" | "350" | "353" | "356" | "359" | "362" | "365" | "368" | "371" | "374" | "377" | "386" | "389" | "392" | "395" => "rainy".to_string(),
        "200" | "227" | "230" => "snowy".to_string(),
        _ => "cloud_alert".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_icon_codes() {
        assert_eq!(get_weather_icon("113"), "sunny");
        assert_eq!(get_weather_icon("116"), "partly_cloudy");
        assert_eq!(get_weather_icon("119"), "cloudy");
        assert_eq!(get_weather_icon("143"), "foggy");
        assert_eq!(get_weather_icon("176"), "rainy");
        assert_eq!(get_weather_icon("200"), "snowy");
        assert_eq!(get_weather_icon("999"), "cloud_alert");
    }

    #[test]
    fn weather_skip_cache_and_rate_limit_env() {
        std::env::set_var("AURA_WEATHER_SKIP_CACHE", "1");
        assert!(weather_skip_cache());
        std::env::remove_var("AURA_WEATHER_SKIP_CACHE");
        assert!(!weather_skip_cache());

        std::env::set_var("AURA_WEATHER_API_KEY", "test-key");
        assert_eq!(weather_rate_limit_secs(), 30);
        std::env::remove_var("AURA_WEATHER_API_KEY");
        assert_eq!(weather_rate_limit_secs(), WEATHER_RATE_LIMIT_SECS);
        assert_eq!(weather_cache_ttl_secs(), 900);
    }

    #[test]
    fn current_weather_json_branches() {
        let full = CurrentCondition {
            temp_C: "18".into(),
            temp_F: "64".into(),
            FeelsLikeC: "17".into(),
            FeelsLikeF: "63".into(),
            humidity: "72".into(),
            weatherDesc: vec![WeatherDesc {
                value: "Partly cloudy".into(),
            }],
            weatherCode: "116".into(),
        };
        let json = current_weather_json(&full);
        assert_eq!(json["icon"], "partly_cloudy");
        assert_eq!(json["humidity"], 72);
        assert_eq!(json["description"], "Partly cloudy");

        let no_desc = CurrentCondition {
            weatherDesc: vec![],
            humidity: "not-a-number".into(),
            weatherCode: "113".into(),
            temp_C: "1".into(),
            temp_F: "34".into(),
            FeelsLikeC: "0".into(),
            FeelsLikeF: "32".into(),
        };
        let sparse = current_weather_json(&no_desc);
        assert_eq!(sparse["description"], "");
        assert_eq!(sparse["humidity"], 0);
        assert_eq!(sparse["icon"], "sunny");
    }

    #[test]
    fn parse_wttr_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/weather/wttr_current.json"
        );
        let text = std::fs::read_to_string(path).expect("fixture");
        let data: WttrResponse = serde_json::from_str(&text).expect("json");
        let json = current_weather_json(&data.current_condition[0]);
        assert_eq!(json["icon"], "partly_cloudy");
        assert_eq!(json["humidity"], 72);
        assert!(json["temp"].as_str().unwrap().contains("18"));
    }

    #[test]
    fn parse_wttr_forecast_fixture_has_weather_days() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/weather/wttr_forecast.json"
        );
        let text = std::fs::read_to_string(path).expect("fixture");
        let data: WttrResponse = serde_json::from_str(&text).expect("json");
        assert_eq!(data.weather.len(), 3);
        assert_eq!(data.current_condition[0].weatherCode, "113");
    }
}
