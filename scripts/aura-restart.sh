#!/usr/bin/env bash
# Restart this Aura session (ags + sidecar only). Does not kill bun, qs, or notification daemons.
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"

pkill -f 'ags run .*/app\.ts' 2>/dev/null || true
pkill -f '/run/user/[0-9]+/ags\.js' 2>/dev/null || true
pkill -x ags-sidecar 2>/dev/null || true
systemctl --user stop aura-debug.service 2>/dev/null || true
sleep 0.4
exec "$AURA_DIR/scripts/aura-session.sh"
