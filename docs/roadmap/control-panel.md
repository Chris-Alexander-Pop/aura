# Revamped control panel (roadmap)

## Summary

The **control center** in Caelestia is the main place for connectivity, media, system status, and power. In Aura, many **sidecar services** already exist under `sidecar/src/services/` (see [../feature_matrix.md](../feature_matrix.md) for a shallow index), but the **React** control center is still mostly **stubbed**—see [`ui/src/pages/ControlCenter.tsx`](../../ui/src/pages/ControlCenter.tsx): nav and stub copy are aligned with **this** roadmap; **keybinds** have no sidecar service yet. This document specifies a **revamped, modular** control panel: package intelligence, full network/Bluetooth/device flows, working audio with fallbacks, performance and power policy, keybind management, security and VPN, productivity controls, automations, unified system settings, and a dedicated **notification** view.

Primary implementation home is expected to be the **`ui/`** Vite app loaded in webview windows (see [AGENTS.md](../../AGENTS.md)), backed by `ags-sidecar` JSON-RPC, with `ags msg` for show/hide.

## Current baseline

- **Planning and stack snapshot:** [../feature_matrix.md](../feature_matrix.md) (replaces the old Caelestia parity table; product detail stays here in `docs/roadmap/`).
- **Bar (GTK):** see `src/widget/bar/` — on-monitor widgets; separate from the web control center.
- **Control Center (`ui/`):** route `/control-center` in [`ui/src/main.tsx`](../../ui/src/main.tsx), page [`ControlCenter.tsx`](../../ui/src/pages/ControlCenter.tsx). Grouped **nav** matches the sections below (including **VPN** and **System settings**); only some panes (e.g. **Network** partial) have live sidecar data—rest are roadmap stubs. **Keybinds** tab: UI only until a sidecar service exists.
- **Migration context:** [../MIGRATION_STRATEGY.md](../MIGRATION_STRATEGY.md), [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md).

## Goals

### Pkg dep. graph / manager panel + auto sys updates and such, git-style history of sys changes

**Package and system change intelligence**: browse installed packages, dependency graph (reverse deps, why-installed), and safe updates. **Automatic updates** policy (security vs all, schedule, reboot hints). **History** of system changes in a **git-like** narrative: what was installed/removed, when, and from which transaction—candidate integration with pacman/journal or a small local append-only log written by the sidecar when it performs actions.

### Wifi panel w/ useful tools + fix all login paths

Full Wi-Fi UX: network list, saved networks, signal, forget, and **advanced tools** (captive portal flow, rescan, maybe `iw`/`nmcli` diagnostics). **Login paths** here means every path that needs credentials (802.1X, stored secrets, open vs secured) and must work after session start—ties to [system-and-input-foundation.md](./system-and-input-foundation.md).

### Revamped / fixed bluetooth / device manager panel

Pairing, trust, audio profile selection, battery for devices, and a broader **device manager** slice: USB, input devices, and “problem device” hints. Use [`sidecar/src/services/bluetooth.rs`](../../sidecar/src/services/bluetooth.rs) and expand RPC as needed.

### Audio panel (TEST THAT IT WORKS + ADD BACKUP SOLS, networking hub, streaming service integrations)

**Primary requirement**: provably working default and fallback paths (PipeWire, profiles, pro-audio gotchas). **Backup solutions** if the happy path fails (e.g. quick reset to default sink, restart pipewire from UI with confirmation). **Networking hub** might mean per-app network or capture routing (see VPN below). **Streaming** could mean easy device selection for OBS or browser capture—scope TBD.

### Performance panel (resource allocations per app / priority / presets / configurable power profiles)

Per-process or per-cgroup view, **niceness/affinity** style controls where safe, and **presets** (e.g. “meeting,” “compile,” “game”) that tie into CPU governor and [`sidecar/src/services/power.rs`](../../sidecar/src/services/power.rs) profiles. Must align with [sidebar.md](./sidebar.md) battery tile and system limits (avoid teaching users to break the scheduler).

### Revamped keybind panel

First-class **keybind editing** with a real sidecar service: list Hyprland (and global) binds, detect conflicts, apply with validation, and optional import/export. This closes the **keybinds gap** in the feature matrix.

### Security panel (add hooks to run AV daily / when something new is installed, password + fingerprint management)

**Scheduled and event-driven** security tasks: daily AV scan (ClamAV or user’s choice), optional scan on package install. **Password** and **fingerprint** (fprintd) management: enrollment status, re-enroll, test. All actions need clear privilege boundaries (PolicyKit, sudo).

### VPN panel (ie some apps on launch are always routed thru vpn, other security options / whatever)

VPN status, connect/disconnect, and **split routing** or **per-app** rules (e.g. only browser through VPN, or “this binary always uses tun0”). May integrate with NetworkManager, `wg-quick`, or user scripts; document threat model (DNS leaks, kill switch).

### Productivity panel (env config / usage, blocking etc.. basically ColdTurkey in linux)

**Focus and blocking** across sites and apps: schedules, allowlists, panic mode, and environment tags (work vs play). “Cold Turkey for Linux” implies strong defaults and clear escape hatches to avoid lockout disasters.

### Automations (basically AutoMate/n8n within linux, vaultwarden, lightweight, any other apps like mail-golem)

**User-defined automations**: triggers (time, file, RPC) and actions (script, notification, sidecar). References to **n8n**-class workflow and **Vaultwarden** are about **integration points** (API keys, webhooks) and optional small daemons—**not** shipping a full n8n inside the shell; prefer pluggability and resource caps.

### System settings (aura settings, linux settings, wayland settings, etc..)

Umbrella for **Aura-specific** config (JSON/theme/layout), **distro** settings (locale, time, user accounts link-outs), and **Wayland**-relevant options (scale, focus follows mouse) where they belong in a compositor-centric world.

### Notification panel

Inbox and rules: filter by app, **DND** schedule, history, and **clear all**. Unifies with “fix notifications” in [system-and-input-foundation.md](./system-and-input-foundation.md).

## Out of scope / risks

- **Per-app network routing** and VPN **kill switches** can brick connectivity; need safe mode and obvious recovery.
- **Process priority** and CPU pinning: avoid encouraging unstable configs on laptops.
- **AV on install** hooks must not make package operations unbearably slow without async UX.

## Dependencies

- [../feature_matrix.md](../feature_matrix.md) (stack snapshot) and [`../../sidecar/src/services/`](../../sidecar/src/services/); **Control Center** UI in [`../../ui/src/pages/ControlCenter.tsx`](../../ui/src/pages/ControlCenter.tsx).
- [system-and-input-foundation.md](./system-and-input-foundation.md) for auth and audio.
- [calendar-panel.md](./calendar-panel.md) if calendar and notifications share prefs.
- Hyprland config for keybind round-trip.

## Open questions

1. Is package management **Arch-only** (`pacman`/`yay`) or should the panel abstract multiple backends?
2. For “git-style” history, is a **local append-only log** in `~/.config/ags` acceptable, or must it integrate with an existing system journal only?
3. Per-app VPN: implement via **NetworkManager** split routes, **nftables** user rules, or a named **clash**/`mihomo`-class tool?
4. Keybind source of truth: **only Hyprland** or also GTK/global shortcuts?
