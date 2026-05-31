# Session, lock, and sleep

**Status:** Planned  
**Depends on:** [shell_platform.md](shell_platform.md) (`Session.Lock` → hyprlock)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.9, §3.8, §2.13

---

## Goal

Complete **shell session** RPCs beyond lock:

1. `Session.Logout` / `Suspend` / `Reboot` / `PowerOff` via `loginctl` + polkit.
2. `Aura.ToggleWindow` — document AGS IPC contract (delegate, no GTK in sidecar).
3. `Apps.Launch` allowlist (share with launcher plan).
4. **`lock.rs`** — `Lock.GetConfig` / `SetConfig`, `Lock.TestFingerprint`, `Sleep.Inhibit` / `GetInhibitors`.
5. **GameMode** — `GameMode.GetStatus` client alignment (use `IsEnabled`; no new alias RPC per ADR).

---

## Non-goals

| Item | Reason |
|------|--------|
| Greeter/PAM changes | Operational docs only (§4.1) |
| Hibernate by default | Explicit opt-in |

---

## Test strategy

| RPC | Harness |
|-----|---------|
| `Session.*` | All on `DENIED_PREFIXES` / `DENIED_EXACT` — never in fast integration |
| Lock config | **New** `lock_storage_test.rs` if persisted; or extend `settings` namespace |
| Sleep inhibitors | `shell_rpc_shapes.rs` **extend** — `Sleep.GetInhibitors` readonly |
| GameMode | `gamemode_rpc_shapes.rs` — `IsEnabled` shape |
| Unit | `resolve_lock_command`, `resolve_session_action` in `shell.rs` `#[cfg(test)]` |

Optional `#[ignore]` test: `Session.Lock` with `hyprlock` dry-run mock (argv only).

---

## Vertical slices

### Slice A — Session actions + polkit — **P1**

Structured errors; deny-list unchanged.

### Slice B — Lock/sleep RPCs — **P2**

logind inhibitors; fprintd test RPC.

### Slice C — UI wiring — **P2**

Power flyout, lock panel roadmap.

---

## Suggested order

A → B → C

---

## Acceptance

- `shell_rpc_shapes.rs` covers new readonly methods  
- Default `cargo test` never suspends or reboots host  
- Manual matrix row for lock + suspend once
