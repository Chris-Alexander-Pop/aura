#!/usr/bin/env bash
# Region screenshot → clipboard (grim + slurp + wl-copy). Same Super+Shift+S chord as Caelestia’s region capture.
set -euo pipefail
area=$(slurp) || exit 0
grim -g "$area" - | wl-copy --type image/png
notify-send -u low Screenshot "Copied to clipboard"
