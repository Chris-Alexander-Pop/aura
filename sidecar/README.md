# ags-sidecar

Rust backend for Aura: JSON-RPC over stdin (GTK shell) and HTTP on `127.0.0.1:9080` (React UI).

## Build

```bash
cd sidecar
cargo build          # debug → target/debug/ags-sidecar
cargo build --release
```

System packages (Arch): `libssl-dev` / `openssl` + `pkg-config` for `openssl-sys`.

## Run

```bash
./target/debug/ags-sidecar
# HTTP: http://127.0.0.1:9080/api/Power.GetBatteryState
# CLI helper: ./target/debug/ags-sidecar client Power.GetBatteryState
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `AURA_SIDECAR` | Absolute path to the binary (GTK shell; see `src/lib/sidecar.ts`) |
| `AURA_STORAGE_DB` | SQLite path for `Storage.*` and namespaced prefs (tests use a temp file) |
| `RUST_LOG` | Tracing filter, e.g. `ags_sidecar=debug` (see `tracing_subscriber`) |
| `AURA_HYPRLAND_EVENTS` | Set `0` to disable Hyprland socket2 push listener |
| `AURA_LAUNCHER_DESKTOP_DIRS` | Colon-separated directories of `.desktop` files (tests use `tests/fixtures/desktop`) |
| `VICINAE_SOCKET` | *(planned)* Unix socket path for Vicinae daemon RPC — `Vicinae.Exec` not implemented yet |

## Binary resolution (GTK / `src/lib/sidecar.ts`)

1. `AURA_SIDECAR` — absolute path to the binary  
2. `$XDG_CONFIG_HOME/ags/sidecar/target/{debug,release}/ags-sidecar`  
3. `$HOME/Engineering/Productivity/ags/sidecar/target/{debug,release}/ags-sidecar` (dev clone)

Symlink or copy builds into `~/.config/ags/sidecar/target/` when using `~/.config/ags/aura`.

## Stack defaults

**Arch-first** today: NetworkManager, PipeWire + WirePlumber, `powerprofilesctl`, pacman, Podman, GNOME Keyring, Freedesktop notifications D-Bus. Other distros may need alternate code paths later — see [../docs/ARCHITECTURE_DECISIONS.md](../docs/ARCHITECTURE_DECISIONS.md).

Polkit: privileged helpers (VPN, firewall, some package actions) expect a working polkit agent when those RPCs are invoked from the UI.

## Real-time push (WebSocket + GTK)

Services emit JSON events on `ws://127.0.0.1:9080/ws` and on **stdout** for the GTK sidecar (wrapped as JSON-RPC notifications).

Message shape:

```json
{ "method": "Power.BatteryState", "params": { "percent": 80, "charging": false, "time_remaining": "2h" } }
```

Examples: `Power.BatteryState`, `Power.Profile`, `Network.StateChanged`, `Bluetooth.StateChanged`, `Audio.StateChanged`, `Notifications.Changed`, `Performance.MetricsChanged`, `Hyprland.StateChanged`, `Calendar.EventsChanged`, `Productivity.TimerTick`.

## API contract (manifest + `api.ts`)

From the repo root:

```bash
./scripts/generate-rpc-manifest.sh   # refresh sidecar/rpc-manifest.json from registry.register
./scripts/check-api-rpc-contract.sh # every ui/src/lib/api.ts RPC is in the manifest
```

Both run as part of `./scripts/sidecar-test-fast.sh`.

## Tests

### Fast gate (CI / pre-push)

```bash
./scripts/sidecar-test-fast.sh
```

Runs manifest generation, `api.ts` contract check, `cargo test --lib`, and `cargo test --tests` (skips `#[ignore]` slow host sweeps).

Manual equivalent from `sidecar/`:

```bash
cargo test --lib
cargo test --tests
```

### Full suite (slow host sweep)

```bash
cd sidecar
cargo test
cargo test --test integration_test readonly_gap_methods_resolve_slow_host -- --ignored
```

### Safety / deny-list

Integration tests call the in-process registry on your real machine. **`call_method` / `call_rpc` refuse blocklisted RPCs** (session/power, network connect, package install, Hyprland dispatch, etc.). See [`tests/README.md`](tests/README.md) and `tests/common/mod.rs` (`DENIED_EXACT`, `DENIED_PREFIXES`, `is_denied_rpc_method`).

Destructive RPCs require `call_method_unchecked` inside `#[ignore]` tests with documented isolation.

### Fixtures

Golden CLI samples live under `tests/fixtures/` (e.g. `network/nmcli_*`, `power_supply/*`). Parser contracts: `tests/integration_contracts.rs`.

### Coverage (`cargo-llvm-cov`)

Requires LLVM coverage tools and `cargo-llvm-cov`:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview   # needs rustup; Arch `rust` alone lacks llvm-cov
```

From the repo root:

```bash
chmod +x scripts/sidecar-coverage.sh   # once
./scripts/sidecar-coverage.sh
./scripts/sidecar-coverage.sh --summary-only
```

Outputs:

- HTML: `sidecar/target/coverage/html/index.html`
- LCOV: `sidecar/target/coverage/lcov.info`

Baseline notes: [../docs/sidecar_coverage_baseline.md](../docs/sidecar_coverage_baseline.md).

## Hyprland, notifications, keybinds

See product docs in [../docs/roadmap/](../docs/roadmap/) and [../AGENTS.md](../AGENTS.md) for panel-level behavior.
