# Sidecar implementation plans

Plans are **vertical slices** aligned with [BACKEND_TODO.md](../BACKEND_TODO.md). Each plan lists goals, non-goals, test layout (see [sidecar/tests/README.md](../../sidecar/tests/README.md)), and suggested PR order.

## Completed

| Plan | Status | BACKEND_TODO focus |
|------|--------|-------------------|
| [control_center_foundation.md](control_center_foundation.md) | Done (2026-05-28) | §3.1–3.2, §2.16–2.18 (Notifications, Keybinds, Logs/Security/Performance aggregates) |
| [control_center_depth.md](control_center_depth.md) | Done (2026-05-29) | §2.19–2.21, §2.23 (DevOps, Automation, Productivity, Calendar CRUD) |
| [compositor_bar_live.md](compositor_bar_live.md) | Done (2026-05-29) | §2.8, §2.10 (Hyprland typed RPCs, dispatch allowlist, WS, media transport) |
| [shell_platform.md](shell_platform.md) | Done (2026-05-30) | §1, §3.4–3.5, §2.23 push, §2.9 lock (Settings, Capture, contract gate, `Calendar.EventsChanged`) |
| [foundation_contracts.md](foundation_contracts.md) | Done (2026-05-31) | §0, §1 — README, manifest gate, `api.ts` sweep, fixtures, unknown-method errors |
| [automation_workflows.md](automation_workflows.md) | Done (2026-05-31) | §2.21 — SQLite workflows, cron tick, trigger/run logging, UI history |
| [session_lock_extended.md](session_lock_extended.md) | Done (2026-05-31) | §2.9, §3.8 — logind session actions, lock/sleep RPCs |
| [p0_daily_hardening.md](p0_daily_hardening.md) | Done (2026-05-31) | §2.5–2.6 — `System.GetStats`, multi-monitor brightness, fixtures |
| [dashboard_aggregation.md](dashboard_aggregation.md) | Done (2026-05-31) | §3.13 — `Dashboard.GetQuickStatus`, `Sidebar.GetTileData` |

## Remaining (suggested order)

| Order | Plan | Priority | Depends on | BACKEND_TODO focus |
|------:|------|----------|------------|-------------------|
| 1 | [control_center_hardening.md](control_center_hardening.md) | P2 | p0, foundation | §2.15–2.18, §2.12, §2.11, §2.16 — Packages, Security defensive, Performance, Weather, Processes, Logs |
| 2 | [launcher_vicinae.md](launcher_vicinae.md) | P2 | foundation, shell_platform | §3.3 — Launcher / Vicinae |
| 3 | [vpn_service.md](vpn_service.md) | P1 | p0, foundation | §2.7 — VPN profiles and connect |
| 4 | [calendar_sync_todos.md](calendar_sync_todos.md) | P2–P3 | control_center_depth, shell_platform | §2.23 CalDAV/ICS, §3.6 Todos |
| 5 | [vault_p3_panels.md](vault_p3_panels.md) | P3 | foundation | §3.7, §2.22, §3.9–3.11, §3.14 stubs |
| 6 | [offensive_security.md](offensive_security.md) | P4 | control_center_hardening | §2.17 — `offensive-security` feature flag |

Run the fast gate after each slice: `./scripts/sidecar-test-fast.sh`.

**Note:** Stabilize flaky `calendar_storage_test` / `automation_storage_test` debounce timing when touching calendar/automation.

## In progress

| Plan | Agent focus |
|------|-------------|
| [control_center_hardening.md](control_center_hardening.md) | Packages, Security defensive, Performance, Weather, Processes, Logs |
| [launcher_vicinae.md](launcher_vicinae.md) | `launcher.rs`, Vicinae bridge |
| [vpn_service.md](vpn_service.md) | `vpn.rs` profiles and status |

## Test layout (all plans)

| Pattern | Use for |
|---------|---------|
| `tests/{service}_rpc_shapes.rs` | Read-only `call_method` / `call_rpc` + JSON field asserts |
| `tests/{service}_storage_test.rs` | Mutations via `setup_temp_storage_db()` + `call_method_unchecked` where needed |
| `tests/integration_test.rs` | `P0_METHODS`, `READONLY_GAP_FAST` / `READONLY_GAP_SLOW_HOST` |
| `tests/integration_contracts.rs` | CLI output fixtures (no registry) |
| `tests/hyprland_internal_test.rs` | Allowlist / event-line parsers (no compositor) |
| `tests/server_http_test.rs` | Ephemeral HTTP + `/ws` push (`127.0.0.1:0` only) |
| `#[cfg(test)]` in `src/services/*.rs` | Parsers, argv builders, debouncers |

Do **not** add bucket-named integration crates. Extend the domain file that owns the behavior.

Mutating RPCs stay on `tests/common/mod.rs` `DENIED_EXACT` / `DENIED_PREFIXES` unless documented under `#[ignore]` with isolation.

## Subagent workflow

Implementation subagents **must not** `git commit`. The parent coordinator commits scoped files after each plan completes.
