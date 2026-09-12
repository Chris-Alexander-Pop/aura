#!/usr/bin/env bash
# Restart this Aura session (ags + sidecar only). Does not kill bun, qs, or notification daemons.
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"

pkill -u "$(id -u)" -x ags 2>/dev/null || true
pkill -u "$(id -u)" -f 'gjs -m /run/user/[0-9]+/ags\.js' 2>/dev/null || true
pkill -u "$(id -u)" -x ags-sidecar 2>/dev/null || true
systemctl --user stop aura-debug.service 2>/dev/null || true
sleep 0.4
exec "$AURA_DIR/scripts/aura-session.sh"
