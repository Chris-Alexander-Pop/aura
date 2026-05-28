use anyhow::Result;
use rusqlite::{params, Connection};
use std::env;
use std::path::{Path, PathBuf};
use tokio::task;

const SCHEMA_VERSION: i32 = 1;

fn db_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("AURA_STORAGE_DB") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(dir) = env::var("XDG_DATA_HOME") {
        return Ok(Path::new(&dir).join("ags-sidecar").join("state.db"));
    }
    let home = env::var("HOME")?;
    Ok(Path::new(&home)
        .join(".local")
        .join("share")
        .join("ags-sidecar")
        .join("state.db"))
}

fn init_db_sync(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA user_version = 1;
        CREATE TABLE IF NOT EXISTS kv (
            namespace TEXT NOT NULL,
            key       TEXT NOT NULL,
            value     TEXT NOT NULL,
            PRIMARY KEY (namespace, key)
        );
        CREATE TABLE IF NOT EXISTS schema_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;

    let current: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current < SCHEMA_VERSION {
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
    }

    conn.execute(
        "INSERT INTO schema_meta(key, value) VALUES ('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

async fn with_conn<F, T>(path: PathBuf, f: F) -> Result<T>
where
    F: FnOnce(&Connection) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    task::spawn_blocking(move || {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        init_db_sync(&conn)?;
        f(&conn)
    })
    .await?
}

/// Initialize the SQLite database (idempotent).
pub async fn init() -> Result<()> {
    let path = db_path()?;
    with_conn(path, |_| Ok(())).await?;
    Ok(())
}

/// Store a JSON value under (namespace, key).
pub async fn set_kv(namespace: &str, key: &str, value: &serde_json::Value) -> Result<()> {
    let ns = namespace.to_string();
    let k = key.to_string();
    let v = value.to_string();
    let path = db_path()?;

    with_conn(path, move |conn| {
        conn.execute(
            "INSERT INTO kv(namespace, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(namespace, key) DO UPDATE SET value = excluded.value",
            params![ns, k, v],
        )?;
        Ok(())
    })
    .await
}

/// Load a JSON value from (namespace, key).
pub async fn get_kv(namespace: &str, key: &str) -> Result<Option<serde_json::Value>> {
    let ns = namespace.to_string();
    let k = key.to_string();
    let path = db_path()?;

    let text = with_conn(path, move |conn| {
        let mut stmt =
            conn.prepare("SELECT value FROM kv WHERE namespace = ?1 AND key = ?2")?;
        let mut rows = stmt.query(params![ns, k])?;
        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    })
    .await?;

    if let Some(text) = text {
        Ok(Some(serde_json::from_str(&text)?))
    } else {
        Ok(None)
    }
}

/// List all keys in a namespace.
pub async fn list_keys(namespace: &str) -> Result<Vec<String>> {
    let ns = namespace.to_string();
    let path = db_path()?;

    with_conn(path, move |conn| {
        let mut stmt = conn.prepare("SELECT key FROM kv WHERE namespace = ?1 ORDER BY key")?;
        let rows = stmt.query_map(params![ns], |row| row.get(0))?;
        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    })
    .await
}

/// List distinct namespaces, optionally filtered by prefix.
pub async fn list_namespaces(prefix: Option<&str>) -> Result<Vec<String>> {
    let prefix = prefix.map(|s| s.to_string());
    let path = db_path()?;

    with_conn(path, move |conn| {
        let mut namespaces = Vec::new();
        if let Some(p) = prefix {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT namespace FROM kv WHERE namespace LIKE ?1 ORDER BY namespace",
            )?;
            let pattern = format!("{p}%");
            let rows = stmt.query_map(params![pattern], |row| row.get(0))?;
            for row in rows {
                namespaces.push(row?);
            }
        } else {
            let mut stmt =
                conn.prepare("SELECT DISTINCT namespace FROM kv ORDER BY namespace")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            for row in rows {
                namespaces.push(row?);
            }
        }
        Ok(namespaces)
    })
    .await
}

/// Delete a key. Returns true if a row was removed.
pub async fn delete_kv(namespace: &str, key: &str) -> Result<bool> {
    let ns = namespace.to_string();
    let k = key.to_string();
    let path = db_path()?;

    with_conn(path, move |conn| {
        let n = conn.execute(
            "DELETE FROM kv WHERE namespace = ?1 AND key = ?2",
            params![ns, k],
        )?;
        Ok(n > 0)
    })
    .await
}

/// Return all JSON values in a namespace (skips corrupt rows).
pub async fn scan_namespace(namespace: &str) -> Result<Vec<serde_json::Value>> {
    let ns = namespace.to_string();
    let path = db_path()?;

    with_conn(path, move |conn| {
        let mut stmt =
            conn.prepare("SELECT value FROM kv WHERE namespace = ?1 ORDER BY key")?;
        let rows = stmt.query_map(params![ns], |row| row.get::<_, String>(0))?;
        let mut items = Vec::new();
        for row in rows {
            let text = row?;
            match serde_json::from_str(&text) {
                Ok(v) => items.push(v),
                Err(e) => tracing::warn!("skipping corrupt kv in {}: {}", ns, e),
            }
        }
        Ok(items)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "ags-sidecar-test-{}-{}.db",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AURA_STORAGE_DB", path.to_string_lossy().to_string());
    }

    #[tokio::test]
    async fn round_trip_set_list_scan_delete() {
        setup();
        init().await.unwrap();

        set_kv("test_ns", "a", &serde_json::json!({"x": 1}))
            .await
            .unwrap();
        set_kv("test_ns", "b", &serde_json::json!({"x": 2}))
            .await
            .unwrap();

        let keys = list_keys("test_ns").await.unwrap();
        assert_eq!(keys, vec!["a", "b"]);

        let items = scan_namespace("test_ns").await.unwrap();
        assert_eq!(items.len(), 2);

        assert!(delete_kv("test_ns", "a").await.unwrap());
        assert!(!delete_kv("test_ns", "missing").await.unwrap());

        let keys = list_keys("test_ns").await.unwrap();
        assert_eq!(keys, vec!["b"]);
    }

    #[tokio::test]
    async fn list_namespaces_with_prefix() {
        setup();
        init().await.unwrap();

        set_kv("auto_a", "k", &serde_json::json!(1))
            .await
            .unwrap();
        set_kv("auto_b", "k", &serde_json::json!(2))
            .await
            .unwrap();
        set_kv("other", "k", &serde_json::json!(3))
            .await
            .unwrap();

        let ns = list_namespaces(Some("auto")).await.unwrap();
        assert!(ns.contains(&"auto_a".to_string()));
        assert!(ns.contains(&"auto_b".to_string()));
        assert!(!ns.contains(&"other".to_string()));
    }
}
