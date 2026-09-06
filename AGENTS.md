# AGENTS — Aura / AGS development

Concise operating manual for humans and AI assistants working in this repository.

## Stack

| Layer | Location | Role |
|--------|----------|------|
| AGS entry | `app.ts` | GTK4 shell: CSS, per-monitor windows, `requestHandler` for `ags msg` |
| GTK widgets | `src/widget/` | OSD, optional GTK bar (`AURA_GTK_BAR=1`), WebKit window shells |
| React panels | `ui/` (Vite) | **Default bar**, control center, calendar, dropdown — `ui/dist` in WebKit |
| Backend | `sidecar/` | `ags-sidecar` — JSON-RPC, services under `src/services/` |

Docs: `docs/MIGRATION_STRATEGY.md`, `docs/COMPONENT_MAPPING.md` (stack / component map); `todo.md` and `docs/roadmap/` (product); `docs/feature_matrix.md` (shallow planning + implementation snapshot); **`docs/ARCHITECTURE_DECISIONS.md`** (resolved stack/product choices); **`docs/BACKEND_TODO.md`** (sidecar implementation checklist).

## Launch and verify

- Full dev stack: `./aura` (sidecar build, UI build, `bun run watch`). Does not kill other user processes.
- Frontend only: `bun run watch` (Tailwind + `ags run app.ts`). Set `AURA_SIDECAR` or build into `$XDG_CONFIG_HOME/ags/sidecar/target/`.
- Hyprland: `aura-launch` runs `~/.config/ags/aura` in `foot`. Aura-owned Hypr fragments: `hypr/` (see below).

Smoke IPC (with AGS running): `ags msg toggle control-center` (also `sidebar`, `dropdown`, `calendar` — window names in `src/widget/webview/*.tsx`).

### Crash dumps (local only)

Sidecar panics, AGS/JS fatals, and unexpected sidecar exits write JSON under `~/.local/share/aura/crashes/` (override with `AURA_CRASH_DIR`). Last 50 dumps are kept. No remote upload.

## Hypr config (Aura-owned)

Aura owns the **full Hyprland Lua compositor config** plus polkit/lock/PAM under `hypr/`:

```bash
./scripts/aura-hypr-link.sh          # XDG symlinks + ~/.config/hypr/hyprland.lua stub
./scripts/aura-hypr-link.sh --check  # verify links + Aura Lua entry
hyprctl reload
```

| Path | Role |
|------|------|
| `hypr/hyprland.lua` | Canonical compositor entry |
| `hypr/hyprland/*.lua` | Modules (keybinds, execs, monitors, …) |
| `~/.config/hypr/hyprland.lua` | Thin stub — do not put real config here |

Full checklist: [`hypr/README.md`](hypr/README.md). Polkit: build hyprtoolkit agent with `./scripts/build-hypr-polkit.sh` (Arch’s package is still Qt).

**Workspace binds:** use `hl.dsp.focus` / `hl.dsp.window.move` — classic `hyprctl dispatch workspace N` fails on Lua-config Hyprland.

## Critical: sidecar binary path

Resolution order (see `docs/ARCHITECTURE_DECISIONS.md`):

1. `AURA_SIDECAR` env var (absolute path) — **`./aura` sets this** for the session
2. `$XDG_CONFIG_HOME/ags/sidecar/target/{release,debug}/ags-sidecar`

A clone that is not `~/.config/ags` must set `AURA_SIDECAR` (or symlink the binary). There is no `Engineering/Productivity` fallback.

**Google Calendar:** set `AURA_GOOGLE_OAUTH_CLIENT_ID` (Desktop OAuth client, Calendar API enabled). Optional `AURA_GOOGLE_OAUTH_CLIENT_SECRET`. Connect from the calendar panel; refresh tokens go in GNOME Keyring (`secret-tool`).

**Stack defaults (Arch):** NetworkManager, PipeWire+WirePlumber, GNOME Keyring, hyprlock, Podman (not Docker by default), Freedesktop notifications D-Bus.

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
| Sidecar tests | `cd sidecar && cargo test` | Integration tests in `tests/integration_test.rs`; safety rules in `sidecar/tests/README.md` |
| Sidecar coverage | `./scripts/sidecar-coverage.sh` | Needs `cargo-llvm-cov` + `llvm-tools-preview`; HTML under `sidecar/target/coverage/html/` |

### Running the sidecar + UI together

1. Start sidecar: `cd sidecar && ./target/debug/ags-sidecar` (background or separate terminal)
2. Start Vite: `cd ui && bun run dev`
3. Open `http://localhost:5173` — the UI loads and API calls proxy through to the sidecar

### Gotchas

- The sidecar build requires `libssl-dev` and `pkg-config` system packages (for `openssl-sys` crate).
- Use `bun` (not `npm`/`npx`) for all JS commands — bun is the project’s package manager and its lockfiles (`bun.lock`) are committed.
- `bun run tsc --noEmit` works for type checking without needing a separate Node.js installation.
- The sidecar binary serves `ui/dist` as static files; it finds the directory by walking up from its own executable path. When running from `sidecar/target/debug/`, it correctly resolves `../../ui/dist`.
