//! Local crash dumps under `~/.local/share/aura/crashes/` (no network upload).

use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;

const MAX_CRASH_DUMPS: usize = 50;
const ENV_CRASH_DIR: &str = "AURA_CRASH_DIR";

#[derive(Debug, Clone, Serialize)]
pub struct CrashDump {
    pub ts: String,
    pub component: String,
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    pub host: String,
    pub aura: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,
}

/// Resolve crash dump directory: `AURA_CRASH_DIR` or `$XDG_DATA_HOME/aura/crashes`.
pub fn crash_dir() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_CRASH_DIR) {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("aura").join("crashes");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("aura")
        .join("crashes")
}

fn hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".into())
}

fn short_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{nanos:08x}")
}

fn sanitize_component(component: &str) -> String {
    component
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Write a crash dump and prune to [`MAX_CRASH_DUMPS`]. Returns the path written.
pub fn write_crash_dump(dump: &CrashDump) -> std::io::Result<PathBuf> {
    let dir = crash_dir();
    fs::create_dir_all(&dir)?;

    let ts_slug = dump
        .ts
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let filename = format!(
        "{}_{}_{}.json",
        ts_slug,
        sanitize_component(&dump.component),
        short_id()
    );
    let path = dir.join(filename);

    let json = serde_json::to_vec_pretty(dump)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut file = fs::File::create(&path)?;
    file.write_all(&json)?;
    file.write_all(b"\n")?;

    prune_crash_dumps(&dir, MAX_CRASH_DUMPS)?;
    Ok(path)
}

/// Best-effort write used from panic hooks (never panics).
pub fn write_crash_dump_best_effort(dump: &CrashDump) {
    if let Err(e) = write_crash_dump(dump) {
        eprintln!("aura: failed to write crash dump: {e}");
    }
}

pub fn prune_crash_dumps(dir: &Path, keep: usize) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect();

    if entries.len() <= keep {
        return Ok(());
    }

    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let remove_count = entries.len() - keep;
    for entry in entries.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

/// Install a panic hook that writes a local crash dump, then chains the previous hook.
pub fn install_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "panic".to_string()
            };

            let location = info.location().map(|l| {
                format!("{}:{}:{}", l.file(), l.line(), l.column())
            });

            let dump = CrashDump {
                ts: Utc::now().to_rfc3339(),
                component: "sidecar".into(),
                kind: "panic".into(),
                message: match location {
                    Some(loc) => format!("{message} @ {loc}"),
                    None => message,
                },
                stack: None,
                host: hostname(),
                aura: format!("ags-sidecar {}", env!("CARGO_PKG_VERSION")),
                exit_status: None,
                signal: None,
            };
            write_crash_dump_best_effort(&dump);
            prev(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_crash_dir<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(ENV_CRASH_DIR, dir.path());
        f(dir.path());
        std::env::remove_var(ENV_CRASH_DIR);
    }

    #[test]
    fn crash_dir_respects_env() {
        with_crash_dir(|path| {
            assert_eq!(crash_dir(), path);
        });
    }

    #[test]
    fn write_crash_dump_creates_json() {
        with_crash_dir(|dir| {
            let dump = CrashDump {
                ts: "2026-07-16T22:00:00Z".into(),
                component: "sidecar".into(),
                kind: "panic".into(),
                message: "test panic".into(),
                stack: Some("frame0".into()),
                host: "testhost".into(),
                aura: "ags-sidecar 0.1.0".into(),
                exit_status: None,
                signal: None,
            };
            let path = write_crash_dump(&dump).expect("write");
            assert!(path.exists());
            assert_eq!(path.parent().unwrap(), dir);
            let body = fs::read_to_string(&path).expect("read");
            assert!(body.contains("test panic"));
            assert!(body.contains("\"kind\": \"panic\""));
        });
    }

    #[test]
    fn prune_keeps_newest_fifty() {
        with_crash_dir(|dir| {
            for i in 0..55 {
                let dump = CrashDump {
                    ts: format!("2026-07-16T22:00:{i:02}Z"),
                    component: "sidecar".into(),
                    kind: "panic".into(),
                    message: format!("panic {i}"),
                    stack: None,
                    host: "testhost".into(),
                    aura: "ags-sidecar 0.1.0".into(),
                    exit_status: None,
                    signal: None,
                };
                write_crash_dump(&dump).expect("write");
                // Ensure distinct mtime ordering on fast filesystems
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let count = fs::read_dir(dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .count();
            assert_eq!(count, MAX_CRASH_DUMPS);
        });
    }

    #[test]
    fn panic_hook_writes_dump() {
        with_crash_dir(|dir| {
            install_panic_hook();
            let _ = std::panic::catch_unwind(|| panic!("hook smoke"));
            let count = fs::read_dir(dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .count();
            assert!(count >= 1, "expected at least one crash dump after panic");
            let body = fs::read_dir(dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .map(|e| fs::read_to_string(e.path()).unwrap())
                .unwrap();
            assert!(body.contains("hook smoke"));
            assert!(body.contains("\"kind\": \"panic\""));
        });
    }
}
