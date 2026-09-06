# VPN service (profiles, connect, status)

**Status:** Implemented (2026-05-31)  
**Depends on:** [p0_daily_hardening.md](p0_daily_hardening.md) (Network keyring), [foundation_contracts.md](foundation_contracts.md) (polkit helper)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.7, ADR per-app VPN phases

---

## Goal

Implement the VPN state machine in `vpn.rs`:

1. `Vpn.GetProfiles` from config dir.
2. `Vpn.Connect` / `Vpn.Disconnect` — OpenConnect/OpenVPN/WireGuard with polkit/sudo wrapper.
3. Status RPC — connected, IP, interface; optional NM split-tunnel later.
4. `Network.GetStatus` VPN fields when deferred item completes.
5. Push `Vpn.StatusChanged` on connect/disconnect (like Network).

---

## Non-goals

| Item | Reason |
|------|--------|
| Kill switch | Feature flag + explicit UX (dangerous) |
| mihomo/clash profile | ADR phase 3 — design doc first |

---

## Test strategy

| Layer | Where |
|-------|--------|
| State machine | `#[cfg(test)]` in `vpn.rs` — transitions with mock `VpnProcess` trait |
| Read-only | `vpn_rpc_shapes.rs` **extend** — `GetProfiles`, `GetStatus` shapes |
| Mutations | `Vpn.Connect` / `Vpn.Disconnect` on `DENIED_EXACT` |
| Integration | `#[ignore]` connect failure paths with fake profile paths in temp dir |

Never call real `Vpn.Connect` in `./scripts/sidecar-test-fast.sh`.

---

## Vertical slices

### Slice A — Profile discovery + GetStatus — **P1**

Read configs; disconnected status always ok in CI.

### Slice B — Connect/disconnect + polkit — **P1**

Central allowlist for binaries (`openconnect`, `wg-quick`, …).

### Slice C — Network aggregate + WS — **P2**

Emit `Vpn.StatusChanged`; extend `network_rpc_shapes.rs` if status merged.

---

## Suggested order

A → B → C

---

## Acceptance

- `vpn_rpc_shapes.rs` passes on host without VPN connected  
- Deny-list blocks connect in default integration harness  
- Manual: connect lab profile with polkit prompt once
