#!/usr/bin/env bash
# Aura hyprlock wrapper (install → ~/.local/bin/hyprlock via aura-hypr-link.sh).
#
# Background is a static image (hypr/hyprlock-bg.png) set in hyprlock.conf —
# hyprlock's own `screenshot` blur is broken on this hybrid-GPU laptop.
# Lock surfaces are created on every output (laptop + HP HDMI). Patched
# hyprlock presents those frames via wl_shm so the Quadro HDMI does not
# need a working wl_egl_window.
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

# Prefer a just-built user binary so lock patches do not wait on sudo pacman -U.
HYPRLOCK_BIN=/usr/bin/hyprlock
if [[ -x "${HOME}/.local/libexec/hyprlock" ]]; then
  HYPRLOCK_BIN="${HOME}/.local/libexec/hyprlock"
fi

hyprlock_running() {
  pidof -q /usr/bin/hyprlock && return 0
  [[ "$HYPRLOCK_BIN" != /usr/bin/hyprlock ]] && pidof -q "$HYPRLOCK_BIN"
}

# Real binary only — never match this wrapper script.
if hyprlock_running; then
  exit 0
fi

# Waydroid surfaceflinger used to crash-loop under session-lock / DPMS and flood
# TTYs. Soft-park the user session for the lock; restore afterward.
waydroid_was_running=0
if systemctl --user is-active --quiet waydroid-session.service 2>/dev/null; then
  waydroid_was_running=1
  timeout 3 systemctl --user stop waydroid-session.service >/dev/null 2>&1 || true
elif command -v waydroid >/dev/null 2>&1; then
  if waydroid status 2>/dev/null | grep -q 'Session:[[:space:]]*RUNNING'; then
    waydroid_was_running=1
    timeout 3 waydroid session stop >/dev/null 2>&1 || true
  fi
fi

# Hyprland renders on the Quadro. Mesa Wayland EGL cannot init against those
# NVIDIA fds. NVIDIA wayland-egl window surfaces then fail swap on the Intel
# eDP (incomplete FBO / EGL_BAD_SURFACE). Force Mesa GBM on the iGPU and let
# patched hyprlock present via wl_shm on every output.
unset __VK_LAYER_NV_optimus
unset GBM_BACKEND
unset __NV_PRIME_RENDER_OFFLOAD
unset HYPRLOCK_SKIP_OUTPUTS
export __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json
export __GLX_VENDOR_LIBRARY_NAME=mesa
export GALLIUM_DRIVER="${GALLIUM_DRIVER:-iris}"
if [[ -e /dev/dri/by-path/pci-0000:00:02.0-render ]]; then
  export HYPRLOCK_GBM_DEVICE=/dev/dri/by-path/pci-0000:00:02.0-render
else
  export HYPRLOCK_GBM_DEVICE=/dev/dri/renderD128
fi

script="$HOME/.config/hypr/scripts/graceful-external-displays-off.sh"
# Do not --lock HDMI. Disabling the HP dropped the seat (password field
# gone / keys ignored) and skipped a lock surface, so Hyprland waited
# forever and the fail UI was only on the laptop panel.

touch "$WANTED_FILE"
status=0
attempts=0
max_attempts=4

HYPRLOCK_LOG="$LOCK_DIR/hyprlock.log"

set +e
while (( attempts < max_attempts )); do
  if hyprlock_running; then
    status=0
    break
  fi
  "$HYPRLOCK_BIN" "$@" >"$HYPRLOCK_LOG" 2>&1
  status=$?
  # Clean unlock / normal exit — do not relaunch.
  if [[ "$status" -eq 0 ]]; then
    break
  fi
  attempts=$((attempts + 1))
  logger -t aura-hyprlock "hyprlock exited status=$status; restore attempt ${attempts}/${max_attempts}; log=$HYPRLOCK_LOG"
  sleep 0.6
done
set -e

if [[ "$waydroid_was_running" -eq 1 ]]; then
  if systemctl --user cat waydroid-session.service >/dev/null 2>&1; then
    systemctl --user start waydroid-session.service >/dev/null 2>&1 || true
  else
    (sleep 2 && waydroid session start >/dev/null 2>&1) &
  fi
fi

# Keep the wanted marker only while hyprlock is actually up. After a clean
# unlock or a failed launch, drop it. If an older wrapper left HDMI disabled,
# put it back.
if hyprlock_running; then
  :
else
  rm -f "$WANTED_FILE"
  [[ -x "$script" ]] && "$script" --restore || true
fi

exit "$status"
