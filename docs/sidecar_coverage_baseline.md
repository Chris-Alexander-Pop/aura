# Sidecar coverage baseline

**Date:** 2026-06-09 (85% gate **passed** — final exec-fixture push)
**Repo:** `~/.config/ags` (`sidecar/`)  
**Toolchain:** `cargo-llvm-cov` + `llvm-tools-preview` (see `sidecar/README.md`)

## Test run summary

| Suite | Count | Result |
|-------|------:|--------|
| Library unit tests (`src/**/*.rs`) | 120+ | pass |
| `integration_contracts` | 31 | pass (1 ignored) |
| `integration_test` | 46 | pass (1 ignored: `readonly_gap_methods_resolve_slow_host`) |
| `exec_coverage_final_push_test` | 26 | pass |
| `exec_coverage_boost_test` | 12 | pass |
| `rpc_contract_test` | 15 | pass |
| `server_http_test` | 6 | pass |
| `*_rpc_shapes` (per service) | 40+ | pass |
| `hyprland_internal_test` | 4 | pass |
| `*_storage_test` (automation, calendar, productivity) | 3 | pass |
| Binary (`main.rs`) | 0 | `test = false` on `[[bin]]` |
| Doc tests | 0 | — |
| **Total test functions** | **270+** | **pass** (2 ignored in default `cargo test`) |

Fast gate (`./scripts/sidecar-test-fast.sh`): same as default `cargo test` on the targets above — skips the slow host sweep via `#[ignore]`.

Full gate adds:

```bash
cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored
```

## Coverage summary

| Metric | Value |
|--------|------:|
| **Total line coverage** | **87.87%** (core; offensive excluded) |
| Total region coverage | 85.85% |
| Total function coverage | 86.95% |

Prior baseline (2026-06-08, pre-final push): 82.86% lines / 80.82% regions / 80.89% functions.

Generated via:

```bash
./scripts/sidecar-coverage-gate.sh
```

(`scripts/sidecar-coverage.sh` uses `CARGO_TARGET_DIR=sidecar/target/llvm-cov`, runs `--all-features` tests, then `--summary-only` or LCOV + HTML.)

**85% gate** (enforced):

```bash
./scripts/sidecar-coverage-gate.sh          # pass: line/region/function >= 85% (core)
SIDECAR_COVERAGE_MIN=65 ./scripts/sidecar-coverage-gate.sh   # override threshold
```

Coverage runs use `--test-threads=1` in `sidecar-coverage.sh` to avoid SQLite / gamemode static races during `cargo llvm-cov`.

### Final-push highlights

New exec-fixture integration tests in `exec_coverage_final_push_test.rs`:

- **Bluetooth** — scan/stop, adapter power/discoverable, pair/connect/disconnect/remove (catch-all `bluetoothctl` fixture)
- **Automation** — workflow lifecycle (update, enable/disable, trigger, history, delete)
- **Power** — `SetProfile` via `powerprofilesctl` fixtures
- **Security** — firewall enable/disable, add/remove rules, ClamScan (`pkexec`/`ufw` fixtures)
- **VPN** — education/openconnect dry-run connect/disconnect
- **Calendar** — update event + sync calendars (temp storage)

Exec fixture additions: `bluetoothctl/empty`, `ufw/empty`, `asdbctl/empty`, `ddcutil/empty` catch-alls in `manifest.json`.

### HTML report

```bash
./scripts/sidecar-coverage.sh
```

- HTML: `sidecar/target/coverage/html/index.html`
- LCOV: `sidecar/target/coverage/lcov.info`

---

## Goals and gates

| Gate | Command | Purpose |
|------|---------|---------|
| Fast | `./scripts/sidecar-test-fast.sh` | Pre-push; no ~5 min host sweep |
| Full tests | `cd sidecar && cargo test` + optional `--ignored` slow gap |
| Coverage report | `./scripts/sidecar-coverage.sh` | LCOV + HTML |
| Coverage gate | `./scripts/sidecar-coverage-gate.sh` | **Pass** if line/region/function ≥ 85% (core) |

Authoring guidance: `sidecar/tests/README.md` (“one test, one branch”).

---

## Integration gap analysis

**Scope:** `sidecar/rpc-manifest.json` methods matching `Get*`, `List*`, or `Validate*` (117 total).

Slow gap sweep: `cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored`

---

## Recommended follow-ups

1. **Raise total toward 90%** — `main.rs` CLI, `server.rs` HTTP edge cases, brightness async detect paths (avoid live `brightnessctl` hangs; prefer unit tests + dry-run only).
2. **Run slow gap in CI nightly** — `readonly_gap_methods_resolve_slow_host -- --ignored`, not on every PR.
3. **main.rs** — keep `test = false`; optional smoke only if a zero-side-effect CLI path is added later.
