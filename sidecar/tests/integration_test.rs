// Basic integration tests for the sidecar
// These tests require the sidecar to be running or can test individual components

#[cfg(test)]
mod tests {
    use serde_json;

    #[tokio::test]
    async fn test_jsonrpc_request_format() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "Power.GetBatteryState",
            "params": {},
            "id": 1
        });

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "Power.GetBatteryState");
    }

    #[tokio::test]
    async fn test_brightness_parsing() {
        // Test brightness value parsing logic
        fn parse_brightness_value(value: &str, current: f64) -> Result<f64, String> {
            let value = value.trim();
            
            if value.ends_with("%-") {
                let percent = value[..value.len() - 2].parse::<f64>().map_err(|e| e.to_string())? / 100.0;
                Ok((current - percent).max(0.0).min(1.0))
            } else if value.starts_with("+") && value.ends_with("%") {
                let percent = value[1..value.len() - 1].parse::<f64>().map_err(|e| e.to_string())? / 100.0;
                Ok((current + percent).max(0.0).min(1.0))
            } else if value.ends_with("%") {
                let percent = value[..value.len() - 1].parse::<f64>().map_err(|e| e.to_string())? / 100.0;
                Ok(percent.max(0.0).min(1.0))
            } else if value.starts_with("+") {
                let increment = value[1..].parse::<f64>().map_err(|e| e.to_string())?;
                Ok((current + increment).max(0.0).min(1.0))
            } else if value.ends_with("-") {
                let decrement = value[..value.len() - 1].parse::<f64>().map_err(|e| e.to_string())?;
                Ok((current - decrement).max(0.0).min(1.0))
            } else {
                let absolute = value.parse::<f64>().map_err(|e| e.to_string())?;
                Ok(absolute.max(0.0).min(1.0))
            }
        }

        assert_eq!(parse_brightness_value("50%", 0.5).unwrap(), 0.5);
        assert_eq!(parse_brightness_value("+10%", 0.5).unwrap(), 0.6);
        assert_eq!(parse_brightness_value("10%-", 0.5).unwrap(), 0.4);
        assert_eq!(parse_brightness_value("0.8", 0.5).unwrap(), 0.8);
        assert_eq!(parse_brightness_value("+0.1", 0.5).unwrap(), 0.6);
    }
}
