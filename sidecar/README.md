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
# HTTP: POST http://127.0.0.1:9080/api/Power.GetBatteryState  (header X-Aura-Token)
# Token: GET /api/meta  or  $XDG_RUNTIME_DIR/aura-http-token
# API docs (Swagger UI): http://127.0.0.1:9080/docs
# OpenAPI spec: http://127.0.0.1:9080/api/openapi.json
# CLI helper (stdio JSON-RPC, not HTTP): ./target/debug/ags-sidecar client Power.GetBatteryState
```

HTTP is a privileged local API. See [SECURITY.md](../SECURITY.md).

Regenerate the OpenAPI spec after adding RPC methods:

```bash
./scripts/generate-rpc-manifest.sh
./scripts/generate-openapi.sh
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `AURA_HTTP_TOKEN` | Optional fixed token for HTTP RPC (`X-Aura-Token`). Default: random per process, also at `$XDG_RUNTIME_DIR/aura-http-token` |
| `AURA_STORAGE_DB` | SQLite path for `Storage.*` and namespaced prefs (tests use a temp file) |
| `RUST_LOG` | Tracing filter, e.g. `ags_sidecar=debug` (see `tracing_subscriber`) |
| `AURA_RPC_LOG_PARAMS` | Set `1` to log RPC params at `debug` (secrets redacted; never logs `Storage.*` / `Settings.*` bodies) |
| `AURA_AUDIO_ADVANCED` | Set `1` to allow mutating `Audio.Effects.*` / `Audio.Profiles.*` (EasyEffects / saved profiles) |
| `AURA_NETWORK_TEST_SSID` | Test SSID for `#[ignore]` `network_connect_test_ssid_live` (`AURA_NETWORK_TEST_PASSWORD` optional) |
| `AURA_AUDIO_VOLUME_TEST` | Set `1` for `#[ignore]` `audio_set_sink_volume_round_trip` |
| `AURA_HYPRLAND_EVENTS` | Set `0` to disable Hyprland socket2 push listener |
| `AURA_LAUNCHER_DESKTOP_DIRS` | Colon-separated directories of `.desktop` files (tests use `tests/fixtures/desktop`) |
| `VICINAE_SOCKET` | Unix socket for `Launcher.VicinaeQuery` (falls back to `Launcher.Query`) |
| `AURA_WEATHER_API_KEY` | Optional Bearer token for custom `AURA_WEATHER_WTTR_URL` |
| `AURA_WEATHER_SKIP_CACHE` | Set `1` in tests to bypass weather cache |
| `AURA_LOGS_FOLLOW_FIXTURE` | NDJSON/text lines for `Logs.FollowLogs` WS tests |
| `AURA_CALDAV_FIXTURE` / `AURA_CALDAV_URL` | CalDAV read-only sync (fixture or live REPORT) |
| `AURA_AUTOMATION_WEBHOOK_SECRET` | Shared secret for `127.0.0.1` webhook (`AURA_AUTOMATION_WEBHOOK_PORT`, default `19081`) |
| `AURA_PERFORMANCE_DRY_RUN` | Set `1` so `Performance.ApplyPreset` returns targets without polkit |
| `AURA_GAMEMODE_DRY_RUN` | Set `1` so `GameMode.Enable` / `Disable` skip `hyprctl` side effects |
| `AURA_EXTENSIONS` | Colon-separated local stdio extras (`""` disables). Each binary speaks NDJSON JSON-RPC and `Extension.ListMethods`. |
| `AURA_EXTENSIONS_DIR` | Directory of `*.json` manifests (`{"command":["/path/to/bin"]}`). Default: `$XDG_CONFIG_HOME/ags/extensions` (gitignored). |

## Binary resolution (GTK / `src/lib/sidecar.ts`)

1. `AURA_SIDECAR` — absolute path to the binary  
2. `$XDG_CONFIG_HOME/ags/sidecar/target/{debug,release}/ags-sidecar`  
3. `$HOME/.config/ags/sidecar/target/{debug,release}/ags-sidecar` when `XDG_CONFIG_HOME` is unset

Symlink or copy builds into `~/.config/ags/sidecar/target/` when using `~/.config/ags/aura`.

## Stack defaults

**Arch-first** today: NetworkManager, PipeWire + WirePlumber, `powerprofilesctl`, pacman, Podman, GNOME Keyring, Freedesktop notifications D-Bus. Other distros may need alternate code paths later — see [../docs/ARCHITECTURE_DECISIONS.md](../docs/ARCHITECTURE_DECISIONS.md).

Polkit: elevated commands go through `utils/polkit.rs` (`run_privileged` → `pkexec` when on PATH, else `sudo`). Used by firewall toggles (`Security.*`), package install/upgrade (`Packages.*`), and some performance helpers. A polkit agent must be available when those RPCs run from the UI — recommend **hyprpolkitagent** (see [`../hypr/README.md`](../hypr/README.md) and `scripts/aura-hypr-link.sh`).

### CI

GitHub Actions: [`.github/workflows/sidecar.yml`](../.github/workflows/sidecar.yml) runs `./scripts/sidecar-test-fast.sh` on pushes touching `sidecar/` or API contract scripts.

### Manual hardware matrix (pre-release smoke)

Run on a Hyprland session with the real stack (NetworkManager, PipeWire, etc.). Use read-only RPCs in scripts where possible; mutating rows need explicit operator approval.

| Area | Procedure | Expected |
|------|-----------|----------|
| Wi-Fi connect | Control center → Network → connect to known SSID | `Network.StateChanged` WS event; bar shows connected SSID |
| Bluetooth pair | Scan → pair trusted device | Device appears connected; `Bluetooth.StateChanged` |
| Battery | Unplug AC, wait for poll | `Power.BatteryState` on `/ws` (GTK `battery-state` signal) |
| VPN dry-run | `Vpn.GetStatus` / connect with test profile | Status JSON updates; no crash if agent missing (graceful error) |
| Screenshot | `Capture.Screenshot` region → clipboard | `grim`/`slurp`/`wl-copy` path OK or `tool_missing` |
| Packages | `Packages.GetUpgradable` (read-only) | List returns; install only with polkit agent |

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

## Build cache

`sidecar/.cargo/config.toml` enables incremental builds and the **lld** linker. Test/coverage scripts source `scripts/rust-cache-env.sh`, which enables **sccache** when installed.

```bash
pacman -S sccache lld    # once on Arch
sccache --show-stats     # hit rate after a few builds
```

Coverage uses a separate target dir (`sidecar/target/llvm-cov`) so the first llvm-cov run still compiles; sccache shares rustc artifacts across default and llvm-cov targets. After each coverage run, `scripts/sidecar-target-prune.sh --coverage-only` drops the llvm-cov build trees (HTML/LCOV under `target/coverage/` are kept).

### Target dir size cap

Integration tests link ~200 MB binaries; repeated `cargo test` / `cargo-watch` rebuilds can leave many stale copies under `target/debug/deps`. `./scripts/sidecar-target-prune.sh` (also run from `./aura` and `./scripts/sidecar-test-fast.sh`) removes llvm-cov build trees and, when `sidecar/target/` exceeds **20 GB** (`AURA_TARGET_MAX_GB`), dedupes stale test artifacts. Optional: `cargo install cargo-sweep` for extra standby/old-file cleanup.

```bash
./scripts/sidecar-target-prune.sh              # prune if over limit
./scripts/sidecar-target-prune.sh --force      # dedupe now
AURA_TARGET_MAX_GB=10 ./scripts/sidecar-target-prune.sh
AURA_TARGET_PRUNE_DRY_RUN=1 ./scripts/sidecar-target-prune.sh --force
```

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
