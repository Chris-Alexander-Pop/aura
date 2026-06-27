#!/usr/bin/env bash
# Restart Aura shell (replaces qs -c caelestia kill / caelestia shell -d).
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"

killall ags gjs ags-sidecar 2>/dev/null || true
sleep 0.4
exec "$AURA_DIR/scripts/aura-session.sh"
