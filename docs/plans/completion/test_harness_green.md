# Test harness green (fast gate)

**Status:** Planned  
**Depends on:** [foundation_contracts.md](../foundation_contracts.md)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §0.7, snapshot “Known fast-gate failures”, §7 item 1

---

## Goal

Make `./scripts/sidecar-test-fast.sh` reliably green on a typical dev machine without weakening safety rules.

---

## Failures to fix

| Test | Issue | Approach |
|------|--------|----------|
| `settings_storage_test::settings_get_defaults_without_row` | Expects `theme: "dark"`; code default `catppuccin-mocha` | Align test to `Settings` default schema in `settings.rs` OR document both as valid |
| `calendar_storage_test::calendar_create_emits_events_changed_after_debounce` | WS notify timeout | Per-test `notify` bus; increase debounce wait; or call `notify::emit` synchronously in test hook |
| `automation_storage_test::*` | SQLite disk I/O / isolation | Unique `AURA_STORAGE_DB` per test; `automation_test_lock` pattern like launcher |

---

## Vertical slices

### Slice A — Settings default assertion

- Fix [settings_storage_test.rs](../../../sidecar/tests/settings_storage_test.rs) to match [settings.rs](../../../sidecar/src/services/settings.rs) `default_settings()`.
- Add `Settings.GetSchema` test that theme enum includes shipped default.

### Slice B — Calendar WS debounce

- [calendar.rs](../../../sidecar/src/services/calendar.rs): export debounce interval or test-only flush.
- [calendar_storage_test.rs](../../../sidecar/tests/calendar_storage_test.rs): subscribe to notify before create; use test bus if needed.

### Slice C — Automation storage isolation

- Mirror `launcher.rs` mutex/env isolation in automation storage tests.
- Ensure cron tick does not run during temp-DB tests (`#[cfg(test)]` guard or disable spawn in test registry).

---

## Acceptance

- `./scripts/sidecar-test-fast.sh` exits 0 three consecutive runs locally
- Update BACKEND_TODO snapshot “Known fast-gate failures” to empty or “none known”
- No new `#[ignore]` without comment in `tests/README.md`

---

## BACKEND_TODO checkboxes

- [ ] §0.7 CI job doc — defer to [process_platform_harness.md](process_platform_harness.md)
- [x] Replace placeholder brightness smoke — already done; verify still in `integration_test.rs`
