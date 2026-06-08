use crate::services::ServiceRegistry;
use crate::utils::process;

pub(crate) fn parse_sublist3r_output(output: &str, domain: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('[')
                || !trimmed.contains(domain)
            {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

pub(crate) fn parse_dnsrecon_bruteforce(output: &str, domain: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            if !line.contains(domain) {
                return None;
            }
            let host = line
                .split(" A ")
                .next()
                .and_then(|prefix| prefix.split_whitespace().last())
                .filter(|token| token.contains(domain))?;
            Some(host.to_string())
        })
        .collect()
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.Subdomain.Enumerate", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let tools: Option<Vec<String>> = params
            .as_ref()
            .and_then(|p| p.get("tools").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut subdomains = Vec::new();

        if tools.as_ref().map(|t| t.contains(&"sublist3r".to_string())).unwrap_or(true) {
            if let Ok(output) = process::exec_command(&["sublist3r", "-d", &domain]).await {
                subdomains.extend(parse_sublist3r_output(&output, &domain));
            }
        }

        if tools.as_ref().map(|t| t.contains(&"amass".to_string())).unwrap_or(false) {
            if let Ok(output) = process::exec_command(&["amass", "enum", "-d", &domain]).await {
                for line in output.lines() {
                    if !line.trim().is_empty() {
                        subdomains.push(line.trim().to_string());
                    }
                }
            }
        }

        Ok(serde_json::to_value(&subdomains)?)
    });

    registry.register("Security.Offensive.Subdomain.BruteForce", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let wordlist: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("wordlist").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing wordlist"))?,
        )?;

        let output = process::exec_command(&["dnsrecon", "-d", &domain, "-D", &wordlist, "-t", "brt"]).await?;
        Ok(serde_json::to_value(&parse_dnsrecon_bruteforce(&output, &domain))?)
    });

    registry.register("Security.Offensive.Subdomain.CertificateTransparency", |params| async move {
        let domain: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("domain").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing domain"))?,
        )?;

        let url = format!("https://crt.sh/?q={}&output=json", domain);
        let response = reqwest::get(&url).await?;
        let json: Vec<serde_json::Value> = response.json().await?;

        let mut subdomains = Vec::new();
        for entry in json {
            if let Some(name) = entry.get("name_value").and_then(|v| v.as_str()) {
                subdomains.push(name.to_string());
            }
        }

        Ok(serde_json::to_value(&subdomains)?)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/security/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read_to_string(&path).expect("fixture")
    }

    #[test]
    fn parse_sublist3r_output_filters_domain_lines() {
        let output = fixture("sublist3r_sample.txt");
        let subs = parse_sublist3r_output(&output, "example.com");
        assert_eq!(subs.len(), 2);
        assert!(subs.iter().all(|s| s.contains("example.com")));
    }

    #[test]
    fn parse_dnsrecon_bruteforce_extracts_hostnames() {
        let output = fixture("dnsrecon_brute_sample.txt");
        let subs = parse_dnsrecon_bruteforce(&output, "example.com");
        assert_eq!(subs, vec!["www.example.com", "api.example.com"]);
    }
}
