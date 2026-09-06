# BACKEND_TODO completion plans

Plans to close remaining items in [BACKEND_TODO.md](../../BACKEND_TODO.md) after the initial [plans program](../README.md) (15 slices, done 2026-05-31).

**Legend:** `[x]` / `[~]` / `[ ]` in BACKEND_TODO — update checkboxes when each plan lands.

**Subagents:** Do not `git commit`; parent commits scoped files per plan.

## Suggested order

| Order | Plan | BACKEND_TODO | Est. |
|------:|------|--------------|------|
| 1 | [test_harness_green.md](test_harness_green.md) | §0.7 flakes, §6 fast gate | Small |
| 2 | [client_sync_api.md](client_sync_api.md) | §5, §3.3–3.7 UI, §2.25 | Medium |
| 3 | [process_platform_harness.md](process_platform_harness.md) | §0.2–0.4, §0.5 logging, §0.6 live WS | Medium |
| 4 | [p0_network_audio_depth.md](p0_network_audio_depth.md) | §2.1–2.4 open P0 | Large |
| 5 | [control_center_polish.md](control_center_polish.md) | §2.12–2.21 polish, §2.16 FollowLogs | Medium |
| 6 | [integrations_phase2.md](integrations_phase2.md) | §2.23 CalDAV, §3.3 Vicinae, §3.7 Vault RW | Large (split PRs) |
| 7 | [greenfield_stubs_platform.md](greenfield_stubs_platform.md) | §3.9–3.11, §3.14, §4 | Small (mostly docs) |

## Definition of “BACKEND_TODO complete”

Not every `[ ]` must become `[x]`. Treat as **complete** when:

1. **P0–P2 product paths** — CC + bar + daily shell work with `api.ts` and green `./scripts/sidecar-test-fast.sh`
2. **Explicit deferrals** — CalDAV, Communication hub depth, IDE/voice/exocortex remain `[~]` with ADR + roadmap link

Roadmap-only UI (sidebar task manager, lock panel chrome) stays `[ ]` in BACKEND_TODO but is **out of sidecar scope**.

## Test layout

Same as [sidecar/tests/README.md](../../../sidecar/tests/README.md): `*_rpc_shapes.rs`, `*_storage_test.rs`, `integration_test.rs`, `server_http_test.rs`, deny-list in `common/mod.rs`.
