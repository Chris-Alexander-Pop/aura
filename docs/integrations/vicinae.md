# Vicinae launcher integration

Aura’s launcher can query [Vicinae](https://github.com/vicinae/vicinae) when a Unix socket is available, and falls back to the built-in desktop-entry index otherwise.

## Environment

| Variable | Purpose |
|----------|---------|
| `VICINAE_SOCKET` | Absolute path to Vicinae’s Unix domain socket |

When unset or unreachable, `Launcher.VicinaeQuery` delegates to `Launcher.Query` (desktop `.desktop` index).

## RPC

### Query

- **`Launcher.VicinaeQuery`** — `{ query, limit? }` → `{ results, source: "vicinae" | "desktop" }`
- Request payload on the socket: JSON line `{"query":"…","limit":N}\n`
- Response: JSON object with a `results` array matching launcher result shape (`id`, `name`, `icon?`, …).

### Exec

- **`Vicinae.Exec`** — `{ id, source?: "vicinae" | "desktop" }` → `{ ok: true }`
- **`Launcher.Run`** — accepts optional `source`; `"vicinae"` routes to `Vicinae.Exec`.

Exec order for Vicinae items:

1. Unix socket (newline JSON), tried payloads: `{"exec":"<id>"}`, `{"launch_app":{"app_id":"<id>"}}`, `{"launchApp":{"id":"<id>"}}`
2. CLI fallback: `vicinae vicinae://launch/<id>` then `vicinae launch <id>` (allowlisted)

Desktop items always use `gtk-launch <id>`.

## UI

React launcher (`ui/src/pages/Launcher.tsx`) merges Vicinae + desktop results, shows a **vicinae** badge, and calls `Vicinae.Exec` or `Launcher.Run` by source.

## See also

- [launcher_vicinae.md](../plans/launcher_vicinae.md) (original plan)
- [BACKEND_TODO.md](../BACKEND_TODO.md) §3.3
