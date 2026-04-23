import type { PaneId } from "./navigation"

/** Short scope lines aligned with docs/roadmap/control-panel.md (stubs only). */
export const ROADMAP_STUB_BLURBS: {
  [K in Exclude<PaneId, "network" | "weather">]: string
} = {
  packages:
    "Package and dependency graph, update policy, and a readable history of system changes (git-style narrative). The packages sidecar surface is still partial—UI will grow with the backend.",
  settings:
    "Umbrella for Aura settings (layout, theme, shell behavior), distro basics (locale, time), and Wayland-relevant options that belong in a compositor-centric shell.",
  vpn: "VPN status, connect and disconnect, split routing, and per-app rules. Uses the sidecar VPN service when the panel is fleshed out; mind DNS leaks and kill-switch UX.",
  bluetooth:
    "Pairing, trust, audio profiles, device battery, and a broader “device manager” path for USB and input glitches. Sidecar Bluetooth today; expand RPC as the UI does.",
  audio:
    "PipeWire-first: default devices, per-app volume where needed, and explicit reset or fallback if something breaks. Optional streaming and capture targets once stable.",
  notifications:
    "Notification inbox, rules, do-not-disturb, history, and clear-all—aligned with shell notification work and the `notification` sidecar/WS path.",
  performance:
    "Per-app resource view, safe presets, and power profiles that tie into performance and power sidecar services—avoid encouraging unstable scheduler tricks.",
  keybinds:
    "List, edit, and export Hyprland (and later global) binds. There is no dedicated keybinds service in the sidecar yet—this pane is the UI lead until RPC exists.",
  security:
    "Antivirus policy hooks, fingerprint and password state, and other security status from the `security` sidecar; destructive actions go through polkit/confirmations.",
  productivity:
    "Focus, blocking, and environment schedules—Cold-Turkey-style for Linux, with a clear “escape hatch” so you can’t lock yourself out accidentally.",
  automations:
    "Time- and event-driven automations, optional hooks to tools like n8n or your own webhooks, without bloating the shell with a full workflow server.",
  calendar:
    "Events and cal-style views backed by the calendar service. The full GCal/Notion-class experience may share code with the dedicated calendar webview route.",
  logs: "Tails, filters, and “copy this bundle” for support—`logs` service over RPC rather than reimplementing journald.",
  devops:
    "Local, home-lab, and light cloud deployment status from the `devops` sidecar—read-heavy first, with guarded actions if credentials allow.",
  communication:
    "Unified inboxes and bridges, privacy-first, opt-in integrations—greenfield; think Beeper-like in scope, not feature parity in v1.",
  fitness: "Health and fitness goals using the `fitness` service; distinct from a full health app but visible from the same shell session.",
}
