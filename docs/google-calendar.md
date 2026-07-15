# Google Calendar (Aura)

Read-only sync via OAuth2 PKCE + Calendar API.

## One-time setup

1. In [Google Cloud Console](https://console.cloud.google.com/), create/select a project.
2. Enable **Google Calendar API**.
3. Create an OAuth client ID of type **Desktop app**.
4. Export the client id (and secret if shown):

```bash
export AURA_GOOGLE_OAUTH_CLIENT_ID="….apps.googleusercontent.com"
# optional for some client types:
# export AURA_GOOGLE_OAUTH_CLIENT_SECRET="…"
```

5. Restart `ags-sidecar` so it picks up the env vars (e.g. from your Hyprland/uwsm session env).

## Use

1. Open the calendar panel (bar clock / `ags msg toggle calendar`).
2. Tap **Connect** — a browser window opens for Google consent.
3. After approval, Aura stores a refresh token in the GNOME Keyring and runs an initial sync (past 30 days → next 90 days).
4. Use **Sync** anytime; **logout** disconnects and removes `gcal_*` events.

## RPC

| Method | Role |
|--------|------|
| `Calendar.GoogleStartAuth` | Start loopback OAuth (background) |
| `Calendar.GoogleAuthStatus` | `{ connected, configured, pending, email, last_sync, error }` |
| `Calendar.GoogleDisconnect` | Clear keyring + Google events |
| `Calendar.GetCalendars` | Google calendar list when connected |
| `Calendar.SyncCalendars` | Google pull if connected, else CalDAV |

Write-back of local creates to Google is out of scope for this pass.
