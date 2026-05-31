# Sidecar coverage baseline

**Date:** 2026-05-29 (Wave 5 hyprland/mpris refresh)  
**Repo:** `~/.config/ags` (`sidecar/`)  
**Toolchain:** `cargo-llvm-cov` + `llvm-tools-preview` (see `sidecar/README.md`)

## Test run summary

| Suite | Count | Result |
|-------|------:|--------|
| Library unit tests (`src/**/*.rs`) | 118 | pass |
| `integration_contracts` | 31 | pass (1 ignored) |
| `integration_test` | 46 | pass (1 ignored: `readonly_gap_methods_resolve_slow_host`) |
| `rpc_contract_test` | 15 | pass |
| `server_http_test` | 6 | pass |
| `*_rpc_shapes` (per service) | 40+ | pass |
| `hyprland_internal_test` | 4 | pass |
| `*_storage_test` (automation, calendar, productivity) | 3 | pass |
| Binary (`main.rs`) | 0 | `test = false` on `[[bin]]` |
| Doc tests | 0 | — |
| **Total test functions** | **250** | **248 passed, 2 ignored** (default `cargo test`) |

Fast gate (`./scripts/sidecar-test-fast.sh`): same as default `cargo test` on the targets above — skips the slow host sweep via `#[ignore]`.

Full gate adds:

```bash
cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored
```

`readonly_gap_methods_resolve` (fast set) runs in **~6s**; `readonly_gap_methods_resolve_slow_host` is **~5 min** on a typical dev machine.

## Coverage summary

| Metric | Value |
|--------|------:|
| **Total line coverage** | **46.64%** (5481 / 10272 lines) |
| Total region coverage | 46.30% |
| Total function coverage | 40.29% |

Generated via:

```bash
./scripts/sidecar-coverage.sh --summary-only
```

(`scripts/sidecar-coverage.sh` uses `CARGO_TARGET_DIR=sidecar/target/llvm-cov`, runs `--lcov` then `--html --no-run`; no conflicting `--output-path` with `--html`.)

### Infrastructure modules (previously 0%)

| Module | Line % | Notes |
|--------|-------:|-------|
| `rpc.rs` | 89.71% | In-process + `server_http_test` / RPC paths |
| `server.rs` | 65.57% | Ephemeral Axum bind in `server_http_test.rs` |
| `notify.rs` | 73.44% | WebSocket push + unit paths |
| `main.rs` | 0.00% | Entry binary; excluded from `cargo test` (`test = false`) |

### HTML report

```bash
./scripts/sidecar-coverage.sh
```

- HTML: `sidecar/target/coverage/html/index.html`
- LCOV: `sidecar/target/coverage/lcov.info`

### Top modules by line coverage

| Rank | Module | Line % |
|-----:|--------|-------:|
| 1 | `lib.rs` | 97.89 |
| 2 | `services/mod.rs` | 95.65 |
| 3 | `utils/process.rs` | 97.53 |
| 4 | `utils/storage.rs` | 92.88 |
| 5 | `utils/transactions.rs` | 91.87 |
| 6 | `rpc.rs` | 89.71 |
| 7 | `services/storage.rs` | 84.11 |
| 8 | `notify.rs` | 73.44 |
| 9 | `services/mpris.rs` | 78.79 |
| 10 | `services/system.rs` | 76.86 |
| 11 | `services/power.rs` | 75.56 |
| 12 | `services/network.rs` | 76.71 |
| 13 | `services/keybinds.rs` | 72.07 |
| 14 | `services/processes.rs` | 67.12 |
| 15 | `services/brightness.rs` | 65.70 |
| 16 | `server.rs` | 65.57 |
| 17 | `services/bluetooth.rs` | 63.21 |
| 18 | `services/hyprland.rs` | 53.25 |
| 19 | `services/packages.rs` | 51.43 |
| 20 | `services/logs.rs` | 50.30 |

### Bottom modules by line coverage

| Rank | Module | Line % | Notes |
|-----:|--------|-------:|-------|
| 1 | `main.rs` | 0.00 | Binary entry; not in test harness |
| 2 | `services/security.rs` | 22.50 | Large surface; mostly parser/probe unit tests |
| 3 | `services/devops.rs` | 19.44 | |
| 4 | `services/communication.rs` | 19.69 | |
| 5 | `utils/keyring.rs` | 18.10 | `secret-tool` paths |
| 6 | `services/automation.rs` | 25.55 | |
| 7 | `services/productivity.rs` | 24.81 | |
| 8 | `services/vpn.rs` | 28.22 | |
| 9 | `services/fitness.rs` | 26.36 | |
| 10 | `services/gamemode.rs` | 22.00 | |
| 11 | `services/weather.rs` | 39.65 | HTTP fixtures + integration |
| 12 | `services/audio.rs` | 37.22 | wpctl/pactl heavy |
| 13 | `services/shell.rs` | 30.34 | |
| 14 | `utils/privileged.rs` | 50.00 | |
| 15 | `services/performance.rs` | 41.53 | |
| 16 | `services/notifications.rs` | 46.07 | |
| 17 | `services/calendar.rs` | 48.06 | |
| 18 | `services/audio.rs` | 37.22 | |

---

## Goals and gates

| Gate | Command | Purpose |
|------|---------|---------|
| Fast | `./scripts/sidecar-test-fast.sh` | Pre-push; no ~5 min host sweep |
| Full tests | `cd sidecar && cargo test` + optional `--ignored` slow gap |
| Coverage | `./scripts/sidecar-coverage.sh` | LCOV + HTML; track toward **~90%** line goal |

Authoring guidance: `sidecar/tests/README.md` (“one test, one branch”).

---

## Integration gap analysis

**Scope:** `sidecar/rpc-manifest.json` methods matching `Get*`, `List*`, or `Validate*` (117 total).

**Dedicated integration test:** a `#[tokio::test]` that calls `call_method(..., "Service.Method", ...)` with JSON shape assertions (not only bulk resolve loops).

### Coverage tiers

| Tier | Count | Description |
|------|------:|-------------|
| Dedicated shape tests | 31+ | Own test function + field/type assertions |
| Bulk-only (resolve, no shape) | 16 | Listed in `P0_METHODS` or `WAVE4_READONLY_METHODS` only |
| Readonly gap fast set | 16 | `READONLY_GAP_FAST` in `readonly_gap_methods_resolve` |
| Readonly gap slow host | 57 | `READONLY_GAP_SLOW_HOST` in `readonly_gap_methods_resolve_slow_host` (`#[ignore]`) |
| No integration test | remainder | See manifest vs `integration_test.rs` |

Slow gap sweep: `cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored`

---

## External service contract matrix

CLI/tools the sidecar invokes via `utils/process::exec_command`, `Command::new`, or `utils/privileged` (`pkexec`/`sudo`). Fixture = checked-in file under `sidecar/tests/fixtures/`. Integration = dedicated or bulk RPC test that hits the live tool on the dev machine.

| Service | Command shape (representative) | Fixture? | Integration test? | Notes |
|---------|-------------------------------|:--------:|:-------------------:|-------|
| **nmcli** | `nmcli radio wifi`; `nmcli -t … dev wifi`; … | Yes | Yes — `Network.GetStatus`, `ListSaved`, `ScanNetworks` | Parsers in `network.rs` + `integration_contracts` |
| **bluetoothctl** | `list/show/devices/info`; `scan on/off` | Yes | Partial — adapters/devices; scan in slow gap | |
| **wpctl** / **pactl** | status, streams, defaults | Yes | Partial — `Audio.GetDevices`, `GetStreams` | |
| **playerctl** | metadata, transport | No | Partial — `Media.GetNowPlaying` | |
| **pacman** | `-Q`, `-Qu`, `-Ss`, … | Yes | Partial + slow gap | `parse_pacman_qu` validates version-shaped lines |
| **journalctl** | `-n`, `-o json`, `--grep` | Yes | Partial — `Logs.Get`; more in slow gap | |
| **hyprctl** | `-j` workspaces/clients | No | Partial — `hyprland_rpc_shapes` + bulk smoke | |
| **powerprofilesctl** | `get` / `set` | Yes | Yes — `Power.GetProfile` | |
| **HTTP (weather)** | `wttr.in`, etc. | Yes | `Weather.Get*` in slow gap + shape test | |
| **docker/podman/kubectl** | DevOps RPCs | Partial | `DevOps.GetStatus` + slow gap | |

### Fixture inventory (`sidecar/tests/fixtures/`)

Unchanged from prior baseline — see `sidecar/tests/fixtures/` tree (audio, bluetooth, brightness, devops, keybinds, logs, network, packages, power, weather).

---

## Recommended follow-ups

1. **Raise total toward 90%** — security/automation/productivity services; prefer fixture unit tests + one RPC shape test per branch (`sidecar/tests/README.md`).
2. **Run slow gap in CI nightly** — `readonly_gap_methods_resolve_slow_host -- --ignored`, not on every PR.
3. **main.rs** — keep `test = false`; optional smoke only if a zero-side-effect CLI path is added later.
