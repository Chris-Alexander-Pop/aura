use crate::services::ServiceRegistry;
use crate::utils::storage;
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Deserialize)]
struct WttrResponse {
    current_condition: Vec<CurrentCondition>,
    weather: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CurrentCondition {
    temp_C: String,
    temp_F: String,
    FeelsLikeC: String,
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
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Weather.Get", |params| async move {
        let city: Option<String> = params
            .and_then(|p| p.get("city").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        // Check cache (15 minute TTL)
        let last_update = LAST_UPDATE.read().await;
        if let Some(instant) = *last_update {
            if instant.elapsed().as_secs() < 900 {
                if let Some(cached) = CACHED_WEATHER.read().await.as_ref() {
                    return Ok(cached.clone());
                }
            }
        }
        drop(last_update);

        // Determine city
        let city_name = if let Some(c) = city {
            c
        } else {
            // Try to get city from IP
            let ip_response = reqwest::get("https://ipinfo.io/json").await.ok();
            if let Some(response) = ip_response {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(c) = json.get("city").and_then(|v| v.as_str()) {
                        c.to_string()
                    } else {
                        "London".to_string() // Default fallback
                    }
                } else {
                    "London".to_string()
                }
            } else {
                "London".to_string()
            }
        };

        // Fetch weather
        let url = format!("https://wttr.in/{}?format=j1", city_name);
        let response = reqwest::get(&url).await?;
        let weather_data: WttrResponse = response.json().await?;

        let cc = &weather_data.current_condition[0];
        let weather_json = serde_json::json!({
            "temp": format!("{}°C", cc.temp_C),
            "feels_like": format!("{}°C", cc.FeelsLikeC),
            "humidity": cc.humidity.parse::<i32>().unwrap_or(0),
            "description": cc.weatherDesc.get(0).map(|d| d.value.clone()).unwrap_or_else(|| "Unknown".to_string()),
            "icon": get_weather_icon(&cc.weatherCode),
        });

        // Update cache
        *CACHED_WEATHER.write().await = Some(weather_json.clone());
        *LAST_UPDATE.write().await = Some(std::time::Instant::now());

        Ok(weather_json)
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

        let city_name = if let Some(c) = city {
            c
        } else {
            get_default_city().await
        };
        let url = format!("https://wttr.in/{}?format=j1", city_name);
        let response = reqwest::get(&url).await?;
        let weather_data: WttrResponse = response.json().await?;

        // Extract forecast data
        let forecast = weather_data.weather.iter().take(days as usize).collect::<Vec<_>>();
        Ok(serde_json::to_value(&forecast)?)
    });

    registry.register("Weather.GetHourly", |_params| async move {
        let city = get_default_city().await;
        let url = format!("https://wttr.in/{}?format=j1", city);
        let response = reqwest::get(&url).await?;
        let weather_data: WttrResponse = response.json().await?;

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

async fn get_default_city() -> String {
    storage::init().await.ok();
    if let Ok(Some(city)) = storage::get_kv("weather", "location").await {
        if let Some(city_str) = city.as_str() {
            return city_str.to_string();
        }
    }

    // Fallback to IP-based detection
    if let Ok(response) = reqwest::get("https://ipinfo.io/json").await {
        if let Ok(json) = response.json::<serde_json::Value>().await {
            if let Some(c) = json.get("city").and_then(|v| v.as_str()) {
                return c.to_string();
            }
        }
    }

    "London".to_string()
}

fn get_weather_icon(code: &str) -> String {
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
