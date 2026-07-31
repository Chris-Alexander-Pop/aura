#!/usr/bin/env bash
# Aura hyprlock wrapper (install → ~/.local/bin/hyprlock via aura-hypr-link.sh).
#
# Background is a static image (hypr/hyprlock-bg.png) set in hyprlock.conf —
# hyprlock's own `screenshot` blur is broken on this hybrid-GPU laptop.
# Turn off external displays before locking so HDMI does not show garbage.
#
# Fingerprint empty-Enter restart comes from hyprlock-patched:
#   https://github.com/Chris-Alexander-Pop/hyprlock/tree/patched
#   ~/.local/share/pkgbuilds/hyprlock-patched/update.sh
#
# Multi-instance note: concurrent hyprlock processes fight over ext-session-lock
# and abort (SIGABRT → coredump storm on the console). Keep a flock for the
# whole lock session (do not exec — that would drop the flock).
#
# Crash restore: if hyprlock dies while the compositor still holds the session
# lock (allow_session_lock_restore=true), relaunch a few times so tty1 is not
# left with a dead lock surface. A user timer (hyprlock-watchdog) covers the
# case where this wrapper itself is gone.
set -euo pipefail

LOCK_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/aura"
LOCK_FILE="$LOCK_DIR/hyprlock.lock"
WANTED_FILE="$LOCK_DIR/session-lock-wanted"
mkdir -p "$LOCK_DIR"

exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  exit 0
fi

# Real binary only — never match this wrapper script.
if pidof -q /usr/bin/hyprlock; then
  exit 0
fi

# Waydroid surfaceflinger used to crash-loop under session-lock / DPMS and flood
# TTYs. Soft-park the user session for the lock; restore afterward.
waydroid_was_running=0
if systemctl --user is-active --quiet waydroid-session.service 2>/dev/null; then
  waydroid_was_running=1
  systemctl --user stop waydroid-session.service >/dev/null 2>&1 || true
elif command -v waydroid >/dev/null 2>&1; then
  if waydroid status 2>/dev/null | grep -q 'Session:[[:space:]]*RUNNING'; then
    waydroid_was_running=1
    waydroid session stop >/dev/null 2>&1 || true
  fi
fi

script="$HOME/.config/hypr/scripts/graceful-external-displays-off.sh"
[[ -x "$script" ]] && "$script" --dpms || true

touch "$WANTED_FILE"
status=0
attempts=0
max_attempts=4

set +e
while (( attempts < max_attempts )); do
  if pidof -q /usr/bin/hyprlock; then
    status=0
    break
  fi
  /usr/bin/hyprlock "$@"
  status=$?
  # Clean unlock / normal exit — do not relaunch.
  if [[ "$status" -eq 0 ]]; then
    break
  fi
  attempts=$((attempts + 1))
  logger -t aura-hyprlock "hyprlock exited status=$status; restore attempt ${attempts}/${max_attempts}"
  sleep 0.6
done
set -e

# Keep the wanted marker on crash so hyprlock-watchdog can restore after the
# wrapper exits. Clear only on a clean unlock.
if [[ "$status" -eq 0 ]]; then
  rm -f "$WANTED_FILE"
fi

if [[ "$waydroid_was_running" -eq 1 ]]; then
  if systemctl --user cat waydroid-session.service >/dev/null 2>&1; then
    systemctl --user start waydroid-session.service >/dev/null 2>&1 || true
  else
    (sleep 2 && waydroid session start >/dev/null 2>&1) &
  fi
fi

exit "$status"
