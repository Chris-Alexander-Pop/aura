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
cargo test --test integration_contracts
cargo test --test rpc_contract_test
cargo test --test server_http_test
cargo test --test integration_test
```

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
| **~90% line** (long-term) | Product goal for `sidecar/src/`; track via `cargo llvm-cov` |
| **~48% baseline** (2026-05) | Current total after rpc/server/notify + contract tests; see `docs/sidecar_coverage_baseline.md` |
| **Fast gate** | `./scripts/sidecar-test-fast.sh` — no slow host sweep |
| **Full coverage run** | `./scripts/sidecar-coverage.sh` — LCOV + HTML (two llvm-cov passes; see script) |

```bash
./scripts/sidecar-coverage.sh --summary-only   # quick total %
./scripts/sidecar-coverage.sh                  # LCOV + HTML under sidecar/target/coverage/
```

See `sidecar/README.md` for `cargo-llvm-cov` install steps.

## Test authoring: one test, one branch

When raising coverage toward 90%, prefer **small tests that hit one decision branch** rather than large integration loops:

- **Unit / contract** — parser or helper with a fixture under `tests/fixtures/` (`integration_contracts.rs`, `#[cfg(test)]` in the service module).
- **Integration shape** — one `#[tokio::test]` per RPC with explicit JSON field assertions (see dedicated tests in `integration_test.rs`).
- **Bulk resolve loops** — only for panic-free smoke; split slow host batches into `#[ignore]` (see `readonly_gap_methods_resolve_slow_host`).

Avoid duplicating the same branch in both a fixture unit test and a 70-method host sweep unless the integration path adds real wiring value.
