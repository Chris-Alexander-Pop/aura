# Sidecar integration tests

Integration tests call the in-process `ServiceRegistry` (no HTTP server, no live sidecar binary). They still run on your real machine and may invoke subprocesses for **read-only** RPC handlers (e.g. `nmcli`, `pactl`, sysfs).

## Safety rules (live dev machine)

When adding or extending tests:

1. **No power or session actions** — do not call `Session.*`, `Power.SetProfile`, suspend/reboot/logout helpers, etc.
2. **No package changes** — no `Packages.Install`, `Remove`, `Upgrade`, `Update`, or AUR install RPCs.
3. **No privileged network mutations** — no `Network.Connect`, `Disconnect`, `Forget`, or `ToggleWifi`.
4. **No Hyprland side effects** — no `Hyprland.Dispatch`, `Keybinds.Reload`, or keybind writes (`Keybinds.Set` / `Unset` / `Import`).
5. **No killing or controlling user processes** — no `Process.Kill`, `Performance.KillProcess`, or systemd service start/stop/restart RPCs.
6. **No offensive / scanning security RPCs** — nothing under `Security.Offensive.*` or port-scan helpers.
7. **Prefer read-only RPCs** — `Get*`, `List*`, `Scan*` (read-only), and shape assertions on returned JSON.
8. **Storage** — `Storage.Set` / `Delete` are allowed only against a **temporary** DB via `AURA_STORAGE_DB` (see `storage_scan_namespace_round_trip` in `integration_test.rs`).
9. **Destructive or host-mutating RPCs** must use `call_method_unchecked` only inside `#[ignore]` tests with a comment explaining isolation (fixture paths, temp dirs, etc.).

The harness enforces this: `call_method` / `call_rpc` panic if a blocklisted method is used. See `tests/common/mod.rs` (`is_denied_rpc_method`).

### Process-global state (serialize parallel tests)

Some handlers keep in-process state shared across all tests in one `cargo test` process:

| Lock | Module | Why |
|------|--------|-----|
| `gamemode_test_lock()` | `GameMode.*` | `GAMEMODE_ENABLED` static; call `reset_gamemode_state_for_tests()` after acquiring |
| `automation_test_lock()` | automation storage | SQLite + cron tick side effects |
| `launcher_test_lock()` | launcher | `AURA_LAUNCHER_DESKTOP_DIRS` override |

Acquire the lock at the start of each affected test (see `gamemode_rpc_shapes.rs`, `automation_storage_test.rs`).

## HTTP / WebSocket server tests

`tests/server_http_test.rs` exercises `server.rs` and push delivery via `notify.rs`:

- The Axum app is started in-process with `server::bind_ephemeral()` → `127.0.0.1:0` (OS-assigned port).
- **Never binds :9080**, so an `ags-sidecar` you already have running for Hyprland/AGS is unaffected.
- Only read-safe RPCs are called over HTTP (`Sidecar.GetVersion`, `Storage.Get` with `AURA_STORAGE_DB` pointing at a temp file).
- WebSocket test connects to `/ws`, then `notify::emit` — no destructive RPCs.

## Running

### Fast gate (default CI / pre-push)

Unit tests, golden CLI contracts, manifest parity, HTTP server, and integration tests **without** `#[ignore]` slow host sweeps:

```bash
./scripts/sidecar-test-fast.sh
```

Equivalent manual run from `sidecar/`:

```bash
cargo test --lib
cargo test --tests
```

## Layout

| File pattern | Purpose |
|--------------|---------|
| `integration_contracts.rs` | Golden CLI fixture parsers (no RPC) |
| `rpc_contract_test.rs` | Manifest / API contract parity |
| `server_http_test.rs` | Axum HTTP + WebSocket push |
| `integration_test.rs` | P0 smoke, storage helpers, readonly gap sweeps |
| `{service}_rpc_shapes.rs` | Read-only `Service.Method` JSON shape tests |
| `{service}_storage_test.rs` | Mutating CRUD via temp DB (`call_method_unchecked`) |
| `hyprland_internal_test.rs` | Dispatch allowlist + socket2 event parsing (no compositor) |
| `security_*.rs` | Security service contracts and RPC guards |

`readonly_gap_methods_resolve` covers the **fast** readonly gap set (`READONLY_GAP_FAST`). The **slow** set (`READONLY_GAP_SLOW_HOST`, subprocess/HTTP/docker/journalctl/pacman/security) is in:

```bash
cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored --nocapture
```

### Full suite (including slow host sweep)

```bash
cd sidecar
cargo test
cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored
```

## Coverage goals

| Target | Role |
|--------|------|
| **70% line/region/function** (Stream A gate) | Enforced by `./scripts/sidecar-coverage-gate.sh`; current baseline ~64% lines |
| **~90% line** (long-term) | Product goal for `sidecar/src/`; track via `cargo llvm-cov` |
| **Fast gate** | `./scripts/sidecar-test-fast.sh` — no slow host sweep |
| **Full coverage run** | `./scripts/sidecar-coverage.sh` — LCOV + HTML (`--all-features`; see script) |

```bash
./scripts/sidecar-coverage.sh --summary-only   # quick total %
./scripts/sidecar-coverage.sh                  # LCOV + HTML under sidecar/target/coverage/
./scripts/sidecar-coverage-gate.sh             # fail when any total metric < 70%
```

### Offensive-security + `--all-features`

Default builds exclude `Security.Offensive.*` from the registry and manifest. With `--features offensive-security` (or `--all-features` in coverage runs):

- `security_rpc.rs`: `offensive_registry_includes_offensive_methods` asserts handlers are registered; `common::guard_tests` still deny `call_method` for offensive RPCs.
- `security_offensive_rpc_shapes.rs`: feature-gated shapes (audit log, nmap fixture, rate limit).

See `sidecar/README.md` for `cargo-llvm-cov` install steps.

## Test authoring: one test, one branch

When raising coverage toward 90%, prefer **small tests that hit one decision branch** rather than large integration loops:

- **Unit / contract** — parser or helper with a fixture under `tests/fixtures/` (`integration_contracts.rs`, `#[cfg(test)]` in the service module).
- **Integration shape** — one `#[tokio::test]` per RPC in `tests/{service}_rpc_shapes.rs` with explicit JSON field assertions.
- **Bulk resolve loops** — only for panic-free smoke; split slow host batches into `#[ignore]` (see `readonly_gap_methods_resolve_slow_host`).

Avoid duplicating the same branch in both a fixture unit test and a 70-method host sweep unless the integration path adds real wiring value.
