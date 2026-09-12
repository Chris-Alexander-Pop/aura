#!/usr/bin/env bash
# Region screenshot → clipboard (grim + slurp + wl-copy). Super+Shift+S.
# Hyprland execs this on every keypress; flock so spam cannot stack slurps.
set -euo pipefail

LOCK_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/aura"
LOCK_FILE="$LOCK_DIR/region-screenshot.lock"
mkdir -p "$LOCK_DIR"

exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  exit 0
fi

# Sidecar Capture.Screenshot can start slurp too.
pidof -q slurp && exit 0

area=$(slurp) || exit 0
grim -g "$area" - | wl-copy --type image/png
notify-send -u low Screenshot "Copied to clipboard"
