# Planning and implementation snapshot

> **Note:** The filename `feature_matrix.md` is legacy. This file is a **shallow** technical and planning index—not a full product backlog.

## Where the roadmap lives

- **Product direction and per-area specs:** [docs/roadmap/README.md](roadmap/README.md) (and files under [docs/roadmap/](roadmap/)).
- **Stack / component map:** [docs/MIGRATION_STRATEGY.md](MIGRATION_STRATEGY.md) and [docs/COMPONENT_MAPPING.md](COMPONENT_MAPPING.md).

## At a glance

| Layer | Role | Pointers |
| :---- | :--- | :------- |
| **Shell (GTK / AGS)** | Layer-shell hosts, OSD, `ags msg` from [app.ts](../app.ts). **Bar is React/WebKit by default** (`AURA_GTK_BAR=1` for GTK) | [src/widget/webview/](../src/widget/webview/), optional [src/widget/bar/](../src/widget/bar/) |
| **Web UI (React)** | Vite app in WebKit; bar strip, control center, calendar, dropdown | [ui/src/pages/](../ui/src/pages/) |
| **Sidecar (Rust)** | JSON-RPC, hardware and system services | [sidecar/src/services/](../sidecar/src/services/) |

## Known technical gaps (concise)

- **Keybind management:** `keybinds.rs` service exists; UI supports live load + Lua override writes. Full editor UX still evolving.
- **Depth vs breadth:** sidecar RPC surface is broad (see [BACKEND_TODO.md](BACKEND_TODO.md)); control center panes vary from live to stub — see roadmap pages.

## Doc flow

```text
docs/roadmap/*  (what to build)
     ↘
docs/feature_matrix.md  (this file: shallow stack + gaps)
     ↗
MIGRATION_STRATEGY + COMPONENT_MAPPING  (stack / component map)
```
