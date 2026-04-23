# Top dropdown (roadmap)

## Summary

A **top-of-screen dropdown** (Caelestia-style “dropdown” region) that acts as a **mission control**: a **configurable main dashboard**, **quickviews** into other roadmap surfaces (calendar, communication, DevOps), and **quick settings / toggles**. Subpanels should be **modular** so users can add, remove, and ideally **reorder** (drag-and-drop) without code changes.

This maps to the **dropdown** webview pattern in Aura ([`src/widget/webview/`](../../src/widget/webview/)) and `ags msg toggle dropdown` per [AGENTS.md](../../AGENTS.md).

## Current baseline

Bar is implemented; **rich dropdown** content is a product decision more than a matrix row. [../feature_matrix.md](../feature_matrix.md) emphasizes **control center** gaps; the dropdown is a **distinct** surface that can embed or **link** to those modules.

## Goals

### Quick configurable main dashboard

User-chosen **widgets**: time, weather, next calendar block, resource sparkline, **dnd** toggle, etc. **Layouts** should be simple (grid or single column) with **presets** (minimal vs power user).

### Configurable quickviews for other panels (ie calendar, communication, devops..)

**Embedded summaries** or **preview panes** that deep-link to full panels: e.g. next three events, unread comms count, cluster **red/green**. Avoid duplicating full UIs—**one source of truth** in each panel doc.

### Configurable quickviews from control panel subpanels (make subpanels modular so its super easy to configure, drag and drop?)

**Same module system** as the control center: each subpanel (network, audio, …) registers a **small** and **large** card for the dropdown. **Drag-and-drop** ordering in v1 or v2; at minimum **enable/disable** and order list in settings.

### Quick settings / quick toggles

One-tap **Wi-Fi**, **Bluetooth**, **DND**, **dark mode**, **night light**, **power profile**—mirroring [sidebar.md](./sidebar.md) tiles but **vertical** and **transient**.

## Out of scope / risks

- **Too many embeds** slow WebKit; cap concurrent quickviews or lazy-load.
- **Drag-and-drop** in GTK + WebKit may need a clear **owner** (all React vs native handles).

## Dependencies

- [sidebar.md](./sidebar.md), [control-panel.md](./control-panel.md), [calendar-panel.md](./calendar-panel.md), [devops-panel.md](./devops-panel.md), [communication-panel.md](./communication-panel.md).
- Sidecar aggregation for **lightweight** status payloads.

## Open questions

1. **Single** dropdown instance or **per-monitor**?
2. Config format: **JSON schema** in `~/.config/ags` with UI editor?
3. Should quickviews be **iframes** of full panels (bad for perf) or **dedicated compact components**?
