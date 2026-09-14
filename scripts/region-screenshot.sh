#!/usr/bin/env bash
# Region screenshot → clipboard (grim + slurp + wl-copy). Super+Shift+S.
# One slurp at a time. A second press kills the stuck picker instead of no-op.
set -euo pipefail

LOCK_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/aura"
LOCK_FILE="$LOCK_DIR/region-screenshot.lock"
mkdir -p "$LOCK_DIR"

exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  killall -q slurp 2>/dev/null || true
  flock -w 2 9 || exit 0
fi

area=$(slurp) || exit 0
# Do not hold the lock through grim. A hung screencopy used to block every later shot.
flock -u 9

grim -g "$area" - | wl-copy --type image/png
notify-send -u low Screenshot "Copied to clipboard"
