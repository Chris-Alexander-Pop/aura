# Integrations phase 2 — CalDAV, Vicinae, Vault RW

**Status:** Planned  
**Depends on:** [calendar_sync_todos.md](../calendar_sync_todos.md), [launcher_vicinae.md](../launcher_vicinae.md), [vault_p3_panels.md](../vault_p3_panels.md)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §2.23, §3.3, §3.7, §7 item 4

---

## Goal

Move **deferred product integrations** from read-only / stub to usable depth, in **separate PRs** per integration.

---

## Track A — CalDAV / ICS

**Current:** `Ics.Import`, local calendar storage, optional file path.

| Step | Work |
|------|------|
| A1 | `Calendar.SyncCalDav` — read-only pull from URL (env or SQLite config) |
| A2 | Basic auth + token in keyring (`utils/keyring.rs`) |
| A3 | Conflict policy: last-write-wins documented |
| A4 | Fixtures: minimal CalDAV PROPFIND/REPORT responses in `tests/fixtures/caldav/` |
| A5 | `#[ignore]` integration with test server |

**Defer:** Full two-way sync, recurrence expansion edge cases.

**BACKEND_TODO:** §2.23 CalDAV bullets.

---

## Track B — Vicinae launcher

**Current:** `Launcher.*` + `launcher.json` pins; no socket.

| Step | Work |
|------|------|
| B1 | Document Vicinae protocol in `docs/integrations/vicinae.md` |
| B2 | `Launcher.VicinaeQuery` — Unix socket or HTTP per upstream |
| B3 | Fallback to `Launcher.Query` when socket absent |
| B4 | GTK/React: optional “Vicinae mode” in launcher host |

**BACKEND_TODO:** §3.3 Vicinae socket, UI.

---

## Track C — Vault read-write

**Current:** `Vault.List`, `Vault.BackupStatus` read-only.

| Step | Work |
|------|------|
| C1 | `Vault.GetEntry` / `Vault.SetEntry` — keyring-backed, never log values |
| C2 | `Vault.Export` / `Vault.Import` — user-initiated, path allowlist |
| C3 | Audit log table (who/when/method, not secret) |
| C4 | React Vault pane: list + unlock flow |
| C5 | `vault_storage_test.rs` with temp keyring mock |

**Security:** Deny-list vault export in fast tests; manual matrix only.

**BACKEND_TODO:** §3.7 transfer, RW, UI.

---

## Test strategy

- Each track: dedicated `*_rpc_shapes.rs` extensions + storage tests
- No real network in `sidecar-test-fast.sh`

---

## Acceptance

- Each track mergeable independently
- BACKEND_TODO §2.23 / §3.3 / §3.7: core items `[x]` or `[~]` with ADR

---

## Suggested order

C1–C3 (vault backend) → [client_sync_api.md](client_sync_api.md) C4 → A1–A3 → B1–B2
