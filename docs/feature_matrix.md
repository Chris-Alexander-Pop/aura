# Planning and implementation snapshot

> **Note:** The filename `feature_matrix.md` is legacy. This file is a **shallow** technical and planning index—not a full product backlog.

## Where the roadmap lives

- **Product direction and per-area specs:** [todo.md](../todo.md) and [docs/roadmap/README.md](roadmap/README.md) (and files under [docs/roadmap/](roadmap/)).
- **Quickshell/Caelestia → AGS context:** [docs/MIGRATION_STRATEGY.md](MIGRATION_STRATEGY.md) and [docs/COMPONENT_MAPPING.md](COMPONENT_MAPPING.md) for legacy module mapping, not a live parity table.

## At a glance

| Layer | Role | Pointers |
| :---- | :--- | :------- |
| **Shell (GTK / AGS)** | Bar, OSD, launcher, WebKit host windows, `ags msg` from [app.ts](../app.ts) | [src/widget/](../src/widget/) — e.g. [bar/](../src/widget/bar/) |
| **Web UI (React)** | Vite app in WebKit; routes like `/control-center` in [ui/src/main.tsx](../ui/src/main.tsx) | [ui/src/pages/](../ui/src/pages/) — control center: [ui/src/pages/ControlCenter.tsx](../ui/src/pages/ControlCenter.tsx) (nav and stubs follow [docs/roadmap/control-panel.md](roadmap/control-panel.md)) |
| **Sidecar (Rust)** | JSON-RPC, hardware and system services | [sidecar/src/services/](../sidecar/src/services/) |

## Known technical gaps (concise)

- **Keybind management:** the React **Control Center** includes a Keybinds pane, but there is **no** dedicated `keybinds` (or similar) service under [sidecar/src/services/](../sidecar/src/services/) at present; editing Hyprland binds from Aura remains **unimplemented** at the backend.
- **Depth vs breadth:** many sidecar services exist; the corresponding **UIs** are still mostly **stubs** except where noted in the control center. Details belong in the relevant [docs/roadmap/](roadmap/) page, not duplicated here.

## Doc flow

```text
todo.md  →  docs/roadmap/*  (what to build)
     ↘
docs/feature_matrix.md  (this file: shallow stack + gaps)
     ↗
MIGRATION_STRATEGY + COMPONENT_MAPPING  (legacy QML only)
```
