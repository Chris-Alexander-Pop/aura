# Caelestia -> AGS Component Mapping

This document maps every file in the source `@caelestia` directory to its destination in the `@ags` architecture.

## Legend
- **[TS]**: TypeScript Component (Frontend)
- **[RS]**: Rust Service (Backend)
- **[CSS]**: Tailwind Utility / CSS
- **[CFG]**: Configuration File

## Services (`/services`)

| Source File (`services/`) | Destination (`ags/`) | Type | Notes |
| :--- | :--- | :--- | :--- |
| `VPN.qml` | `sidecar/src/services/vpn.rs` | **[RS]** | Complete logic port. State machine, command execution (`openconnect`). |
| `Audio.qml` | `sidecar/src/services/audio.rs` | **[RS]** | Pipewire/Wireplumber control. |
| `Network.qml` | `sidecar/src/services/network.rs` | **[RS]** | NetworkManager client. |
| `PowerService.qml` | `sidecar/src/services/power.rs` | **[RS]** | Battery & Power Profiles. |
| `Brightness.qml` | `sidecar/src/services/backlight.rs` | **[RS]** | `brightnessctl` wrapper. |
| `Hypr.qml` | `(Built-in AGS Service)` | **[TS]** | Use `import { Hyprland } from 'resource:///...'` |
| `SystemUsage.qml` | `sidecar/src/services/system.rs` | **[RS]** | CPU/RAM stats using `sysinfo` crate. |
| `GameMode.qml` | `sidecar/src/services/gamemode.rs` | **[RS]** | Gamemode status monitoring. |
| `MediaController.qml` | `(Built-in AGS Service)` | **[TS]** | Mpris service is built-in to AGS. |
| `Notifs.qml` | `(Built-in AGS Service)` | **[TS]** | Notifications service is built-in. |
| `Keybinds.qml` | `src/services/Keybinds.ts` | **[TS]** | Local listener for shell shortcuts. |
| `Weather.qml` | `sidecar/src/services/weather.rs` | **[RS]** | Fetch OpenWeatherMap (or similar) JSON. |
| `Colors.qml` | `tailwind.config.js` | **[CFG]** | Define color palette. |

## Modules / UI (`/modules`)

| Source Module (`modules/`) | Destination Component (`src/components/`) | Notes |
| :--- | :--- | :--- |
| `bar/*` | `Bar/index.tsx` | Main status bar. |
| `controlcenter/*` | `ControlCenter/index.tsx` | Quick settings panel. |
| `dashboard/*` | `Dashboard/index.tsx` | Widget dashboard. |
| `notifications/*` | `Notifications/NotificationPopups.tsx` | Toast notifications. |
| `launcher/*` | `Launcher/AppLauncher.tsx` | App search interface. |
| `lock/*` | `LockScreen/index.tsx` | Visual overlay for lock screen (atop `hyprlock`). |
| `osd/*` | `OSD/Indicator.tsx` | Volume/Brightness overlay indicators. |
| `session/*` | `PowerMenu/index.tsx` | Logout/Reboot/Shutdown modal. |
| `background/*` | `Desktop/Background.tsx` | Desktop wallpaper/widgets. |
| `utilities/fuzzysort.js` | `src/utils/fuzzysort.ts` | Port library or use `fzf` binding. |

## Configuration (`/config`)

| Source File | Destination |
| :--- | :--- |
| `BarConfig.qml` | `src/config/bar.ts` |
| `Appearance.qml` | `src/config/theme.ts` |
| `LauncherConfig.qml`| `src/config/launcher.ts` |
| `general.qml` | `src/config/general.ts` |

## Rust Sidecar Architecture (`sidecar/`)

The sidecar will expose a unified JSON-RPC interface.

```rust
// Structure
sidecar/
  src/
    main.rs        // Entry point, RPC loop
    services/
      mod.rs
      vpn.rs       // VpnService trait
      audio.rs     // AudioService trait
      system.rs    // SystemService trait
    types.rs       // Shared structs (serialize/deserialize)
```
