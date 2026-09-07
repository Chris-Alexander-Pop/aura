---
name: ags-dev-launch
description: >-
  Runs and debugs the Aura AGS shell (./aura, bun run watch, Hyprland aura-launch),
  including sidecar binary path under ~/.config/ags and ags msg smoke tests. Use
  when the user wants to launch the project, see the UI, fix startup errors, or
  verify IPC after changes.
---

# Aura / AGS — launch and verify

## Quick commands

| Goal | Command |
|------|---------|
| Fresh clone / not `~/.config/ags` | `./setup` (symlink, Hypr links, sidecar + UI) |
| Full dev (release sidecar, optional cargo-watch, UI build, Tailwind + AGS) | `./aura` from repo root |
| Same without React UI build | `./aura --no-ui-build` |
| Same without Rust watch | `./aura --no-rust-watch` |
| Debug sidecar (symbols, ~200 MB binary) | `./aura --debug` |
| CSS + AGS only (no `./aura` pipeline) | `bun run watch` |
| Stop a background `./aura --bg` session | `./aura --kill` (PID file only; no `killall`) |

Ensure `bun install` at repo root and in `ui/`. `./aura` uses bun (`$RUNNER run build`) when bun is on PATH.

## Hyprland vs repo cwd

- `./setup` makes `~/.config/ags` this clone (symlink if needed).
- `aura-launch` starts `foot` and runs **`~/.config/ags/aura`**.
- If you skip setup, set `AURA_SIDECAR` / `AURA_DIR` or run `./aura` from the tree AGS actually loads.

## Sidecar binary

1. `AURA_SIDECAR` (set by `./aura` for that session)
2. `$AURA_DIR` then `$XDG_CONFIG_HOME/ags/sidecar/target/{release,debug}/ags-sidecar`

## What `./aura` does not do

It does **not** `killall` bun, ags, notification daemons, or competing shells. Stop other desktops yourself if they contend for the bar. `--kill` only signals PIDs recorded from `--bg`.
