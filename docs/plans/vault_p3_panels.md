# Vault, communication, fitness, and deferred panels

**Status:** Implemented (2026-05-31, slice A)  
**Depends on:** [foundation_contracts.md](foundation_contracts.md), keyring helper  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §3.7, §2.22, §2.24, §3.9–3.11, §3.14

---

## Goal

Greenfield and **stub-to-real** services for roadmap panels that are not daily CC:

| Service | RPC focus |
|---------|-----------|
| `vault.rs` | rclone registry, `Vault.List`, transfer, backup status |
| `communication.rs` | Deepen `GetUnread`, bridges read-only phase 1 |
| `fitness.rs` | Goals, workout history in SQLite |
| `ide.rs` | `Ide.List`, `Ide.Launch` allowlist |
| `debug.rs` | `Debug.CollectBundle`, `HealthCheck` |
| `voice.rs` | Engines list, local transcribe (P4) |
| Stubs | Exocortex, Skiller, Secure-A — document only per §3.14 |

ADR: **communication hub** remains separate app long-term; sidecar stays thin aggregation.

---

## Non-goals

| Item | Reason |
|------|--------|
| Plaintext secrets in logs | Redaction required |
| Full Matrix send path v1 | Read-only first |

---

## Test strategy

| Service | Files |
|---------|--------|
| Vault | **New** `vault_rpc_shapes.rs` (readonly list/status); transfer on deny-list |
| Communication | **Extend** `communication_rpc_shapes.rs` — unread map keys |
| Fitness | **Extend** `fitness_rpc_shapes.rs` + `fitness_storage_test.rs` |
| IDE / Debug / Voice | New `*_rpc_shapes.rs` when registered; argv/unit only for voice path |
| Stubs | No tests until product spec — track in BACKEND_TODO §3.14 only |

Use `READONLY_GAP_SLOW_HOST` for rclone/docker-touching probes.

---

## Vertical slices

### Slice A — Vault read-only — **P3**

List remotes, backup status without transfer.

### Slice B — Communication read bridges — **P3**

Per-app unread; mock adapters.

### Slice C — Fitness + IDE — **P3**

Storage tests + allowlisted launch.

### Slice D — Debug bundle + Voice — **P4**

Redacted bundle; local STT behind feature flag.

---

## Suggested order

A → B → C → D (each may be its own PR)

---

## Acceptance

- New services register in manifest; shape tests pass with empty/stub data  
- No offensive or vault transfer in fast gate
