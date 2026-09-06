//! Optional local JSON-RPC stdio extras.
//!
//! Core RPC methods stay compile-time. Extra methods are advertised by an
//! out-of-tree binary via `Extension.ListMethods` and proxied at runtime.
//!
//! Discovery (first match wins per command):
//! - `AURA_EXTENSIONS` — colon-separated executables (`""` disables extras)
//! - else `AURA_EXTENSIONS_DIR` or `$XDG_CONFIG_HOME/ags/extensions/*.json`

use crate::services::ServiceRegistry;
use crate::types::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

const HANDSHAKE_METHOD: &str = "Extension.ListMethods";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const CALL_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Deserialize)]
struct Manifest {
    command: ManifestCommand,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ManifestCommand {
    Path(String),
    Argv(Vec<String>),
}

impl ManifestCommand {
    fn argv(&self) -> Result<Vec<String>> {
        match self {
            ManifestCommand::Path(p) => {
                if p.trim().is_empty() {
                    anyhow::bail!("empty command");
                }
                Ok(vec![p.clone()])
            }
            ManifestCommand::Argv(args) => {
                if args.is_empty() || args[0].trim().is_empty() {
                    anyhow::bail!("empty command");
                }
                Ok(args.clone())
            }
        }
    }
}

struct StdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl StdioClient {
    async fn spawn(argv: &[String]) -> Result<Self> {
        let program = argv
            .first()
            .ok_or_else(|| anyhow!("extension command is empty"))?;
        let mut cmd = Command::new(program);
        if argv.len() > 1 {
            cmd.args(&argv[1..]);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawn extension {}", program))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("extension stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("extension stdout missing"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 0,
        })
    }

    async fn call(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        self.next_id += 1;
        let id = self.next_id;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: method.to_string(),
            params,
            id: Some(json!(id)),
        };
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        let mut response_line = String::new();
        let n = timeout(CALL_TIMEOUT, self.stdout.read_line(&mut response_line))
            .await
            .map_err(|_| anyhow!("extension timed out on {method}"))??;
        if n == 0 {
            anyhow::bail!("extension closed stdout during {method}");
        }
        let response: JsonRpcResponse = serde_json::from_str(response_line.trim())
            .with_context(|| format!("extension response for {method}"))?;
        if let Some(err) = response.error {
            anyhow::bail!("extension {method}: {} ({})", err.message, err.code);
        }
        response
            .result
            .ok_or_else(|| anyhow!("extension {method} returned empty result"))
    }
}

impl Drop for StdioClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn env_commands() -> Option<Vec<Vec<String>>> {
    match std::env::var("AURA_EXTENSIONS") {
        Ok(raw) if raw.trim().is_empty() => Some(Vec::new()),
        Ok(raw) => Some(
            raw.split(':')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| vec![s.to_string()])
                .collect(),
        ),
        Err(_) => None,
    }
}

fn extensions_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("AURA_EXTENSIONS_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("ags").join("extensions")
}

fn commands_from_dir(dir: &Path) -> Vec<Vec<String>> {
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => return Vec::new(),
    };
    entries.sort();
    let mut out = Vec::new();
    for path in entries {
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match load_manifest_argv(&path) {
            Ok(argv) => out.push(argv),
            Err(err) => tracing::warn!("skip extension {}: {err:#}", path.display()),
        }
    }
    out
}

fn load_manifest_argv(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    let manifest: Manifest = serde_json::from_str(&text)?;
    manifest.command.argv()
}

fn discover_commands() -> Vec<Vec<String>> {
    if let Some(from_env) = env_commands() {
        return from_env;
    }
    commands_from_dir(&extensions_dir())
}

fn parse_method_list(value: &Value) -> Result<Vec<String>> {
    let arr = if let Some(arr) = value.as_array() {
        arr
    } else if let Some(arr) = value.get("methods").and_then(|v| v.as_array()) {
        arr
    } else {
        anyhow::bail!("Extension.ListMethods must return an array or {{methods: [...]}}");
    };
    let mut names = Vec::new();
    for item in arr {
        let name = item
            .as_str()
            .ok_or_else(|| anyhow!("method name must be a string"))?;
        if name.is_empty() || name == HANDSHAKE_METHOD || name.contains('\n') {
            continue;
        }
        names.push(name.to_string());
    }
    Ok(names)
}

async fn attach_one(registry: &mut ServiceRegistry, argv: Vec<String>) {
    let label = argv
        .last()
        .map(|s| {
            Path::new(s)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(s)
                .to_string()
        })
        .unwrap_or_else(|| "extension".into());

    let mut client = match StdioClient::spawn(&argv).await {
        Ok(c) => c,
        Err(err) => {
            tracing::warn!("extension {label}: {err:#}");
            return;
        }
    };

    let listed = match timeout(HANDSHAKE_TIMEOUT, client.call(HANDSHAKE_METHOD, None)).await {
        Ok(Ok(value)) => parse_method_list(&value),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(anyhow!("handshake timed out")),
    };
    let methods = match listed {
        Ok(m) => m,
        Err(err) => {
            tracing::warn!("extension {label}: {err:#}");
            return;
        }
    };

    let mut installed = 0usize;
    let client = Arc::new(Mutex::new(client));
    for method in methods {
        if registry.handler_for(&method).is_some() {
            tracing::warn!("extension {label}: skip overlapping method {method}");
            continue;
        }
        let client = client.clone();
        let name = method.clone();
        registry.register(&method, move |params| {
            let client = client.clone();
            let name = name.clone();
            async move { client.lock().await.call(&name, params).await }
        });
        installed += 1;
    }
    if installed == 0 {
        tracing::warn!("extension {label}: advertised no new methods");
        return;
    }
    tracing::info!("extension {label}: attached {installed} methods");
    registry.record_extension(label, installed);
}

/// Spawn configured extras and proxy their advertised methods. Failures are logged.
pub async fn attach(registry: &mut ServiceRegistry) {
    for argv in discover_commands() {
        attach_one(registry, argv).await;
    }
}

#[cfg(test)]
mod tests {
    use super::{load_manifest_argv, parse_method_list, ManifestCommand};
    use serde_json::json;
    use std::io::Write;

    #[test]
    fn parse_list_object_and_array() {
        assert_eq!(
            parse_method_list(&json!({"methods": ["Ext.Ping", "Extension.ListMethods"]})).unwrap(),
            vec!["Ext.Ping"]
        );
        assert_eq!(
            parse_method_list(&json!(["Ext.Ping"])).unwrap(),
            vec!["Ext.Ping"]
        );
    }

    #[test]
    fn manifest_path_and_argv() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mock.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"command":["python3","/tmp/mock.py"]}}"#).unwrap();
        assert_eq!(
            load_manifest_argv(&path).unwrap(),
            vec!["python3", "/tmp/mock.py"]
        );
        let _ = ManifestCommand::Path("/bin/true".into());
    }
}
