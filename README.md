# Aura

Arch Linux + Hyprland desktop shell: AGS (GTK4) chrome, React panels in WebKit, and a Rust sidecar (`ags-sidecar`).

This is **not** a multi-distro desktop environment. It assumes Hyprland 0.55+ (Lua config), NetworkManager, PipeWire, and GNOME Keyring. The sidecar HTTP API on `127.0.0.1:9080` can lock, suspend, and otherwise control the session — see [SECURITY.md](SECURITY.md).

## Layout

| Layer | Path | What it is |
|--------|------|------------|
| GTK / AGS | `app.ts`, `src/widget/` | Layer-shell hosts, OSD, `ags msg`. Default **bar is React in WebKit**; GTK bar only if `AURA_GTK_BAR=1` |
| React UI | `ui/` | Control center, calendar, dropdown, bar strip — built to `ui/dist` |
| Sidecar | `sidecar/` | JSON-RPC over stdio (GTK) and POST HTTP (UI) |
| Hyprland | `hypr/` | Canonical compositor Lua (linked into `~/.config/hypr`) |

## Install

AGS loads `~/.config/ags`. From any clone:

```bash
# deps (Arch): ags, bun, rust, hyprland, webkitgtk-6.0, …
# Astal typelibs: yay -S libastal-tray-git libastal-mpris-git

./setup
hyprctl reload
./aura
```

`./setup` symlinks this repo to `~/.config/ags` if needed, seeds gitignored `hypr/hyprland/*-local.lua` from the examples, builds sidecar + UI, and runs `scripts/aura-hypr-link.sh`. Use `./setup --link-only` to skip the build.

`./aura` rebuilds as needed, sets `AURA_SIDECAR` for that session, and runs Tailwind + `ags run app.ts`. It does **not** kill other processes. `./aura --kill` only stops a previous `./aura --bg` session (PIDs in `/tmp/aura-dev.pid`).

Frontend only (sidecar already on the resolver path):

```bash
bun run watch
```

Host Hyprland overlays (gitignored): `execs-local.lua`, `monitors-local.lua`, `user-local.lua`, `env-local.lua` — copy from the `*.example` files.

## Sidecar

1. `AURA_SIDECAR` — absolute path  
2. `$AURA_DIR/sidecar/target/{release,debug}/ags-sidecar` (defaults with `~/.config/ags` after `./setup`)

HTTP is POST-only with `X-Aura-Token` from `GET /api/meta`. Details: [sidecar/README.md](sidecar/README.md).

## License

[MIT](LICENSE). Color tokens are [Catppuccin Mocha](https://github.com/catppuccin/catppuccin) (also MIT).
