# Launcher and Vicinae integration

**Status:** Implemented (2026-05-31, Vicinae deferred)  
**Depends on:** [shell_platform.md](shell_platform.md) (`Settings.*` for layout), [foundation_contracts.md](foundation_contracts.md)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §3.3, [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md) (Vicinae socket)

---

## Goal

New **`launcher.rs`** service for app search and Vicinae orchestration:

1. `Launcher.Query` — fuzzy match `.desktop` entries.
2. `Launcher.Run` — allowlisted launch (`gtk-launch`, `xdg-open`).
3. `Vicinae.Exec` — long-lived socket/RPC with subprocess fallback (ADR).
4. `Launcher.Recent` / `Pin` — SQLite namespace `launcher`.
5. Wire GTK launcher + future overpowered Vicinae flows in `todo.md`.

---

## Non-goals

| Item | Reason |
|------|--------|
| Full command palette UI | Product iteration after RPC stable |
| Arbitrary user shell | Allowlist only |

---

## Test strategy

| Layer | Where |
|-------|--------|
| Desktop entry parse | `#[cfg(test)]` in `launcher.rs` + `tests/fixtures/desktop/` |
| Read-only RPC | **New** `launcher_rpc_shapes.rs` — `Launcher.Query` with `{ "query": "term" }` |
| Storage | **New** `launcher_storage_test.rs` — recent/pin round-trip, temp DB |
| Vicinae | Mock Unix socket or `#[ignore]` integration with `VICINAE_SOCKET` env |
| Mutations | `Launcher.Run`, `Vicinae.Exec` on `DENIED_EXACT` |

Register readonly methods in `READONLY_GAP_FAST` only after deterministic stub env (empty query → `[]`).

---

## Vertical slices

### Slice A — Desktop index + Query — **P2**

Build index at startup (refresh RPC optional).

### Slice B — Run allowlist — **P2**

Shared pattern with `shell.rs` `Apps.Launch` allowlist.

### Slice C — Vicinae bridge — **P2**

Socket client + timeout; document env vars in `sidecar/README.md`.

### Slice D — Recent/pin + UI — **P2**

`api.ts` + launcher widget / webview host.

---

## Suggested order

A → B → C → D

---

## Acceptance

- `Launcher.Query` returns array shape in CI without desktop dir (empty ok)  
- `launcher_storage_test.rs` passes in fast gate  
- Contract script includes new methods
