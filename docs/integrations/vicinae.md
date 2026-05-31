# Vicinae launcher integration

Aura’s launcher can query [Vicinae](https://github.com/vicinae/vicinae) when a Unix socket is available, and falls back to the built-in desktop-entry index otherwise.

## Environment

| Variable | Purpose |
|----------|---------|
| `VICINAE_SOCKET` | Absolute path to Vicinae’s Unix domain socket |

When unset or unreachable, `Launcher.VicinaeQuery` delegates to `Launcher.Query` (desktop `.desktop` index).

## RPC

- **`Launcher.VicinaeQuery`** — `{ query, limit? }` → `{ results, source: "vicinae" | "desktop" }`
- Request payload on the socket: JSON line `{"query":"…","limit":N}\n`
- Response: JSON object with a `results` array matching launcher result shape.

## UI

GTK/React launcher hosts may show a “Vicinae mode” when `source === "vicinae"`. No socket protocol is implemented in the default build beyond newline-delimited JSON.

## See also

- [launcher_vicinae.md](../plans/launcher_vicinae.md) (original plan)
- [BACKEND_TODO.md](../BACKEND_TODO.md) §3.3
