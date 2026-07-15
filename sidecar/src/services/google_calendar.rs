//! Google Calendar OAuth2 (PKCE + loopback) and read-only event sync.

use crate::services::calendar::{self, Calendar, CalendarEvent};
use crate::utils::{keyring, storage};
use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const AUTH_META_NS: &str = "calendar";
const AUTH_META_KEY: &str = "google_auth_meta";
const CALENDARS_CACHE_KEY: &str = "google_calendars_cache";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const CALENDAR_LIST_URL: &str = "https://www.googleapis.com/calendar/v3/users/me/calendarList";
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/userinfo.email";
const AUTH_TIMEOUT_SECS: u64 = 180;
static AUTH_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref ACCESS_TOKEN_CACHE: Mutex<Option<CachedAccessToken>> = Mutex::new(None);
}

#[derive(Debug, Clone)]
struct CachedAccessToken {
    token: String,
    expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleAuthMeta {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub last_sync: Option<i64>,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    token_type: Option<String>,
}

/// Build PKCE verifier + S256 challenge (URL-safe base64, no padding).
pub fn generate_pkce_pair() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = pkce_challenge(&verifier);
    (verifier, challenge)
}

pub fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub fn build_auth_url(
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> String {
    format!(
        "{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
        urlencoding_encode(client_id),
        urlencoding_encode(redirect_uri),
        urlencoding_encode(SCOPE),
        urlencoding_encode(code_challenge),
        urlencoding_encode(state),
    )
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn oauth_client_id() -> Option<String> {
    std::env::var("AURA_GOOGLE_OAUTH_CLIENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn oauth_client_secret() -> Option<String> {
    std::env::var("AURA_GOOGLE_OAUTH_CLIENT_SECRET")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn load_meta() -> Result<GoogleAuthMeta> {
    storage::init().await?;
    Ok(storage::get_kv(AUTH_META_NS, AUTH_META_KEY)
        .await?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

async fn save_meta(meta: &GoogleAuthMeta) -> Result<()> {
    storage::init().await?;
    storage::set_kv(AUTH_META_NS, AUTH_META_KEY, &serde_json::to_value(meta)?).await?;
    Ok(())
}

pub async fn auth_status() -> Result<Value> {
    let configured = oauth_client_id().is_some();
    let refresh = keyring::lookup_google_oauth_refresh().await?.is_some();
    let meta = load_meta().await.unwrap_or_default();
    Ok(json!({
        "connected": refresh,
        "configured": configured,
        "pending": meta.pending,
        "email": meta.email,
        "last_sync": meta.last_sync,
        "error": meta.error,
    }))
}

/// Kick off browser OAuth in the background; returns immediately.
pub async fn start_auth() -> Result<Value> {
    if oauth_client_id().is_none() {
        bail!("AURA_GOOGLE_OAUTH_CLIENT_ID is not set");
    }
    if AUTH_IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return Ok(json!({
            "started": true,
            "message": "auth already in progress"
        }));
    }

    let mut meta = load_meta().await.unwrap_or_default();
    meta.pending = true;
    meta.error = None;
    save_meta(&meta).await?;

    tokio::spawn(async {
        let result = run_oauth_flow().await;
        AUTH_IN_FLIGHT.store(false, Ordering::SeqCst);
        let mut meta = load_meta().await.unwrap_or_default();
        meta.pending = false;
        match result {
            Ok(email) => {
                meta.email = email;
                meta.error = None;
            }
            Err(e) => {
                meta.error = Some(e.to_string());
            }
        }
        let _ = save_meta(&meta).await;
    });

    Ok(json!({
        "started": true,
        "message": "complete sign-in in your browser"
    }))
}

async fn run_oauth_flow() -> Result<Option<String>> {
    let client_id = oauth_client_id().ok_or_else(|| anyhow!("missing client id"))?;
    let (verifier, challenge) = generate_pkce_pair();
    let mut state_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");
    let auth_url = build_auth_url(&client_id, &redirect_uri, &challenge, &state);

    open_browser(&auth_url).await?;

    let code = tokio::time::timeout(
        Duration::from_secs(AUTH_TIMEOUT_SECS),
        wait_for_oauth_code(listener, &state),
    )
    .await
    .map_err(|_| anyhow!("timed out waiting for Google sign-in"))??;

    let tokens = exchange_code(&client_id, &redirect_uri, &code, &verifier).await?;
    let refresh = tokens
        .refresh_token
        .ok_or_else(|| anyhow!("Google did not return a refresh token; revoke Aura access and retry"))?;
    keyring::store_google_oauth_refresh(&refresh).await?;

    *ACCESS_TOKEN_CACHE.lock().await = Some(CachedAccessToken {
        token: tokens.access_token.clone(),
        expires_at: chrono::Utc::now().timestamp() + tokens.expires_in.unwrap_or(3600) - 60,
    });

    let email = fetch_user_email(&tokens.access_token).await.ok();
    Ok(email)
}

async fn open_browser(url: &str) -> Result<()> {
    let status = tokio::process::Command::new("xdg-open")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            tracing::info!("open this URL to authorize Google Calendar: {url}");
            Ok(())
        }
    }
}

async fn wait_for_oauth_code(listener: TcpListener, expected_state: &str) -> Result<String> {
    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut buf = vec![0u8; 8192];
        let n = socket.read(&mut buf).await.unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req.lines().next().unwrap_or("");
        if !path.contains("/oauth/callback") {
            let _ = write_oauth_response(&mut socket, 404, "Not found").await;
            continue;
        }
        let query = path
            .split_whitespace()
            .nth(1)
            .and_then(|p| p.split('?').nth(1))
            .unwrap_or("");
        let params = parse_query(query);
        if let Some(err) = params.get("error") {
            let _ = write_oauth_response(
                &mut socket,
                400,
                &format!("Authorization failed: {err}. You can close this tab."),
            )
            .await;
            bail!("Google OAuth error: {err}");
        }
        let state = params.get("state").map(String::as_str).unwrap_or("");
        if state != expected_state {
            let _ = write_oauth_response(&mut socket, 400, "Invalid state. Close this tab.").await;
            bail!("OAuth state mismatch");
        }
        let code = params
            .get("code")
            .cloned()
            .ok_or_else(|| anyhow!("missing code in OAuth callback"))?;
        let _ = write_oauth_response(
            &mut socket,
            200,
            "Aura is connected to Google Calendar. You can close this tab.",
        )
        .await;
        return Ok(code);
    }
}

async fn write_oauth_response(
    socket: &mut tokio::net::TcpStream,
    status: u16,
    body: &str,
) -> Result<()> {
    let reason = if status == 200 { "OK" } else { "Error" };
    let html = format!(
        "<!DOCTYPE html><html><body style=\"font-family:system-ui;padding:2rem\"><p>{body}</p></body></html>"
    );
    let resp = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );
    socket.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn parse_query(q: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in q.split('&') {
        if part.is_empty() {
            continue;
        }
        let mut it = part.splitn(2, '=');
        let k = it.next().unwrap_or("");
        let v = it.next().unwrap_or("");
        map.insert(
            percent_decode(k),
            percent_decode(v),
        );
    }
    map
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

async fn exchange_code(
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> Result<TokenResponse> {
    let client = reqwest::Client::new();
    let mut form = vec![
        ("code", code.to_string()),
        ("client_id", client_id.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("grant_type", "authorization_code".into()),
        ("code_verifier", verifier.to_string()),
    ];
    if let Some(secret) = oauth_client_secret() {
        form.push(("client_secret", secret));
    }
    let resp = client.post(TOKEN_URL).form(&form).send().await?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        bail!("token exchange failed: {text}");
    }
    Ok(resp.json().await?)
}

async fn refresh_access_token() -> Result<String> {
    {
        let cache = ACCESS_TOKEN_CACHE.lock().await;
        if let Some(c) = cache.as_ref() {
            if c.expires_at > chrono::Utc::now().timestamp() {
                return Ok(c.token.clone());
            }
        }
    }

    let client_id = oauth_client_id().ok_or_else(|| anyhow!("missing client id"))?;
    let refresh = keyring::lookup_google_oauth_refresh()
        .await?
        .ok_or_else(|| anyhow!("not connected to Google"))?;

    let client = reqwest::Client::new();
    let mut form = vec![
        ("client_id", client_id),
        ("grant_type", "refresh_token".into()),
        ("refresh_token", refresh),
    ];
    if let Some(secret) = oauth_client_secret() {
        form.push(("client_secret", secret));
    }
    let resp = client.post(TOKEN_URL).form(&form).send().await?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        bail!("token refresh failed: {text}");
    }
    let tokens: TokenResponse = resp.json().await?;
    *ACCESS_TOKEN_CACHE.lock().await = Some(CachedAccessToken {
        token: tokens.access_token.clone(),
        expires_at: chrono::Utc::now().timestamp() + tokens.expires_in.unwrap_or(3600) - 60,
    });
    Ok(tokens.access_token)
}

async fn fetch_user_email(access_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        bail!("userinfo failed: {}", resp.status());
    }
    let v: Value = resp.json().await?;
    v.get("email")
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("no email in userinfo"))
}

pub async fn disconnect() -> Result<Value> {
    keyring::clear_google_oauth_refresh().await?;
    *ACCESS_TOKEN_CACHE.lock().await = None;
    AUTH_IN_FLIGHT.store(false, Ordering::SeqCst);
    storage::init().await?;
    let _ = storage::delete_kv(AUTH_META_NS, AUTH_META_KEY).await;
    let _ = storage::delete_kv(AUTH_META_NS, CALENDARS_CACHE_KEY).await;
    // Remove previously synced Google events
    if let Ok(keys) = storage::list_keys("calendar_events").await {
        for key in keys {
            if key.starts_with("gcal_") {
                let _ = storage::delete_kv("calendar_events", &key).await;
            }
        }
    }
    calendar::schedule_calendar_events_emit_public("google_disconnect");
    Ok(json!({ "success": true }))
}

pub async fn list_calendars() -> Result<Vec<Calendar>> {
    if keyring::lookup_google_oauth_refresh().await?.is_none() {
        return Ok(vec![]);
    }
    if let Ok(Some(cached)) = storage::get_kv(AUTH_META_NS, CALENDARS_CACHE_KEY).await {
        if let Ok(list) = serde_json::from_value::<Vec<Calendar>>(cached) {
            if !list.is_empty() {
                return Ok(list);
            }
        }
    }
    let token = refresh_access_token().await?;
    fetch_and_cache_calendars(&token).await
}

async fn fetch_and_cache_calendars(access_token: &str) -> Result<Vec<Calendar>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(CALENDAR_LIST_URL)
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        bail!("calendarList failed: {}", resp.status());
    }
    let body: Value = resp.json().await?;
    let list = parse_calendar_list(&body);
    storage::init().await?;
    storage::set_kv(
        AUTH_META_NS,
        CALENDARS_CACHE_KEY,
        &serde_json::to_value(&list)?,
    )
    .await?;
    Ok(list)
}

/// Parse Google `calendarList` JSON into Aura [`Calendar`] rows.
pub fn parse_calendar_list(body: &Value) -> Vec<Calendar> {
    let mut out = Vec::new();
    let Some(items) = body.get("items").and_then(|i| i.as_array()) else {
        return out;
    };
    for item in items {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        let name = item
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let color = item
            .get("backgroundColor")
            .and_then(|v| v.as_str())
            .unwrap_or("#cba6f7")
            .to_string();
        out.push(Calendar { id, name, color });
    }
    out
}

/// Parse a Google Calendar `events.list` item into a local [`CalendarEvent`].
pub fn parse_google_event(item: &Value, calendar_id: &str) -> Option<CalendarEvent> {
    let id = item.get("id")?.as_str()?;
    let title = item
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("(No title)")
        .to_string();
    let description = item
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let (start, end) = parse_event_times(item)?;
    Some(CalendarEvent {
        id: format!("gcal_{id}"),
        title,
        start,
        end,
        description,
        calendar_id: Some(calendar_id.to_string()),
        reminder_minutes: None,
    })
}

fn parse_event_times(item: &Value) -> Option<(i64, i64)> {
    let start_obj = item.get("start")?;
    let end_obj = item.get("end")?;
    let start = parse_google_time(start_obj)?;
    let end = parse_google_time(end_obj).unwrap_or(start + 3600);
    Some((start, end))
}

/// Parse Google event `date` (all-day) or `dateTime` into unix seconds.
pub fn parse_google_time(obj: &Value) -> Option<i64> {
    if let Some(dt) = obj.get("dateTime").and_then(|v| v.as_str()) {
        return chrono::DateTime::parse_from_rfc3339(dt)
            .ok()
            .map(|d| d.timestamp());
    }
    if let Some(d) = obj.get("date").and_then(|v| v.as_str()) {
        // All-day: treat as local midnight UTC date
        let naive = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?;
        let dt = naive.and_hms_opt(0, 0, 0)?.and_utc();
        return Some(dt.timestamp());
    }
    None
}

pub async fn sync_google_read_only() -> Result<Value> {
    if keyring::lookup_google_oauth_refresh().await?.is_none() {
        return Ok(json!({
            "success": true,
            "synced": 0,
            "provider": "none",
            "message": "Google not connected"
        }));
    }

    let token = refresh_access_token().await.context("refresh access token")?;
    let calendars = fetch_and_cache_calendars(&token).await?;
    let now = chrono::Utc::now();
    let time_min = (now - chrono::Duration::days(30)).to_rfc3339();
    let time_max = (now + chrono::Duration::days(90)).to_rfc3339();

    let client = reqwest::Client::new();
    let mut synced = 0usize;
    storage::init().await?;

    for cal in &calendars {
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            urlencoding_encode(&cal.id)
        );
        let mut page_token: Option<String> = None;
        loop {
            let mut req = client
                .get(&url)
                .bearer_auth(&token)
                .query(&[
                    ("singleEvents", "true"),
                    ("orderBy", "startTime"),
                    ("timeMin", time_min.as_str()),
                    ("timeMax", time_max.as_str()),
                    ("maxResults", "250"),
                ]);
            if let Some(pt) = &page_token {
                req = req.query(&[("pageToken", pt.as_str())]);
            }
            let resp = req.send().await?;
            if !resp.status().is_success() {
                tracing::warn!("events.list failed for {}: {}", cal.id, resp.status());
                break;
            }
            let body: Value = resp.json().await?;
            if let Some(items) = body.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    if let Some(ev) = parse_google_event(item, &cal.id) {
                        storage::set_kv(
                            "calendar_events",
                            &ev.id,
                            &serde_json::to_value(&ev)?,
                        )
                        .await?;
                        synced += 1;
                    }
                }
            }
            page_token = body
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if page_token.is_none() {
                break;
            }
        }
    }

    let mut meta = load_meta().await.unwrap_or_default();
    meta.last_sync = Some(chrono::Utc::now().timestamp());
    meta.error = None;
    save_meta(&meta).await?;
    calendar::schedule_calendar_events_emit_public("google_sync");

    Ok(json!({
        "success": true,
        "synced": synced,
        "provider": "google",
        "calendars": calendars.len(),
        "policy": "last-write-wins"
    }))
}

/// Prefer Google when connected; otherwise fall back to CalDAV.
pub async fn sync_calendars() -> Result<Value> {
    if keyring::lookup_google_oauth_refresh().await?.is_some() {
        return sync_google_read_only().await;
    }
    crate::services::caldav::sync_caldav_read_only().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_stable() {
        let challenge = pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk");
        // Known S256 challenge for the RFC 7636 appendix B verifier
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn auth_url_contains_pkce_params() {
        let url = build_auth_url(
            "client.apps.googleusercontent.com",
            "http://127.0.0.1:1234/oauth/callback",
            "challenge",
            "state123",
        );
        assert!(url.contains("code_challenge=challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("access_type=offline"));
    }

    #[test]
    fn parse_calendar_list_fixture() {
        let body: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/google/calendar_list.json"
        ))
        .unwrap();
        let list = parse_calendar_list(&body);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "primary@example.com");
        assert_eq!(list[0].name, "Personal");
    }

    #[test]
    fn parse_google_event_fixture() {
        let body: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/google/events_list.json"))
                .unwrap();
        let items = body.get("items").unwrap().as_array().unwrap();
        let ev = parse_google_event(&items[0], "primary@example.com").unwrap();
        assert_eq!(ev.id, "gcal_evt1");
        assert_eq!(ev.title, "Standup");
        assert!(ev.start > 0);
        assert!(ev.end > ev.start);
    }

    #[test]
    fn parse_google_all_day() {
        let obj = json!({ "date": "2026-07-15" });
        let ts = parse_google_time(&obj).unwrap();
        assert!(ts > 0);
    }
}
