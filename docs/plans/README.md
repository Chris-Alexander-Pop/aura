# Sidecar implementation plans

Plans are **vertical slices** aligned with [BACKEND_TODO.md](../BACKEND_TODO.md). Each plan lists goals, non-goals, test layout (see [sidecar/tests/README.md](../../sidecar/tests/README.md)), and suggested PR order.

## Completed

| Plan | Status | BACKEND_TODO focus |
|------|--------|-------------------|
| [control_center_foundation.md](control_center_foundation.md) | Done (2026-05-28) | §3.1–3.2, §2.16–2.18 |
| [control_center_depth.md](control_center_depth.md) | Done (2026-05-29) | §2.19–2.21, §2.23 |
| [compositor_bar_live.md](compositor_bar_live.md) | Done (2026-05-29) | §2.8, §2.10 |
| [shell_platform.md](shell_platform.md) | Done (2026-05-30) | §1, §3.4–3.5, §2.23 push, §2.9 |
| [foundation_contracts.md](foundation_contracts.md) | Done (2026-05-31) | §0, §1 |
| [automation_workflows.md](automation_workflows.md) | Done (2026-05-31) | §2.21 |
| [session_lock_extended.md](session_lock_extended.md) | Done (2026-05-31) | §2.9, §3.8 |
| [p0_daily_hardening.md](p0_daily_hardening.md) | Done (2026-05-31) | §2.5–2.6 |
| [dashboard_aggregation.md](dashboard_aggregation.md) | Done (2026-05-31) | §3.13 |
| [control_center_hardening.md](control_center_hardening.md) | Done (2026-05-31) | §2.15–2.18, §2.11–2.12, §2.16 (slices A–C) |
| [launcher_vicinae.md](launcher_vicinae.md) | Done (2026-05-31) | §3.3 (slices A–B; Vicinae deferred) |
| [vpn_service.md](vpn_service.md) | Done (2026-05-31) | §2.7 (slices A–B) |

## Remaining

| Order | Plan | Priority | Depends on | BACKEND_TODO focus |
|------:|------|----------|------------|-------------------|
| 1 | [calendar_sync_todos.md](calendar_sync_todos.md) | P2–P3 | control_center_depth, shell_platform | §2.23 ICS, §3.6 Todos |
| 2 | [vault_p3_panels.md](vault_p3_panels.md) | P3 | foundation | §3.7, §2.22, stubs |
| 3 | [offensive_security.md](offensive_security.md) | P4 | control_center_hardening | §2.17 feature flag |

## In progress

| Plan | Focus |
|------|--------|
| [calendar_sync_todos.md](calendar_sync_todos.md) | ICS import, `todos.rs`, optional CalDAV defer |
| [vault_p3_panels.md](vault_p3_panels.md) | Vault read-only + communication/fitness stubs |
| [offensive_security.md](offensive_security.md) | `offensive-security` Cargo feature gate |

Run `./scripts/sidecar-test-fast.sh` after each slice.

## Test layout

| Pattern | Use for |
|---------|---------|
| `tests/{service}_rpc_shapes.rs` | Read-only RPC shape tests |
| `tests/{service}_storage_test.rs` | Temp DB mutations via `call_method_unchecked` |
| `tests/integration_test.rs` | `P0_METHODS`, readonly gap sweeps |
| `#[cfg(test)]` in services | Parsers, argv builders, state machines |

Subagents **must not** `git commit` — parent commits scoped files per plan.
