#!/usr/bin/env bash
# Full-screen screenshot → clipboard (grim + wl-copy).
set -euo pipefail
grim - | wl-copy --type image/png
notify-send -u low Screenshot "Full screen copied to clipboard"
