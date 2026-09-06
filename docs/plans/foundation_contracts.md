# Foundation and API contracts

**Status:** Done (2026-05-31)  
**Depends on:** Existing `rpc-manifest.json`, `scripts/check-api-rpc-contract.sh`, `sidecar/tests/rpc_contract_test.rs`  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §0.1–0.7, §1, §5

---

## Goal

Close **platform hygiene** gaps so every future slice lands with stable CI and client parity:

1. **Sidecar README** — build, HTTP `:9080`, env vars (`AURA_SIDECAR`, `AURA_STORAGE_DB`, `RUST_LOG`), polkit expectations.
2. **Binary path** — document `AURA_SIDECAR` + XDG path; optional repo-walk resolver (ADR).
3. **RPC manifest in CI** — `scripts/generate-rpc-manifest.sh` fails on drift.
4. **§1 contract table** — every `ui/src/lib/api.ts` method registered; defined errors when tools missing (no 500 from missing method).
5. **Test harness** — fixtures dir, WS integration test for real notify delivery, optional mocked `exec_command` trait.

---

## Non-goals

| Item | Reason |
|------|--------|
| New product RPCs | Belongs in domain plans |
| Full 90% coverage | Incremental per service |

---

## Vertical slices

### Slice A — Documentation and resolver — **P0**

| Task | Details |
|------|---------|
| A.1 | `sidecar/README.md`: build, fast vs full test, coverage script, deny-list summary |
| A.2 | Document `src/lib/sidecar.ts` path resolution; dev clone symlink pattern |
| A.3 | Arch vs multi-distro stance (pacman branches today) |

**Tests:** None (docs only).

---

### Slice B — Manifest and contract gate — **P0**

| Task | Details |
|------|---------|
| B.1 | CI runs `generate-rpc-manifest.sh` + `check-api-rpc-contract.sh` |
| B.2 | Extend `rpc_contract_test.rs` for any new namespaces as they land |
| B.3 | Add `api.ts` readonly sweep in `integration_test.rs` (`api_ts_readonly_methods_resolve`) — one `call_method` per safe `api.ts` export; skip entries in `common::is_denied_rpc_method` |

**Tests:**

| File | What |
|------|------|
| `rpc_contract_test.rs` | Manifest regex coverage, denied-method documentation |
| `integration_test.rs` | New `api_ts_readonly_methods_resolve` loop (mirror `P0_METHODS` pattern) |
| `scripts/check-api-rpc-contract.sh` | Stays green in `./scripts/sidecar-test-fast.sh` |

---

### Slice C — Fixtures and shared test infra — **P1**

| Task | Details |
|------|---------|
| C.1 | `sidecar/tests/fixtures/` — `nmcli/`, `pactl/`, `pacman/`, `power_supply/` samples |
| C.2 | `integration_contracts.rs` — one test per fixture parser |
| C.3 | WS integration: extend `server_http_test.rs` — trigger host change or call `notify::emit` after registry handler (pattern: `websocket_receives_calendar_events_changed`) |

**Tests:**

| File | What |
|------|------|
| `integration_contracts.rs` | Pure parse, no RPC |
| `server_http_test.rs` | `/ws` receives `{ method, params }` |
| `integration_test.rs` | Do not add slow host methods to `READONLY_GAP_FAST` without review |

---

### Slice D — Registry and logging — **P2**

| Task | Details |
|------|---------|
| D.1 | Unknown method → structured error code |
| D.2 | Request logging behind `RUST_LOG` with secret redaction |
| D.3 | Structured RPC error mapping in `rpc.rs` |

**Tests:** Unit tests on error mapping in `rpc.rs` `#[cfg(test)]` if added.

---

## Suggested order

1. Slice A (README)  
2. Slice B (contract gate)  
3. Slice C (fixtures + WS)  
4. Slice D (registry polish)

---

## Acceptance

- `./scripts/sidecar-test-fast.sh` green  
- Contract script passes on clean tree  
- New contributor can find build/test/docs in `sidecar/README.md` without reading chat history
