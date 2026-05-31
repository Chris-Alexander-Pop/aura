# AGS Sidecar Backend — Implementation & Test Checklist

> **Purpose:** Exhaustive backlog for `ags-sidecar` (`sidecar/`) so each item can be picked up, implemented, and verified with unit + integration tests before wiring the React/GTK UI.
>
> **Sources:** [todo.md](../todo.md), [docs/roadmap/](roadmap/), [feature_matrix.md](feature_matrix.md), [MIGRATION_STRATEGY.md](MIGRATION_STRATEGY.md), **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)**, `sidecar/src/services/*`, `ui/src/lib/api.ts`, `src/lib/sidecar.ts`.
>
> **How to use:** `[x]` done · `[~]` partial or deferred · `[ ]` not done. Prefer one vertical slice per PR. Run `cd sidecar && cargo test` after each slice.
>
> **Plans:** First program [docs/plans/README.md](plans/README.md) **implemented** (2026-05-31). Remaining work: [docs/plans/completion/README.md](plans/completion/README.md). This checklist is **not** fully complete.

---

## Executive summary (2026-05-31 audit)

| Lens | Status |
|------|--------|
| **docs/plans/** (15 vertical slices) | **Complete** — sidecar RPC + tests; see plan files for scope |
| **This checklist** | **In progress** — many shipped items were never ticked; P3/P4 and UI sync remain |
| **Ready for every roadmap panel** | **No** — several RPC namespaces lack `api.ts` wiring; Communication/Vicinae/CalDAV deferred per ADR |

### Shipped (sidecar RPC + tests)

Control-center foundation/depth/hardening, P0 System/Brightness, Hyprland bar sync, Settings/Capture, Dashboard aggregates, Launcher (no Vicinae), VPN profiles/connect, Automation SQLite + cron, Session/Lock/Sleep, Todos + ICS calendar, Vault read-only, offensive-security feature gate, foundation contract harness.

### Partial / deferred

Network `Connect` (802.1X, captive portal), VPN per-app routing, CalDAV/Google sync, `Vicinae.Exec`, Vault transfer/backup, Communication hub depth (ADR: thin stubs), Weather/Logs follow depth, Automation webhooks, audio effects profiles, GTK `sidecar.ts` push parity.

### Still open (prioritized — see §7 → [completion plans](plans/completion/README.md))

Fast-gate flakes, `api.ts` for Launcher/Todos/Vault/Dashboard/Capture, §0.4 process allowlist + polkit helper, CI workflow doc, CalDAV/Vicinae/Vault phase 2, P2 polish, P4 offensive hardening, greenfield ADRs (IDE, voice, exocortex).

---

## Snapshot (2026-05-31)

| Item | Status |
|------|--------|
| Registered RPC methods (default build) | **281** in [`sidecar/rpc-manifest.json`](../sidecar/rpc-manifest.json) (`Security.Offensive.*` excluded; use `AURA_OFFENSIVE_MANIFEST=1` when generating with feature) |
| Services registered | `power`, `network`, `bluetooth`, `audio`, `brightness`, `system`, `vpn`, `hyprland`, `shell`, `lock`, `mpris`, `processes`, `weather`, `gamemode`, `storage`, `packages`, `logs`, `security`, `performance`, `devops`, `productivity`, `automation`, `communication`, `calendar`, `ics`, `todos`, `fitness`, `keybinds`, `notifications`, `settings`, `dashboard`, `capture`, `launcher`, `vault` |
| Integration tests | **300+** test fns — `*_rpc_shapes.rs`, `*_storage_test.rs`, `integration_test`, `server_http_test`, `integration_contracts` ([sidecar/tests/README.md](../sidecar/tests/README.md)) |
| `ui/src/lib/api.ts` vs sidecar | **`./scripts/check-api-rpc-contract.sh`** — OK (29 methods); readonly sweep: `api_ts_readonly_methods_resolve` in `integration_test.rs` |
| Storage | SQLite KV — list/delete/scan, migrations |
| Real-time push | `notify` — Power, Network, BT, Audio, Notifications, Performance, Productivity, Hyprland, Calendar |
| Coverage | ~**50%** line ([sidecar_coverage_baseline.md](sidecar_coverage_baseline.md); re-run `./scripts/sidecar-coverage.sh`) |
| **Fast gate** | `./scripts/sidecar-test-fast.sh` — **not fully green** (see below) |

**Known fast-gate failures (2026-05-31):**

- `settings_storage_test::settings_get_defaults_without_row` — default theme assertion (`catppuccin-mocha` vs `dark`)
- Intermittent: `calendar_storage_test::calendar_create_emits_events_changed_after_debounce`, `automation_storage_test::*` (WS timing / SQLite isolation)

---

## Priority tiers

| Tier | Meaning | Examples |
|------|---------|----------|
| **P0** | Blocks UI today or causes 500s | Contract mismatches; Power/Network/Audio correctness |
| **P1** | Core shell daily use | Notifications, brightness, Hyprland, session |
| **P2** | Control center panes | Packages, Logs, Security, Performance, VPN |
| **P3** | Roadmap / greenfield | Vault transfers, CalDAV, Communication depth |
| **P4** | Stubs / sensitive / optional | Exocortex, offensive UI, voice |

---

## 0. Foundation & test harness

### 0.1 Project hygiene

- [x] Document **Arch-first** stance in [sidecar/README.md](../sidecar/README.md) (pacman branches; multi-distro not a goal)
- [x] `sidecar/README.md`: build, HTTP `:9080`, env vars, deny-list, coverage
- [x] **Binary path:** `AURA_SIDECAR`, XDG path, dev clone walk in `src/lib/sidecar.ts` (see ADR)
- [x] Machine-readable **RPC manifest** + regeneration in `./scripts/sidecar-test-fast.sh`
- [x] **`DevOps`** casing in `ui/src/lib/api.ts` (not `Devops`)

### 0.2 Shared types (`sidecar/src/types.rs`)

- [~] Audit DTOs vs `ui/src/lib/api-types.ts` (ongoing per service)
- [~] `#[serde(deny_unknown_fields)]` on stable outward types where feasible (Settings `AuraSettings`, `Todos`, `Launcher` result; more per service)
- [~] Mirror types in `src/lib/types.ts` (partial; VPN and others updated ad hoc)
- [x] Version RPC: `Sidecar.GetVersion` (+ HTTP tests in `server_http_test.rs`)

### 0.3 Storage (`sidecar/src/utils/storage.rs`)

- [x] `list_keys(namespace) -> Vec<String>`
- [x] `list_namespace(prefix) -> Vec<(namespace, key)>`
- [x] `delete_kv(namespace, key)`
- [x] `scan_namespace(namespace) -> Vec<serde_json::Value>`
- [x] Migrations table + schema version
- [x] Unit tests: round-trip, concurrent writes, corrupt JSON handling

### 0.4 Process & privileges (`utils/process.rs`, `utils/polkit.rs`, `utils/keyring.rs`)

- [x] Central allowlist: `run_allowlisted` / `run_allowlisted_detached` in `utils/process.rs` (launcher, capture migrated)
- [x] Default timeout (30s) + max output (1 MiB) on `exec_command` / allowlisted runs
- [x] Polkit helper: `utils/polkit.rs` `run_privileged` (security firewall, packages, performance)
- [~] Keyring: network/VPN paths (not full OAuth)
- [x] Unit tests: `MockCommandRunner` + allowlist rejection in `utils/process.rs`
- [~] Remaining services still use `exec_command` without allowlist (hyprland, logs, …)

### 0.5 Service registry

- [x] Unknown method: `MethodNotFound` → structured JSON-RPC / HTTP 404
- [x] Request logging: method + duration at `debug`; `AURA_RPC_LOG_PARAMS=1` with redaction (`utils/rpc_log.rs`)
- [x] `Security.Offensive.*` behind `offensive-security` feature ([security_offensive.rs](../sidecar/src/services/security_offensive.rs))
- [x] ~~Register alias handlers~~ — **decided:** rename clients, no aliases (ADR); `Automation.ListRules` aliases `GetWorkflows` in Rust only

### 0.6 Real-time events

- [x] Notify payload: `{ method, params }` on `/ws` and stdout forwarder
- [x] Emit on: battery, network, Bluetooth, Audio (partial); Performance, Productivity, Hyprland, Calendar
- [~] GTK `sidecar.ts` listens for some events; not all React namespaces wired
- [x] WS tests: `server_http_test.rs` (synthetic `notify::emit`; Calendar, Power, etc.)
- [~] Integration test: **live** host change → WS (`server_http_test.rs` covers synthetic `Power.BatteryState` only)

### 0.7 Test infrastructure

- [x] Unit tests: `#[cfg(test)]` in service modules + `integration_contracts.rs`
- [x] Integration tests via full `build_registry()` + `call_method` deny-list
- [x] HTTP integration: `server_http_test.rs` on `127.0.0.1:0`
- [x] Fixture directory `sidecar/tests/fixtures/` (hyprland, nmcli, audio, power_supply, brightness, packages, security, vpn, calendar, desktop, vault, …)
- [x] CI job: `.github/workflows/sidecar.yml` → `./scripts/sidecar-test-fast.sh`
- [x] Manual test matrix: `sidecar/README.md` hardware table
- [x] P0 / brightness smoke via registry (`integration_test.rs`, `brightness_rpc_shapes.rs`)
- [x] `api_ts_readonly_methods_resolve` for safe `api.ts` RPCs

---

## 1. API contract alignment (P0)

Historical gaps (2025 checklist) — **resolved for current `api.ts`:**

| Client | Resolution |
|--------|------------|
| `Packages.GetUpdates` | Client uses `Packages.GetUpgradable` or equivalent path |
| `Logs.Get` | Implemented |
| `Security.GetStatus` | Aggregate implemented |
| `Performance.GetMetrics` | Aggregate implemented |
| `DevOps.GetStatus` | Implemented; TS uses `DevOps.*` |
| `Productivity.GetStats` | Aggregate implemented |
| `Automation.ListRules` / `Trigger` | Aliased to `GetWorkflows` / `RunWorkflow`; UI uses `GetWorkflows` + `Trigger` |
| `Communication.GetUnread` | Stub map implemented |
| `GameMode` | UI uses `GameMode.IsEnabled` (no `GetStatus` alias per ADR) |
| `Storage` config | Aura prefs use `Settings.*`; generic `Storage.*` for KV |
| `Automation.Trigger` | Registered; aliases `RunWorkflow` |

- [x] Contract table satisfied for current React client
- [x] `scripts/check-api-rpc-contract.sh` (29 `api.ts` methods)
- [x] `api_ts_readonly_methods_resolve` integration test
- [x] Manifest parity via `rpc_contract_test.rs` + fast gate manifest regen

**Not in `api.ts` yet (sidecar only):** `Launcher.*`, `Todos.*`, `Vault.*`, `Dashboard.*`, `Sidebar.*`, full `Capture.*` — see §5.

---

## 2. Existing services — implement, harden, test

For each service: **(a)** integration **(b)** typed responses **(c)** fixture tests **(d)** registry tests **(e)** `api.ts` / GTK if user-facing.

### 2.1 Power (`power.rs`) — P0

- [x] Battery % from sysfs / UPower
- [x] Charging state + time remaining
- [x] Power profiles (`powerprofilesctl` / fallbacks)
- [x] `Power.SetProfile`
- [x] `Power.BatteryState` notifications
- [x] Sysfs fixtures (`tests/fixtures/power_supply/`, `integration_contracts.rs`)
- [~] Integration: `GetProfile` / `SetProfile` (deny-listed in default harness; host-only)

### 2.2 Network (`network.rs`) — P0

- [x] `Network.GetStatus` (wifi, ethernet, IPs, public IP)
- [x] `Network.ScanNetworks`
- [~] `Network.Connect`: WPA/WPA3/open + keyring; clearer nmcli errors; **802.1X + captive portal deferred**
- [x] Saved networks list/forget/auto-connect
- [~] Captive portal detection (deferred — no `Network.GetCaptivePortal` yet)
- [x] Keyring for passwords
- [x] `Network.ToggleWifi`
- [~] VPN fields in `GetStatus` (VPN service separate)
- [x] nmcli fixtures in `integration_contracts.rs`
- [x] Connect fixtures + `#[ignore]` live test (`AURA_NETWORK_TEST_SSID`; deny-list unchanged)

### 2.3 Bluetooth (`bluetooth.rs`) — P0

- [x] Adapter power/discoverable
- [~] Pairing flow + agent (PIN UI deferred; documented in `bluetooth.rs`)
- [x] Device battery % when available
- [~] Audio profile selection (A2DP/HSP deferred; no RPC stub)
- [x] `Bluetooth.GetDeviceInfo`
- [x] Stop scan, remove device
- [x] `bluetoothctl` fixtures + `bluetooth_rpc_shapes.rs`

### 2.4 Audio (`audio.rs`) — P0

- [x] `Audio.GetDevices` / `GetStreams` (PipeWire / pactl)
- [x] Default device, mute sync, per-stream volume
- [x] `Audio.SetDefaultDevice`, `Audio.Refresh`
- [x] Sink/source volume RPCs
- [~] Effects submodule (mutating RPCs gated on `AURA_AUDIO_ADVANCED=1`; EasyEffects verification incomplete)
- [~] Profiles/scenarios (mutating RPCs gated on `AURA_AUDIO_ADVANCED=1`; real-config tests thin)
- [x] Media via `mpris.rs` + `Audio.Media.*` + bar `api.ts`
- [x] pactl/wpctl fixture coverage (`pactl_list_sinks`, extended `wpctl_status`, `integration_contracts`)
- [~] Volume round-trip integration (`#[ignore]` + `AURA_AUDIO_VOLUME_TEST=1`)

### 2.5 Brightness (`brightness.rs`) — P1

- [x] Multi-monitor `Brightness.Get` / `Set` (brightnessctl; `monitor: all`)
- [~] Relative adjustments (`+10%`) — parser exists; Set wiring partial
- [ ] Hyprland/GNOME keybind integration (document binds)
- [x] Fixtures + `brightness_rpc_shapes.rs`; `AURA_BRIGHTNESS_DRY_RUN`

### 2.6 System (`system.rs`) — P1

- [x] `System.GetStats` (sysinfo + `/proc` + sensors/GPU/df)
- [~] Per-disk detail (basic via df in stats)
- [~] Load averages / uptime (partial in implementation)
- [~] Unit thresholds (`system_rpc_shapes.rs`)

### 2.7 VPN (`vpn.rs`) — P1

- [~] State machine (OpenConnect/OpenVPN/WireGuard; not full Caelestia port)
- [x] `Vpn.GetProfiles` from config dir + defaults
- [x] `Vpn.Connect` / `Disconnect` (allowlist, `AURA_VPN_DRY_RUN`, deny-list in tests)
- [x] `Vpn.GetStatus` (+ optional interface/IP)
- [ ] Per-app routing / kill switch (roadmap)
- [x] State transition unit tests + `vpn_rpc_shapes.rs`
- [x] `#[ignore]` connect failure / dry-run tests

### 2.8 Hyprland (`hyprland.rs`) — P1

- [x] Typed workspaces/clients/monitors/active window
- [x] `Dispatch` allowlist
- [x] `Hyprland.StateChanged` WebSocket
- [x] hyprctl JSON fixtures + `hyprland_rpc_shapes.rs` + `hyprland_internal_test.rs`
- [ ] Live `hyprctl` on CI (optional)

### 2.9 Shell / session (`shell.rs`, `lock.rs`) — P1

- [x] `Session.Lock` → `hyprlock` / `loginctl`
- [x] Logout/suspend/reboot/poweroff via logind
- [ ] `Aura.ToggleWindow` → AGS IPC contract
- [~] `Apps.Launch` (registered; overlap with `Launcher.Run`)
- [x] Session tests in `shell.rs` `#[cfg(test)]`; `shell_rpc_shapes.rs`
- [x] `Lock.GetConfig` / `SetConfig`, `Lock.TestFingerprint`, `Sleep.GetInhibitors`, `Sleep.Inhibit`

### 2.10 MPRIS / media (`mpris.rs`) — P1

- [x] `Media.GetNowPlaying` + `Audio.Media.*`
- [x] Bar `api.ts` transport
- [x] playerctl fixtures + `audio_rpc_shapes.rs`

### 2.11 Processes (`processes.rs`) — P1

- [x] `Process.ListTop` (shared collector with Performance)
- [x] `Process.Kill` with `confirmation_token`
- [x] Aligned with `Performance.GetProcesses`
- [ ] Sidebar task manager UI (roadmap)

### 2.12 Weather (`weather.rs`) — P2

- [~] `Weather.Get` (env key; basic implementation)
- [~] Forecast/hourly/locations — verify or prune dead RPCs
- [ ] Cache + rate limit hardening
- [~] `weather_rpc_shapes.rs`

### 2.13 GameMode (`gamemode.rs`) — P2

- [~] `GameMode.IsEnabled` / toggle (dbus when present)
- [ ] `GameMode.GetStatus` alias (ADR: clients use `IsEnabled`)
- [~] `gamemode_rpc_shapes.rs`

### 2.14 Storage (`storage.rs`) — P2

- [x] Generic KV `Storage.Get` / `Set` / `Delete` / `ScanNamespace`
- [~] Aura app schema lives in `Settings.*` (not `Storage.GetConfig` alias)
- [~] Schema validation on generic storage (minimal)
- [x] Storage round-trip tests in `integration_test.rs`

### 2.15 Packages (`packages.rs`) — P2

- [x] `Packages.GetUpgradable` / UI path for updates
- [x] Install/remove/update helpers
- [x] `GetPackageDependencies`, `GetReverseDependencies`, `GetAutoUpdatePolicy`
- [x] Transaction history jsonl
- [ ] AUR/Flatpak/Snap depth
- [x] pacman/pactree fixtures + `packages_rpc_shapes.rs`
- [ ] Dry-run install integration

### 2.16 Logs (`logs.rs`) — P2

- [x] `Logs.Get` + journal JSON levels
- [ ] `FollowLogs` WebSocket stream
- [x] Filters: lines, priority, unit, grep
- [~] Export/search RPCs — audit dead methods
- [x] Journal fixture tests
- [x] Logs pane uses `Logs.Get`

### 2.17 Security (`security.rs`) — P2

**Defensive**

- [x] `Security.GetStatus` aggregate
- [~] Firewall enable/disable (polkit paths; verify on host)
- [~] SSH / encryption probes in aggregate
- [ ] Port scan safety caps (offensive gated separately)
- [~] `ListFingerprints`, `GetPasswordPolicy`, `RunClamAV` RPCs
- [ ] Scan-on-package-install hook
- [x] Defensive parser fixtures (`security_contracts.rs`, `security_rpc.rs`)

**Offensive (P4)**

- [x] `Security.Offensive.*` behind `offensive-security` feature
- [ ] Allowlist binaries; refuse-as-root default
- [ ] Audit log per invocation
- [ ] Pentest panel React exposure
- [x] Default build excludes offensive from manifest + registry tests

### 2.18 Performance (`performance.rs`) — P2

- [x] `Performance.GetMetrics` + `MetricsChanged` WS
- [~] Real `/proc` / meminfo paths (improved; presets open)
- [~] `SetCpuGovernor` (polkit path exists)
- [x] Process list via shared collector
- [~] Systemd service control RPCs (verify polkit)
- [ ] Presets RPC (meeting/compile/game)
- [x] meminfo fixture + `performance_rpc_shapes.rs`

### 2.19 DevOps (`devops.rs`) — P2

- [x] `DevOps.GetStatus` (Podman-first)
- [~] Docker/K8s listing when `AURA_ALLOW_DOCKER=1`
- [~] Git roots via `AURA_GIT_ROOTS`
- [ ] Systemd timers / cron listing
- [ ] Cloud connector stubs
- [~] `devops_rpc_shapes.rs`

### 2.20 Productivity (`productivity.rs`) — P2

- [x] `Productivity.GetStats`, timers, WS `TimerTick`
- [x] Tasks CRUD + storage
- [ ] Focus mode / site blocking
- [ ] Screen time / ActivityWatch
- [~] Timer tests via storage/rpc shapes

### 2.21 Automation (`automation.rs`) — P2

- [x] `GetWorkflows` / `ListRules` from SQLite
- [x] Delete/update/enable workflows
- [x] `Trigger` → `RunWorkflow` + run log
- [x] Cron tick (60s); schema validation tests
- [x] Script runner allowlist + timeout
- [ ] Webhook ingress
- [ ] File-watch triggers
- [x] `automation_storage_test.rs`, `automation_rpc_shapes.rs`
- [x] AutomationsPane wired

### 2.22 Communication (`communication.rs`) — P3

- [x] `Communication.GetUnread` stub map
- [~] `GetMessages` / `GetConversations` stubs
- [ ] Matrix bridge / real bridges (ADR: separate app long-term)
- [ ] Mark read / send per bridge
- [x] `communication_rpc_shapes.rs`

### 2.23 Calendar (`calendar.rs`, `ics.rs`) — P2

- [x] SQLite CRUD + upcoming + reminders
- [x] `Calendar.EventsChanged` WS
- [x] ICS import/export (`ics.rs`, fixtures, storage tests)
- [ ] CalDAV/Google sync (`TODO` in `SyncCalendars`)
- [x] Reminders → notifications
- [~] Recurrence / full CalDAV

### 2.24 Fitness (`fitness.rs`) — P3

- [~] Goals persistence (`fitness_storage_test.rs`)
- [ ] Device sync / workouts
- [ ] Google Fit / Strava

### 2.25 Calendar + Fitness UI API

- [~] `api.ts` calendar (partial — not full CRUD surface)
- [ ] `api.ts` fitness
- [~] `api-types.ts` coverage

---

## 3. New services

### 3.1 Keybinds — P1

- [x] List, categories, Set/Unset, Validate, Export/Import, Reload
- [ ] keyd integration (optional)
- [x] Fixtures + `keybinds_rpc_shapes.rs`
- [~] `api.ts` (GTK deferred)

### 3.2 Notifications — P1

- [x] D-Bus listener, List, Dismiss, DND, rules, WS, tests
- [x] Control Center pane

### 3.3 Launcher / Vicinae — P2

- [x] `Launcher.Query`, `Run`, `Recent`, `Pin` (SQLite recent/pin)
- [ ] `Vicinae.Exec` (see `VICINAE_SOCKET` in README)
- [ ] `api.ts` wrappers

### 3.4 Capture — P2

- [x] `Capture.Screenshot`, `RecordStart`/`RecordStop`, `ListDevices`
- [x] argv tests in module; `capture_rpc_shapes.rs`
- [ ] `api.ts` / keybind UX

### 3.5 Settings — P2

- [x] `Settings.Get` / `Set` / `GetSchema` / `Reset`
- [x] `SettingsPane` wired
- [ ] Hypr proxy / distro links (read-only pointers)

### 3.6 Todos — P2

- [x] CRUD, projects, `ParseDueDate`, reminders → notifications, `Todos.Changed`
- [ ] CalDAV VTODO
- [ ] `api.ts` + calendar subpanel UI

### 3.7 Vault — P3

- [~] `Vault.List`, `Vault.Backup.Status` (read-only slice)
- [ ] `Vault.Transfer`, `Backup.Start`, encryption helpers
- [ ] `api.ts`

### 3.8 Lock / session extended — P2

- [x] `Lock.*`, `Sleep.*` RPCs
- [ ] Hibernate targets / lock panel UI

### 3.9–3.11 IDE, debug, voice — P3/P4

- [ ] Not started (roadmap only)

### 3.12 Multitasking

- [ ] Extend `hyprland.rs` or new module — not started

### 3.13 Dashboard — P2

- [x] `Dashboard.GetQuickStatus`, `Sidebar.GetTileData`
- [x] `dashboard_rpc_shapes.rs`
- [ ] `api.ts` + dropdown/sidebar UI wiring

### 3.14 Stub panels

- [ ] Document-only: exocortex, skiller, secure-a, marginal gains (roadmap specs)

---

## 4. Cross-cutting platform work

### 4.1 Login & session

- [ ] Greeter/PAM/keyring doc
- [ ] `Login.GetLastFailures`
- [ ] Post-login NM keyring hook

### 4.2 Package hygiene

- [ ] `Packages.Audit`, `System.Cleanup`

### 4.3 FN keys

- [ ] `Input.GetFnLock` / `SetFnLock`
- [ ] F-row → sidecar actions doc

### 4.4 Migration

- [ ] Optional `Migration.Checklist` RPC

---

## 5. Client sync checklist

When a backend slice lands, update consumers. **2026-05-31 gap table:**

| RPC namespace | Sidecar | `api.ts` | GTK `sidecar.ts` | CC / bar UI |
|---------------|---------|----------|------------------|-------------|
| Power, Network, BT, Audio | Yes | Yes | Partial | Yes |
| Hyprland, Media | Yes | Yes | Partial | Bar |
| Logs, Security, Performance, DevOps, Productivity | Yes | Yes | — | CC panes |
| Automation | Yes | Yes | — | AutomationsPane |
| Calendar (read) | Yes | Partial | — | CalendarNav |
| Settings | Yes | Yes | — | SettingsPane |
| Notifications, Keybinds | Yes | Partial | — | CC |
| **Launcher** | Yes | **No** | **No** | — |
| **Todos** | Yes | **No** | **No** | — |
| **Vault** | Yes | **No** | **No** | — |
| **Dashboard / Sidebar** | Yes | **No** | **No** | — |
| **Capture** | Yes | **Minimal** | **No** | — |
| VPN | Yes | Partial | — | Partial |

- [~] `sidecar/src/types.rs` — per-service updates
- [~] `src/lib/types.ts` — partial
- [ ] `src/lib/sidecar.ts` — push parity for new events
- [~] `ui/src/lib/api.ts` + `api-types.ts` — core CC done; gaps above
- [~] React panes — many wired; Launcher/Todos/Vault/Dashboard pending
- [~] GTK bar/flyouts — Hyprland/media partial
- [~] `.cursor/skills/ags-sidecar-rpc/SKILL.md`

---

## 6. Test case catalog (templates)

Unchanged template for new slices. **P0 smoke** (`integration_test.rs`):

- [x] `Power.GetBatteryState`, `Network.GetStatus`
- [x] `Audio.GetDevices`
- [x] `Hyprland.GetWorkspaces` (host-dependent)
- [x] `System.GetStats`
- [~] `Brightness.Set` (deny-listed; `#[ignore]` dry-run in `brightness_rpc_shapes.rs`)

### 6.2 Control Center pane coverage

| Pane | Primary RPCs | Test status |
|------|----------------|-------------|
| Network | `Network.*` | `network_rpc_shapes.rs` |
| Bluetooth | `Bluetooth.*` | fixtures + shapes |
| Audio | `Audio.*` | `audio_rpc_shapes.rs` |
| VPN | `Vpn.*` | `vpn_rpc_shapes.rs` |
| Packages | `Packages.*` | `packages_rpc_shapes.rs` |
| Logs | `Logs.Get` | journal fixtures |
| Security | `Security.GetStatus` | `security_rpc.rs` |
| Performance | `Performance.GetMetrics` | `performance_rpc_shapes.rs` |
| DevOps | `DevOps.GetStatus` | `devops_rpc_shapes.rs` |
| Automations | `Automation.*` | storage + shapes |
| Communication | `Communication.GetUnread` | shapes |
| Calendar | `Calendar.*` | storage + ICS |
| Productivity | `Productivity.GetStats` | shapes + storage |
| Settings | `Settings.*` | `settings_*_test.rs` |
| Keybinds | `Keybinds.*` | `keybinds_rpc_shapes.rs` |
| Weather | `Weather.Get` | `weather_rpc_shapes.rs` |

---

## 7. Suggested work order (post-plans, 2026-05-31)

The first [docs/plans/README.md](plans/README.md) program is complete. **Implementation plans for the rest of this checklist:** [docs/plans/completion/README.md](plans/completion/README.md).

| Priority | Completion plan | BACKEND_TODO |
|----------|-----------------|--------------|
| 1 | [test_harness_green.md](plans/completion/test_harness_green.md) | §0.7 flakes, fast gate |
| 2 | [client_sync_api.md](plans/completion/client_sync_api.md) | §5, §3.3–3.7 UI gaps |
| 3 | [process_platform_harness.md](plans/completion/process_platform_harness.md) | §0.2–0.7 foundation |
| 4 | [p0_network_audio_depth.md](plans/completion/p0_network_audio_depth.md) | §2.1–2.4 P0 depth |
| 5 | [control_center_polish.md](plans/completion/control_center_polish.md) | §2.12–2.21 polish |
| 6 | [integrations_phase2.md](plans/completion/integrations_phase2.md) | CalDAV, Vicinae, Vault (extends [calendar_sync_todos](plans/calendar_sync_todos.md), [launcher_vicinae](plans/launcher_vicinae.md), [vault_p3_panels](plans/vault_p3_panels.md)) |
| 7 | [offensive_security_phase2.md](plans/completion/offensive_security_phase2.md) | §2.17 P4 offensive |
| 8 | [greenfield_stubs_platform.md](plans/completion/greenfield_stubs_platform.md) | §3.9–3.14, §4 deferrals |

**Definition of done:** see completion README — P0–P2 product paths + green fast gate + explicit deferrals; not every `[ ]` must ship.

---

## 8. Resolved decisions (2026-05-28)

See **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** for full rationale.

| # | Question | **Decision** |
|---|----------|----------------|
| 1 | Package backend | **Arch only** |
| 2 | Transaction history | Local append-only log |
| 3 | Per-app VPN | Phased: NM → nftables → optional clash profile |
| 4 | Keybinds | Hyprland + `aura-binds.conf`; keyd optional |
| 5 | Notifications | Freedesktop D-Bus; swaync recommended |
| 6 | Vicinae | Long-lived socket/RPC (not implemented) |
| 7 | Voice | Local default; cloud opt-in |
| 8 | DevOps | Podman-first |

---

## 9. Progress log

| Date | Item | Notes |
|------|------|-------|
| 2026-05-31 | **Completion plans** | [docs/plans/completion/](plans/completion/) — 8 slices mapped to §7 |
| 2026-05-31 | **Checklist audit** | Reconciled with codebase; first plans program complete |
| 2026-05-31 | Batch 4 — calendar/todos ICS, vault RO, offensive gate | plan commits on `master` |
| 2026-05-31 | Batch 3 — CC hardening, launcher, VPN | |
| 2026-05-31 | Batch 2 — P0 hardening, dashboard | |
| 2026-05-31 | Batch 1 — foundation, automation, session | |
| 2026-05-30 | Shell platform — Settings, Capture, calendar WS | [shell_platform.md](plans/shell_platform.md) |
| 2026-05-29 | Compositor bar, CC depth | [compositor_bar_live.md](plans/compositor_bar_live.md), [control_center_depth.md](plans/control_center_depth.md) |
| 2026-05-28 | CC foundation, test harness | [control_center_foundation.md](plans/control_center_foundation.md) |

---

*Update this file as items ship. Plans: [first program](plans/README.md) (done) · [completion phase](plans/completion/README.md) (remaining).*
