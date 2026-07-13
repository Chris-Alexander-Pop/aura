#!/usr/bin/env bash
# Restart Aura shell (replaces qs -c caelestia kill / caelestia shell -d).
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"

# Do NOT `killall gjs` — other apps (e.g. Surfshark) also use gjs.
pkill -f 'ags run .*/app\.ts' 2>/dev/null || true
pkill -f '/run/user/[0-9]+/ags\.js' 2>/dev/null || true
pkill -x ags-sidecar 2>/dev/null || true
systemctl --user stop aura-debug.service 2>/dev/null || true
sleep 0.4
exec "$AURA_DIR/scripts/aura-session.sh"
