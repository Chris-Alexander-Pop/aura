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

## Install (this machine / clone)

AGS loads config from `~/.config/ags`. Clone or symlink the repo there, or set `AURA_SIDECAR` to the built binary.

```bash
# deps (Arch): ags, bun, rust, hyprland, webkitgtk-6.0, …
# Astal typelibs: yay -S libastal-tray-git libastal-mpris-git

./scripts/aura-hypr-link.sh
cd sidecar && cargo build --release
cd ../ui && bun run build
./aura
```

`./aura` builds the sidecar, sets `AURA_SIDECAR` for that session, builds `ui/`, and runs Tailwind + `ags run app.ts`. It does **not** kill other processes. `./aura --kill` only stops a previous `./aura --bg` session (PIDs in `/tmp/aura-dev.pid`).

Frontend only (needs an already-built sidecar on the resolver path):

```bash
bun run watch
```

Hypr host overrides: copy `hypr/hyprland/monitors-local.lua.example` and `hypr/hyprland/execs-local.lua.example` (gitignored).

## Sidecar

1. `AURA_SIDECAR` — absolute path  
2. `$XDG_CONFIG_HOME/ags/sidecar/target/{release,debug}/ags-sidecar`

HTTP is POST-only with `X-Aura-Token` from `GET /api/meta`. Details: [sidecar/README.md](sidecar/README.md).

## License

[MIT](LICENSE). Aura is an original AGS / React / Rust implementation. The control-center layout and hover-flyout feel take after [Caelestia](https://github.com/caelestia-dots/shell); the color tokens are [Catppuccin Mocha](https://github.com/catppuccin/catppuccin) (also MIT). Early `docs/MIGRATION_STRATEGY.md` notes are historical planning, not a source import.
