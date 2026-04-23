---
name: ags-ui-layer-choice
description: >-
  Picks the correct layer for a shell feature — GTK bar/OSD/launcher vs React
  pages in ui/ vs WebKit window shells. Use when the user asks where to implement
  something, when planning a new panel, or when navigating bar vs control center
  vs webview code.
---

# Where does this feature live?

## Decision tree

1. **Always on the monitor edge, Hyprland layer shell style, small GTK pieces**  
   → `src/widget/bar/`, `src/widget/osd/OSD.tsx`, or related `src/widget/` GTK code. Theme via root Tailwind → `style/style.css`.

2. **Large panel, settings grid, web-like layout served as a page**  
   → `ui/src/pages/` (React) + routing/entry in `ui/src/main.tsx`. Style with **`ui/`** Tailwind (`ui/tailwind.config.js`).

3. **Window chrome, transparency, layer-shell name, `ags msg toggle …`, WebKit host**  
   → `src/widget/webview/*Window.tsx` (e.g. `ControlCenterWindow.tsx`). Window `name` must match `app.get_window` / keybinds (`control-center`, `sidebar`, `dropdown`, `calendar`).

4. **Hot-corner strip or GTK-only overlay trigger**  
   → e.g. `src/widget/dropdown/DropdownTrigger.tsx` — GTK that talks to named windows.

5. **Backend state, hardware, NetworkManager, audio, VPN**  
   → Prefer `sidecar/src/services/*.rs` and expose via RPC; bind from GTK or React through `sidecar.ts` / `api.ts`.

## Parity and planning

- `docs/feature_matrix.md` — what is done vs backend-only vs missing UI.
- `docs/COMPONENT_MAPPING.md` — legacy Caelestia → this repo mapping.

## Anti-pattern

Duplicating the same control UI in both GTK (`src/widget`) and React (`ui/`) unless migrating — pick one layer per surface and keep the other thin.
