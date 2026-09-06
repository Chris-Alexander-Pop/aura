# Aura component map

Where shell surfaces and sidecar services live. Paths are the current tree, not a backlog.

## Legend

- **[TS]**: GTK / AGS TypeScript
- **[RS]**: Rust sidecar service
- **[UI]**: React in `ui/`
- **[CFG]**: Config / theme

## Sidecar services

| Area | Path | Type |
| :--- | :--- | :--- |
| VPN | `sidecar/src/services/vpn.rs` | **[RS]** |
| Audio | `sidecar/src/services/audio.rs` | **[RS]** |
| Network | `sidecar/src/services/network.rs` | **[RS]** |
| Power | `sidecar/src/services/power.rs` | **[RS]** |
| Backlight | `sidecar/src/services/backlight.rs` | **[RS]** |
| System stats | `sidecar/src/services/system.rs` | **[RS]** |
| GameMode | `sidecar/src/services/gamemode.rs` | **[RS]** |
| Weather | `sidecar/src/services/weather.rs` | **[RS]** |
| Keybinds | `sidecar/src/services/keybinds.rs` | **[RS]** |
| Hyprland | AGS `Hyprland` + `sidecar/src/services/hyprland.rs` | **[TS]/[RS]** |
| MPRIS | Astal / sidecar media RPCs | **[TS]/[RS]** |
| Notifications | Freedesktop D-Bus in sidecar | **[RS]** |

## Shell and UI

| Surface | Location | Notes |
| :--- | :--- | :--- |
| Bar | `src/widget/webview/BarWebViewWindow.tsx`, `ui/src/pages/BarStrip.tsx` | Default; GTK bar if `AURA_GTK_BAR=1` |
| Control center | `src/widget/webview/` + `ui/src/pages/ControlCenter.tsx` | `ags msg toggle control-center` |
| Calendar | `ui/` calendar page + webview host | `ags msg toggle calendar` |
| Dropdown | webview + `ui/` | `ags msg toggle dropdown` |
| OSD | `src/widget/osd/` | Volume / brightness |
| Launcher | GTK host + Vicinae | |
| Session / lock | `hypr/hyprlock.conf`, sidecar `Session.*` | |
| Theme | `src/lib/theme.ts`, `style/input.css`, `ui/` Tailwind | Catppuccin Mocha + Material 3 roles |

## Sidecar layout

```
sidecar/
  src/
    main.rs
    rpc.rs
    services/
    types.rs
```
