# AGENTS — Aura / AGS development

Concise operating manual for humans and AI assistants working in this repository.

## Stack

| Layer | Location | Role |
|--------|----------|------|
| AGS entry | `app.ts` | GTK4 shell: CSS, per-monitor windows, `requestHandler` for `ags msg` |
| GTK widgets | `src/widget/` | Bar, OSD, launcher, WebKit shells, dropdown trigger |
| React panels | `ui/` (Vite) | Built to `ui/dist`, loaded in WebKit overlay windows |
| Backend | `sidecar/` | `ags-sidecar` — JSON-RPC, services under `src/services/` |

Docs: `docs/MIGRATION_STRATEGY.md`, `docs/COMPONENT_MAPPING.md` (legacy mapping); `todo.md` and `docs/roadmap/` (product); `docs/feature_matrix.md` (shallow planning + implementation snapshot).

## Launch and verify

- Full dev stack: `./aura` (see script for cleanup of competing shells, sidecar build, UI build, `bun run watch`).
- Frontend only: `bun run watch` (Tailwind + `ags run app.ts`).
- Hyprland: `aura-launch` runs `~/.config/ags/aura` in `foot` (see `hyprland_aura.conf`).

Smoke IPC (with AGS running): `ags msg toggle control-center` (also `sidebar`, `dropdown`, `calendar` — window names in `src/widget/webview/*.tsx`).

## Critical: sidecar binary path

`src/lib/sidecar.ts` spawns the binary only from:

`$HOME/.config/ags/sidecar/target/debug/ags-sidecar` (then release fallback).

If this repo is checked out elsewhere, either mirror that path (symlink or copy build outputs) or plan a code change to resolve the binary relative to the active AGS config directory.

## Task tool playbook (subagents)

Cursor does not load custom subagent types from the repo. Use the built-in Task types:

| Type | Use when |
|------|-----------|
| `explore` | Broad codebase search, mapping files, readonly reconnaissance |
| `shell` | Git, `cargo`, long-running or batch shell work |
| `generalPurpose` | Cross-cutting edits spanning Rust, TS, and UI |

Project skills live under `.cursor/skills/` (launch, RPC, UI layer choice).

## Cursor rules and indexing

- Rules: `.cursor/rules/*.mdc` (architecture, GTK TS, sidecar, React UI, styling).
- `.cursorignore` excludes `node_modules`, `sidecar/target`, lockfiles to keep context lean.

## Hooks (deferred)

There is no `hooks.json` in this repo yet. Auto-format on save would need a formatter dependency (e.g. Prettier) and a tested `afterFileEdit` hook; command hooks that parse stdin often depend on `jq`. Add hooks only after choosing tooling and verifying in Cursor’s Hooks UI.


## Cursor Cloud specific instructions

### Environment limitations

The AGS GTK4 shell (`ags run app.ts`, `bun run watch`) cannot run in cloud VMs — it requires Hyprland, Wayland, GTK4, and GJS. The testable components in cloud are:

| Component | How to run | Notes |
|-----------|-----------|-------|
| Rust sidecar | `cd sidecar && cargo build && ./target/debug/ags-sidecar` | Serves HTTP on `:9080`, requires `libssl-dev` + `pkg-config` |
| React UI (dev) | `cd ui && bun run dev` | Vite on `:5173`, proxies `/api` and `/ws` to sidecar |
| React UI (build) | `cd ui && bun run build` | Outputs to `ui/dist` |
| TailwindCSS (GTK) | `bun run build:css` (from repo root) | Produces `style/style.css` |
| TypeScript check | `cd ui && bun run tsc --noEmit` | Type-checks the React UI |
| Sidecar tests | `cd sidecar && cargo test` | Integration tests in `tests/integration_test.rs` |

### Running the sidecar + UI together

1. Start sidecar: `cd sidecar && ./target/debug/ags-sidecar` (background or separate terminal)
2. Start Vite: `cd ui && bun run dev`
3. Open `http://localhost:5173` — the UI loads and API calls proxy through to the sidecar

### Gotchas

- The sidecar build requires `libssl-dev` and `pkg-config` system packages (for `openssl-sys` crate).
- Use `bun` (not `npm`/`npx`) for all JS commands — bun is the project’s package manager and its lockfiles (`bun.lock`) are committed.
- `bun run tsc --noEmit` works for type checking without needing a separate Node.js installation.
- The sidecar binary serves `ui/dist` as static files; it finds the directory by walking up from its own executable path. When running from `sidecar/target/debug/`, it correctly resolves `../../ui/dist`.
