//! App launcher: desktop entry index, fuzzy query, allowlisted launch, recent/pin prefs.

use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const LAUNCHER_NS: &str = "launcher";
const RECENT_KEY: &str = "recent_ids";
const PINS_KEY: &str = "pins";
const MAX_RECENT: usize = 30;
const MAX_PINS: usize = 64;
const DEFAULT_QUERY_LIMIT: usize = 50;
const MAX_QUERY_LIMIT: usize = 100;

lazy_static::lazy_static! {
    static ref DESKTOP_INDEX: Mutex<DesktopIndexCache> = Mutex::new(DesktopIndexCache::default());
}

#[derive(Default)]
struct DesktopIndexCache {
    dirs_fingerprint: String,
    entries: Vec<DesktopEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LauncherResult {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    generic_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    categories: Vec<String>,
    pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    score: Option<i32>,
}

fn desktop_dirs_fingerprint(dirs: &[PathBuf]) -> String {
    dirs.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\0")
}

/// Colon-separated override (`AURA_LAUNCHER_DESKTOP_DIRS`), else XDG application dirs.
pub fn desktop_search_dirs() -> Vec<PathBuf> {
    if let Ok(raw) = std::env::var("AURA_LAUNCHER_DESKTOP_DIRS") {
        return raw
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect();
    }

    let mut dirs = Vec::new();
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        dirs.push(Path::new(&data_home).join("applications"));
    } else if let Ok(home) = std::env::var("HOME") {
        dirs.push(Path::new(&home).join(".local/share/applications"));
    }
    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for base in data_dirs.split(':').filter(|s| !s.is_empty()) {
            dirs.push(Path::new(base).join("applications"));
        }
    }
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));
    dirs.sort();
    dirs.dedup();
    dirs
}

/// Clear cached index (tests and `AURA_LAUNCHER_DESKTOP_DIRS` changes).
pub fn invalidate_desktop_index_cache() {
    let mut cache = DESKTOP_INDEX.lock().expect("desktop index lock");
    cache.dirs_fingerprint.clear();
    cache.entries.clear();
}

fn load_index() -> Vec<DesktopEntry> {
    let dirs = desktop_search_dirs();
    let fp = desktop_dirs_fingerprint(&dirs);
    let mut cache = DESKTOP_INDEX.lock().expect("desktop index lock");
    if cache.dirs_fingerprint != fp {
        cache.entries = build_desktop_index(&dirs);
        cache.dirs_fingerprint = fp;
    }
    cache.entries.clone()
}

pub fn build_desktop_index(dirs: &[PathBuf]) -> Vec<DesktopEntry> {
    let mut by_id = std::collections::BTreeMap::<String, DesktopEntry>::new();
    for dir in dirs {
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            if let Some(parsed) = parse_desktop_file(&path) {
                by_id.insert(parsed.id.clone(), parsed);
            }
        }
    }
    by_id.into_values().collect()
}

/// Parse a freedesktop `.desktop` file (Application entries only).
pub fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let stem = path.file_stem()?.to_str()?;
    let content = std::fs::read_to_string(path).ok()?;
    let map = parse_desktop_ini(&content)?;
    let entry_type = map.get("Type").map(|s| s.as_str()).unwrap_or("Application");
    if entry_type != "Application" {
        return None;
    }
    if is_truthy(map.get("NoDisplay")) || is_truthy(map.get("Hidden")) {
        return None;
    }
    let name = map.get("Name")?.clone();
    if map.get("Exec").is_none() {
        return None;
    }
    let id = stem.to_string();
    Some(DesktopEntry {
        id,
        name,
        generic_name: map.get("GenericName").cloned(),
        comment: map.get("Comment").cloned(),
        icon: map.get("Icon").cloned(),
        categories: map
            .get("Categories")
            .map(|c| c.split(';').filter(|s| !s.is_empty()).map(str::to_owned).collect())
            .unwrap_or_default(),
        keywords: map
            .get("Keywords")
            .map(|c| c.split(';').filter(|s| !s.is_empty()).map(str::to_owned).collect())
            .unwrap_or_default(),
    })
}

fn parse_desktop_ini(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let mut in_desktop_entry = false;
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            in_desktop_entry = section == "Desktop Entry";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        if key.contains('[') {
            continue;
        }
        map.insert(key.trim().to_string(), unescape_desktop_value(value.trim()));
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn unescape_desktop_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('s') => out.push(' '),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_truthy(value: Option<&String>) -> bool {
    matches!(
        value.map(|s| s.as_str()),
        Some("true" | "True" | "1" | "yes" | "Yes")
    )
}

pub fn fuzzy_score(query: &str, entry: &DesktopEntry) -> Option<i32> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    let comment = entry.comment.as_deref().unwrap_or("");
    let generic = entry.generic_name.as_deref().unwrap_or("");
    let keywords = entry.keywords.join(" ");
    let fields = [
        entry.name.as_str(),
        generic,
        entry.id.as_str(),
        keywords.as_str(),
        comment,
    ];
    let mut score = 0i32;
    for token in q.split_whitespace() {
        let mut token_matched = false;
        for field in &fields {
            let f = field.to_lowercase();
            if f.starts_with(&token) {
                score += 100;
                token_matched = true;
                break;
            }
            if f.contains(&token) {
                score += 40;
                token_matched = true;
                break;
            }
        }
        if !token_matched {
            return None;
        }
    }
    Some(score)
}

fn validate_app_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 128 {
        bail!("invalid app id");
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        bail!("invalid app id");
    }
    Ok(())
}

async fn load_id_list(key: &str) -> Result<Vec<String>> {
    storage::init().await?;
    let Some(raw) = storage::get_kv(LAUNCHER_NS, key).await? else {
        return Ok(Vec::new());
    };
    let ids: Vec<String> = serde_json::from_value(raw)?;
    Ok(ids)
}

async fn save_id_list(key: &str, ids: Vec<String>) -> Result<()> {
    storage::init().await?;
    storage::set_kv(LAUNCHER_NS, key, &json!(ids)).await?;
    Ok(())
}

async fn load_pins() -> Result<HashSet<String>> {
    Ok(load_id_list(PINS_KEY).await?.into_iter().collect())
}

async fn touch_recent(id: &str) -> Result<()> {
    validate_app_id(id)?;
    let mut ids = load_id_list(RECENT_KEY).await?;
    ids.retain(|x| x != id);
    ids.insert(0, id.to_string());
    ids.truncate(MAX_RECENT);
    save_id_list(RECENT_KEY, ids).await
}

async fn set_pinned(id: &str, pinned: bool) -> Result<()> {
    validate_app_id(id)?;
    let mut ids: Vec<String> = load_id_list(PINS_KEY).await?;
    if pinned {
        ids.retain(|x| x != id);
        ids.insert(0, id.to_string());
        ids.truncate(MAX_PINS);
    } else {
        ids.retain(|x| x != id);
    }
    save_id_list(PINS_KEY, ids).await
}

fn entry_to_result(entry: &DesktopEntry, pinned: bool, score: Option<i32>) -> LauncherResult {
    LauncherResult {
        id: entry.id.clone(),
        name: entry.name.clone(),
        generic_name: entry.generic_name.clone(),
        comment: entry.comment.clone(),
        icon: entry.icon.clone(),
        categories: entry.categories.clone(),
        pinned,
        score,
    }
}

async fn query_entries_async(query: &str, limit: usize) -> Result<Vec<LauncherResult>> {
    let index = load_index();
    let pins = load_pins().await?;
    let q = query.trim();
    let mut scored: Vec<(i32, DesktopEntry)> = Vec::new();

    if q.is_empty() {
        for entry in index {
            scored.push((0, entry));
        }
        scored.sort_by(|a, b| {
            let ap = pins.contains(&a.1.id);
            let bp = pins.contains(&b.1.id);
            bp.cmp(&ap).then_with(|| a.1.name.cmp(&b.1.name))
        });
    } else {
        for entry in index {
            if let Some(s) = fuzzy_score(q, &entry) {
                scored.push((s, entry));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    }

    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(s, e)| {
            entry_to_result(
                &e,
                pins.contains(&e.id),
                if q.is_empty() { None } else { Some(s) },
            )
        })
        .collect())
}

async fn recent_items() -> Result<Vec<LauncherResult>> {
    let index = load_index();
    let by_id: std::collections::HashMap<_, _> = index.into_iter().map(|e| (e.id.clone(), e)).collect();
    let pins = load_pins().await?;
    let ids = load_id_list(RECENT_KEY).await?;
    let mut items = Vec::new();
    for id in ids {
        if let Some(entry) = by_id.get(&id) {
            items.push(entry_to_result(entry, pins.contains(&id), None));
        }
    }
    Ok(items)
}

async fn vicinae_query(socket_path: &str, query: &str, limit: usize) -> Result<Vec<LauncherResult>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let payload = json!({ "query": query, "limit": limit }).to_string() + "\n";
    let mut stream = UnixStream::connect(socket_path).await?;
    stream.write_all(payload.as_bytes()).await?;
    stream.shutdown().await?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    let response: serde_json::Value = serde_json::from_slice(&buf)?;
    let results_value = response
        .get("results")
        .cloned()
        .unwrap_or_else(|| json!([]));
    Ok(serde_json::from_value(results_value)?)
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Launcher.VicinaeQuery", |params| async move {
        let query = params
            .as_ref()
            .and_then(|p| p.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let limit = params
            .as_ref()
            .and_then(|p| p.get("limit"))
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_QUERY_LIMIT)
            .clamp(1, MAX_QUERY_LIMIT);

        if let Ok(socket_path) = std::env::var("VICINAE_SOCKET") {
            if let Ok(results) = vicinae_query(&socket_path, query, limit).await {
                return Ok(json!({ "results": results, "source": "vicinae" }));
            }
        }

        let results = query_entries_async(query, limit).await?;
        Ok(json!({ "results": results, "source": "desktop" }))
    });

    registry.register("Launcher.Query", |params| async move {
        let query = params
            .as_ref()
            .and_then(|p| p.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let limit = params
            .as_ref()
            .and_then(|p| p.get("limit"))
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_QUERY_LIMIT)
            .clamp(1, MAX_QUERY_LIMIT);
        let results = query_entries_async(query, limit).await?;
        Ok(json!({ "results": results }))
    });

    registry.register("Launcher.Run", |params| async move {
        let id = params
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing id"))?;
        validate_app_id(id)?;

        let index = load_index();
        if !index.iter().any(|e| e.id == id) {
            bail!("unknown app id: {id}");
        }

        process::run_allowlisted_detached(&["gtk-launch", id]).await?;
        let _ = touch_recent(id).await;
        Ok(json!({ "ok": true }))
    });

    registry.register("Launcher.Recent", |_params| async move {
        let items = recent_items().await?;
        Ok(json!({ "items": items }))
    });

    registry.register("Launcher.Pin", |params| async move {
        let id = params
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing id"))?;
        let pinned = params
            .as_ref()
            .and_then(|p| p.get("pinned"))
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("missing pinned boolean"))?;
        validate_app_id(id)?;
        set_pinned(id, pinned).await?;
        Ok(json!({ "ok": true, "id": id, "pinned": pinned }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_fixture_firefox() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/desktop/firefox.desktop");
        let entry = parse_desktop_file(&path).expect("firefox");
        assert_eq!(entry.id, "firefox");
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.generic_name.as_deref(), Some("Web Browser"));
        assert!(entry.categories.contains(&"Network".to_string()));
    }

    #[test]
    fn parse_skips_no_display() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/desktop/hidden-app.desktop");
        assert!(parse_desktop_file(&path).is_none());
    }

    #[test]
    fn fuzzy_score_requires_all_tokens() {
        let entry = DesktopEntry {
            id: "code".into(),
            name: "Visual Studio Code".into(),
            generic_name: Some("Text Editor".into()),
            comment: None,
            icon: None,
            categories: vec![],
            keywords: vec!["editor".into()],
        };
        assert!(fuzzy_score("visual code", &entry).is_some());
        assert!(fuzzy_score("visual emacs", &entry).is_none());
    }

    #[test]
    fn validate_app_id_rejects_path_chars() {
        assert!(validate_app_id("firefox").is_ok());
        assert!(validate_app_id("../evil").is_err());
    }

    #[test]
    fn build_index_from_fixture_dir() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/desktop");
        let entries = build_desktop_index(&[dir]);
        let ids: HashSet<_> = entries.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains("firefox"));
        assert!(ids.contains("aura-test"));
        assert!(!ids.contains("hidden-app"));
    }

    #[test]
    fn unescape_desktop_value_spaces() {
        assert_eq!(unescape_desktop_value("foo\\sbar"), "foo bar");
    }

    #[test]
    fn parse_desktop_ini_ignores_localized_keys() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            "[Desktop Entry]\nType=Application\nName=App\nName[en]=Ignored\nExec=true\n"
        )
        .unwrap();
        let entry = parse_desktop_file(f.path()).unwrap();
        assert_eq!(entry.name, "App");
    }
}
