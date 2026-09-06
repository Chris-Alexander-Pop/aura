#!/usr/bin/env bash
# Region screenshot → clipboard (grim + slurp + wl-copy). Super+Shift+S.
set -euo pipefail
area=$(slurp) || exit 0
grim -g "$area" - | wl-copy --type image/png
notify-send -u low Screenshot "Copied to clipboard"
