# Client sync — api.ts, api-types, GTK

**Status:** Planned  
**Depends on:** Sidecar RPCs already registered (Launcher, Todos, Vault, Dashboard, Capture)  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §5, §3.3–3.6, §3.13, §3.4, §7 item 2

---

## Goal

Wire **React** and **GTK** clients to sidecar namespaces that exist only in Rust today.

| Namespace | Sidecar | Target consumers |
|-----------|---------|------------------|
| `Launcher.*` | Yes | Launcher UI / future Vicinae host |
| `Todos.*` | Yes | Calendar todo subpanel |
| `Vault.*` | Yes | Vault panel (read-only first) |
| `Dashboard.*` / `Sidebar.*` | Yes | Top dropdown, sidebar tiles |
| `Capture.*` | Yes | Keybinds, CC card |

---

## Non-goals

| Item | Reason |
|------|--------|
| Vicinae socket | [integrations_phase2.md](integrations_phase2.md) |
| Full calendar CRUD in api.ts | Extend incrementally; `GetEvents` may already exist |
| Fitness panel | Low priority |

---

## Vertical slices

### Slice A — `api.ts` + `api-types.ts`

Add typed helpers mirroring existing patterns (`call` / `callData`):

```text
launcherQuery, launcherRun, launcherRecent, launcherPin
todosList, todosCreate, todosUpdate, todosDelete, todosListProjects, todosParseDueDate
vaultList, vaultBackupStatus
dashboardGetQuickStatus, sidebarGetTileData
captureScreenshot, captureRecordStart, captureRecordStop, captureListDevices
```

- Zod or manual parsers in [api-types.ts](../../../ui/src/lib/api-types.ts) for each response shape.
- Extend `scripts/check-api-rpc-contract.sh` if it only scans a fixed regex — include new methods OR document as “sidecar-only until UI lands” with separate manifest grep.

### Slice B — React UI

| Pane / surface | File |
|--------------|------|
| Dropdown | Consume `dashboardGetQuickStatus`; drop N+1 polls where possible |
| Sidebar | `sidebarGetTileData` per tile id |
| Calendar nav | Todos list + create (minimal) |
| Settings or new card | Capture screenshot trigger (readonly list devices) |
| Launcher | Optional: query box calling `launcherQuery` |

### Slice C — GTK `src/lib/sidecar.ts`

- Typed wrappers for Launcher/Capture if bar needs them.
- `connectWs` handlers: `Todos.Changed`, `Dashboard.QuickStatusChanged` (if push added).

### Slice D — Contract tests

- Add readonly methods to `api_ts_readonly_methods_resolve` in [integration_test.rs](../../../sidecar/tests/integration_test.rs) once exposed in `api.ts`.
- Run `bun run tsc --noEmit` in `ui/`.

---

## Test strategy

- Existing `*_rpc_shapes.rs` unchanged (sidecar-only).
- Optional: thin `ui` contract test script grepping `api.` vs manifest (BACKEND_TODO §1 optional item).

---

## Acceptance

- All new `api.ts` methods pass `check-api-rpc-contract.sh` or documented exclusion list
- At least one React surface calls each new namespace (can be minimal)
- BACKEND_TODO §5 gap table: all “No” → “Yes” for listed namespaces

---

## Suggested order

A → D → B (panes in parallel after types) → C
