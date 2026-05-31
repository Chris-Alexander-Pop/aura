# Process, privileges, and platform harness

**Status:** Planned  
**Depends on:** [foundation_contracts.md](../foundation_contracts.md)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §0.2–0.4, §0.5–0.6, §0.7, §7 item 3

---

## Goal

Centralize **safe subprocess** usage and close foundation gaps so new services do not reinvent allowlists.

---

## Vertical slices

### Slice A — Exec allowlist trait (`utils/process.rs`)

- Introduce `CommandRunner` trait or `run_allowlisted(argv: &[&str], opts: ExecOpts)` used by launcher, capture, vpn, packages.
- Central table: binary path + max output bytes + timeout (defaults: 30s, 1 MiB).
- `#[cfg(test)]` mock returning fixture stdout.

**BACKEND_TODO:** §0.4 first three bullets.

### Slice B — Polkit helper

- `utils/polkit.rs`: `run_privileged(cmd)` wrapper used by security firewall, packages, vpn (migrate ad hoc pkexec).
- Document in [sidecar/README.md](../../../sidecar/README.md).

### Slice C — Request logging + redaction

- In [rpc.rs](../../../sidecar/src/rpc.rs) or middleware: log method name + duration at `debug!`; never log Storage/Settings values or keyring material.
- Env: `AURA_RPC_LOG_PARAMS=1` for dev-only param logging with redaction list.

**BACKEND_TODO:** §0.5 request logging.

### Slice D — Live WebSocket integration test

- Extend [server_http_test.rs](../../../sidecar/tests/server_http_test.rs): after `Power` poll tick or synthetic sysfs change, assert WS receives `Power.BatteryState` (may use `notify::emit` from power service test hook if live poll too heavy for CI).

**BACKEND_TODO:** §0.6 live host change test (or mark `[~]` if synthetic-only).

### Slice E — CI + manual matrix

- Add `.github/workflows/sidecar.yml` (or document in README “run locally before push” if no GH Actions desired).
- [sidecar/README.md](../../../sidecar/README.md): manual matrix table (Wi-Fi connect, BT pair, battery, VPN dry-run).

**BACKEND_TODO:** §0.7 CI + manual matrix.

### Slice F — Types hardening (partial)

- Add `#[serde(deny_unknown_fields)]` on top-level DTOs in [types.rs](../../../sidecar/src/types.rs) used by `api-types.ts` (batch by service, start with Settings/AuraSettings, Todos, Launcher result).
- Mirror critical fields in [src/lib/types.ts](../../../src/lib/types.ts).

**BACKEND_TODO:** §0.2.

### Slice G — GTK binary resolver

- [src/lib/sidecar.ts](../../../src/lib/sidecar.ts): implement ADR repo-walk or require `AURA_SIDECAR` (document only if product defers code change).

**BACKEND_TODO:** §0.1 binary path `[~]` → `[x]`.

---

## Acceptance

- At least launcher + capture migrated to central exec helper
- Mock runner unit tests pass
- README documents CI command and manual matrix
- BACKEND_TODO §0.4–0.7 mostly `[x]` or explicit `[~]`

---

## Suggested order

A → B → C → D → E → F (ongoing) → G
