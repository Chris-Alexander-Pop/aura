# Caelestia-style sidebar (roadmap)

## Summary

The **sidebar** is a primary **Caelestia-inspired** surface: persistent or dismissible, tile-based, and information-dense. This roadmap keeps that **look and feel** while making **geometry, palette, and tile sets user-configurable**, and deepening integration with **workspaces, windows, and special panels** (control center, calendar, etc.).

Implementation may combine GTK shell chrome in [`src/widget/`](../../src/widget/) with React content in [`ui/`](../../ui/) loaded in a webview window, coordinated via `ags msg` and the sidecar (see [AGENTS.md](../../AGENTS.md)).

## Current baseline

The [feature matrix](../feature_matrix.md) shows **bar** pieces implemented; a full **control center / sidebar** product UI is not. Sidecar services for power, network, Bluetooth, calendar, and more exist for when the UI lands. Legacy Caelestia modules in Quickshell are the **UX reference**, not the implementation.

## Goals

### Size and position and colour etc. configurable

User-controlled **layout**: width, edge anchoring, margin, blur or solid background, accent color, and typography scale. Settings should persist under Aura config and apply without full session restart where possible.

### Tile grouping / organization / expandable

**Groups** of tiles (e.g. connectivity, power, productivity) with **collapse/expand**, drag reorder, and optional **nested** groups. Support an “overflow” or “more” affordance so the default view stays clean.

### Window/workspace viewer w/ better integration for all the special panels

A **live** view of workspaces and windows (Hyprland-aware) with quick focus, move, and **deep links** into panels (Wi-Fi detail, calendar, performance) so the sidebar is a **hub** rather than isolated tiles.

### Task manager system, hook into Wayland

**Process and resource** view appropriate for daily use: CPU/RAM per app, kill/renice with guardrails, and integration with compositor **toplevel** metadata. “Hook into Wayland” means correct use of **wlr/hyprland** protocols and sidecar polling, not ad-hoc `ps` only.

### Organizeable / configurable app tile system w/ popup

**Pinned applications** as tiles with icon, launch, and optional **context popup** (new window, close all, workspace move). Fully **reorderable** and matching the same config system as other tiles.

### Caelestia-like Wifi tile (organized better?, put into the actual wifi/bt/battery tile group?)

A **connectivity cluster**: Wi-Fi, Bluetooth, and battery as a **single logical group** (or clearly linked sub-tiles) to reduce hunting. Wi-Fi sub-UI should stay **scannable** (signal, SSID, security) and defer deep config to [control-panel.md](./control-panel.md).

### Battery tile with actually working power profiles and such

Battery %, time remaining if available, and **power profile** (balanced, performance, power saver) wired to real backend calls—see [`sidecar/src/services/power.rs`](../../sidecar/src/services/power.rs) and `setPowerProfile` in the client. No dead toggles.

### Calendar tile to view upcoming tasks / date and time / open calendar panel

**At-a-glance** next events and current date/time, with a **clear action** to open the full [calendar-panel.md](./calendar-panel.md) experience.

### Power / logout / etc. tile

**Session actions**: lock, suspend, log out, reboot, shutdown—confirmations for destructive actions, alignment with [lock-panel.md](./lock-panel.md) and system policy (e.g. `logind`).

## Out of scope / risks

- **Killing processes** from the shell is dangerous; require confirmation and never default-kill PID1.
- **Over-customization** can make support impossible; ship **sane defaults** and “reset layout.”

## Dependencies

- [control-panel.md](./control-panel.md) for deep network/device/audio settings.
- [calendar-panel.md](./calendar-panel.md) for full calendar.
- [top-dropdown.md](./top-dropdown.md) if quickviews mirror sidebar tiles.
- Hyprland IPC and sidecar services for power and connectivity.

## Open questions

1. Should the sidebar be **one webview** or **GTK-composed** with embedded web for heavy views?
2. Maximum **tile count** before performance degrades on low-end machines?
3. Should workspace/window thumbnails be **live** or **icon-only** for memory use?
