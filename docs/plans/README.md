# Sidecar implementation plans

All backend plans in this directory are **implemented** (2026-05-31). Track ongoing product work in [BACKEND_TODO.md](../BACKEND_TODO.md) and [docs/roadmap/](../roadmap/).

## Completed plans

| Plan | BACKEND_TODO focus |
|------|-------------------|
| [control_center_foundation.md](control_center_foundation.md) | Notifications, Keybinds, CC aggregates |
| [control_center_depth.md](control_center_depth.md) | DevOps, Automation, Productivity, Calendar CRUD |
| [compositor_bar_live.md](compositor_bar_live.md) | Hyprland WS, dispatch allowlist, media |
| [shell_platform.md](shell_platform.md) | Settings, Capture, calendar push, Session.Lock |
| [foundation_contracts.md](foundation_contracts.md) | §0–§1 harness and contract gate |
| [automation_workflows.md](automation_workflows.md) | Workflow SQLite, cron tick |
| [session_lock_extended.md](session_lock_extended.md) | Session actions, lock/sleep |
| [p0_daily_hardening.md](p0_daily_hardening.md) | System.GetStats, brightness |
| [dashboard_aggregation.md](dashboard_aggregation.md) | Dashboard / sidebar aggregates |
| [control_center_hardening.md](control_center_hardening.md) | Packages, Security defensive, Performance |
| [launcher_vicinae.md](launcher_vicinae.md) | Launcher query/run (Vicinae deferred) |
| [vpn_service.md](vpn_service.md) | VPN profiles and connect |
| [calendar_sync_todos.md](calendar_sync_todos.md) | ICS import/export, Todos (CalDAV deferred) |
| [vault_p3_panels.md](vault_p3_panels.md) | Vault read-only, fitness storage test |
| [offensive_security.md](offensive_security.md) | `offensive-security` feature flag |

## Verification

```bash
./scripts/sidecar-test-fast.sh
cd sidecar && cargo test --features offensive-security   # optional offensive build
```

Regenerate manifest with offensive methods (rare): `AURA_OFFENSIVE_MANIFEST=1 ./scripts/generate-rpc-manifest.sh`

## Test layout

See [sidecar/tests/README.md](../../sidecar/tests/README.md). Subagents do not commit; the parent coordinator commits scoped files per plan.
