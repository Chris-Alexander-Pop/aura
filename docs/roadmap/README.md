# Aura roadmap (from `todo.md`)

This directory expands [todo.md](../../todo.md) into per-area product notes. These documents are **forward-looking**: they describe where Aura (AGS shell + `ui/` panels + `sidecar/`) and the wider system are headed, not only what is implemented today.

## Design direction

The current interface is a dense, tile-based “shell OS”: control center, sidebar tiles, and bar flyouts. Layout, grouping, colors, and which surfaces live in GTK (`src/widget/`) versus React (`ui/`) versus the sidecar will shift as features land. Roadmap docs call out **configurable and modular** panels where the todo asks for drag-and-drop, quickviews, or per-device controls.

For a **shallow** technical snapshot, see [../feature_matrix.md](../feature_matrix.md). Stack notes: [../MIGRATION_STRATEGY.md](../MIGRATION_STRATEGY.md) and [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md). Product scope stays in `todo.md` and this `docs/roadmap/` set.

**Implementation:** resolved backend/stack choices in [../ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md); sidecar task checklist in [../BACKEND_TODO.md](../BACKEND_TODO.md).

## Index

| Document | Focus |
| -------- | ----- |
| [system-and-input-foundation.md](./system-and-input-foundation.md) | Login, packages, Vicinae, notifications, audio keys, capture, storage, voice, multitasking, FN key |
| [sidebar.md](./sidebar.md) | Sidebar: layout, tiles, workspaces, task manager, connectivity tiles |
| [control-panel.md](./control-panel.md) | Revamped control center: packages, network, devices, audio, performance, keybinds, security, VPN, productivity, automations, settings, notifications |
| [calendar-panel.md](./calendar-panel.md) | Rich calendar, sync, scheduling aids, fitness, todo subpanel |
| [devops-panel.md](./devops-panel.md) | Local/home/cloud deployments and resource views |
| [communication-panel.md](./communication-panel.md) | Unified messaging (Beeper-like) on Linux |
| [top-dropdown.md](./top-dropdown.md) | Top dropdown: dashboard, quickviews, modular subpanels |
| [sidebar-popup.md](./sidebar-popup.md) | Quick brightness and volume per device |
| [vault-panel.md](./vault-panel.md) | Unified cloud storage, transfers, backups, sharing |
| [ide-popup.md](./ide-popup.md) | IDE/project picker and agent launchers |
| [system-debugging-popup.md](./system-debugging-popup.md) | Autodebug and backup restore entry points |
| [lock-panel.md](./lock-panel.md) | Lock/sleep, biometrics, visibility hardening |
| [exocortex-panel.md](./exocortex-panel.md) | Second-brain / capture (stub) |
| [skiller-panel.md](./skiller-panel.md) | Skills/training (stub) |
| [secure-a-panel.md](./secure-a-panel.md) | Research and analytical tooling (sensitive) |
| [marginal-gains-panel.md](./marginal-gains-panel.md) | KOLB, marginal gains, studying workflows (stub) |
