use crate::services::ServiceRegistry;
use crate::utils::process;

pub(crate) fn osint_email_response(email: &str, breaches: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "email": email,
        "breaches": breaches
    })
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.OSINT.Email", |params| async move {
        let email: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("email").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing email"))?,
        )?;

        let url = format!("https://haveibeenpwned.com/api/v3/breachedaccount/{}", email);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.ok();

        let breaches = if let Some(resp) = response {
            if resp.status().is_success() {
                resp.json::<Vec<serde_json::Value>>().await.unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(osint_email_response(&email, breaches))
    });

    registry.register("Security.Offensive.OSINT.IP", |params| async move {
        let ip: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("ip").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing ip"))?,
        )?;

        let url = format!("https://ipinfo.io/{}/json", ip);
        let response = reqwest::get(&url).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });

    registry.register("Security.Offensive.OSINT.Domain", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let whois_output = process::exec_command(&["whois", &domain]).await?;
        Ok(serde_json::json!({ "whois": whois_output }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osint_email_response_shape() {
        let breaches = vec![serde_json::json!({ "Name": "TestBreach" })];
        let result = osint_email_response("user@example.com", breaches.clone());
        assert_eq!(result["email"], "user@example.com");
        assert_eq!(result["breaches"].as_array().unwrap().len(), 1);
        assert_eq!(result["breaches"][0]["Name"], "TestBreach");
    }

    #[test]
    fn osint_email_response_empty_breaches() {
        let result = osint_email_response("nobody@example.com", vec![]);
        assert!(result["breaches"].as_array().unwrap().is_empty());
    }
}
