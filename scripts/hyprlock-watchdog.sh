#!/usr/bin/env bash
# Relaunch hyprlock when the compositor session is locked but the lock client
# is gone (SIGABRT / dbus death / kill). Safe no-op when unlocked or already
# running. Installed as a user systemd timer by aura-hypr-link.sh.
set -euo pipefail

LOCK_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/aura"
WANTED_FILE="$LOCK_DIR/session-lock-wanted"

if pidof -q /usr/bin/hyprlock; then
  exit 0
fi

locked=0
if [[ -f "$WANTED_FILE" ]]; then
  locked=1
fi

sid="${XDG_SESSION_ID:-}"
if [[ "$locked" -eq 0 && -n "$sid" ]] && command -v loginctl >/dev/null 2>&1; then
  if [[ "$(loginctl show-session "$sid" -p LockedHint --value 2>/dev/null || true)" == "yes" ]]; then
    locked=1
  fi
fi

# Hyprland may keep the ext-session-lock without updating LockedHint after a
# client crash. Detect "inputs refused" indirectly: recent lockdead warning is
# unreliable from a timer; instead try restore whenever hypridle thinks we are
# idle-locked via the wanted marker, or when the Hyprland instance exists and
# a sticky lock flag was left by the wrapper.
if [[ "$locked" -eq 0 ]]; then
  exit 0
fi

# Prefer Aura wrapper (flock + HDMI blank + waydroid park).
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
if [[ -z "${HYPRLAND_INSTANCE_SIGNATURE:-}" && -d "$XDG_RUNTIME_DIR/hypr" ]]; then
  HYPRLAND_INSTANCE_SIGNATURE="$(ls -1 "$XDG_RUNTIME_DIR/hypr" 2>/dev/null | head -1 || true)"
  export HYPRLAND_INSTANCE_SIGNATURE
fi
if [[ -z "${WAYLAND_DISPLAY:-}" ]]; then
  export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-1}"
fi

logger -t aura-hyprlock-watchdog "lock client missing while session lock wanted — relaunching hyprlock"
exec "${HOME}/.local/bin/hyprlock"
