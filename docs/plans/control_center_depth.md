# Control Center depth (DevOps, Automation, Productivity, Calendar)

**Status:** Implemented (2026-05-29)  
**Depends on:** [control_center_foundation.md](control_center_foundation.md), P0 contract + test harness  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §7 steps 7–8

## Delivered

### Automation
- `Automation.ListRules` / `Automation.Trigger` RPCs
- Safe action runner (allowlist, 30s timeout), `automation_runs.jsonl` history
- `DeleteWorkflow` via storage; UI create/run/enable/delete in `AutomationsPane.tsx`

### Productivity
- `Productivity.GetTasks` / `DeleteTask` via SQLite; `GetStats` includes task counts
- `Productivity.TimerTick` WebSocket (debounced); focus mode prefs
- UI tasks + focus toggle in `ProductivityPane.tsx`

### DevOps
- Podman-first `list_containers_preferred()`; `AURA_ALLOW_DOCKER=1` for Docker fallback
- `DevOps.GetStatus`: `container_runtime`, `tool_missing`, `git_dirty_count`, `AURA_GIT_ROOTS`
- Read-only `DevopsPane.tsx` with container list

### Calendar
- `Calendar.DeleteEvent`, `Calendar.GetUpcomingEvents`, `reminder_minutes` on create
- 60s reminder tick → `notifications::record_notification` (spawned from `main.rs`)
- UI quick-add + delete in `CalendarNavPane.tsx`

### Tests
- `tests/automation_storage_test.rs`, `productivity_storage_test.rs`, `calendar_storage_test.rs`, `automation_rpc_shapes.rs`, `productivity_rpc_shapes.rs`, `calendar_rpc_shapes.rs`

## Verification

```bash
cd sidecar && cargo test
./scripts/sidecar-test-fast.sh
./scripts/check-api-rpc-contract.sh
```
