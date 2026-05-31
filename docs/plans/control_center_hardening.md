# Control Center service hardening

**Status:** Planned  
**Depends on:** [control_center_foundation.md](control_center_foundation.md), [p0_daily_hardening.md](p0_daily_hardening.md) (fixtures)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.11–2.18, §2.12, §2.15–2.16

---

## Goal

Move CC panes from **aggregates + stubs** to **real data paths** where BACKEND_TODO still has gaps:

1. **Packages** — dependency graph, pacman fixtures, optional AUR/Flatpak gates.
2. **Security (defensive)** — firewall toggle verification, SSH/sudo logs, fprintd list, ClamAV scan RPC (opt-in).
3. **Performance** — real `/proc` paths, governor RPC behind polkit, align with `processes.rs`.
4. **Processes** — `Process.ListTop`, safe `Process.Kill` with UI confirmation token.
5. **Weather** — API key from env/keyring, cache, fixture tests.
6. **Logs** — `FollowLogs` WebSocket stream (optional).

---

## Non-goals

| Item | Reason |
|------|--------|
| Offensive security | [offensive_security.md](offensive_security.md) |
| Communication hub | ADR separate app |
| GameMode | Small; can tag on end of slice |

---

## Test strategy

| Service | File | Notes |
|---------|------|-------|
| Packages | `integration_contracts.rs` + extend `integration_test.rs` | `pacman -Qu` fixture; install RPCs denied |
| Security | `security_contracts.rs`, `security_rpc.rs` | ufw/nftables stdout fixtures; no `Security.Offensive.*` |
| Performance | `system_rpc_shapes.rs` or dedicated shapes if split | Real metric fields; `Performance.Set*` denied |
| Processes | `process_rpc_shapes.rs` | `ListTop` readonly; `Process.Kill` denied |
| Weather | `weather_rpc_shapes.rs` | API JSON fixtures in `tests/fixtures/weather/` |
| Logs | `logs_rpc_shapes.rs` | `FollowLogs` only in `server_http_test.rs` if WS stream added |

Slow security/package probes → `READONLY_GAP_SLOW_HOST`.

---

## Vertical slices

### Slice A — Packages depth — **P2**

`GetPackageDependencies`, reverse deps, auto-update policy RPC (read-only policy get first).

### Slice B — Security defensive — **P2**

Firewall enable/disable with polkit helper; fprintd list; read-only password policy.

### Slice C — Performance + Processes — **P2**

Real CPU/GPU/disk; dedupe `Performance.GetProcesses` vs `Process.ListTop`.

### Slice D — Weather + Logs follow — **P2**

`Weather.Get` with cache; optional `Logs.FollowLogs` WS (extend `server_http_test.rs`).

---

## Suggested order

A → C → B → D

---

## Acceptance

- `Security.GetStatus` fields match probes on dev machine (manual row in BACKEND_TODO §6)  
- `Performance.GetMetrics` includes non-zero CPU when host loaded  
- Fast gate unchanged; slow sweep documented in `tests/README.md`
