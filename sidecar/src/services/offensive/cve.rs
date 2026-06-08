use crate::services::ServiceRegistry;

pub(crate) fn cve_api_url(cve_id: &str) -> String {
    format!("https://cve.circl.lu/api/cve/{}", cve_id)
}

pub(crate) fn cve_search_url(product: &str, version: Option<&str>) -> String {
    let query = if let Some(v) = version {
        format!("{} {}", product, v)
    } else {
        product.to_string()
    };
    format!("https://cve.circl.lu/api/search/{}", query)
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.CVE.Search", |params| async move {
        let cve_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("cve_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing cve_id"))?,
        )?;

        let response = reqwest::get(&cve_api_url(&cve_id)).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });

    registry.register("Security.Offensive.CVE.SearchByProduct", |params| async move {
        let product: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("product").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing product"))?,
        )?;

        let version: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("version").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let response = reqwest::get(&cve_search_url(&product, version.as_deref())).await?;
        let json: serde_json::Value = response.json().await?;

        Ok(json)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cve_api_url_includes_id() {
        assert_eq!(
            cve_api_url("CVE-2024-1234"),
            "https://cve.circl.lu/api/cve/CVE-2024-1234"
        );
    }

    #[test]
    fn cve_search_url_with_and_without_version() {
        assert_eq!(
            cve_search_url("openssl", None),
            "https://cve.circl.lu/api/search/openssl"
        );
        assert_eq!(
            cve_search_url("openssl", Some("1.1.1")),
            "https://cve.circl.lu/api/search/openssl 1.1.1"
        );
    }
}
