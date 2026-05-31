//! CalDAV read-only pull (fixtures + optional live URL). Conflict policy: last-write-wins on import.

use crate::services::ics::{self, ParsedVevent};
use crate::utils::{keyring, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalDavConfig {
    pub url: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub calendar_id: Option<String>,
}

pub async fn load_caldav_config() -> Result<Option<CalDavConfig>> {
    if let Ok(url) = std::env::var("AURA_CALDAV_URL") {
        return Ok(Some(CalDavConfig {
            url,
            username: std::env::var("AURA_CALDAV_USER").ok(),
            calendar_id: std::env::var("AURA_CALDAV_CALENDAR_ID").ok(),
        }));
    }
    storage::init().await?;
    storage::get_kv("calendar", "caldav_config")
        .await?
        .map(|v| serde_json::from_value(v))
        .transpose()
        .map_err(Into::into)
}

/// Parse minimal CalDAV `calendar-data` ICS bodies from a REPORT multistatus XML fixture.
pub fn parse_caldav_report_events(xml: &str) -> Result<Vec<ParsedVevent>> {
    let mut events = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = xml[search_from..].find("BEGIN:VCALENDAR") {
        let begin = search_from + rel;
        let Some(rel_end) = xml[begin..].find("END:VCALENDAR") else {
            break;
        };
        let end = begin + rel_end + "END:VCALENDAR".len();
        let chunk = &xml[begin..end];
        events.extend(ics::parse_ics(chunk)?);
        search_from = end;
    }
    Ok(events)
}

pub async fn sync_caldav_read_only() -> Result<Value> {
    let Some(config) = load_caldav_config().await? else {
        return Ok(serde_json::json!({
            "success": true,
            "synced": 0,
            "message": "no CalDAV URL configured"
        }));
    };

    let body = if let Ok(fixture) = std::env::var("AURA_CALDAV_FIXTURE") {
        tokio::fs::read_to_string(&fixture).await?
    } else {
        let client = reqwest::Client::new();
        let mut req = client
            .request(reqwest::Method::POST, &config.url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(minimal_calendar_query_body());

        if let Some(user) = &config.username {
            if let Some(pass) = keyring::lookup_caldav_password(user).await? {
                req = req.basic_auth(user, Some(pass));
            } else if let Ok(token) = std::env::var("AURA_CALDAV_TOKEN") {
                req = req.bearer_auth(token);
            }
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("CalDAV REPORT failed: {}", resp.status());
        }
        resp.text().await?
    };

    let parsed = parse_caldav_report_events(&body)?;
    let calendar_id = config.calendar_id.clone().unwrap_or_else(|| "caldav".into());
    let imported = ics::import_events_to_storage(&parsed, Some(calendar_id)).await?;

    Ok(serde_json::json!({
        "success": true,
        "synced": imported,
        "policy": "last-write-wins"
    }))
}

fn minimal_calendar_query_body() -> String {
    r#"<?xml version="1.0" encoding="utf-8" ?>
<C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop>
    <D:getetag/>
    <C:calendar-data/>
  </D:prop>
  <C:filter>
    <C:comp-filter name="VCALENDAR">
      <C:comp-filter name="VEVENT"/>
    </C:comp-filter>
  </C:filter>
</C:calendar-query>"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caldav_fixture_report() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/caldav/report_multistatus.xml"
        );
        let xml = std::fs::read_to_string(path).expect("fixture");
        let events = parse_caldav_report_events(&xml).expect("parse");
        assert!(!events.is_empty());
        assert!(!events[0].summary.is_empty());
    }
}
