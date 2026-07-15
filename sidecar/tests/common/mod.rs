//! Shared helpers for sidecar integration tests.
//!
//! See `tests/README.md` for safety rules on a live dev machine.

use ags_sidecar::build_registry;
use ags_sidecar::notify;
use ags_sidecar::services::ServiceRegistry;
use ags_sidecar::types::JsonRpcRequest;
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Once;

static INIT_NOTIFY: Once = Once::new();

/// RPC method prefixes that must not run from default integration tests.
const DENIED_PREFIXES: &[&str] = &["Session.", "Security.Offensive."];

/// Exact RPC methods blocked for default integration tests.
const DENIED_EXACT: &[&str] = &[
    "Apps.Launch",
    "Aura.ToggleWindow",
    "Brightness.Set",
    "Hyprland.Dispatch",
    "Launcher.Pin",
    "Launcher.Run",
    "Keybinds.Export",
    "Keybinds.Import",
    "Keybinds.Reload",
    "Keybinds.Set",
    "Keybinds.Unset",
    "Logs.ClearLogs",
    "Logs.ExportLogs",
    "Logs.FollowLogs",
    "Network.Connect",
    "Network.Disconnect",
    "Network.Forget",
    "Network.ToggleWifi",
    "Notifications.ClearAll",
    "Notifications.Dismiss",
    "Notifications.InvokeAction",
    "Notifications.SetDnd",
    "Notifications.SetRules",
    "Packages.Install",
    "Packages.InstallAur",
    "Packages.Remove",
    "Packages.Update",
    "Packages.Upgrade",
    "Performance.KillProcess",
    "Performance.RestartService",
    "Performance.SetCpuFrequency",
    "Performance.SetCpuGovernor",
    "Performance.SetProcessPriority",
    "Performance.StartService",
    "Performance.StopService",
    "Power.SetProfile",
    "Process.Kill",
    "Session.Lock",
    "Session.Logout",
    "Session.PowerOff",
    "Session.Reboot",
    "Session.Suspend",
    "Vpn.Connect",
    "Vpn.Disconnect",
    "Bluetooth.Connect",
    "Bluetooth.Disconnect",
    "Bluetooth.Pair",
    "Bluetooth.Remove",
    "Bluetooth.SetAdapterDiscoverable",
    "Bluetooth.SetAdapterPower",
    "DevOps.PullImage",
    "DevOps.RemoveImage",
    "DevOps.RestartContainer",
    "DevOps.StartContainer",
    "DevOps.StopContainer",
    "Automation.RunScript",
    "Automation.RunWorkflow",
    "Automation.CreateScript",
    "Automation.CreateWorkflow",
    "Automation.DeleteWorkflow",
    "Automation.DisableWorkflow",
    "Automation.EnableWorkflow",
    "Automation.UpdateWorkflow",
    "Security.AddFirewallRule",
    "Security.DisableFirewall",
    "Security.EnableFirewall",
    "Security.RemoveFirewallRule",
    "Security.RunClamScan",
    "Security.ScanPorts",
    "Audio.Refresh",
    "Audio.CreateLoopback",
    "Audio.CreateNullSink",
    "Audio.RouteStream",
    "Audio.SetDefaultDevice",
    "Audio.SetSinkMute",
    "Audio.SetSinkVolume",
    "Audio.SetSourceMute",
    "Audio.SetSourceVolume",
    "Audio.SetStreamMute",
    "Audio.SetStreamVolume",
    "Audio.Media.Next",
    "Audio.Media.PlayPause",
    "Audio.Media.Previous",
    "Audio.Profiles.ApplyScenario",
    "Audio.Profiles.Delete",
    "Audio.Profiles.Load",
    "Audio.Profiles.Save",
    "Audio.Effects.LoadPreset",
    "Audio.Effects.Reset",
    "Audio.Effects.SavePreset",
    "Audio.Effects.SetEqBand",
    "Audio.Effects.SetMicGain",
    "Audio.Effects.SetNoiseGate",
    "Audio.Effects.SetNoiseSuppression",
    "Audio.Effects.Start",
    "Audio.Effects.Stop",
    "Communication.LaunchApp",
    "Communication.MarkRead",
    "Communication.MuteNotifications",
    "Communication.SendMessage",
    "Communication.SetNotificationSettings",
    "Calendar.CreateEvent",
    "Calendar.DeleteEvent",
    "Calendar.ImportIcs",
    "Calendar.SetReminder",
    "Calendar.SyncCalendars",
    "Calendar.UpdateEvent",
    "Todos.Create",
    "Todos.CreateProject",
    "Todos.Delete",
    "Todos.Update",
    "Capture.Screenshot",
    "Capture.RecordStart",
    "Capture.RecordStop",
    "Settings.Set",
    "Settings.Reset",
    "Lock.SetConfig",
    "Lock.TestFingerprint",
    "Sleep.Inhibit",
    "Productivity.CancelTimer",
    "Productivity.CreatePomodoro",
    "Productivity.CreateTask",
    "Productivity.CreateTimer",
    "Productivity.DeleteTask",
    "Productivity.SetFocusMode",
    "Productivity.UpdateTask",
    "Fitness.SetGoal",
    "Fitness.StartWorkout",
    "Fitness.StopWorkout",
    "Fitness.SyncDevice",
    "Vault.Backup.Start",
    "Vault.SetEntry",
    "Vault.Transfer",
    "Performance.ApplyPreset",
    "GameMode.Disable",
    "GameMode.Enable",
    "GameMode.Toggle",
    "Weather.AddLocation",
    "Weather.RemoveLocation",
    "Weather.SetLocation",
    "Weather.SetUnits",
];

/// Short reason for blocklisted RPCs (used by contract tests documenting `api.ts` gaps).
pub fn denied_rpc_reason(method: &str) -> Option<&'static str> {
    match method {
        "Apps.Launch" => Some("spawns arbitrary desktop application"),
        "Launcher.Run" => Some("spawns gtk-launch for a desktop application"),
        "Launcher.Pin" => Some("mutates launcher pin list in SQLite"),
        "Aura.ToggleWindow" => Some("toggles AGS WebKit shell windows"),
        "Audio.SetStreamMute" | "Audio.SetStreamVolume" => Some("mutates PipeWire stream volume"),
        "Bluetooth.Connect" | "Bluetooth.Disconnect" => Some("changes Bluetooth device connection"),
        "Bluetooth.Scan" => Some("starts adapter discovery (host radio side effect)"),
        "Brightness.Set" => Some("changes monitor brightness"),
        "Hyprland.Dispatch" => Some("mutates Hyprland compositor state"),
        "Keybinds.Export" | "Keybinds.Import" | "Keybinds.Reload" | "Keybinds.Set" | "Keybinds.Unset" => {
            Some("reads or writes Hyprland keybind config")
        }
        "Logs.ClearLogs" | "Logs.ExportLogs" | "Logs.FollowLogs" => Some("clears, exports, or tails journal"),
        "Network.Connect" | "Network.Disconnect" | "Network.Forget" | "Network.ToggleWifi" => {
            Some("mutates NetworkManager connections")
        }
        "Notifications.ClearAll" | "Notifications.Dismiss" | "Notifications.InvokeAction"
        | "Notifications.SetDnd" | "Notifications.SetRules" => Some("mutates notification store or DND"),
        "Power.SetProfile" => Some("changes system power profile"),
        "Process.Kill" => Some("sends signal to user process"),
        "Security.RunClamScan" => Some("runs ClamAV scan on host paths"),
        "Session.Lock" | "Session.Logout" | "Session.PowerOff" | "Session.Reboot" | "Session.Suspend" => {
            Some("session / power action")
        }
        "Lock.SetConfig" | "Lock.TestFingerprint" => Some("mutates lock screen prefs or runs fingerprint verify"),
        "Sleep.Inhibit" => Some("holds logind sleep inhibitor"),
        "Vpn.Connect" | "Vpn.Disconnect" => Some("mutates VPN connection"),
        "Vault.Backup.Start" => Some("starts restic/borg backup job on host"),
        "Vault.Transfer" => Some("runs rclone/rsync transfer between remotes or paths"),
        m if m.starts_with("Session.") => Some("session / power action"),
        m if m.starts_with("Security.Offensive.") => Some("offensive security tooling"),
        _ => None,
    }
}

/// Returns true if `method` must not be invoked via [`call_method`] / [`call_rpc`].
pub fn is_denied_rpc_method(method: &str) -> bool {
    if DENIED_PREFIXES.iter().any(|p| method.starts_with(p)) {
        return true;
    }
    DENIED_EXACT.contains(&method)
}

/// Panics in tests when a blocklisted RPC would run on the host.
pub fn assert_safe_rpc_method(method: &str) {
    assert!(
        !is_denied_rpc_method(method),
        "refusing destructive or host-mutating RPC in integration test: {method}. \
         Use call_method_unchecked only in #[ignore] tests with isolation documented in tests/README.md"
    );
}

use std::sync::Mutex;

static STORAGE_TEST_LOCK: Mutex<()> = Mutex::new(());
static LAUNCHER_TEST_LOCK: Mutex<()> = Mutex::new(());
static AUTOMATION_TEST_LOCK: Mutex<()> = Mutex::new(());
static GAMEMODE_TEST_LOCK: Mutex<()> = Mutex::new(());
static PERFORMANCE_TEST_LOCK: Mutex<()> = Mutex::new(());
static WEATHER_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Serialize launcher tests that override `AURA_LAUNCHER_DESKTOP_DIRS`.
pub fn launcher_test_lock() -> std::sync::MutexGuard<'static, ()> {
    LAUNCHER_TEST_LOCK.lock().unwrap()
}

/// Serialize automation storage tests (SQLite + cron tick side effects).
pub fn automation_test_lock() -> std::sync::MutexGuard<'static, ()> {
    AUTOMATION_TEST_LOCK.lock().unwrap()
}

/// Serialize gamemode RPC tests (process-global `GAMEMODE_ENABLED` static).
pub fn gamemode_test_lock() -> std::sync::MutexGuard<'static, ()> {
    GAMEMODE_TEST_LOCK.lock().unwrap()
}

/// Serialize performance preset tests that use `AURA_PERFORMANCE_DRY_RUN`.
pub fn performance_test_lock() -> std::sync::MutexGuard<'static, ()> {
    PERFORMANCE_TEST_LOCK.lock().unwrap()
}

/// Serialize weather RPC tests (shared in-memory cache + wiremock env).
pub fn weather_test_lock() -> std::sync::MutexGuard<'static, ()> {
    WEATHER_TEST_LOCK.lock().unwrap()
}

static KEYRING_TEST_LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();

/// Serialize tests that prepend a mock `secret-tool` onto `PATH`.
pub async fn keyring_test_lock() -> tokio::sync::MutexGuard<'static, ()> {
    KEYRING_TEST_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

fn write_mock_secret_tool(dir: &std::path::Path, store_path: &std::path::Path) {
    let script = format!(
        r#"#!/bin/sh
STORE="{store}"
case "$1" in
  store)
    shift
    key=""
    while [ $# -gt 0 ]; do
      case "$1" in
        --label) shift 2 ;;
        *) key="${{key}}|$1|$2"; shift 2 ;;
      esac
    done
    secret=$(cat)
    echo "${{key}}|${{secret}}" >> "$STORE"
    exit 0
    ;;
  lookup)
    shift
    key=""
    while [ $# -gt 0 ]; do
      key="${{key}}|$1|$2"
      shift 2
    done
    line=$(grep -F "${{key}}|" "$STORE" 2>/dev/null | tail -1)
    if [ -z "$line" ]; then exit 1; fi
    echo "${{line##*|}}"
    exit 0
    ;;
esac
exit 1
"#,
        store = store_path.display()
    );
    let path = dir.join("secret-tool");
    std::fs::write(&path, script).expect("script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

/// Prepends a temp-dir mock `secret-tool` onto `PATH` for the lifetime of this guard.
pub struct MockKeyring {
    _dir: tempfile::TempDir,
    prev_path: Option<String>,
}

impl MockKeyring {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = dir.path().join("store.txt");
        write_mock_secret_tool(dir.path(), &store);
        let prev_path = std::env::var("PATH").ok();
        let path = prev_path.clone().unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.path().display(), path));
        Self {
            _dir: dir,
            prev_path,
        }
    }
}

impl Drop for MockKeyring {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
    }
}

pub struct StorageTestDb {
    _guard: std::sync::MutexGuard<'static, ()>,
    path: std::path::PathBuf,
}

impl Drop for StorageTestDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        std::env::remove_var("AURA_STORAGE_DB");
    }
}

fn storage_file_id() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u128)
        .unwrap_or(0)
}

/// Isolated SQLite DB via `AURA_STORAGE_DB` for storage-backed RPC branches.
pub async fn setup_temp_storage_db() -> StorageTestDb {
    let guard = STORAGE_TEST_LOCK.lock().unwrap();
    let path = std::env::temp_dir().join(format!(
        "ags-it-{}-{}.db",
        std::process::id(),
        storage_file_id()
    ));
    let _ = std::fs::remove_file(&path);
    std::env::set_var("AURA_STORAGE_DB", path.to_string_lossy().to_string());
    ags_sidecar::utils::storage::init()
        .await
        .expect("storage init");
    StorageTestDb { _guard: guard, path }
}

pub fn load_fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {rel}: {e}"))
}

/// Default subprocess fixture tree for mocked exec integration tests.
pub fn default_exec_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/exec")
}

/// RAII guard for `AURA_EXEC_FIXTURE_DIR` (see `sidecar/tests/README.md`).
pub struct ExecFixtureGuard {
    previous: Option<String>,
}

impl ExecFixtureGuard {
    pub fn activate() -> Self {
        Self::activate_dir(default_exec_fixture_dir())
    }

    pub fn activate_dir(dir: PathBuf) -> Self {
        let previous = std::env::var(ags_sidecar::utils::process::EXEC_FIXTURE_ENV).ok();
        std::env::set_var(
            ags_sidecar::utils::process::EXEC_FIXTURE_ENV,
            dir.to_string_lossy().as_ref(),
        );
        Self { previous }
    }
}

impl Drop for ExecFixtureGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.previous.take() {
            std::env::set_var(ags_sidecar::utils::process::EXEC_FIXTURE_ENV, prev);
        } else {
            std::env::remove_var(ags_sidecar::utils::process::EXEC_FIXTURE_ENV);
        }
    }
}

pub fn test_registry() -> ServiceRegistry {
    INIT_NOTIFY.call_once(|| {
        notify::init_for_tests();
    });
    build_registry()
}

/// Invoke an RPC on the in-process registry (blocklist enforced).
pub async fn call_method(
    registry: &ServiceRegistry,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    assert_safe_rpc_method(method);
    dispatch_rpc(registry, method, params).await
}

/// Alias for [`call_method`].
#[allow(dead_code)]
pub async fn call_rpc(
    registry: &ServiceRegistry,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    call_method(registry, method, params).await
}

/// Invoke an RPC without the safety blocklist (only for `#[ignore]` tests).
#[allow(dead_code)]
pub async fn call_method_unchecked(
    registry: &ServiceRegistry,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    if is_denied_rpc_method(method) {
        eprintln!(
            "warning: call_method_unchecked invoking blocklisted method {method}; \
             ensure test is #[ignore] and isolated"
        );
    }
    dispatch_rpc(registry, method, params).await
}

async fn dispatch_rpc(
    registry: &ServiceRegistry,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: method.to_string(),
        params,
        id: Some(Value::Number(1.into())),
    };
    registry.handle_request(request).await
}

/// Assert every key is present on a JSON object (integration shape tests).
pub fn assert_json_object_keys(value: &Value, keys: &[&str]) {
    let obj = value.as_object().expect("JSON object");
    for key in keys {
        assert!(obj.contains_key(*key), "missing key {key}");
    }
}

/// Load RPC method names referenced in `ui/src/lib/api.ts`.
pub fn load_api_ts_methods() -> std::collections::BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/lib/api.ts");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let re = regex::Regex::new(
        r#""((?:Power|Network|Bluetooth|Audio|System|Weather|Vpn|Calendar|Packages|Notifications|Keybinds|Logs|Security|Performance|DevOps|Productivity|Automation|Communication|Fitness|Brightness|Hyprland|Session|Aura|Apps|Media|Process|Settings|Capture|Launcher|Todos|Vault|Dashboard|Sidebar)\.[A-Za-z.]+)""#,
    )
    .expect("api method regex");
    re.captures_iter(&text)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Heuristic: read-only RPC safe for default integration `call_method` sweeps.
pub fn is_safe_readonly_rpc(method: &str) -> bool {
    if is_denied_rpc_method(method) {
        return false;
    }
    let verb = method.rsplit('.').next().unwrap_or("");
    const MUTATING: &[&str] = &[
        "Set", "Create", "Delete", "Update", "Install", "Remove", "Upgrade", "Connect",
        "Disconnect", "Forget", "Toggle", "Enable", "Disable", "Launch", "Kill", "Start",
        "Stop", "Restart", "Apply", "Load", "Save", "Import", "Clear", "Dismiss", "Invoke",
        "Run", "Pull", "Pair", "Sync", "Mark", "Send", "Mute", "Capture", "Add", "Reload",
        "Refresh", "Route", "Follow", "Export", "Unset", "PowerOff", "Logout", "Lock",
        "Reboot", "Suspend", "ScanPorts", "Trigger", "Inhibit", "TestFingerprint",
    ];
    if MUTATING.iter().any(|v| verb == *v || verb.starts_with(v)) {
        return false;
    }
    verb.starts_with("Get")
        || verb.starts_with("List")
        || verb.starts_with("Validate")
        || verb.starts_with("Scan")
        || verb.starts_with("Search")
        || verb.starts_with("Filter")
        || verb == "IsEnabled"
        || verb == "ExportIcs"
        || matches!(verb, "Query" | "Recent" | "ParseDueDate" | "Status")
}

/// Collect RPC method names invoked via [`call_method`] / [`call_rpc`] in integration test sources.
pub fn integration_test_methods_from_sources(sources: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let re = regex::Regex::new(r#""([A-Z][a-zA-Z0-9]*\.[A-Za-z0-9.]+)""#).expect("method regex");
    for cap in re.captures_iter(sources) {
        let method = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if method.contains('.') && !method.starts_with("test_ns_") && !method.starts_with("it_") {
            out.insert(method.to_string());
        }
    }
    out
}

#[cfg(test)]
mod guard_tests {
    use super::*;

    #[test]
    fn denies_session_and_offensive_prefixes() {
        assert!(is_denied_rpc_method("Session.Reboot"));
        assert!(is_denied_rpc_method("Security.Offensive.Nmap.Scan"));
        assert!(is_denied_rpc_method("GameMode.Enable"));
        assert!(!is_denied_rpc_method("GameMode.IsEnabled"));
    }

    #[test]
    fn allows_read_only_examples() {
        assert!(!is_denied_rpc_method("Power.GetBatteryState"));
        assert!(!is_denied_rpc_method("Storage.Set"));
        assert!(!is_denied_rpc_method("Logs.Get"));
    }

    #[test]
    fn assert_safe_rpc_method_panics_on_denied() {
        let result = std::panic::catch_unwind(|| assert_safe_rpc_method("Session.PowerOff"));
        assert!(result.is_err());
    }
}
