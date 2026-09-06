# Sidecar implementation plans

The **first plans program** (15 slices below) is **implemented** (2026-05-31). Remaining BACKEND_TODO work is broken into **[completion plans](completion/README.md)** — test harness, client sync, platform harness, P0 depth, polish, integrations phase 2, greenfield ADRs. Checklist: [BACKEND_TODO.md](../BACKEND_TODO.md). Product specs: [docs/roadmap/](../roadmap/).

## Completion phase (remaining BACKEND_TODO)

| Plan | Focus |
|------|--------|
| [completion/test_harness_green.md](completion/test_harness_green.md) | Fast gate flakes (settings, calendar WS, automation DB) |
| [completion/client_sync_api.md](completion/client_sync_api.md) | `api.ts` / GTK for Launcher, Todos, Vault, Dashboard, Capture |
| [completion/process_platform_harness.md](completion/process_platform_harness.md) | Exec allowlist, polkit, RPC logging, CI matrix |
| [completion/p0_network_audio_depth.md](completion/p0_network_audio_depth.md) | Network.Connect, audio fixtures, BT optional |
| [completion/control_center_polish.md](completion/control_center_polish.md) | FollowLogs, weather cache, presets, webhooks |
| [completion/integrations_phase2.md](completion/integrations_phase2.md) | CalDAV, Vicinae socket, Vault RW |
| [completion/greenfield_stubs_platform.md](completion/greenfield_stubs_platform.md) | Communication/IDE/voice ADRs, §4 deferrals |

Suggested order: [completion/README.md](completion/README.md).

## Completed plans (first program)

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

## Verification

```bash
./scripts/sidecar-test-fast.sh
```

## Test layout

See [sidecar/tests/README.md](../../sidecar/tests/README.md). Subagents do not commit; the parent coordinator commits scoped files per plan.
