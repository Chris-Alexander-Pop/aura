# Aura stack strategy

## Executive Summary

Aura is an AGS/TypeScript + Rust sidecar desktop shell. The stack is chosen for type safety, a stable JSON-RPC contract, and a split between GTK chrome and React panels.

## Layers

| Feature | Implementation | Benefit |
| :--- | :--- | :--- |
| **Shell chrome** | TypeScript (GJS) via AGS / GTK4 | Layer-shell windows, `ags msg` |
| **Panels** | React in WebKit (`ui/`) | Dense settings UIs without GTK layout pain |
| **Backend** | Rust sidecar (`ags-sidecar`) | Concurrency, allowlisted exec, tests |
| **IPC** | JSON-RPC (stdio + POST HTTP on loopback) | Typed contracts between GTK, UI, and services |
| **Styling** | TailwindCSS + Catppuccin Mocha tokens | Shared palette across GTK CSS and Vite |

## Build order (historical)

### Phase 1: Sidecar

Core services (VPN, audio, power, network) behind JSON-RPC. Deliverable: `ags-sidecar` with a CLI/client for smoke queries.

### Phase 2: AGS host

`app.ts`, Tailwind → `style/style.css`, `SidecarClient` in TypeScript.

### Phase 3: Surfaces

Status bar, notification/control center, launcher, OSD — React in WebKit by default; GTK for OSD and hosts.

### Phase 4: Hardening

RPC stress, sidecar auto-restart on disconnect, polkit for privileged actions (VPN, packages).

## Risk Management

- **GTK theming is awkward** — prefer Tailwind utilities; keep GTK CSS small.
- **Sidecar crash** — AGS restarts the binary; widgets grey out on disconnect.
- **Privileged commands** — polkit / allowlisted helpers, not ad-hoc sudo from the UI.
