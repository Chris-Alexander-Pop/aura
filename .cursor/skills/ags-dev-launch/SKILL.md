---
name: ags-dev-launch
description: >-
  Runs and debugs the Aura AGS shell (./aura, bun run watch, Hyprland aura-launch),
  including sidecar binary path under ~/.config/ags, competing shell cleanup, and
  ags msg smoke tests. Use when the user wants to launch the project, see the UI,
  fix startup errors, or verify IPC after changes.
---

# Aura / AGS — launch and verify

## Quick commands

| Goal | Command |
|------|---------|
| Full dev (release sidecar, optional cargo-watch, UI build, Tailwind + AGS) | `./aura` from repo root |
| Same without React UI build | `./aura --no-ui-build` |
| Same without Rust watch | `./aura --no-rust-watch` |
| Debug sidecar (symbols, ~200 MB binary) | `./aura --debug` |
| CSS + AGS only (no `./aura` pipeline) | `bun run watch` |

Ensure `bun install` has been run at repo root; `ui/` uses `npm` for `npm run build` inside `./aura`.

## Hyprland vs repo cwd

- `aura-launch` starts `foot` and runs **`~/.config/ags/aura`**, not necessarily this clone.
- If you edit a different directory, sync or run `./aura` from the tree AGS actually loads.

## Sidecar binary (common failure)

`src/lib/sidecar.ts` spawns only:

1. `$HOME/.config/ags/sidecar/target/release/ags-sidecar`
2. Else `$HOME/.config/ags/sidecar/target/debug/ags-sidecar`

`./aura` runs `cargo build --release` by default and sets `AURA_SIDECAR` to the built binary for that session. Use `./aura --debug` for a debug build. If the repo is not `~/.config/ags`, align checkout or copy the binary into the expected path until the resolver is improved.

## What `./aura` cleans up

Stops `qs` / `quickshell` / `caelestia`, several notification daemons, and orphaned `ags` / `bun` / sidecar-related processes (see `aura` script). Expect Quickshell/Caelestia to exit when starting Aura.

## Smoke checks (AGS already running)

```bash
ags msg toggle control-center
```

Other window names: `sidebar`, `dropdown`, `calendar` (see `src/widget/webview/*Window.tsx`).

## Dependencies

`ags`, `bun` (or `npm`), `cargo`; optional `cargo install cargo-watch` for auto-rebuild on Rust changes.
