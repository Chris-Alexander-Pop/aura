# Phase 3: Pixel-Perfect UI Port (Full Behavior Integrated)

Phase 3 components are implemented and **fully integrated**: all windows are registered and AGS services (Hyprland, SystemTray, Notifications, Applications) are loaded at startup.

## What Was Added

### 1. Status Bar (`src/components/Bar/`)
- **Bar** – Main bar window (anchor top/left/right) with left/center/right sections.
- **Workspaces** – Hyprland workspace buttons (pass `Service.import('hyprland')` when enabling).
- **Clock** – Time display (config: `config/bar.ts`).
- **Tray** – System tray icons (pass `Service.import('systemtray')` when enabling).
- **Power** – Battery state + sidecar; optional gear button for control center.

### 2. Notification Center (`src/components/Notifications/`)
- **NotificationPopups** – Toast-style notifications (pass `Service.import('notifications')` when enabling).

### 3. Control Center (`src/components/ControlCenter/`)
- **NavRail** – Pane list: Network, Bluetooth, Audio, Performance, Power, Weather.
- **Panes** – Audio (volume + mute via sidecar), Network (Wi‑Fi toggle + scan), Power (battery + power profile via sidecar). Others are placeholders.

### 4. App Launcher (`src/components/Launcher/`)
- **AppLauncher** – Search entry + filtered app list (pass `Service.import('applications')` when enabling).

### Config
- `src/config/bar.ts` – Bar entries, workspaces, clock, tray, sizes.
- `src/config/launcher.ts` – Launcher enabled, maxShown.
- `src/config/controlCenter.ts` – Pane list, sizes.

## Current Behavior

- **main.ts** runs an async `start()` that loads `hyprland`, `systemtray`, `notifications`, and `applications` via `Service.import()`, then calls `App.config()` with all four windows.
- **Bar** shows workspaces (reactive to Hyprland), clock (updates every second), tray (reactive to SystemTray items), launcher button (⌘), and power (battery + gear to open Control Center).
- **Control Center** and **App Launcher** start **hidden** (`visible: false`); use the bar or keybinds to show them.
- **Power** gear button calls `App.toggleWindow('control-center')`.
- **Launcher** bar button calls `App.toggleWindow('app-launcher')`.

### Keybinds (set in your compositor, e.g. Hyprland)

- Toggle Control Center: `ags -t control-center` (e.g. Super+m)
- Toggle App Launcher: `ags -t app-launcher` (e.g. Super+a)

## Dependencies

- **Sidecar** – Must be running for Bar (Power), Control Center (Audio, Network, Power).
- **AGS services** – Loaded automatically; Hyprland, SystemTray, Notifications, Applications. If a service fails to load, that part of the UI uses placeholders or empty state.

## Next (Phase 4)

- Parallel run with Caelestia for visual comparison.
- Stress-test RPC; then cutover (disable Caelestia, enable AGS).
