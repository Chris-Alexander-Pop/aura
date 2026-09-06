# Calendar sync, ICS, and Todos

**Status:** Implemented (2026-05-31, CalDAV deferred)  
**Depends on:** [control_center_depth.md](control_center_depth.md), [shell_platform.md](shell_platform.md) (`Calendar.EventsChanged`)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.23, §3.6, §2.25

---

## Goal

1. **CalDAV / Google OAuth** — `GetCalendars`, `SyncCalendars` (P3 phase; keyring tokens).
2. **ICS** — `ImportIcs` / export parsers (`quick-xml`).
3. **`todos.rs`** — CRUD, projects, due dates, NL date parse; reminders → notifications.
4. **UI** — extend `api.ts` calendar CRUD; calendar panel todo subpanel per roadmap.

---

## Non-goals

| Item | Reason |
|------|--------|
| Full recurrence engine v1 | Later; local CRUD already works |
| Reclaim/time-tracking | ADR out of scope |

---

## Test strategy

| Layer | Where |
|-------|--------|
| ICS parse | `tests/fixtures/calendar/*.ics` + `#[cfg(test)]` in `calendar.rs` |
| Local CRUD | **Extend** `calendar_storage_test.rs` (already uses temp DB) |
| Todos | **New** `todos_storage_test.rs`, `todos_rpc_shapes.rs` |
| Sync | `#[ignore]` tests with mock HTTP server or recorded CalDAV fixtures |
| Mutations | `Calendar.SyncCalendars`, `ImportIcs`, todo writes on deny-list except storage tests |

`Calendar.CreateEvent` / `UpdateEvent` / `DeleteEvent` — storage tests use `call_method_unchecked` (already pattern in `calendar_storage_test.rs`).

WS: emit `Calendar.EventsChanged` on sync completion (extend `server_http_test.rs`).

---

## Vertical slices

### Slice A — ICS import/export — **P2**

### Slice B — `todos.rs` service — **P2**

SQLite namespace `todos`; link reminders to `notifications.rs`.

### Slice C — CalDAV/Google sync — **P3**

OAuth in keyring; read-only sync first.

### Slice D — UI + api.ts — **P2**

`CalendarNavPane` + todo subpanel.

---

## Suggested order

A → B → D → C

---

## Acceptance

- ICS fixture round-trip in unit tests  
- `todos_storage_test.rs` in fast gate  
- CalDAV behind `#[ignore]` until OAuth mock exists
