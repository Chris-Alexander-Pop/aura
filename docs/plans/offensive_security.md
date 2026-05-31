# Offensive security feature gate

**Status:** Planned (P4)  
**Depends on:** [control_center_hardening.md](control_center_hardening.md) (defensive security first), [foundation_contracts.md](foundation_contracts.md)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.17, [pentest-panel.md](../roadmap/pentest-panel.md)

---

## Goal

1. Move `Security.Offensive.*` (~60+ methods) behind Cargo feature **`offensive-security`**.
2. Default build: methods **not registered** (or return `feature_disabled`).
3. Allowlist binaries; refuse run-as-root by default; audit log per invocation.
4. Document legal/ethical use; **no React exposure** until security review.

---

## Non-goals

| Item | Reason |
|------|--------|
| Enabling in CI fast gate | `cargo test --features offensive-security` optional job only |
| Defensive firewall work | [control_center_hardening.md](control_center_hardening.md) |

---

## Test strategy

| Build | Tests |
|-------|--------|
| Default | `security_rpc.rs` — assert `Security.Offensive.*` not callable / not in manifest |
| `--features offensive-security` | Separate `security_offensive_contracts.rs` (optional) with **mocked** binaries only |
| Deny-list | Keep `Security.Offensive.` prefix on `DENIED_PREFIXES` even when feature enabled for integration harness |

Never run real port scans or exploit tooling in automated tests.

---

## Vertical slices

### Slice A — Feature flag + compile-time registration — **P4**

`cfg(feature = "offensive-security")` around module + registry.

### Slice B — Allowlist + audit log — **P4**

Append-only log in `~/.local/share/ags-sidecar/offensive-audit.jsonl`.

### Slice C — Docs + pentest panel gate — **P4**

Roadmap + ADR cross-link; UI hidden behind env `AURA_OFFENSIVE_UI=1` if ever needed.

---

## Suggested order

A → B → C

---

## Acceptance

- Default `cargo test` and `./scripts/sidecar-test-fast.sh` do not register offensive methods  
- Optional CI job documented in `sidecar/README.md`  
- Manual pentest panel remains disabled in production builds
