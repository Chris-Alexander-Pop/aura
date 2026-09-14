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

# NVIDIA render + Intel eDP: GSK GL and WebKit DMA-BUF paint fully transparent
# surfaces (bar exists in hyprctl layers with alpha 0).
export GSK_RENDERER="${GSK_RENDERER:-cairo}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"

# Match the real AGS process, not `concurrently "… ags run app.ts"`.
ags_shell_running() {
  pgrep -u "$(id -u)" -x ags >/dev/null 2>&1 && return 0
  pgrep -u "$(id -u)" -f 'gjs -m /run/user/[0-9]+/ags\.js' >/dev/null 2>&1
}

WATCHDOG_PIDFILE="${XDG_RUNTIME_DIR:-/tmp}/aura-ags-watchdog.pid"
LOG_FILE="/tmp/aura-ags.log"
SUPERVISE="${AURA_AGS_SUPERVISE:-1}"

watchdog_alive() {
  local pid
  pid="$(cat "$WATCHDOG_PIDFILE" 2>/dev/null || true)"
  [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

if watchdog_alive || ags_shell_running; then
  exit 0
fi

run_ags() {
  if [[ -f "$LOG_FILE" ]]; then
    local sz
    sz=$(wc -c < "$LOG_FILE" 2>/dev/null || echo 0)
    if (( sz > 2000000 )); then
      : > "$LOG_FILE"
    fi
  fi
  echo "=== aura ags $(date -Iseconds) ===" >> "$LOG_FILE"
  env AURA_SIDECAR="$AURA_SIDECAR" \
    GSK_RENDERER="$GSK_RENDERER" \
    WEBKIT_DISABLE_DMABUF_RENDERER="$WEBKIT_DISABLE_DMABUF_RENDERER" \
    WEBKIT_DISABLE_COMPOSITING_MODE="$WEBKIT_DISABLE_COMPOSITING_MODE" \
    ags run "$AURA_DIR/app.ts" >>"$LOG_FILE" 2>&1
}

if [[ "$SUPERVISE" != "1" ]]; then
  echo "=== aura ags $(date -Iseconds) ===" >> "$LOG_FILE"
  nohup env AURA_SIDECAR="$AURA_SIDECAR" \
    GSK_RENDERER="$GSK_RENDERER" \
    WEBKIT_DISABLE_DMABUF_RENDERER="$WEBKIT_DISABLE_DMABUF_RENDERER" \
    WEBKIT_DISABLE_COMPOSITING_MODE="$WEBKIT_DISABLE_COMPOSITING_MODE" \
    ags run "$AURA_DIR/app.ts" >>"$LOG_FILE" 2>&1 &
  disown -h $! 2>/dev/null || true
  exit 0
fi

# Restart after SIGSEGV on output unplug (GTK/WebKit dead GdkSurface).
# Detach from aura-session so the watchdog is not SIGHUP'd when this script exits.
watchdog_body() {
  echo "$BASHPID" > "$WATCHDOG_PIDFILE"
  trap 'rm -f "$WATCHDOG_PIDFILE"' EXIT
  trap '' HUP
  local count=0
  local window_start now
  window_start=$(date +%s)
  while true; do
    run_ags || true
    now=$(date +%s)
    if (( now - window_start > 60 )); then
      count=0
      window_start=$now
    fi
    count=$((count + 1))
    if (( count > 8 )); then
      notify-send -u critical Aura "Shell crashed repeatedly. Not restarting." 2>/dev/null || true
      exit 1
    fi
    sleep 1
  done
}

export -f run_ags watchdog_body
export AURA_DIR AURA_SIDECAR GSK_RENDERER WEBKIT_DISABLE_DMABUF_RENDERER WEBKIT_DISABLE_COMPOSITING_MODE LOG_FILE WATCHDOG_PIDFILE

if command -v setsid >/dev/null 2>&1; then
  setsid -f bash -c 'watchdog_body' </dev/null >/dev/null 2>&1
else
  nohup bash -c 'watchdog_body' </dev/null >/dev/null 2>&1 &
  disown
fi
