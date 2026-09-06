# Control center and service polish

**Status:** Implemented  
**Depends on:** [control_center_hardening.md](../control_center_hardening.md), [client_sync_api.md](client_sync_api.md) (for panes using new APIs)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §2.12–2.21, §2.16, §2.18–2.19, §7 item 6

---

## Goal

Depth and polish for **P2 control center** backends not covered in the first hardening slice.

---

## Vertical slices

### Slice A — Logs follow stream

- `Logs.FollowLogs`: WebSocket stream of journal lines (reuse server `/ws` + background journalctl task).
- Cancel on client disconnect.
- Test: [server_http_test.rs](../../../sidecar/tests/server_http_test.rs) receives ≥1 line from mock journal fixture injector.

**BACKEND_TODO:** §2.16 `FollowLogs`.

### Slice B — Weather

- [weather.rs](../../../sidecar/src/services/weather.rs): env `AURA_WEATHER_API_KEY`, in-memory cache TTL, rate limit.
- Fixture: `tests/fixtures/weather/wttr.json`.
- Prune or implement dead forecast RPCs in manifest.

**BACKEND_TODO:** §2.12.

### Slice C — Performance presets

- `Performance.ApplyPreset` → maps to Power profile + CPU governor (reuse existing RPCs).
- Presets: `meeting`, `compile`, `game` JSON in SQLite or static.

**BACKEND_TODO:** §2.18 presets.

### Slice D — DevOps depth

- Docker socket graceful errors; optional timer/cron list via `systemctl list-timers`.
- Fixtures for `docker ps` / `podman ps`.

**BACKEND_TODO:** §2.19.

### Slice E — Automation webhooks

- Localhost-only HTTP webhook in [automation.rs](../../../sidecar/src/services/automation.rs) (127.0.0.1:PORT, shared secret).
- File-watch triggers: defer or single-path with `notify` crate.

**BACKEND_TODO:** §2.21 webhook, file-watch.

### Slice F — Productivity / packages / security leftovers

- Productivity focus mode: **defer** (dangerous) unless UX sign-off.
- Packages: AUR helper gate; dry-run install `#[ignore]` test.
- Security: scan-on-install hook opt-in; port scan caps review.

### Slice G — Calendar / fitness API

- Extend `api.ts` calendar CRUD (create/update/delete) if not in [client_sync_api.md](client_sync_api.md).
- Fitness: `api.ts` goals read — minimal.

**BACKEND_TODO:** §2.25, §2.24, §2.15, §2.17 partial.

---

## Acceptance

- Each slice has at least one automated test
- Dead RPCs removed from manifest or implemented
- BACKEND_TODO P2 polish items addressed or marked `[~]`

---

## Suggested order

A → B → C → D → E → F → G
