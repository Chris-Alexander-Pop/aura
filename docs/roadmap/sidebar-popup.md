# Sidebar popup (roadmap)

## Summary

A **small floating or anchored popup** associated with the sidebar (or bar) for **immediate hardware controls**: **brightness** and **volume**, with **per-device** selection (laptop panel vs external monitor audio, HDMI vs Bluetooth headphones). Minimal chrome, large hit targets, and **persistence** of last-used device where sensible.

Likely **GTK-near** for latency (OSD-style) or a **lightweight** webview; keep visual language consistent with the rest of Aura and allow **theme** sync.

## Current baseline

[../feature_matrix.md](../feature_matrix.md): **Brightness** and **Audio** have sidecar backends; **OSD** UI marked pending. **Media keys** issues are tracked in [system-and-input-foundation.md](./system-and-input-foundation.md).

## Goals

### Brightness, volume (quick configurable per device)

- **Brightness**: internal display and **DDCCI** or `ddcutil` for external where supported—clear UX when a display is not controllable.
- **Volume**: default sink/source, **per-app** volume (PipeWire/pavucontrol-class) in a **second row** or tab if needed.
- **Per-device quick switch**: dropdown or cycling key for “where should sound go?”

## Out of scope / risks

- **DDC/CI** reliability varies; never fail silently—show “not supported.”
- **Per-app** mixer UIs get complex fast; keep v1 to **device** + **default** only if needed.

## Dependencies

- [`sidecar/src/services/brightness.rs`](../../sidecar/src/services/brightness.rs), [`audio.rs`](../../sidecar/src/services/audio.rs).
- [control-panel.md](./control-panel.md) for deep audio routing.
- Hyprland/OSD for **on-screen** feedback when keys are pressed.

## Open questions

1. **Popup** triggered by **click** on bar icons, **edge hover**, or **dedicated key**?
2. **Multi-monitor** brightness: single slider with **target** selector or one popup per display?
3. Should this **merge** with a global OSD spec or stay separate?
