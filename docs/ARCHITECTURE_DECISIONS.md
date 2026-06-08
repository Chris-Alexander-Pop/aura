# Architecture decisions (Aura / AGS)

> **Status:** Living document. Last updated: **2026-05-28** after product Q&A.  
> **Consumers:** `docs/BACKEND_TODO.md`, sidecar implementation, `ui/`, GTK shell.

---

## 1. Canonical machine stack (probed 2026-05-28)

| Layer | Decision | Evidence on host |
|--------|-----------|------------------|
| **OS** | **Arch Linux only** for package manager sidecar | `/etc/os-release` |
| **Display manager** | **SDDM** | User confirmed |
| **Secrets** | **GNOME Keyring** (`org.freedesktop.secrets`) | `gnome-keyring-daemon` active; D-Bus `org.freedesktop.secrets` |
| **Network** | **NetworkManager** (`nmcli`) | `NetworkManager` active, `nmcli` connected |
| **Audio** | **PipeWire + WirePlumber**; **EasyEffects** optional advanced | `pipewire` + `wireplumber` active; `wpctl`, `pactl`, `easyeffects` present |
| **Compositor** | **Hyprland** | Project baseline |
| **Lock** | **`hyprlock`** (Aura-owned theme) | `hyprlock` installed; `swaylock` not installed |
| **Containers** | **Podman** (not Docker daemon) | User on Podman; sidecar uses **Podman socket/API**, not `/var/run/docker.sock` unless user explicitly enables Docker compatibility |
| **Capture** | **`grim` + `slurp` + `wf-recorder`** | `grim`, `slurp` installed; **`wf-recorder` not installed** — install before recording RPC is tested |
| **Fingerprint** | **`fprintd`** | Implement in v1 (install/enable if missing on host) |

### Notifications (user asked for guidance)

**Installed but not running at probe time:** `mako`, `swaync`, `dunst` (all present via pacman).

**Recommendation for Hyprland + Aura control center:**

| Daemon | Use when |
|--------|----------|
| **swaync** (recommended) | You want a **notification center** UI, grouping, and closer fit to “fix notifications” + Control Center panel |
| **mako** | You only need **lightweight toasts**, minimal UI |
| **dunst** | You prefer classic dunst scripting |

**Sidecar approach (decided):** Implement a **Freedesktop Notifications D-Bus** listener (`org.freedesktop.Notifications`) so the backend works with **any** compliant daemon; Aura UI does not hard-code one brand.

**Action for you:** Pick one daemon to **autostart in Hyprland** (recommend `swaync`). Until one runs, desktop notifications may appear broken—that matches the “fix notifications” todo.

---

## 2. Sidecar binary location

| Priority | Resolution |
|----------|------------|
| 1 | `AURA_SIDECAR` environment variable → absolute path to `ags-sidecar` |
| 2 | `$XDG_CONFIG_HOME/ags/sidecar/target/release/ags-sidecar`, then `debug/` |
| 3 | **Dev:** walk from AGS config directory / repo root for `sidecar/target/{debug,release}/ags-sidecar` (code change tracked in `BACKEND_TODO` §0.1) |

**Dev workflow (now):** Symlink or copy build artifacts into `~/.config/ags/sidecar/target/...` **or** set `AURA_SIDECAR` to the Engineering clone build.

**Production direction:** Keep default under **`~/.config/ags`** (XDG config) so AGS and Hyprland configs stay co-located; optional `/usr/local/bin/ags-sidecar` later for packaged installs. Do **not** require the binary inside the git clone for daily use.

---

## 3. API & naming

| Topic | Decision |
|--------|----------|
| RPC prefix casing | **`DevOps.*`** (match Rust module `devops.rs`) — rename TS clients from `Devops.*` |
| Contract drift | **Rename clients** to match Rust; **no long-lived aliases** |
| RPC manifest in CI | **Yes** — generate from `registry.register` |
| Offensive security RPCs | Cargo feature **`offensive-security`**, **default off** |
| Pentest panel | **Yes**, when feature enabled and user opts in |

---

## 4. Packages & system history

| Topic | Decision |
|--------|----------|
| Package backend | **Arch only** (`pacman`, `yay`/`paru` helpers as optional) — remove/low-priority apt/dpkg paths over time |
| Change history (“git-style”) | **Local append-only transaction log** at `~/.local/share/ags-sidecar/transactions.jsonl` (or SQLite table) written when sidecar performs install/remove/upgrade; **optional** `journalctl` correlation for timestamps only |

*Rationale:* Append-only log gives fast UI narrative; journal alone is noisy and hard to map to “who installed `foo`”.

---

## 5. VPN & routing (phased — user wants all capabilities)

Implement in order; all are in scope, not mutually exclusive:

1. **Phase A:** Profile connect/disconnect (OpenConnect/OpenVPN/WireGuard) — existing `vpn.rs`
2. **Phase B:** **NetworkManager** split routing / per-connection domains
3. **Phase C:** **`nftables`** (or `iptables-nft`) per-app/mark rules with safe rollback
4. **Phase D:** Optional **mihomo/clash**-class profile type for users who already run a proxy stack

**Safety:** Kill-switch and per-app rules require **recovery mode** (disable VPN module, reset nftables) documented in UI.

---

## 6. Keybinds & input

| Topic | Decision |
|--------|----------|
| Source of truth | **Hyprland**, with binds maintained in an **Aura-managed file** included from `hyprland.conf` |
| Suggested path | `~/.config/ags/hypr/aura-binds.conf` (generated/applied by `Keybinds.*` RPC) |
| Hyprland include | `source = ~/.config/ags/hypr/aura-binds.conf` (or `~/.config/hypr/aura-binds.conf` — pick one path in implementation and document) |
| **keyd** | **Optional** for FN/media key layer below Hyprland — install when ready; sidecar documents toggle, does not require keyd for v1 keybind editor |

### What is keyd?

**[keyd](https://github.com/rvaiya/keyd)** is a system-wide key remapping daemon (runs as root). It maps physical keys before they reach Hyprland/X11/Wayland—useful for **Fn-lock**, turning media keys into F-keys, or global macros. Hyprland `bind` handles compositor actions; **keyd** handles hardware/firmware quirks. Aura does not replace either; it may later sync a small keyd config snippet for FN row fixes (`todo.md`).

---

## 7. Audio

| Topic | Decision |
|--------|----------|
| v1 scope | **Per-app sink routing** (`Audio.RouteStream`, etc.) |
| Streaming/OBS hub | **Defer** beyond routing |
| Fallback | **Yes** — “restart PipeWire” via sidecar with UI confirmation |
| Stack | PipeWire + WirePlumber; EasyEffects for advanced EQ/effects |

---

## 8. Security / AV

| Topic | Decision |
|--------|----------|
| v1 AV engine | **ClamAV** (`clamav`, `freshclam`) — best Arch packaging and scheduled/on-demand scans |
| Schedule | Daily scan + **optional scan after package install** (async, non-blocking UI) |
| Alternatives | **PRX-SD** / **LMD** noted as future plugins; not v1 default (PRX-SD: stronger real-time story but newer install path; LMD often still wraps ClamAV) |
| Firewall / SSH / encryption | Use existing `security.rs` defensive RPCs |
| Fingerprint | **`fprintd`** integration in v1 |

---

## 9. Capture

| Tool | Role |
|------|------|
| `grim` + `slurp` | Screenshots |
| `wf-recorder` | Screen recording (**install:** `pacman -S wf-recorder`) |

---

## 10. Voice

| Topic | Decision |
|--------|----------|
| Mode | **Local STT default**; **cloud opt-in** |
| Hardware | Prefer **onboard GPU** inference when available (document backend e.g. whisper.cpp / vosk — chosen at implementation) |
| UX | **Push-to-talk** only in v1 (no wake word) |

---

## 11. DevOps

| Topic | Decision |
|--------|----------|
| Runtime | **Podman** first (`podman` CLI, user socket at `/run/user/$UID/podman/podman.sock`) |
| Permissions | Read/write **with strict allowlist**, confirmations for destructive ops, audit log |
| Docker | Do not assume `docker.sock`; support Docker only if socket exists and user enables |

---

## 12. Real-time events (WebSocket + GTK)

Push **all** of the following when backend can detect changes (final product requirement):

- Battery / power profile  
- Network connect/disconnect  
- VPN state  
- Default audio device / notable stream changes  
- Bluetooth device connect/battery  
- Notifications (new/cleared/DND)  
- Hyprland workspace/window (where cheap)

---

## 13. Calendar, comms, productivity integrations

| Area | Decision |
|------|----------|
| Calendar sync v1 | **Google Calendar + CalDAV** |
| Reclaim-style AI scheduling | **Out of scope** — separate app/project later |
| Communication hub (Beeper-like) | **Out of scope** — separate app/project later |
| Comms data residency | N/A until integration |

---

## 14. Vault panel (deferred — definitions)

User asked what these mean; **implementation deferred** to P3.

| Term | Meaning |
|------|---------|
| **Vault backups** | Scheduled **snapshots** of chosen paths (e.g. `~/`, `~/.config`) to local disk or cloud using **restic** or **borg** (orchestrated by sidecar, policies in Aura settings) |
| **Vault mounts** | **rclone mount** (FUSE) exposing cloud storage as a filesystem path in the file manager or Aura file browser |

**Default when built:** Orchestrate **restic** + **rclone**; browser-first UI in Aura; optional FUSE mount.

---

## 15. Lock screen

| Topic | Decision |
|--------|----------|
| Locker | **`hyprlock`** with **Aura-owned** theme/config |
| v1 widgets | **Clock only** (custom widgets later) |
| Music/notifications on lock | **Off** in v1 |

---

## 16. UI shell patterns

| Surface | Decision |
|---------|----------|
| Sidebar (future) | Same as today’s panels: **React in WebKit** (`src/widget/webview/*Window.tsx` + `ui/` routes) — **no GTK sidebar** unless `AURA_GTK_*` escape hatch |
| Current bar | **React** `BarWebViewWindow` default; GTK bar via `AURA_GTK_BAR=1` |
| Top dropdown | **Drag-and-drop** module order in v1 |
| Workspace thumbnails | **Both** modes; **default icon-only**; live previews opt-in |

---

## 17. Vicinae

| Topic | Decision |
|--------|----------|
| Integration | **Long-lived socket/RPC** preferred (avoid cold-start latency) |
| Fallback | Subprocess if daemon unavailable |
| Contract | Define in `launcher.rs` when Vicinae API is stable |

---

## 18. Deferred / out of scope

| Item | Decision |
|------|----------|
| IDE popup / Jules | **Defer** |
| Exocortex, Skiller, Secure-A, Marginal gains | **Ignore** until product spec |
| Communication panel backend | **Separate project** |

---

## 19. Dependencies to install (host)

```bash
# Recording (missing at probe)
sudo pacman -S wf-recorder

# AV (when implementing security scans)
sudo pacman -S clamav freshclam
sudo systemctl enable --now clamav-freshclam.service

# Notifications (pick one — recommend swaync)
sudo pacman -S swaync   # already installed; enable autostart in Hyprland

# Optional later
# sudo pacman -S keyd fprintd
```

---

## 20. Communication hub (P3 — deferred)

| Decision | Detail |
|----------|--------|
| Scope | **No unified inbox in v1.** `Communication.GetUnread` returns a stub map; `GetMessages` / `GetConversations` are empty until a provider is chosen. |
| Long-term | Matrix, email, or a separate “comms app” — not blocked on sidecar P0–P2. |
| Sidecar | Keep read-only/stub RPCs; mutating bridge RPCs stay on the integration deny-list. |

See [roadmap/](roadmap/) and [BACKEND_TODO.md](BACKEND_TODO.md) §3.9.

---

## 21. IDE / dev environment (P3 — deferred)

| Decision | Detail |
|----------|--------|
| Scope | **No `IDE.*` RPC namespace yet.** Editors (Cursor, VS Code, Neovim) differ too much for one contract. |
| Future | If added: read-only “open project / recent files” per editor adapter, not generic “run IDE”. |
| Docs | Product slices live in [roadmap/](roadmap/); implementation waits on panel spec. |

---

## 22. Voice / exocortex (P4 — deferred)

| Decision | Detail |
|----------|--------|
| Privacy | **Local-first** when implemented (e.g. whisper.cpp on GPU); cloud STT only with explicit opt-in. |
| Sidecar | **No voice RPCs** until model path, wake word, and GTK/React UX are defined. |
| Exocortex | Out of sidecar scope until product spec; do not half-ship in `Storage.*`. |

---

## 23. Gamemode / gaming overlay

| Topic | Decision |
|-------|----------|
| Live RPCs | `GameMode.IsEnabled`, `Enable`/`Disable`/`Toggle` — Hyprland keyword tweaks via `hyprctl` |
| Not integrated | Feral GameMode D-Bus, gamescope — see [gamemode.md](gamemode.md) |
| UI | Bar indicator only when `GameMode.IsEnabled` reflects user intent (in-process flag today) |

---

## 24. Open items (still TBD at implementation time)

- Exact Hyprland path for `aura-binds.conf` (under `~/.config/ags/hypr/` vs `~/.config/hypr/`)
- Vicinae socket JSON schema stability (see [integrations/vicinae.md](integrations/vicinae.md))
- Local STT library choice for GPU path
- Whether to autostart `swaync` from Aura or only document Hyprland user config
