#!/usr/bin/env bash
# Aura session autostart — Hyprland exec-once.
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"

resolve_sidecar() {
  local release="$AURA_DIR/sidecar/target/release/ags-sidecar"
  local debug="$AURA_DIR/sidecar/target/debug/ags-sidecar"
  if [[ -n "${AURA_SIDECAR:-}" && -x "$AURA_SIDECAR" ]]; then
    echo "$AURA_SIDECAR"
  elif [[ -x "$release" ]]; then
    echo "$release"
  elif [[ -x "$debug" ]]; then
    echo "$debug"
  else
    echo ""
  fi
}

ensure_built() {
  local sidecar
  sidecar="$(resolve_sidecar)"
  if [[ -n "$sidecar" && -f "$AURA_DIR/ui/dist/index.html" ]]; then
    export AURA_SIDECAR="$sidecar"
    return 0
  fi

  notify-send -u normal Aura "Building sidecar and UI (first launch)…" 2>/dev/null || true

  if [[ -z "$sidecar" ]]; then
    (cd "$AURA_DIR/sidecar" && cargo build --release)
    sidecar="$AURA_DIR/sidecar/target/release/ags-sidecar"
  fi
  export AURA_SIDECAR="$sidecar"

  if [[ ! -f "$AURA_DIR/ui/dist/index.html" ]]; then
    if command -v bun &>/dev/null; then
      (cd "$AURA_DIR/ui" && bun run build)
    else
      (cd "$AURA_DIR/ui" && npm run build)
    fi
  fi
}

cd "$AURA_DIR"

if command -v bun &>/dev/null; then
  bun run build:css >/dev/null 2>&1 || true
elif command -v npm &>/dev/null; then
  npm run build:css >/dev/null 2>&1 || true
fi

ensure_built

if pgrep -f "ags run.*app\\.ts" >/dev/null 2>&1; then
  exit 0
fi

nohup env AURA_SIDECAR="$AURA_SIDECAR" ags run "$AURA_DIR/app.ts" >/tmp/aura-ags.log 2>&1 &
disown -h $! 2>/dev/null || true
