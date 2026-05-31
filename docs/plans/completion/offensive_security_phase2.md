# Offensive security phase 2

**Status:** Planned  
**Depends on:** [offensive_security.md](../offensive_security.md) (feature gate shipped)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §2.17 offensive, §3.12, §7 item 7

---

## Goal

Harden **P4 offensive** namespace: audit trail, caps, no accidental exposure in default UI/manifest.

---

## Vertical slices

### Slice A — Audit log

- SQLite table `offensive_audit`: timestamp, method, arg hash (not raw args), user, result status.
- `SecurityOffensive.GetAuditLog` — paginated, read-only.

### Slice B — Rate limits and caps

- Per-method cooldown (e.g. port scan max hosts, max duration).
- Return structured error `-32099` “rate limited” consistent with RPC conventions.

### Slice C — Manifest and build

- Default `rpc-manifest.json` excludes offensive (already); verify `AURA_OFFENSIVE_MANIFEST=1` CI job optional.
- `cargo build --features offensive` documented in sidecar README.

### Slice D — React gating

- Pentest panel only if `GET /api/meta` returns `offensiveEnabled: true` (new meta field).
- No `api.ts` wrappers in default build OR behind `import.meta.env` flag.

### Slice E — Tests

- [devops_rpc_shapes.rs](../../../sidecar/tests/devops_rpc_shapes.rs) or dedicated `security_offensive_rpc_shapes.rs` with `#[cfg(feature = "offensive")]`.
- Deny-list: all `SecurityOffensive.*` destructive methods.

**BACKEND_TODO:** §2.17 offensive hardening, §3.12 UI gate.

---

## Acceptance

- Audit log written on every offensive mutating call
- Default UI build has no pentest routes
- BACKEND_TODO offensive items `[x]` or explicit security ADR

---

## Suggested order

A → B → C → E → D
