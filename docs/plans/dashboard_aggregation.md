# Dashboard and sidebar aggregation

**Status:** Implemented (2026-05-31)  
**Depends on:** [shell_platform.md](shell_platform.md) (`Settings.*` module ids), compositor bar WS patterns  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §3.13, [top-dropdown.md](../roadmap/top-dropdown.md), [sidebar.md](../roadmap/sidebar.md)

---

## Goal

Reduce dropdown/sidebar **RPC chatter** with small aggregate endpoints:

1. `Dashboard.GetQuickStatus` — battery, network, BT, DND, next calendar event (read-only compose).
2. `Sidebar.GetTileData` — named tiles (`network`, `audio`, `productivity`, …).
3. Optional push: `Dashboard.QuickStatusChanged` debounced (reuse `notify` bus).

---

## Non-goals

| Item | Reason |
|------|--------|
| DnD module editor | UI product work; backend only stores module ids in Settings |
| Replacing per-pane RPCs | Aggregates are cache-friendly shortcuts |

---

## Test strategy

| File | What |
|------|------|
| **New** `dashboard_rpc_shapes.rs` | `GetQuickStatus` object fields; `GetTileData` with `{ "tile": "network" }` |
| `integration_test.rs` | Add to `P0_METHODS` or `READONLY_GAP_FAST` once stable |
| `server_http_test.rs` | Optional WS test for `Dashboard.QuickStatusChanged` |
| Unit | `#[cfg(test)]` in `dashboard.rs` — merge logic with mocked sub-service JSON |

No subprocess mutations; aggregator only calls existing readonly handlers internally.

---

## Vertical slices

### Slice A — `dashboard.rs` service — **P2**

Implement composer calling existing service functions (not subprocess re-entry).

### Slice B — Settings integration — **P2**

Read `dropdown_modules` from `Settings.Get`; filter tiles.

### Slice C — UI — **P2**

Top dropdown + sidebar hosts use single poll or WS invalidate.

---

## Suggested order

A → B → C

---

## Acceptance

- One `GetQuickStatus` call returns all fields asserted in `dashboard_rpc_shapes.rs`  
- Dropdown opens without 6+ parallel CC-grade polls (manual check)
