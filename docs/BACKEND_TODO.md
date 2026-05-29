# AGS Sidecar Backend — Implementation & Test Checklist

> **Purpose:** Exhaustive backlog for `ags-sidecar` (`sidecar/`) so each item can be picked up, implemented, and verified with unit + integration tests before wiring the React/GTK UI.
>
> **Sources:** [todo.md](../todo.md), [docs/roadmap/](roadmap/), [feature_matrix.md](feature_matrix.md), [MIGRATION_STRATEGY.md](MIGRATION_STRATEGY.md), **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** (resolved product/stack choices), current `sidecar/src/services/*`, `ui/src/lib/api.ts`, `src/lib/sidecar.ts`.
>
> **How to use:** Check `[x]` when done. Prefer **one vertical slice** per PR (e.g. “Network saved networks + tests + api.ts”), not half a service. Run `cd sidecar && cargo test` after each slice.

---

## Snapshot (May 2026)

| Item | Status |
|------|--------|
| Registered RPC methods | ~**309** (`sidecar/rpc-manifest.json`) |
| Integration tests | **250+** test fns — contracts, RPC guard, wave4 slice tests, fast vs slow host sweep |
| `ui/src/lib/api.ts` vs sidecar | **Contract script** — `scripts/check-api-rpc-contract.sh` |
| Dedicated services missing | **Launcher/Vicinae**, **Capture**, **Settings/Config**, **Vault**, **Todos** (see §3) |
| Storage layer | SQLite KV with **list/delete/scan** |
| Real-time push | `notify` bus — Power, Network, BT, Audio, Notifications, Performance, Productivity, **Hyprland** |
| Coverage | ~**50%** line (`docs/sidecar_coverage_baseline.md`, `scripts/sidecar-coverage.sh`) |

---

## Priority tiers

| Tier | Meaning | Examples |
|------|---------|----------|
| **P0** | Blocks UI today or causes 500s | Fix API contract mismatches; Power/Network/Audio correctness |
| **P1** | Core shell daily use | Notifications, brightness/volume OSD paths, Hyprland, session |
| **P2** | Control center panes (existing UI) | Packages, Logs, Security aggregate, Performance, VPN profiles |
| **P3** | Roadmap panels (greenfield backend) | Vault, Communication hub, Calendar sync, DevOps cloud |
| **P4** | Stubs / sensitive / optional | Exocortex, Skiller, Secure-A, most `Security.Offensive.*` |

---

## 0. Foundation & test harness

### 0.1 Project hygiene

- [ ] Document canonical **Arch-only vs multi-distro** stance in `sidecar/README.md` (pacman vs apt branches exist today)
- [ ] Add `sidecar/README.md` with: build, `ags-sidecar client`, HTTP `:9080`, env vars, polkit/sudo expectations
- [ ] Resolve **binary path** story: `src/lib/sidecar.ts` hardcodes `~/.config/ags/sidecar/target/...` — document or fix for dev clones under `Engineering/Productivity/ags`
- [ ] Add **OpenAPI or machine-readable RPC manifest** generated from `registry.register` calls (script in CI) to prevent contract drift
- [ ] Standardize method naming: **`DevOps`** everywhere (rename TS clients from `Devops`) — see [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)

### 0.2 Shared types (`sidecar/src/types.rs`)

- [ ] Audit all public DTOs — ensure `serde` renames match `ui/src/lib/api-types.ts` (snake_case)
- [ ] Add `#[serde(deny_unknown_fields)]` on stable outward types where feasible
- [ ] Mirror types in `src/lib/types.ts` for GTK client
- [ ] Add **version field** to HTTP health: `GET /api/System.Ping` or `Sidecar.GetVersion`

### 0.3 Storage (`sidecar/src/utils/storage.rs`)

- [x] `list_keys(namespace) -> Vec<String>`
- [x] `list_namespace(prefix) -> Vec<(namespace, key)>`
- [x] `delete_kv(namespace, key)`
- [x] `scan_namespace(namespace) -> Vec<serde_json::Value>` for calendar events, workflows, tasks
- [x] Migrations table + schema version
- [x] Unit tests: round-trip, concurrent writes, corrupt JSON handling

### 0.4 Process & privileges (`utils/process.rs`, `utils/keyring.rs`)

- [ ] Central **allowlist** for shell commands (security review)
- [ ] Timeout + max output bytes on all `exec_command`
- [ ] Polkit integration helper for privileged ops (VPN, firewall, package install)
- [ ] Keyring: store/retrieve Wi-Fi/VPN/OAuth secrets — never log values
- [ ] Unit tests with **mocked** command runner (inject trait)

### 0.5 Service registry

- [ ] Unknown method: structured error code (not generic bail)
- [ ] Request logging behind `RUST_LOG` with **redaction**
- [ ] Optional **method groups** for pentest (`Security.Offensive.*`) behind feature flag `offensive-security`
- [ ] ~~Register alias handlers~~ — **decided:** rename clients to match Rust, no aliases (see ADR)

### 0.6 Real-time events

- [ ] Define notification schema: `{ method, params }` for WebSocket + future stdin notify
- [x] Emit on: battery change, network connect/disconnect, Bluetooth, Audio (partial — VPN pending)
- [x] GTK `sidecar.ts` already listens for `Power.BatteryState` / `Power.Profile` — **implement emitters** in `power.rs`
- [ ] Integration test: connect WS client, trigger change, assert message

### 0.7 Test infrastructure

- [ ] **Unit tests** per service module under `sidecar/src/services/<name>/tests` or `#[cfg(test)]` in module
- [ ] **Integration tests** using `ServiceRegistry::new()` + register only service under test (no full HTTP server required)
- [ ] **HTTP integration tests** with `axum::test` or `reqwest` against spawned server on random port
- [ ] **Fixture directory** `sidecar/tests/fixtures/` (nmcli output, pactl, hyprctl JSON, pacman -Qu)
- [ ] CI job: `cargo test`, `cargo clippy`, optional `cargo test --features offensive-security`
- [ ] Document **manual test matrix** for hardware-dependent tests (Wi-Fi, BT, battery)
- [ ] Replace placeholder `integration_test.rs` brightness copy with real registry calls

---

## 1. API contract alignment (P0)

These methods are called from **`ui/src/lib/api.ts`** and/or **`src/lib/sidecar.ts`** but are **missing or differently named** in the sidecar today. Fix by implementing the method **or** adding a documented alias + updating clients.

| Client calls | Sidecar today | Action |
|--------------|---------------|--------|
| `Packages.GetUpdates` | `Packages.GetUpgradable` | Implement alias or rename client |
| `Logs.Get` | `Logs.GetSystemLogs`, etc. | Implement `Logs.Get` wrapper returning structured `LogEntry[]` |
| `Security.GetStatus` | `Security.GetFirewallStatus`, … | Implement aggregate `GetStatus` matching `api-types.ts` |
| `Performance.GetMetrics` | `Performance.GetCpuStats`, … | Implement aggregate or update clients |
| `Devops.GetStatus` | `DevOps.*` (15 methods, no `GetStatus`) | Implement summary + fix casing |
| `Productivity.GetStats` | `Productivity.GetScreenTime`, … | Implement aggregate |
| `Automation.ListRules` | `Automation.GetWorkflows` | Implement alias + fix `GetWorkflows` empty list |
| `Communication.GetUnread` | *(none)* | Implement unread map per bridge/app |
| `Gamemode.GetStatus` | `GameMode.IsEnabled` | Alias or rename |
| `Storage.GetConfig` | `Storage.Get` / `Storage.Set` | Alias or implement config schema |
| `Automation.Trigger` | `Automation.RunWorkflow` | Alias |

- [ ] Complete table above (all rows)
- [ ] Add integration test: every `api.ts` method returns `ok: true` on CI stub environment OR returns defined error when tool missing
- [ ] Add `ui` contract test (optional): script that greps `api.` vs generated manifest

---

## 2. Existing services — implement, harden, test

For each service: **(a)** real system integration, **(b)** typed responses, **(c)** unit tests with fixtures, **(d)** integration test via registry, **(e)** expose in `api.ts` + `sidecar.ts` if user-facing.

### 2.1 Power (`power.rs`) — P0

- [x] Battery % from `/sys/class/power_supply` or UPower (zbus)
- [x] Charging state + **time remaining** (fix TODO in source)
- [x] Power profiles: `powerprofilesctl` / TLP / PPD — verify on target hardware
- [x] `Power.SetProfile` persists and reports errors clearly
- [x] Emit `Power.BatteryState` notifications on change (poll or uevent)
- [ ] Unit tests: parse sysfs fixtures
- [ ] Integration tests: `GetBatteryState`, `GetProfile`, `SetProfile` (mock profiles daemon)

### 2.2 Network (`network.rs`) — P0

- [x] `Network.GetStatus`: wifi enabled, active SSID, IPs, **public IP** (optional cached)
- [x] `Network.ScanNetworks`: signal, security type, hidden SSIDs
- [ ] `Network.Connect`: open/WPA2/WPA3; **802.1X** path (WPA via nmcli + keyring done; 802.1X deferred)
- [x] Saved networks: list, forget, auto-connect flag
- [ ] Captive portal detection helper
- [x] Keyring integration for passwords
- [x] `Network.ToggleWifi` reliable on NM
- [x] Ethernet/VPN interface status in `GetStatus` (ethernet done; VPN in status deferred)
- [ ] Unit tests: parse `nmcli` fixtures
- [ ] Integration tests with **nmcli test mode** or mocked commands
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) Wi-Fi panel, [system-and-input-foundation.md](roadmap/system-and-input-foundation.md) login paths

### 2.3 Bluetooth (`bluetooth.rs`) — P0

- [x] Adapter power/discoverable (`bluetoothctl`, `SetAdapterPower` / `SetAdapterDiscoverable`)
- [ ] Pairing flow + agent handling (PIN UI deferred)
- [x] Device battery % when available (`bluetoothctl info` parse)
- [ ] Audio profile selection (A2DP/HSP)
- [x] `Bluetooth.GetDeviceInfo` used by UI
- [x] Stop scan, remove device — `bluetoothctl` + React pane wired
- [x] Unit tests: `bluetoothctl` output fixtures
- [x] Integration tests: `GetAdapters` / `GetDevices` smoke

### 2.4 Audio (`audio.rs`) — P0

- [x] `Audio.GetDevices` / `GetStreams` — PipeWire via `wpctl` / `pactl`
- [x] Default sink/source set + **mute state sync** (`SetSinkMute` / `SetSourceMute`, parse MUTED)
- [x] Per-stream volume/mute (verified + refresh after set)
- [x] `Audio.SetDefaultDevice` — GTK bar needs this
- [x] **Fallback path**: `Audio.Refresh` with optional `restart_wireplumber` param
- [x] Sink volume for default output (`Audio.SetSinkVolume` / `SetSourceVolume`)
- [ ] Effects submodule: verify EasyEffects/wireplumber links or gate behind “advanced”
- [ ] Profiles/scenarios: list/load/apply — test with real configs
- [ ] Media controls: prefer `mpris.rs` or consolidate `Audio.Media.*`
- [ ] Unit tests: parse `pactl`/`wpctl` fixtures
- [ ] Integration tests: volume set round-trip (mock)
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) audio panel, [sidebar-popup.md](roadmap/sidebar-popup.md)

### 2.5 Brightness (`brightness.rs`) — P1

- [ ] Multi-monitor `Brightness.Get`/`Set` via brightnessctl or ddcutil
- [ ] Parse relative adjustments (`+10%`, etc.) — unit tests exist for parser; **wire to Set**
- [ ] Integration with GNOME/Hyprland brightness keys
- [ ] Tests: parser + command builder

### 2.6 System (`system.rs`) — P1

- [ ] `System.GetStats`: CPU, RAM, GPU, temps via `sysinfo` + sensors
- [ ] Per-disk usage (extend beyond single number)
- [ ] Optional: load averages, uptime
- [ ] Unit tests: sysinfo snapshot thresholds

### 2.7 VPN (`vpn.rs`) — P1

- [ ] Port Caelestia state machine (OpenConnect/OpenVPN/WireGuard)
- [ ] `Vpn.GetProfiles` from config dir
- [ ] `Vpn.Connect`/`Disconnect` with polkit/sudo safe wrapper
- [ ] Status: connected, IP, DNS leak hints
- [ ] **Per-app routing** (roadmap) — design doc + nftables/NM split tunnel
- [ ] Kill switch (optional, dangerous — feature flag)
- [ ] Unit tests: state transitions with mock process
- [ ] Integration tests: connect failure paths

### 2.8 Hyprland (`hyprland.rs`) — P1

- [x] `GetWorkspaces` / `GetClients` / `GetActiveWindow` — typed JSON matching React bar
- [x] `Dispatch` allowlist (prevent arbitrary command injection)
- [x] `GetMonitors` for multi-monitor bar
- [x] Event subscription → WebSocket (`Hyprland.StateChanged` via socket2; `AURA_HYPRLAND_EVENTS=0` to disable)
- [x] Unit tests: parse hyprctl JSON fixtures (`tests/fixtures/hyprland/`, `wave5_*` tests)
- [ ] Integration tests with live `hyprctl` on CI host (optional; fixtures cover parsers)

### 2.9 Shell / session (`shell.rs`) — P1

- [ ] `Session.Lock` → `hyprlock`/`loginctl lock-session`
- [ ] Logout/suspend/reboot/poweroff via `loginctl` with polkit
- [ ] `Aura.ToggleWindow` → delegate to AGS IPC (document contract)
- [ ] `Apps.Launch` allowlist (xdg-open, gtk-launch)
- [ ] Tests: verify allowlist rejects unknown ids

### 2.10 MPRIS / media (`mpris.rs`) — P1

- [x] `Media.GetNowPlaying` — `playerctl status` + metadata; `playing`/`paused`/`player_name`
- [x] Play/pause/next/prev wired in `api.ts` → `Audio.Media.*` (bar controls)
- [x] Tests with fixtures (`playerctl_status.txt`, `wave5_media_shapes.rs`)

### 2.11 Processes (`processes.rs`) — P1

- [ ] `Process.ListTop` — stable sort, configurable limit
- [ ] `Process.Kill` — SIGTERM then SIGKILL with confirmation token from UI
- [ ] Align with `Performance.GetProcesses` — dedupe or delegate
- [ ] Roadmap: [sidebar.md](roadmap/sidebar.md) task manager

### 2.12 Weather (`weather.rs`) — P2

- [ ] `Weather.Get` with API key from env/keyring
- [ ] Forecast/hourly/locations methods — implement or remove from registry
- [ ] Cache + rate limit
- [ ] Unit tests: parse API JSON fixtures

### 2.13 GameMode (`gamemode.rs`) — P2

- [ ] `GameMode.IsEnabled` / toggle via `gamemoded` dbus
- [ ] Add `GameMode.GetStatus` alias for GTK client
- [ ] Tests with mocked dbus

### 2.14 Storage (`storage.rs`) — P2

- [ ] Define Aura config schema (`Storage.Get`/`Set` for namespaced JSON)
- [ ] `Storage.GetConfig` alias for GTK
- [ ] Validate schema on write
- [ ] Tests: schema validation

### 2.15 Packages (`packages.rs`) — P2

- [x] `Packages.GetUpgradable` — pacman `-Qu` (Arch only)
- [x] **`Packages.GetUpdates` alias** for UI (client uses `GetUpgradable`)
- [x] Install/remove/update with `pkexec`/`sudo` helper
- [ ] Dependency graph: `GetPackageDependencies` + reverse deps (“why installed”)
- [x] **Transaction history** append-only log in `~/.local/share/ags-sidecar/transactions.jsonl`
- [ ] AUR/Flatpak/Snap methods — verify or gate behind optional tools
- [ ] Auto-update policy RPC (schedule, security-only)
- [ ] Unit tests: parse pacman output fixtures
- [ ] Integration tests: dry-run install mocked
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) package panel

### 2.16 Logs (`logs.rs`) — P2

- [x] **`Logs.Get`** returning `LogEntry[]` for Control Center (journalctl structured)
- [x] Map journal priority to `level` field (`journalctl -o json`, `PRIORITY` → level)
- [ ] `FollowLogs` → WebSocket stream (optional)
- [x] `Logs.Get` filters: `lines`, `priority`, `unit`, `grep`
- [ ] Filter/search/export methods — implement or remove dead RPCs
- [x] Unit tests: journal JSON parser fixture
- [ ] Roadmap: Logs pane

### 2.17 Security (`security.rs`) — P2

**Defensive (daily use)**

- [x] **`Security.GetStatus`** aggregate for UI (firewall, ssh, encryption + fail2ban/clamav/fprintd probes)
- [ ] Firewall enable/disable — verify ufw/nftables
- [ ] SSH status, failed logins, sudo logs
- [ ] Port scan (local only) — safety caps
- [ ] Encryption/LUKS detection
- [ ] Certificate listing
- [ ] **AV integration**: ClamAV scheduled scan RPC
- [ ] **Scan on package install** hook (async, opt-in)
- [ ] **fprintd**: list/enroll/test fingerprint
- [ ] Password policy status (expire, weak configs) — read-only

**Offensive (pentest panel — P4, feature-gated)**

- [ ] Move `Security.Offensive.*` (~60+ methods) behind `offensive-security` feature flag
- [ ] Allowlist binaries; refuse as root by default
- [ ] Audit logging for every offensive invocation
- [ ] Document legal/ethical use in [pentest-panel.md](roadmap/pentest-panel.md)
- [ ] Do **not** expose full surface to React until reviewed

- [ ] Unit tests per defensive parser
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) security panel

### 2.18 Performance (`performance.rs`) — P2

- [x] **`Performance.GetMetrics`** aggregate for UI
- [x] `Performance.MetricsChanged` WS push (30s poll, debounced)
- [ ] CPU/GPU/memory/disk/network stats — real data paths
- [ ] `SetCpuGovernor` / frequency — polkit + safety caps
- [ ] Process list + priority — align with `processes.rs`
- [ ] Systemd service control — confirm polkit
- [ ] **Presets** RPC: meeting/compile/game → power profile + governor
- [ ] Unit tests: parse `/proc`, `ps` fixtures
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) performance panel

### 2.19 DevOps (`devops.rs`) — P2

- [x] **`DevOps.GetStatus`** summary: podman/docker runtime, container count, k8s hint, git dirty count (`AURA_GIT_ROOTS`)
- [ ] Docker: containers/images/stats — handle missing docker.sock gracefully
- [ ] Podman/K8s optional
- [ ] Git repo discovery + status
- [ ] Systemd timers, cron listing
- [ ] **Cloud connector** stubs (AWS/GCP health check) — P3
- [ ] Unit tests: docker/ps fixtures
- [ ] Roadmap: [devops-panel.md](roadmap/devops-panel.md)

### 2.20 Productivity (`productivity.rs`) — P2

- [x] **`Productivity.GetStats`** aggregate (task counts, pomodoro, focus, timers)
- [x] Timers/pomodoro — in-memory ticks; pomodoro prefs in SQLite; **`Productivity.TimerTick`** WS
- [x] Tasks CRUD with storage **list/delete** (`GetTasks`, `DeleteTask`)
- [ ] Focus mode / site blocking — integrate `/etc/hosts` or nftables (dangerous — confirm UX)
- [ ] Screen time / app usage — integrate ActivityWatch or similar (roadmap Cold Turkey)
- [ ] Unit tests: timer state machine
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) productivity panel

### 2.21 Automation (`automation.rs`) — P2

- [ ] Fix `Automation.GetWorkflows` to list from SQLite
- [ ] Fix delete workflow (storage delete)
- [ ] **`Automation.ListRules` alias** for UI
- [ ] **`Automation.Trigger` alias** → `RunWorkflow`
- [ ] Trigger engine: cron, file watch, RPC (use `notify` crate carefully)
- [ ] Script runner sandbox (timeout, allowlist)
- [ ] Webhook ingress (localhost only)
- [ ] Vaultwarden/n8n **integration RPCs** (webhook URLs, not embedded n8n)
- [ ] Unit tests: workflow JSON schema validation
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) automations

### 2.22 Communication (`communication.rs`) — P3

- [ ] **`Communication.GetUnread`** — per-app counts (telegram, discord, …)
- [ ] Implement `GetMessages` / `GetConversations` (even stub per bridge)
- [ ] Matrix (mautrix) bridge adapter — phase 1 read-only
- [ ] Mark read / send message — per bridge
- [ ] Mute/DND integration with notification service
- [ ] `LaunchApp` allowlist
- [ ] Unit tests: unread aggregation logic
- [ ] Roadmap: [communication-panel.md](roadmap/communication-panel.md)

### 2.23 Calendar (`calendar.rs`) — P2

- [x] `Calendar.GetEvents` — load from SQLite via `scan_namespace`
- [x] CRUD: create/update/delete with validation (`DeleteEvent` via storage)
- [x] `Calendar.GetUpcomingEvents` — sorted upcoming window
- [x] Reminders v0 — sidecar tick → notification inbox
- [ ] `GetCalendars`, `SyncCalendars` — CalDAV/Google OAuth (P3)
- [ ] ICS import/export — implement parsers (use `quick-xml`)
- [ ] Reminders → notification service + WS
- [ ] `GetUpcomingEvents` for sidebar tile
- [ ] Unit tests: ICS parse, recurrence (later)
- [ ] Roadmap: [calendar-panel.md](roadmap/calendar-panel.md)

### 2.24 Fitness (`fitness.rs`) — P3

- [ ] `Fitness.GetGoals` — persist goals
- [ ] Activity/steps/heart rate — device sync stubs
- [ ] Workout start/stop/history
- [ ] Integrate Google Fit / Strava (optional, keyring tokens)
- [ ] Roadmap: calendar fitness hooks

### 2.25 Calendar + Fitness UI API

- [ ] Extend `api.ts` with calendar CRUD methods (not only `GetEvents`)
- [ ] Typed parsers in `api-types.ts` for all calendar/fitness DTOs

---

## 3. New services to create

### 3.1 Keybinds (`keybinds.rs`) — P1

- [x] `Keybinds.List` — parse `~/.config/hypr/hyprland.conf` + conf.d (`source =`, depth cap)
- [x] `Keybinds.GetCategories` — window/workspace/media/etc.
- [x] `Keybinds.Set` / `Unset` — safe write to `~/.config/ags/hypr/aura-binds.conf` + backup
- [x] `Keybinds.Validate` — conflict detection + dispatch allowlist hints
- [x] `Keybinds.Export` / `Import`
- [x] `Keybinds.Reload` — `hyprctl reload`
- [ ] Optional: global shortcuts via `keyd` integration
- [x] Unit tests: parse/bind conflict fixtures
- [x] Register in `build_registry()`, `api.ts` (GTK `sidecar.ts` deferred)
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) keybinds pane (live load in UI v1)

### 3.2 Notifications (`notifications.rs`) — P1

- [x] Subscribe to Freedesktop Notification spec (D-Bus: `dbus-monitor` + `NotificationClosed` signals)
- [x] `Notifications.List` — history with filters
- [x] `Notifications.Dismiss` / `ClearAll`
- [x] `Notifications.GetDnd` / `SetDnd` + schedule (SQLite)
- [x] `Notifications.GetRules` / `SetRules` per app
- [x] `Notifications.InvokeAction` (best-effort)
- [x] Mirror to WebSocket (`Notifications.Changed`) + Control Center pane
- [x] Unit tests: parse notification payloads
- [ ] Roadmap: [system-and-input-foundation.md](roadmap/system-and-input-foundation.md), [control-panel.md](roadmap/control-panel.md)

### 3.3 Launcher / Vicinae (`launcher.rs`) — P2

- [ ] `Launcher.Query` — fuzzy app search (desktop entries)
- [ ] `Launcher.Run` — allowlisted commands
- [ ] `Vicinae.Exec` — integration contract with Vicinae binary/RPC
- [ ] `Launcher.Recent` / `Pin`
- [ ] Custom user commands store (SQLite)
- [ ] Roadmap: Vicinae overpowered integration in todo.md

### 3.4 Capture (`capture.rs`) — P2

- [ ] `Capture.Screenshot` — region/window/full (grim+slurp)
- [ ] `Capture.Record` — start/stop (wf-recorder)
- [ ] `Capture.ListDevices` — audio/video inputs
- [ ] Clipboard vs file path options
- [ ] Unit tests: command assembly only
- [ ] Roadmap: screenshot/recording revamp

### 3.5 Settings / Aura config (`settings.rs`) — P2

- [ ] `Settings.Get` / `Settings.Set` — Aura JSON (theme, layout, panel modules)
- [ ] `Settings.GetSchema` — for UI forms
- [ ] `Settings.Reset` namespace
- [ ] Wayland/compositor options proxy (read hypr config)
- [ ] Distro settings links (read-only pointers)
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) system settings

### 3.6 Todos (`todos.rs`) — P2

- [ ] CRUD + projects + due dates
- [ ] Natural language date parsing (library)
- [ ] CalDAV VTODO sync (optional)
- [ ] Reminders → notifications
- [ ] Roadmap: [calendar-panel.md](roadmap/calendar-panel.md) todo subpanel

### 3.7 Vault (`vault.rs`) — P3

- [ ] Provider registry (rclone configs)
- [ ] `Vault.List` / `Transfer` / `Backup.Start` / `Backup.Status`
- [ ] Encryption helpers (age/gpg) — orchestration only
- [ ] Share links metadata (no plaintext keys in logs)
- [ ] Roadmap: [vault-panel.md](roadmap/vault-panel.md)

### 3.8 Lock / session extended (`lock.rs`) — P2

- [ ] `Lock.GetConfig` / `SetConfig` — visibility toggles
- [ ] `Lock.TestFingerprint`
- [ ] `Sleep.Inhibit` / `Sleep.GetInhibitors` — logind
- [ ] Hibernate/suspend targets
- [ ] Roadmap: [lock-panel.md](roadmap/lock-panel.md)

### 3.9 IDE / agents (`ide.rs`) — P3

- [ ] `Ide.List` — detect installed IDEs
- [ ] `Ide.Launch` — folder + IDE id
- [ ] `Agents.Run` — allowlisted CLI agents (Jules, etc.)
- [ ] Roadmap: [ide-popup.md](roadmap/ide-popup.md)

### 3.10 System debug (`debug.rs`) — P3

- [ ] `Debug.CollectBundle` — logs, configs redacted
- [ ] `Debug.RestoreBackup` — orchestrate restic/borg
- [ ] `Debug.HealthCheck` — disk, failed units, last boot
- [ ] Roadmap: [system-debugging-popup.md](roadmap/system-debugging-popup.md)

### 3.11 Voice (`voice.rs`) — P4

- [ ] `Voice.ListEngines` — local/cloud
- [ ] `Voice.Transcribe` — push-to-talk file in
- [ ] Route text to Vicinae/sidecar command
- [ ] Roadmap: voice control in todo.md

### 3.12 Multitasking (`workspaces.rs`) — P3

- [ ] Workspace rules, recent windows, move semantics
- [ ] May extend `hyprland.rs` instead of new module — decide

### 3.13 Dropdown / sidebar aggregation (`dashboard.rs`) — P2

- [ ] `Dashboard.GetQuickStatus` — single RPC for top dropdown (network, bt, battery, dnd, next event)
- [ ] `Sidebar.GetTileData` — named tiles
- [ ] Roadmap: [top-dropdown.md](roadmap/top-dropdown.md), [sidebar.md](roadmap/sidebar.md)

### 3.14 Stub panels (document only until product spec)

- [ ] Exocortex — [exocortex-panel.md](roadmap/exocortex-panel.md)
- [ ] Skiller — [skiller-panel.md](roadmap/skiller-panel.md)
- [ ] Secure A — [secure-a-panel.md](roadmap/secure-a-panel.md)
- [ ] Marginal gains / KOLB — [marginal-gains-panel.md](roadmap/marginal-gains-panel.md)

---

## 4. Cross-cutting platform work

### 4.1 Login & session ([system-and-input-foundation.md](roadmap/system-and-input-foundation.md))

- [ ] Document greeter + PAM + keyring stack for this machine
- [ ] `Login.GetLastFailures` read-only diagnostic RPC
- [ ] Post-login hook: ensure NM keyring unlocked for Wi-Fi

### 4.2 Package hygiene

- [ ] `Packages.Audit` — orphans, explicit install list export
- [ ] `System.Cleanup` — cache sizes, pacman -Sc dry-run

### 4.3 FN keys / media keys

- [ ] `Input.GetFnLock` / `SetFnLock` — keyd or kernel module specific
- [ ] Map F12 row to sidecar actions (document Hypr binds)

### 4.4 Migration / Windows

- [ ] Mostly operational — optional `Migration.Checklist` static RPC

---

## 5. Client sync checklist

When a backend slice lands, update all consumers:

- [ ] `sidecar/src/types.rs`
- [ ] `src/lib/types.ts`
- [ ] `src/lib/sidecar.ts` — typed wrapper + signals if push
- [ ] `ui/src/lib/api.ts` + `ui/src/lib/api-types.ts`
- [ ] React panes under `ui/src/pages/control-center/panes/`
- [ ] GTK bar/flyouts under `ui/src/components/bar/flyouts/` and `src/widget/bar/`
- [ ] `.cursor/skills/ags-sidecar-rpc/SKILL.md` if workflow changes

---

## 6. Test case catalog (templates)

Copy per feature:

```text
Unit:
  [ ] parse_<fixture>
  [ ] validate_<params>_rejects_invalid
  [ ] state_machine_<transition>

Integration (registry):
  [ ] <Method>_returns_ok_with_fixture
  [ ] <Method>_errors_when_tool_missing

HTTP:
  [ ] GET_/api/<Method>
  [ ] POST_/api/<Method>_with_body

Manual (hardware):
  [ ] <describe physical verification>
```

### 6.1 P0 smoke suite (automate first)

- [x] `Power.GetBatteryState`
- [x] `Network.GetStatus`
- [ ] `Audio.GetDevices`
- [ ] `Hyprland.GetWorkspaces`
- [ ] `System.GetStats`
- [ ] `Brightness.Set` (mock monitor name)

### 6.2 Control Center pane coverage

| Pane | Primary RPCs | Tests needed |
|------|----------------|--------------|
| Network | `Network.*` | scan, connect mock |
| Bluetooth | `Bluetooth.*` | adapter list fixture |
| Audio | `Audio.*` | streams fixture |
| VPN | `Vpn.*` | state machine mock |
| Packages | `Packages.GetUpdates` | pacman -Qu fixture |
| Logs | `Logs.Get` | journal fixture |
| Security | `Security.GetStatus` | ufw fixture |
| Performance | `Performance.GetMetrics` | proc fixture |
| DevOps | `DevOps.GetStatus` | podman fixture |
| Automations | `Automation.ListRules` | workflow CRUD |
| Communication | `Communication.GetUnread` | mock counts |
| Calendar nav | `Calendar.GetEvents` | sqlite populate |
| Fitness | `Fitness.GetGoals` | storage |
| Productivity | `Productivity.GetStats` | timers |
| Settings | Power/Network/… aggregates | combined mock |
| Weather | `Weather.Get` | API fixture |
| Keybinds | `Keybinds.*` | **service missing** |

---

## 7. Suggested implementation order (for agents)

1. **§0.7 + §1** — test harness + contract fixes (unblocks all UI panes)
2. **§0.1 sidecar binary resolver** — `AURA_SIDECAR` + repo walk (see ADR)
3. **§0.3 storage list/delete** — unblocks Calendar, Automation, Productivity
4. **§2.1–2.4** — Power, Network, Bluetooth, Audio (daily use)
5. **§3.2 Notifications** + **§3.1 Keybinds**
6. **§2.15–2.18** — Packages, Logs, Security aggregate, Performance
7. **§2.19–2.21** — DevOps (Podman), Productivity, Automation — **skip Communication** (separate app)
8. **§2.23 Calendar** (Google/CalDAV) + **§3.6 Todos**
9. **§3.4–3.5 Capture + Settings** (install `wf-recorder` on host)
10. **§3.7+** — Vault, voice
11. **§2.17 offensive security** — `offensive-security` feature; pentest panel when enabled

---

## 8. Resolved decisions (2026-05-28)

Full rationale, host probe results, and notification guidance: **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)**.

| # | Question | **Decision** |
|---|----------|----------------|
| 1 | Package backend | **Arch only** (`pacman` / optional AUR helper) |
| 2 | Transaction history | **Local append-only log** in `~/.local/share/ags-sidecar/` (+ optional journal correlation) |
| 3 | Per-app VPN | **All phases:** NM split → nftables → optional mihomo/clash profile type |
| 4 | Keybinds | **Hyprland** + Aura-managed include file; **keyd** optional for FN row |
| 5 | Notifications | **Freedesktop D-Bus listener** (any daemon); **recommend swaync** autostart on Hyprland |
| 6 | Vicinae | **Long-lived socket/RPC**; subprocess fallback |
| 7 | Voice | **Local default**, cloud opt-in, **GPU** when available, **push-to-talk** |
| 8 | DevOps | **Podman** socket/API; RW with allowlist + confirmations (not assumed Docker) |

**Also resolved (see ADR):** SDDM; GNOME Keyring; NetworkManager; PipeWire+WirePlumber; hyprlock; `DevOps` naming; rename clients not aliases; `offensive-security` feature flag; sidecar path via `AURA_SIDECAR` + XDG + repo walk; CI RPC manifest; WS push all domains; ClamAV scans; fprintd; grim/slurp/wf-recorder; Google/CalDAV; comms/Reclaim out of scope; vault deferred; hyprlock clock-only; React webview pattern for sidebar; dropdown DnD; workspace thumbnails icon-only default; pentest yes with feature flag; IDE/stub panels deferred/ignore.

---

## 9. Progress log (optional)

| Date | Item completed | Notes |
|------|----------------|-------|
| 2026-05-29 | Wave 5 — Hyprland typed RPCs, dispatch allowlist, WS bar sync, media transport | See `docs/plans/wave_5_compositor_bar_live.md` |
| 2026-05-29 | Wave 4 — DevOps, Automation, Productivity, Calendar | See `docs/plans/wave_4_control_center_depth.md` |
| 2026-05-28 | Wave 3 — Notifications, Keybinds, Logs/Security/Performance | |
| 2026-05-28 | Test harness + ~47% coverage | `sidecar-test-fast.sh`, contract tests |

---

*Generated for Aura/AGS backend implementation planning. Update this file as items ship; link PRs in §9.*
