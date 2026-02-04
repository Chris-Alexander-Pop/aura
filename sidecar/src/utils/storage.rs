use anyhow::Result;
use rusqlite::{params, Connection};
use std::env;
use std::path::{Path, PathBuf};
use tokio::task;

fn db_path() -> Result<PathBuf> {
    // Prefer XDG_DATA_HOME, fall back to ~/.local/share
    if let Ok(dir) = env::var("XDG_DATA_HOME") {
        Ok(Path::new(&dir).join("ags-sidecar").join("state.db"))
    } else {
        let home = env::var("HOME")?;
        Ok(Path::new(&home)
            .join(".local")
            .join("share")
            .join("ags-sidecar")
            .join("state.db"))
    }
}

fn init_db_sync(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS kv (
            namespace TEXT NOT NULL,
            key       TEXT NOT NULL,
            value     TEXT NOT NULL,
            PRIMARY KEY (namespace, key)
        );
        "#,
    )?;
    Ok(())
}

/// Initialize the SQLite database (idempotent).
pub async fn init() -> Result<()> {
    let path = db_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid DB path"))?
        .to_path_buf();

    task::spawn_blocking(move || {
        std::fs::create_dir_all(&dir)?;
        let conn = Connection::open(path)?;
        init_db_sync(&conn)?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;

    Ok(())
}

/// Store a JSON value under (namespace, key).
pub async fn set_kv(namespace: &str, key: &str, value: &serde_json::Value) -> Result<()> {
    let ns = namespace.to_string();
    let k = key.to_string();
    let v = value.to_string();
    let path = db_path()?;

    task::spawn_blocking(move || {
        let conn = Connection::open(path)?;
        init_db_sync(&conn)?;
        conn.execute(
            "INSERT INTO kv(namespace, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(namespace, key) DO UPDATE SET value = excluded.value",
            params![ns, k, v],
        )?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;

    Ok(())
}

/// Load a JSON value from (namespace, key).
pub async fn get_kv(namespace: &str, key: &str) -> Result<Option<serde_json::Value>> {
    let ns = namespace.to_string();
    let k = key.to_string();
    let path = db_path()?;

    let res = task::spawn_blocking(move || -> Result<Option<String>> {
        let conn = Connection::open(path)?;
        init_db_sync(&conn)?;
        let mut stmt = conn.prepare("SELECT value FROM kv WHERE namespace = ?1 AND key = ?2")?;
        let mut rows = stmt.query(params![ns, k])?;
        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    })
    .await??;

    if let Some(text) = res {
        let json = serde_json::from_str(&text)?;
        Ok(Some(json))
    } else {
        Ok(None)
    }
}

