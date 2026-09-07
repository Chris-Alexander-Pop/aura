#!/usr/bin/env bash
# Install this clone as the Aura AGS config (~/.config/ags), seed host Lua
# overlays, optionally build sidecar + UI, and link Hyprland XDG paths.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
XDG_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
AGS_DEST="$XDG_CONFIG/ags"

DO_BUILD=true
LINK_ONLY=false

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

  Point AGS at this clone via ~/.config/ags (symlink if needed), copy missing
  *-local.lua examples, install JS deps, build the sidecar and UI, then run
  aura-hypr-link.sh.

Options:
  --link-only   Symlink + Hyprland XDG links only (no bun/cargo)
  --no-build    Skip sidecar / UI / bun install (still seed local Lua)
  --help, -h
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --link-only) LINK_ONLY=true; DO_BUILD=false; shift ;;
    --no-build)  DO_BUILD=false; shift ;;
    --help|-h)   usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

same_tree() {
  local a b
  a="$(readlink -f "$1")"
  b="$(readlink -f "$2")"
  [[ -n "$a" && -n "$b" && "$a" == "$b" ]]
}

link_ags() {
  mkdir -p "$(dirname "$AGS_DEST")"
  if [[ -e "$AGS_DEST" || -L "$AGS_DEST" ]]; then
    if same_tree "$AGS_DEST" "$REPO_ROOT"; then
      echo "ok  $AGS_DEST is this clone"
      return 0
    fi
    echo "error: $AGS_DEST already exists and is not this repository." >&2
    echo "       $(readlink -f "$AGS_DEST" 2>/dev/null || echo "$AGS_DEST")" >&2
    echo "Move it aside and re-run:  mv $AGS_DEST ${AGS_DEST}.bak" >&2
    exit 1
  fi
  ln -sfn "$REPO_ROOT" "$AGS_DEST"
  echo "ok  $AGS_DEST -> $REPO_ROOT"
}

seed_local() {
  local example="$1"
  local dest="$2"
  if [[ -e "$dest" ]]; then
    return 0
  fi
  if [[ -f "$example" ]]; then
    cp "$example" "$dest"
    echo "ok  seeded $dest"
  fi
}

link_ags

HYPR="$REPO_ROOT/hypr/hyprland"
seed_local "$HYPR/execs-local.lua.example" "$HYPR/execs-local.lua"
seed_local "$HYPR/monitors-local.lua.example" "$HYPR/monitors-local.lua"
seed_local "$HYPR/user-local.lua.example" "$HYPR/user-local.lua"
seed_local "$HYPR/env-local.lua.example" "$HYPR/env-local.lua"

export AURA_DIR="$AGS_DEST"
"$REPO_ROOT/scripts/aura-hypr-link.sh"

if [[ "$LINK_ONLY" == true ]]; then
  echo
  echo "Linked. Edit gitignored hypr/hyprland/*-local.lua, then: hyprctl reload"
  echo "Dev session: $AGS_DEST/aura"
  exit 0
fi

if [[ "$DO_BUILD" == true ]]; then
  if command -v bun >/dev/null 2>&1; then
    (cd "$REPO_ROOT" && bun install)
    (cd "$REPO_ROOT/ui" && bun install && bun run build)
  elif command -v npm >/dev/null 2>&1; then
    (cd "$REPO_ROOT" && npm install)
    (cd "$REPO_ROOT/ui" && npm install && npm run build)
  else
    echo "error: bun or npm required for the UI build (or pass --no-build)" >&2
    exit 1
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found (or pass --no-build)" >&2
    exit 1
  fi
  (cd "$REPO_ROOT/sidecar" && cargo build --release)
fi

echo
echo "Aura is installed at $AGS_DEST"
echo "  compositor:  hyprctl reload"
echo "  shell:       $AGS_DEST/aura"
echo "  host Lua:    hypr/hyprland/*-local.lua (gitignored)"
