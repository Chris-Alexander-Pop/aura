# Security

Aura’s sidecar is a **privileged local control plane**. It can lock/suspend the session, change network and packages, read the keyring vault, and write Hyprland binds. Treat it like a polkit helper that happens to speak HTTP.

## Trust model

- Bound to **loopback only** (`127.0.0.1:9080`). That is necessary, not sufficient: browsers on this machine can still talk to localhost.
- GTK talks JSON-RPC over **stdio** (no HTTP).
- The React UI is served from the same origin as the API (`http://127.0.0.1:9080`) or, in Vite, via the `/api` proxy.

## HTTP RPC rules

1. **POST only** for `/api/{method}`. GET returns `405` so a webpage cannot fire `Session.PowerOff` with `<img src=…>`.
2. **No CORS `*`. ** Cross-origin `fetch` from a random site is rejected at the preflight; if an `Origin` header is present it must be loopback (`localhost` / `127.0.0.1` / `::1`).
3. **`X-Aura-Token`** on every RPC POST. The token is generated at process start, returned from `GET /api/meta` (loopback origin only), and written to `$XDG_RUNTIME_DIR/aura-http-token` (mode `0600`) for curl. Override with `AURA_HTTP_TOKEN` if you need a stable value.
4. WebSocket `/ws` requires `?token=` (browsers cannot set the custom header on the handshake).

Example:

```bash
TOKEN=$(cat "${XDG_RUNTIME_DIR}/aura-http-token")
curl -sS -X POST \
  -H "X-Aura-Token: ${TOKEN}" \
  -H "Content-Type: application/json" \
  http://127.0.0.1:9080/api/Sidecar.GetVersion
```

## Other guards

- `Process.Kill` / `Performance.KillProcess` require a per-process confirmation token from `Process.ListTop` (not `confirm-kill-{pid}`).
- `Hyprland.Dispatch` and `Keybinds.Set` / `Import` refuse `exec`.
- OpenVPN `--auth-user-pass` files go under `$XDG_RUNTIME_DIR/aura/` with mode `0600`, not `/tmp`.
- Optional extras (`AURA_EXTENSIONS` / `~/.config/ags/extensions/*.json`) are **local code execution**. They attach to the same RPC surface; only load binaries you trust.

## Reporting

This is a personal desktop shell. If you find a way to invoke privileged RPC from a remote website without a loopback origin and token, open an issue on the Aura repo.
