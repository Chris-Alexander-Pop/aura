#!/usr/bin/env bash
# Clipboard history picker (fuzzel + cliphist).
set -euo pipefail

mode="${1:-}"

pick() {
  local selected
  selected="$(cliphist list | fuzzel --dmenu --prompt "Clipboard" || true)"
  [[ -z "$selected" ]] && exit 0
  cliphist decode <<<"$selected" | wl-copy
}

case "$mode" in
  -d|--delete)
    selected="$(cliphist list | fuzzel --dmenu --prompt "Delete entry" || true)"
    [[ -z "$selected" ]] && exit 0
    cliphist delete <<<"$selected"
    ;;
  *)
    pick
    ;;
esac
