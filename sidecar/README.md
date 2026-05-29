# ags-sidecar

Rust backend for Aura: JSON-RPC over stdin (GTK shell) and HTTP on `127.0.0.1:9080` (React UI).

## Build

```bash
cd sidecar
cargo build          # debug → target/debug/ags-sidecar
cargo build --release
```

## Run

```bash
./target/debug/ags-sidecar
# HTTP: http://127.0.0.1:9080/api/Power.GetBatteryState
# CLI helper: ./target/debug/ags-sidecar client Power.GetBatteryState
```

## Binary resolution (GTK / `src/lib/sidecar.ts`)

1. `AURA_SIDECAR` — absolute path to the binary  
2. `$XDG_CONFIG_HOME/ags/sidecar/target/{debug,release}/ags-sidecar`  
3. `$HOME/Engineering/Productivity/ags/sidecar/target/{debug,release}/ags-sidecar` (dev clone)

Symlink or copy builds into `~/.config/ags/sidecar/target/` when using `~/.config/ags/aura`.

## Real-time push (WebSocket + GTK)

Services emit JSON events on `ws://127.0.0.1:9080/ws` and on **stdout** for the GTK sidecar (wrapped as JSON-RPC notifications).

Message shape:

```json
{ "method": "Power.BatteryState", "params": { "percent": 80, "charging": false, "time_remaining": "2h" } }
```

Examples: `Power.BatteryState`, `Power.Profile`, `Network.StateChanged`, `Bluetooth.StateChanged`, `Audio.StateChanged`, `Notifications.Changed`, `Performance.MetricsChanged`, `Hyprland.StateChanged`, `Productivity.TimerTick`.

## Hyprland (React bar)

- Typed RPCs: `Hyprland.GetWorkspaces`, `GetClients`, `GetActiveWindow`, `GetActiveWorkspace`, `GetMonitors`.
- `Hyprland.Dispatch` — **allowlisted** workspace/focus/float verbs only; rejects `exec`, `keyword`, shell metacharacters.
- Socket2 listener emits debounced `Hyprland.StateChanged` when `HYPRLAND_INSTANCE_SIGNATURE` is set. Disable with `AURA_HYPRLAND_EVENTS=0`.
- Optional: `playerctl` for `Media.GetNowPlaying` and `Audio.Media.*` transport on the bar.

## Notifications

- Listens on the session bus via `dbus-monitor` (Notify) and zbus signals (`NotificationClosed`).
- RPC: `Notifications.List`, `Notifications.GetDnd` / `SetDnd`, `Notifications.ClearAll`, etc.
- DND/quiet-hours prefs persist in SQLite (`notifications` namespace).

## Keybinds

- Reads Hyprland config (`HYPRLAND_CONFIG` or `~/.config/hypr/hyprland.conf`) and `source =` includes.
- Writes **only** to `~/.config/ags/hypr/aura-binds.conf` (backup on change).
- RPC: `Keybinds.List`, `Keybinds.Validate`, `Keybinds.Set` / `Unset`, `Keybinds.Reload`.

## Tests

```bash
cargo test
./scripts/generate-rpc-manifest.sh    # from repo root
./scripts/check-api-rpc-contract.sh
```

Integration tests use an in-process registry; see [`tests/README.md`](tests/README.md) for safety rules on a live machine.

### Coverage (`cargo-llvm-cov`)

Requires LLVM coverage tools and `cargo-llvm-cov`:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview   # needs rustup; Arch `rust` alone lacks llvm-cov
```

If you use distro Rust without `rustup`, install [rustup](https://rustup.rs/) or point `LLVM_COV` / `LLVM_PROFDATA` at a matching LLVM toolchain.

From the repo root:

```bash
chmod +x scripts/sidecar-coverage.sh   # once
./scripts/sidecar-coverage.sh
```

Summary only (no HTML):

```bash
./scripts/sidecar-coverage.sh --summary-only
```

Outputs:

- HTML report: `sidecar/target/coverage/html/index.html`
- LCOV: `sidecar/target/coverage/lcov.info`

## Stack defaults

Arch Linux, NetworkManager, PipeWire, Podman, GNOME Keyring — see [../docs/ARCHITECTURE_DECISIONS.md](../docs/ARCHITECTURE_DECISIONS.md).
