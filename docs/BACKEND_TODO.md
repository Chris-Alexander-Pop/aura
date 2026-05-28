# AGS Sidecar Backend — Implementation & Test Checklist

> **Purpose:** Exhaustive backlog for `ags-sidecar` (`sidecar/`) so each item can be picked up, implemented, and verified with unit + integration tests before wiring the React/GTK UI.
>
> **Sources:** [todo.md](../todo.md), [docs/roadmap/](roadmap/), [feature_matrix.md](feature_matrix.md), [MIGRATION_STRATEGY.md](MIGRATION_STRATEGY.md), current `sidecar/src/services/*`, `ui/src/lib/api.ts`, `src/lib/sidecar.ts`.
>
> **How to use:** Check `[x]` when done. Prefer **one vertical slice** per PR (e.g. “Network saved networks + tests + api.ts”), not half a service. Run `cd sidecar && cargo test` after each slice.

---

## Snapshot (May 2026)

| Item | Status |
|------|--------|
| Registered RPC methods | ~**273** across existing services |
| Integration tests | **Minimal** — `tests/integration_test.rs` only validates JSON shape + brightness parser |
| `ui/src/lib/api.ts` vs sidecar | **Several method name mismatches** (UI calls methods that do not exist) |
| `src/lib/sidecar.ts` (GTK) vs sidecar | **Same mismatches** on aggregate helpers |
| Dedicated services missing | **Keybinds**, **Notifications**, **Launcher/Vicinae**, **Screenshot/Capture**, **Settings/Config**, **Vault**, **Lock/Session (extended)**, **IDE**, **Todos** |
| Storage layer | SQLite KV only — **no list/delete/scan** (blocks Calendar, Automation, Productivity) |
| Real-time push | HTTP `/ws` exists; **few services emit** `push_notification` today |
| Stdin JSON-RPC + HTTP | Both hit same `ServiceRegistry` — good; tests should cover **both** entrypoints |

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
- [ ] Standardize method naming: **`DevOps` vs `Devops`**, prefer one casing everywhere

### 0.2 Shared types (`sidecar/src/types.rs`)

- [ ] Audit all public DTOs — ensure `serde` renames match `ui/src/lib/api-types.ts` (snake_case)
- [ ] Add `#[serde(deny_unknown_fields)]` on stable outward types where feasible
- [ ] Mirror types in `src/lib/types.ts` for GTK client
- [ ] Add **version field** to HTTP health: `GET /api/System.Ping` or `Sidecar.GetVersion`

### 0.3 Storage (`sidecar/src/utils/storage.rs`)

- [ ] `list_keys(namespace) -> Vec<String>`
- [ ] `list_namespace(prefix) -> Vec<(namespace, key)>`
- [ ] `delete_kv(namespace, key)`
- [ ] `scan_namespace(namespace) -> Vec<serde_json::Value>` for calendar events, workflows, tasks
- [ ] Migrations table + schema version
- [ ] Unit tests: round-trip, concurrent writes, corrupt JSON handling

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
- [ ] Register **alias handlers** for legacy client names (see §1) until clients updated

### 0.6 Real-time events

- [ ] Define notification schema: `{ method, params }` for WebSocket + future stdin notify
- [ ] Emit on: battery change, network connect/disconnect, VPN state, audio default device, Bluetooth device
- [ ] GTK `sidecar.ts` already listens for `Power.BatteryState` / `Power.Profile` — **implement emitters** in `power.rs`
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

- [ ] Battery % from `/sys/class/power_supply` or UPower (zbus)
- [ ] Charging state + **time remaining** (fix TODO in source)
- [ ] Power profiles: `powerprofilesctl` / TLP / PPD — verify on target hardware
- [ ] `Power.SetProfile` persists and reports errors clearly
- [ ] Emit `Power.BatteryState` notifications on change (poll or uevent)
- [ ] Unit tests: parse sysfs fixtures
- [ ] Integration tests: `GetBatteryState`, `GetProfile`, `SetProfile` (mock profiles daemon)

### 2.2 Network (`network.rs`) — P0

- [ ] `Network.GetStatus`: wifi enabled, active SSID, IPs, **public IP** (optional cached)
- [ ] `Network.ScanNetworks`: signal, security type, hidden SSIDs
- [ ] `Network.Connect`: open/WPA2/WPA3; **802.1X** path
- [ ] Saved networks: list, forget, auto-connect flag
- [ ] Captive portal detection helper
- [ ] Keyring integration for passwords
- [ ] `Network.ToggleWifi` reliable on NM
- [ ] Ethernet/VPN interface status in `GetStatus`
- [ ] Unit tests: parse `nmcli` fixtures
- [ ] Integration tests with **nmcli test mode** or mocked commands
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) Wi-Fi panel, [system-and-input-foundation.md](roadmap/system-and-input-foundation.md) login paths

### 2.3 Bluetooth (`bluetooth.rs`) — P0

- [ ] Adapter power/discoverable (methods exist — verify bluez)
- [ ] Pairing flow + agent handling
- [ ] Device battery % when available
- [ ] Audio profile selection (A2DP/HSP)
- [ ] `Bluetooth.GetDeviceInfo` used by UI
- [ ] Stop scan, remove device — wire to React pane
- [ ] Unit tests: `bluetoothctl` output fixtures
- [ ] Integration tests: scan/connect mocked

### 2.4 Audio (`audio.rs`) — P0

- [ ] `Audio.GetDevices` / `GetStreams` — PipeWire via `wpctl` or wireplumber D-Bus
- [ ] Default sink/source set + **mute state sync** (fix “mute BS” in todo)
- [ ] Per-stream volume/mute (already in API — verify)
- [ ] `Audio.SetDefaultDevice` — GTK bar needs this
- [ ] **Fallback path**: detect failure → `Audio.Refresh`, restart pipewire (confirm dialog from UI)
- [ ] Sink volume for default output (not only per-app streams)
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

- [ ] `GetWorkspaces` / `GetClients` / `GetActiveWindow` — typed JSON matching React bar
- [ ] `Dispatch` allowlist (prevent arbitrary command injection)
- [ ] `GetMonitors` for multi-monitor bar
- [ ] Event subscription → WebSocket (`workspace`, `window`, `monitor`)
- [ ] Unit tests: parse hyprctl JSON fixtures
- [ ] Integration tests with `HYPRCTL` mock

### 2.9 Shell / session (`shell.rs`) — P1

- [ ] `Session.Lock` → `hyprlock`/`loginctl lock-session`
- [ ] Logout/suspend/reboot/poweroff via `loginctl` with polkit
- [ ] `Aura.ToggleWindow` → delegate to AGS IPC (document contract)
- [ ] `Apps.Launch` allowlist (xdg-open, gtk-launch)
- [ ] Tests: verify allowlist rejects unknown ids

### 2.10 MPRIS / media (`mpris.rs`) — P1

- [ ] `Media.GetNowPlaying` — playerctl or zbus mpris
- [ ] Play/pause/next/prev or deprecate in favor of `Audio.Media.*`
- [ ] Tests with fixture bus names

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

- [ ] `Packages.GetUpgradable` — implement fully (pacman -Qu, apt, yay check)
- [ ] **`Packages.GetUpdates` alias** for UI
- [ ] Install/remove/update with polkit
- [ ] Dependency graph: `GetPackageDependencies` + reverse deps (“why installed”)
- [ ] **Transaction history** append-only log in `~/.local/share/ags-sidecar/transactions.log`
- [ ] AUR/Flatpak/Snap methods — verify or gate behind optional tools
- [ ] Auto-update policy RPC (schedule, security-only)
- [ ] Unit tests: parse pacman output fixtures
- [ ] Integration tests: dry-run install mocked
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) package panel

### 2.16 Logs (`logs.rs`) — P2

- [ ] **`Logs.Get`** returning `LogEntry[]` for Control Center (journalctl structured)
- [ ] Map journal priority to `level` field
- [ ] `FollowLogs` → WebSocket stream (optional)
- [ ] Filter/search/export methods — implement or remove dead RPCs
- [ ] Unit tests: journal line parser
- [ ] Roadmap: Logs pane

### 2.17 Security (`security.rs`) — P2

**Defensive (daily use)**

- [ ] **`Security.GetStatus`** aggregate for UI (firewall, ssh, encryption, keyring)
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

- [ ] **`Performance.GetMetrics`** aggregate for UI
- [ ] CPU/GPU/memory/disk/network stats — real data paths
- [ ] `SetCpuGovernor` / frequency — polkit + safety caps
- [ ] Process list + priority — align with `processes.rs`
- [ ] Systemd service control — confirm polkit
- [ ] **Presets** RPC: meeting/compile/game → power profile + governor
- [ ] Unit tests: parse `/proc`, `ps` fixtures
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) performance panel

### 2.19 DevOps (`devops.rs`) — P2

- [ ] **`Devops.GetStatus`** (or `DevOps.GetStatus`) summary: docker running, k8s context, git dirty count
- [ ] Docker: containers/images/stats — handle missing docker.sock gracefully
- [ ] Podman/K8s optional
- [ ] Git repo discovery + status
- [ ] Systemd timers, cron listing
- [ ] **Cloud connector** stubs (AWS/GCP health check) — P3
- [ ] Unit tests: docker/ps fixtures
- [ ] Roadmap: [devops-panel.md](roadmap/devops-panel.md)

### 2.20 Productivity (`productivity.rs`) — P2

- [ ] **`Productivity.GetStats`** aggregate
- [ ] Timers/pomodoro — persist state, emit WS on tick
- [ ] Tasks CRUD with storage **list/delete** fixed
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

- [ ] `Calendar.GetEvents` — load from SQLite via `scan_namespace`
- [ ] CRUD: create/update/delete with validation
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

### 3.1 Keybinds (`keybinds.rs`) — P1 — **no service today**

- [ ] `Keybinds.List` — parse `~/.config/hypr/hyprland.conf` + conf.d
- [ ] `Keybinds.GetCategories` — window/workspace/media/etc.
- [ ] `Keybinds.Set` / `Unset` — safe write with backup file
- [ ] `Keybinds.Validate` — conflict detection
- [ ] `Keybinds.Export` / `Import`
- [ ] `Keybinds.Reload` — `hyprctl reload`
- [ ] Optional: global shortcuts via `keyd` integration
- [ ] Unit tests: parse/bind conflict fixtures (see `hyprlandKeybindReference.ts` in UI)
- [ ] Register in `main.rs`, `api.ts`, `sidecar.ts`
- [ ] Roadmap: [control-panel.md](roadmap/control-panel.md) keybinds pane

### 3.2 Notifications (`notifications.rs`) — P1

- [ ] Subscribe to **mako/dunst/swaync** or freedesktop Notification spec (D-Bus)
- [ ] `Notifications.List` — history with filters
- [ ] `Notifications.Clear` / `ClearAll`
- [ ] `Notifications.GetDnd` / `SetDnd` + schedule
- [ ] `Notifications.GetRules` / `SetRules` per app
- [ ] Action invocation (reply, dismiss)
- [ ] Mirror to WebSocket for Control Center + top dropdown
- [ ] Unit tests: parse notification payloads
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

- [ ] `Power.GetBatteryState`
- [ ] `Network.GetStatus`
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
| Devops | `Devops.GetStatus` | docker fixture |
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
2. **§0.3 storage list/delete** — unblocks Calendar, Automation, Productivity
3. **§2.1–2.4** — Power, Network, Bluetooth, Audio (daily use)
4. **§3.2 Notifications** + **§3.1 Keybinds**
5. **§2.15–2.18** — Packages, Logs, Security aggregate, Performance
6. **§2.19–2.22** — DevOps, Productivity, Automation, Communication
7. **§2.23 Calendar** + **§3.6 Todos**
8. **§3.4–3.5 Capture + Settings**
9. **§3.7+** — Vault, IDE, voice, stub panels
10. **§2.17 offensive security** — only if pentest panel active; feature-gated

---

## 8. Open decisions (resolve before large work)

Record answers here as you decide:

| # | Question | Notes |
|---|----------|-------|
| 1 | Arch-only vs multi-distro package backend? | pacman + apt code paths exist |
| 2 | Transaction history: local append-only vs journal-only? | control-panel roadmap |
| 3 | Per-app VPN: NM vs nftables vs mihomo? | control-panel roadmap |
| 4 | Keybind source: Hyprland only or also keyd/GTK? | control-panel roadmap |
| 5 | Notification daemon: mako vs swaync vs built-in? | system foundation |
| 6 | Vicinae: subprocess vs RPC? | system foundation |
| 7 | Voice: local STT only or cloud opt-in? | privacy |
| 8 | DevOps: local docker.sock acceptable? | devops roadmap |

---

## 9. Progress log (optional)

| Date | Item completed | Notes |
|------|----------------|-------|
| | | |

---

*Generated for Aura/AGS backend implementation planning. Update this file as items ship; link PRs in §9.*
