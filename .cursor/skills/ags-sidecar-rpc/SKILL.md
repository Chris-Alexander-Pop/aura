---
name: ags-sidecar-rpc
description: >-
  Extends or fixes the Rust ags-sidecar JSON-RPC surface and keeps TypeScript and
  React clients in sync. Use when editing sidecar/src/rpc.rs, types, services, or
  when the user mentions RPC, sidecar methods, or integration_test failures.
---

# Sidecar RPC workflow

## Change order (contract-first)

1. **Rust**: Define or adjust types in `sidecar/src/types.rs` and handler routing in `sidecar/src/rpc.rs` (and the relevant `src/services/*.rs`).
2. **TypeScript**: Mirror request/response shapes in `src/lib/types.ts` and add or update methods on the client in `src/lib/sidecar.ts` (GObject signals if notifications are part of the contract).
3. **UI consumers**: GTK widgets under `src/widget/` and/or `ui/src/lib/api.ts` (and pages) — update call sites and error handling.
4. **Verify**: `cd sidecar && cargo test` — extend `tests/integration_test.rs` when adding user-visible RPC behavior.

## Files to know

| Concern | Path |
|---------|------|
| RPC dispatch | `sidecar/src/rpc.rs` |
| Shared DTOs | `sidecar/src/types.rs` |
| Service modules | `sidecar/src/services/` |
| TS types | `src/lib/types.ts` |
| Process + stdin/stdout | `src/lib/sidecar.ts` |

## Do not forget

- JSON field names and optional fields must match serde on both sides.
- Long-running or privileged operations stay in Rust; keep GTK/React thin for orchestration.
- After RPC edits, a full `./aura` or rebuilt sidecar binary must land where `sidecar.ts` expects it if testing under the user’s real config path (`~/.config/ags/...`).
