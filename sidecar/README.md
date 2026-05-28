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

Examples: `Power.BatteryState`, `Power.Profile`, `Network.StateChanged`, `Bluetooth.StateChanged`, `Audio.StateChanged`, `Notifications.Changed`, `Performance.MetricsChanged`.

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

## Stack defaults

Arch Linux, NetworkManager, PipeWire, Podman, GNOME Keyring — see [../docs/ARCHITECTURE_DECISIONS.md](../docs/ARCHITECTURE_DECISIONS.md).
