# Communication hub integration (external app)

Aura’s Control Center **Communication** pane does not implement a unified Beeper-like inbox in the sidecar (see [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md)). Unread counts and threading live in a **separate comm app** you build.

## What Aura provides today

| RPC | Purpose |
|-----|---------|
| `Communication.GetCommunicationApps` | Detect installed desktop clients (`telegram`, `discord`, …) |
| `Communication.LaunchApp` | `{ app_name }` → spawn client |
| `Communication.GetUnread` | Returns cached unread map (empty until imported) |
| `Communication.ImportUnread` | `{ counts: { "matrix": 3, "email": 12 } }` — your app pushes counts |

## Contract for your comm app

1. **Push unread counts** periodically via sidecar HTTP JSON-RPC:

```json
{
  "method": "Communication.ImportUnread",
  "params": { "counts": { "matrix": 2, "signal": 1 } }
}
```

2. **Optional:** expose `GET /unread` on a local HTTP port; a future Aura poller could call it. For now, **ImportUnread** is the supported hook.

3. Counts are stored in SQLite (`communication.unread_cache`) and shown in the Communication pane + any dashboard tiles you wire later.

## UI behavior

- Until `ImportUnread` is called, the pane shows **installed apps** to launch and copy explaining the external hub.
- When counts exist, KPI tiles and per-source list render from `GetUnread`.

## See also

- [communication-panel.md](../roadmap/communication-panel.md) — product vision
- [BACKEND_TODO.md](../BACKEND_TODO.md) — deferred hub scope
