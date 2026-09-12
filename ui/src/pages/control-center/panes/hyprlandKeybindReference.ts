/**
 * Static Hyprland-oriented cheatsheet for the control center.
 * Not loaded from `hyprland.conf` or `hyprctl`; safe offline reference + search only.
 */

export type HyprKeybindRow = {
  /** Human-readable chord, e.g. `SUPER + Return` */
  combo: string
  /** What the bind typically does (Hyprland dispatcher / exec summary) */
  action: string
}

export type HyprKeybindSection = {
  id: string
  title: string
  rows: HyprKeybindRow[]
}

/** Shown in UI — keep wording explicit so no one thinks this is a live editor. */
export const HYPR_KEYBIND_REFERENCE_DISCLAIMER =
  "Curated offline reference. Common Hyprland-style chords and dispatchers — your real binds live in `hyprland.conf` and may differ completely."

export const HYPR_KEYBIND_SECTIONS: HyprKeybindSection[] = [
  {
    id: "session",
    title: "Session & apps",
    rows: [
      { combo: "SUPER + Return", action: "Open terminal (`exec` default terminal)" },
      { combo: "SUPER + Q", action: "Close focused window (`window.close`)" },
      { combo: "SUPER + Shift + Q", action: "Force-kill focused window process (`window.kill` / SIGKILL)" },
      { combo: "SUPER + M", action: "Exit compositor (`exit`)" },
      { combo: "SUPER + L", action: "Lock session (`exec` hyprlock / swaylock)" },
      { combo: "SUPER + D", action: "Application launcher (`exec` rofi, wofi, fuzzel, …)" },
      { combo: "SUPER + E", action: "File manager (`exec` your FM)" },
      { combo: "SUPER + B", action: "Browser (`exec` — optional user bind)" },
    ],
  },
  {
    id: "windows",
    title: "Window state",
    rows: [
      { combo: "SUPER + F", action: "Fullscreen (`fullscreen`)" },
      { combo: "SUPER + V", action: "Toggle floating (`togglefloating`)" },
      { combo: "SUPER + P", action: "Pseudo-tile (`pseudo`)" },
      { combo: "SUPER + J", action: "Toggle split orientation (`togglesplit`)" },
      { combo: "SUPER + Shift + T", action: "Toggle window transparency (`exec` or plugin — common custom)" },
      { combo: "SUPER + Ctrl + F", action: "Maximize (`fullscreen`, 1) — common variant" },
      { combo: "SUPER + mouse drag on window", action: "Move window (`movewindow` mouse binding)" },
      { combo: "SUPER + right mouse drag", action: "Resize window (`resizewindow`)" },
    ],
  },
  {
    id: "focus",
    title: "Focus & cycling",
    rows: [
      { combo: "SUPER + ← ↑ → ↓", action: "Focus in direction (`movefocus`)" },
      { combo: "SUPER + H / J / K / L", action: "Focus left / down / up / right (vim-style configs)" },
      { combo: "SUPER + Tab", action: "Next window (`cyclenext`)" },
      { combo: "SUPER + Shift + Tab", action: "Previous window (`cyclenext`, prev)" },
      { combo: "ALT + Tab", action: "Alt-Tab style (`cyclenext` / global keybind)" },
      { combo: "SUPER + [ / ]", action: "Cycle stacks / tabs (layout-specific)" },
    ],
  },
  {
    id: "move",
    title: "Move windows",
    rows: [
      { combo: "SUPER + Shift + arrows", action: "Move window (`movewindow`)" },
      { combo: "SUPER + Shift + H / J / K / L", action: "Move vim-style" },
      { combo: "SUPER + mousewheel on border", action: "Send to relative workspace (optional)" },
    ],
  },
  {
    id: "workspaces",
    title: "Workspaces",
    rows: [
      { combo: "SUPER + 1 … 0", action: "Switch workspace by index (`workspace`)" },
      { combo: "SUPER + Shift + 1 … 0", action: "Move window to workspace (`movetoworkspace`)" },
      { combo: "SUPER + Scroll ↑ / ↓", action: "Prev / next workspace (`workspace`, e+1 / e-1)" },
      { combo: "SUPER + S", action: "Toggle special / scratch workspace (`togglespecialworkspace`)" },
      { combo: "SUPER + Shift + S", action: "Move to special workspace (common variant)" },
    ],
  },
  {
    id: "layout",
    title: "Layout & toggles",
    rows: [
      { combo: "SUPER + Shift + Space", action: "Toggle master layout / dwindle (`layoutopt` / plugin)" },
      { combo: "SUPER + Space", action: "Toggle split / toggle floating (varies — check your config)" },
      { combo: "SUPER + R", action: "Resize mode / submap (many configs)" },
      { combo: "SUPER + G", action: "Toggle group (`togglegroup`) — when enabled" },
      { combo: "SUPER + Alt + ← / →", action: "Change active group window" },
    ],
  },
  {
    id: "mouse",
    title: "Mouse",
    rows: [
      { combo: "Scroll on focused edge", action: "Focus adjacent window (optional edge scroll binds)" },
      { combo: "SUPER + scroll on bar / layer", action: "Workspace or volume — shell-specific" },
    ],
  },
  {
    id: "misc",
    title: "Screenshots & media",
    rows: [
      { combo: "Print", action: "Screenshot (`exec` grim, hyprshot, …)" },
      { combo: "SUPER + Print", action: "Region / window shot (custom)" },
      { combo: "XF86AudioRaiseVolume", action: "Volume up (`exec` wpctl or pamixer)" },
      { combo: "XF86AudioLowerVolume", action: "Volume down" },
      { combo: "XF86AudioMute", action: "Mute" },
      { combo: "SUPER + Ctrl + ← / →", action: "Brightness (often `brightnessctl`)" },
    ],
  },
]

function normalizeQuery(query: string): string {
  return query.trim().toLowerCase().replace(/\s+/g, " ")
}

function rowMatches(query: string, sectionTitle: string, row: HyprKeybindRow): boolean {
  if (!query) return true
  const blob = `${sectionTitle} ${row.combo} ${row.action}`.toLowerCase()
  return blob.includes(query)
}

/**
 * Returns sections that still have at least one row after filtering.
 * Empty query yields a shallow copy of all sections (rows unchanged).
 */
export function filterHyprKeybindSections(
  query: string,
  sections: readonly HyprKeybindSection[] = HYPR_KEYBIND_SECTIONS,
): HyprKeybindSection[] {
  const q = normalizeQuery(query)
  if (!q) return sections.map((s) => ({ ...s, rows: [...s.rows] }))

  return sections
    .map((sec) => ({
      ...sec,
      rows: sec.rows.filter((row) => rowMatches(q, sec.title, row)),
    }))
    .filter((sec) => sec.rows.length > 0)
}
