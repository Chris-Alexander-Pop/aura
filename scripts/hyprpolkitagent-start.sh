#!/usr/bin/env bash
# Start Aura's hyprtoolkit hyprpolkitagent when dist/ still links.
# Otherwise run Arch's Qt agent so polkit prompts still work.
#
# Login must never cmake from here. Compiling hyprtoolkit exceeds systemd's
# default TimeoutStartSec (90s). Restart=on-failure then loops cc1plus forever
# after wiping dist/.
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
BIN="$AURA_DIR/hypr/dist/libexec/hyprpolkitagent"
LIB="$AURA_DIR/hypr/dist/lib"
SYS=/usr/lib/hyprpolkitagent/hyprpolkitagent

agent_ok() {
  [[ -x "$BIN" ]] || return 1
  if ldd "$BIN" 2>/dev/null | grep -qi qt; then
    return 1
  fi
  if ldd "$BIN" 2>/dev/null | grep -q 'not found'; then
    return 1
  fi
  return 0
}

if agent_ok; then
  export LD_LIBRARY_PATH="${LIB}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  exec "$BIN" "$@"
fi

echo "warning: Aura hyprpolkitagent unavailable, using $SYS" >&2
exec "$SYS" "$@"
