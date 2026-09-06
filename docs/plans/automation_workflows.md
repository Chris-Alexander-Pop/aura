# Automation workflows and triggers

**Status:** Planned  
**Depends on:** [shell_platform.md](shell_platform.md) (SQLite, storage delete), [foundation_contracts.md](foundation_contracts.md)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.21, §6.2 Automations pane

---

## Goal

Make **Automation** a real persistence + execution layer, not an empty `GetWorkflows` stub:

1. List/create/update/delete workflows in SQLite (`automation` namespace).
2. `Automation.ListRules` / `Automation.Trigger` already alias list/run — keep ADR (no extra alias RPCs).
3. Safe script runner (timeout, argv allowlist) and optional cron/file-watch triggers (localhost webhook only).
4. Control Center `AutomationsPane` reflects CRUD and run history.

---

## Non-goals

| Item | Reason |
|------|--------|
| Embedded n8n | Webhook URLs only (ADR) |
| Arbitrary shell from UI | Allowlisted actions only |
| Communication bridges | Out of scope (ADR) |

---

## Vertical slices

### Slice A — SQLite workflow CRUD — **P2**

| Task | Details |
|------|---------|
| A.1 | `GetWorkflows` / `ListRules` → `scan_namespace` + typed `Workflow` DTO |
| A.2 | `DeleteWorkflow`, `UpdateWorkflow`, `EnableWorkflow` / `DisableWorkflow` use `delete_kv` / `set` |
| A.3 | JSON schema validation on write |

**Tests:**

| File | What |
|------|------|
| `automation_storage_test.rs` | **Extend** — list after create, delete removes key, enable flag round-trip (`setup_temp_storage_db`) |
| `automation_rpc_shapes.rs` | **Extend** — `GetWorkflows` returns array; `ListRules` same shape |
| `#[cfg(test)]` in `automation.rs` | Schema validation rejects bad JSON |

Mutating RPCs (`CreateWorkflow`, `RunWorkflow`, `RunScript`, …) stay on `DENIED_EXACT`; storage tests use `call_method_unchecked`.

---

### Slice B — Trigger engine — **P2**

| Task | Details |
|------|---------|
| B.1 | Cron scheduler (in-process, single-threaded tick) |
| B.2 | `Automation.Trigger` → `RunWorkflow` with run log append to `automation_runs.jsonl` |
| B.3 | Localhost webhook ingress (127.0.0.1 only, shared secret env) |

**Tests:**

| File | What |
|------|------|
| `#[cfg(test)]` in `automation.rs` | Cron expression next-fire calculation |
| `automation_storage_test.rs` | Trigger increments run count in temp DB |

No default integration test calls `RunWorkflow` on host.

---

### Slice C — UI and client sync — **P2**

| Task | Details |
|------|---------|
| C.1 | `api.ts` / `api-types.ts` — workflow DTOs match Rust |
| C.2 | `AutomationsPane.tsx` — list from sidecar, not hardcoded empty |

**Tests:** `bun run tsc --noEmit` in `ui/`.

---

## Suggested order

A → B → C (one PR per slice when possible).

---

## Acceptance

- Create workflow in UI → survives sidecar restart (temp DB test proves storage path)  
- `./scripts/sidecar-test-fast.sh` green  
- `Automation.GetWorkflows` returns persisted rows in `automation_storage_test.rs`
