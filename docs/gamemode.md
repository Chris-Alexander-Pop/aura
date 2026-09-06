# GameMode / gaming overlay status

## Live today

| RPC | Behavior |
|-----|----------|
| `GameMode.IsEnabled` | In-process flag + Hyprland keyword tweaks when enabled via `Enable`/`Toggle` |
| `GameMode.Enable` / `Disable` / `Toggle` | Mutates Hyprland via `hyprctl keyword` (animations, blur, gaps, tearing) |

There is **no** separate `GameMode.GetStatus` alias — clients use `GameMode.IsEnabled` per [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md).

## Not integrated

- **Feral GameMode** / `gamemode-dbus` — not probed; `gamemode.rs` does not talk to `com.feralinteractive.GameMode`
- **Gamescope** — compositor-level; stays outside sidecar unless a future `Gamescope.*` namespace is added
- **Bar indicator** — show only when `GameMode.IsEnabled` is reliable on your host (today: reflects last toggle in this sidecar process)

## Hyprland vs sidecar

Performance-oriented tweaks are applied through **`hyprctl keyword`** from the sidecar. Session-wide power profiles remain under `Power.SetProfile` / `Performance.ApplyPreset`.
