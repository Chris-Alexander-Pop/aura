# System and input foundation (roadmap)

## Summary

This area covers **cross-cutting system and input behaviors** that are not a single “panel” but block daily use of Aura: authentication flows, package hygiene, launcher/CLI glue (**Vicinae**), reliable notifications and media keys, capture (screenshot/recording), disk migration, voice control, multitasking affordances, and **FN key** handling on the keyboard firmware/OS boundary.

These capabilities span **Hyprland**, **system services**, and the Aura stack ([`app.ts`](../../app.ts), [`src/widget/`](../../src/widget/), [`ui/`](../../ui/), [`sidecar/`](../../sidecar/)). Shell UI pieces may live in the bar, OSD, webview windows, or sidecar-backed panels depending on urgency and modality (see [AGENTS.md](../../AGENTS.md)).

## Current baseline

Aura today is strongest on **bar widgets** (battery, clock, workspaces per [../feature_matrix.md](../feature_matrix.md)). **Control center**, **notifications**, and **OSD** UIs are largely unrealized in the matrix; notification handling is explicitly partial. Network, Bluetooth, audio, VPN, and similar services exist in `sidecar` but lack completed React/GTK surfaces.

This roadmap group is the **underpinning** so those panels and global shortcuts behave predictably before or while revamping layouts elsewhere in this folder.

## Goals

### Clean up / fix login

Ensure graphical and TTY-adjacent login (e.g. greetd, SDDM, or whatever the host uses) is reliable: correct PAM/session, Wayland session handoff, no stale keyring or network-auth edge cases that break Wi-Fi/VPN post-login. Document **known failure modes** (e.g. captive portal vs stored credentials) and align with fixes in [control-panel.md](./control-panel.md) for “all login paths.”

### Review all installed packages + system cleanup

Regular audit: explicit list of installed packages, removal of orphans, pinning of critical versions, and optional **“health report”** export. Candidate scope includes integration with package updates and history described in [control-panel.md](./control-panel.md).

### Better Vicinae integration + custom commands (must be overpowered)

**Vicinae** is the terminal-centric launcher/shell assistant layer in this workflow. The goal is **deep integration**: Aura should surface Vicinae-driven actions (custom commands, snippets, project-scoped flows) with minimal friction—keyboard-first, scriptable, and composable with `ags msg` or sidecar triggers. “Overpowered” means first-class support for user-defined command palettes, not a single hardcoded list.

### Fix notifications

Notification pipeline must be **correct and trustworthy**: ordering, grouping, actions, persistence policy, do-not-disturb, and mirroring what Mako/hybrid backends actually emit on Wayland. See [../feature_matrix.md](../feature_matrix.md) (notifications partial backend). UI likely spans `ui/` and GTK notification surfaces; close coordination with [control-panel.md](./control-panel.md) notification panel.

### Fix mute BS, wire up other F12 buttons

Audio mute and related **media/function-row** behavior should match muscle memory: reliable mute state sync, correct default sink/source, and **F12 row** (or equivalent) mapped to Aura actions where appropriate (e.g. control center, mic toggle, sidecar-driven presets). May involve Hyprland binds, pipewire/pactl, and OSD feedback.

### Revamp screenshot / screen recording stuff

Unified flow: region/window/fullscreen, clipboard vs file, optional annotation hook, and **screen recording** with sensible defaults (codec, audio source, stop hotkey). Should feel as polished as commercial tools while staying scriptable.

### Migrate from Windows and free up that SSD

One-time or ongoing **data migration** off another OS: user data, game libraries, and disk layout so the Linux install regains space. Largely operational, not shell code; may still warrant a small “migration checklist” or sidecar status if automated steps exist.

### Voice control system

Voice as an **input modality** for the shell: push-to-talk or wake word (privacy-sensitive), command routing to Vicinae/sidecar, and clear feedback in the bar or a dedicated overlay. TBD: local vs cloud STT.

### Multitasking system

User-facing **workspace and window management** beyond the current bar: overview, quick move between workspaces, per-app rules, and alignment with [sidebar.md](./sidebar.md) “window/workspace viewer” and any Hyprland workspace plugins.

### Fix (swap) FN key BS

Hardware/firmware often expose **Fn-lock** or awkward media-key defaults. Document target behavior (F-keys vs media keys), interaction with `evdev`/keyd, and user-facing toggle so behavior is consistent across apps and the shell.

## Out of scope / risks

- **Cloud STT** for voice carries privacy and cost tradeoffs; local models may be heavy on disk/GPU.
- **Login and PAM** changes can lock users out; changes need rollback notes and non-graphical recovery path.
- **Package removal** automation can be destructive; prefer dry-run and explicit confirmation patterns.

## Dependencies

- [control-panel.md](./control-panel.md) (network auth, audio, updates).
- [sidebar.md](./sidebar.md) and [top-dropdown.md](./top-dropdown.md) (where multitasking and quick actions surface).
- Hyprland config, pipewire, greeter, and optional `keyd` for FN behavior.

## Login and keyring (post-boot WiFi / VPN)

Stored WiFi passwords use **GNome Keyring** via `secret-tool` (`application=aura`, `type=wifi_password`). If the keyring is **locked at login**, `Network.Connect` cannot retrieve saved passwords until you unlock it (usually automatic on graphical login with `gnome-keyring-daemon`).

**Checklist for reliable post-login networking:**

1. **Greeter session** — SDDM/greetd must start a session that launches `gnome-keyring-daemon` (or `seahorse` unlock on first secret access).
2. **Polkit agent** — **hyprpolkitagent** via `hypr/hyprland/execs-aura.conf` (or `systemctl --user enable --now hyprpolkitagent.service`). Run [`scripts/aura-hypr-link.sh`](../../scripts/aura-hypr-link.sh) so `hyprtoolkit.conf` and agent config resolve from `~/.config/ags/hypr/`.
3. **Aura Hyprland opt-in** — source `hypr/hyprland/execs-aura.conf` and `hypr/hyprland/aura-keybinds.conf` from live `hyprland.conf` (see [`hypr/README.md`](../../hypr/README.md)).
4. **Symptoms when broken** — secured WiFi shows “Password required” after reboot; VPN profiles missing credentials; `Security.GetKeyringStatus` reports keyring unavailable or locked.

Aura surfaces keyring status in Network (control center) and Settings when the sidecar detects a locked or missing keyring.

## Open questions

**Resolved (2026-05-28):** see [../ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md).

1. **SDDM**; **GNOME Keyring** (`org.freedesktop.secrets`).
2. **Vicinae:** long-lived socket/RPC; subprocess fallback.
3. **Voice:** push-to-talk; local STT default, cloud opt-in; GPU when available.
4. **Capture:** `grim` + `slurp` + `wf-recorder` (install `wf-recorder` if missing).
5. **Notifications:** Freedesktop D-Bus listener; recommend **swaync** autostart on Hyprland (see ADR §1).
