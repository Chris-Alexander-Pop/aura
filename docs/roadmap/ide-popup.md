# IDE popup (roadmap)

## Summary

A **fast “open project”** flow from the shell: **choose an IDE** (or editor), **pick a folder**, optionally **spawn a terminal** in that tree, and **launch CLI coding agents** (e.g. **Jules** and similar) with a **consistent** environment. The popup should be **keyboard-first** and work alongside Vicinae in [system-and-input-foundation.md](./system-and-input-foundation.md).

## Current baseline

Aura has a **launcher** path in the matrix; IDEs are not a dedicated feature row. [../feature_matrix.md](../feature_matrix.md) / [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md) for launcher vs control center.

## Goals

### Choose IDE and folder location, launch CLI agents easily..

- **Recents** and **bookmarks** for project roots.
- **IDE** list: VS Code, Zed, Neovim, user-defined **desktop entries** or scripts.
- **One-shot** “open in terminal here” and **debug profile** (env vars) per project (optional).

### Jules & etc.. integration

**Jules** (or comparable **agent** products): start a **task** with **repo** context, **API key** from keyring, and **visible** log/progress in the UI. **TBD** exact SDK; treat as **optional plugin** with graceful degradation if the CLI is absent.

**Candidate scope**: a **templated** command panel where users map **“agent” → command + args + cwd**, not hard-coding a single vendor.

## Out of scope / risks

- **Agent** tools can run **arbitrary** code; require explicit **trust** of directory and a **non-silent** first run.
- Storing **API keys** in dotfiles is unacceptable; use **keyring** + env injection.

## Dependencies

- [system-and-input-foundation.md](./system-and-input-foundation.md) (Vicinae, keybinds).
- Optional [vault-panel.md](./vault-panel.md) for monorepos on cloud storage.
- xdg / `.desktop` parsing for **installed** IDEs on Linux.

## Open questions

1. **Monorepo** subproject detection: manual only or **git** root heuristics?
2. **Flatpak** IDEs: spawn with correct `flatpak run` and portal paths?
3. **Remote** dev (SSH) in v1 or out?
