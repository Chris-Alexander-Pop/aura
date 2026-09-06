#!/usr/bin/env bash
# Link Aura-owned Hypr ecosystem configs from ~/.config/ags/hypr/ into XDG paths
# that hyprpolkitagent, hyprtoolkit, hyprlock, and Qt Quick style read by default.
#
# Compositor Lua config is also Aura-owned:
#   ~/.config/ags/hypr/hyprland.lua + hyprland/*.lua
#   ~/.config/hypr/hyprland.lua is a thin package.path stub (created/verified here).
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
HYPR_DIR="$AURA_DIR/hypr"
MODE="${1:-}"

fail=0

link_file() {
  local target="$1"
  local link="$2"
  local parent
  parent="$(dirname "$link")"
  mkdir -p "$parent"

  if [[ -L "$link" && "$(readlink -f "$link")" == "$(readlink -f "$target")" ]]; then
    return 0
  fi
  if [[ -e "$link" && ! -L "$link" ]]; then
    echo "error: $link exists and is not a symlink (move aside or merge manually)" >&2
    return 1
  fi
  ln -sfn "$target" "$link"
}

link_dir() {
  local target="$1"
  local link="$2"
  local parent
  parent="$(dirname "$link")"
  mkdir -p "$parent"

  if [[ -L "$link" && "$(readlink -f "$link")" == "$(readlink -f "$target")" ]]; then
    return 0
  fi
  if [[ -e "$link" && ! -L "$link" ]]; then
    echo "error: $link exists and is not a symlink (move aside or merge manually)" >&2
    return 1
  fi
  ln -sfn "$target" "$link"
}

check_link() {
  local target="$1"
  local link="$2"
  if [[ -L "$link" && "$(readlink -f "$link")" == "$(readlink -f "$target")" ]]; then
    echo "ok  $link -> $target"
    return 0
  fi
  echo "missing or wrong  $link (expected -> $target)" >&2
  return 1
}

ensure_xdg_lua_stub() {
  local stub="$HOME/.config/hypr/hyprland.lua"
  local aura_entry="$HYPR_DIR/hyprland.lua"
  mkdir -p "$(dirname "$stub")"
  if [[ ! -f "$aura_entry" ]]; then
    echo "error: missing $aura_entry" >&2
    return 1
  fi
  # Always refresh the stub so package.path points at Aura.
  cat >"$stub" <<'EOF'
-- XDG entrypoint only. Canonical config lives in Aura:
--   ~/.config/ags/hypr/hyprland.lua
--   ~/.config/ags/hypr/hyprland/*.lua
local home = os.getenv("HOME") or "/home/user"
local aura = home .. "/.config/ags/hypr"
package.path = table.concat({
  aura .. "/?.lua",
  aura .. "/?/init.lua",
  package.path,
}, ";")
dofile(aura .. "/hyprland.lua")
EOF
  echo "ok  wrote $stub → loads $aura_entry"
}

check_xdg_lua_stub() {
  local stub="$HOME/.config/hypr/hyprland.lua"
  local aura_entry="$HYPR_DIR/hyprland.lua"
  if [[ ! -f "$aura_entry" ]]; then
    echo "missing  $aura_entry" >&2
    return 1
  fi
  if [[ ! -f "$stub" ]]; then
    echo "missing  $stub (run without --check to create)" >&2
    return 1
  fi
  if grep -qF ".config/ags/hypr" "$stub" && grep -qF "dofile" "$stub"; then
    echo "ok  $stub loads Aura Lua config"
    return 0
  fi
  echo "warn  $stub does not look like the Aura loader stub" >&2
  return 1
}

check_hypr_lua() {
  local entry="$HYPR_DIR/hyprland.lua"
  local keybinds="$HYPR_DIR/hyprland/keybinds.lua"
  local execs="$HYPR_DIR/hyprland/execs.lua"
  if [[ -f "$entry" ]]; then
    echo "ok  Aura entry $entry"
  else
    echo "missing  $entry" >&2
    fail=1
  fi
  if [[ -f "$keybinds" ]]; then
    echo "ok  $keybinds"
  else
    echo "missing  $keybinds" >&2
    fail=1
  fi
  if [[ -f "$execs" ]]; then
    echo "ok  $execs"
  else
    echo "missing  $execs" >&2
    fail=1
  fi
  # Legacy hyprlang fragments — optional once Lua owns session/binds
  local conf="${HYPRLAND_CONFIG:-$HOME/.config/hypr/hyprland.conf}"
  if [[ -f "$conf" ]] && grep -qE '^\s*source\s*=' "$conf" 2>/dev/null; then
    echo "note  $conf still present (legacy hyprlang reference — live config is Lua)"
  fi
}

install_systemd_dropin() {
  local src="$HYPR_DIR/systemd/user/hyprpolkitagent.service.d/override.conf"
  local dropin_dir="$HOME/.config/systemd/user/hyprpolkitagent.service.d"
  if [[ ! -f "$src" ]]; then
    echo "warn  missing $src — skipping systemd drop-in" >&2
    return 0
  fi
  mkdir -p "$dropin_dir"
  ln -sfn "$src" "$dropin_dir/override.conf"
  systemctl --user daemon-reload 2>/dev/null || true
}

install_waydroid_session_unit() {
  local src="$HYPR_DIR/systemd/user/waydroid-session.service"
  local dest="$HOME/.config/systemd/user/waydroid-session.service"
  if [[ ! -f "$src" ]]; then
    echo "warn  missing $src — skipping waydroid session unit" >&2
    return 0
  fi
  mkdir -p "$(dirname "$dest")"
  ln -sfn "$src" "$dest"
  systemctl --user daemon-reload 2>/dev/null || true
  systemctl --user enable waydroid-session.service 2>/dev/null || true
  echo "ok  $dest -> $src (enabled)"
}

install_hyprlock_watchdog() {
  local unit_src="$HYPR_DIR/systemd/user/hyprlock-watchdog.service"
  local timer_src="$HYPR_DIR/systemd/user/hyprlock-watchdog.timer"
  local unit_dest="$HOME/.config/systemd/user/hyprlock-watchdog.service"
  local timer_dest="$HOME/.config/systemd/user/hyprlock-watchdog.timer"
  if [[ ! -f "$unit_src" || ! -f "$timer_src" ]]; then
    echo "warn  missing hyprlock-watchdog units — skipping" >&2
    return 0
  fi
  if [[ -f "$AURA_DIR/scripts/hyprlock-watchdog.sh" ]]; then
    chmod +x "$AURA_DIR/scripts/hyprlock-watchdog.sh"
  fi
  mkdir -p "$(dirname "$unit_dest")"
  ln -sfn "$unit_src" "$unit_dest"
  ln -sfn "$timer_src" "$timer_dest"
  systemctl --user daemon-reload 2>/dev/null || true
  systemctl --user enable --now hyprlock-watchdog.timer 2>/dev/null || true
  echo "ok  $timer_dest -> $timer_src (enabled)"
}

apply_links() {
  if [[ ! -d "$HYPR_DIR" ]]; then
    echo "error: $HYPR_DIR not found — is AURA_DIR correct?" >&2
    exit 1
  fi

  link_file "$HYPR_DIR/hyprtoolkit.conf" "$HOME/.config/hypr/hyprtoolkit.conf"
  link_file "$HYPR_DIR/application-style.conf" "$HOME/.config/hypr/application-style.conf"
  link_dir "$HYPR_DIR/hyprpolkitagent" "$HOME/.config/hyprpolkitagent"
  link_file "$HYPR_DIR/hyprlock.conf" "$HOME/.config/hypr/hyprlock.conf"
  # Wrapper: flock + HDMI blank + park Waydroid (hyprlock itself is still hyprlang).
  if [[ -f "$AURA_DIR/scripts/hyprlock.sh" ]]; then
    chmod +x "$AURA_DIR/scripts/hyprlock.sh"
    mkdir -p "$HOME/.local/bin"
    link_file "$AURA_DIR/scripts/hyprlock.sh" "$HOME/.local/bin/hyprlock"
  fi
  install_systemd_dropin
  install_waydroid_session_unit
  install_hyprlock_watchdog
  ensure_xdg_lua_stub

  echo "Linked Aura hypr configs from $HYPR_DIR"
  echo "Compositor Lua: $HYPR_DIR/hyprland.lua (via ~/.config/hypr/hyprland.lua stub)"
  echo "hyprlock: still hyprlang ($HYPR_DIR/hyprlock.conf) — not Lua"
  echo "hyprlock-watchdog.timer: restores lock UI if hyprlock dies while locked"
  echo
  echo "Then: systemctl --user restart hyprpolkitagent && pkexec true"
  echo "      hyprctl reload"
}

run_check() {
  if [[ ! -d "$HYPR_DIR" ]]; then
    echo "error: $HYPR_DIR not found" >&2
    exit 1
  fi

  check_link "$HYPR_DIR/hyprtoolkit.conf" "$HOME/.config/hypr/hyprtoolkit.conf" || fail=1
  check_link "$HYPR_DIR/application-style.conf" "$HOME/.config/hypr/application-style.conf" || fail=1
  check_link "$HYPR_DIR/hyprpolkitagent" "$HOME/.config/hyprpolkitagent" || fail=1
  check_link "$HYPR_DIR/hyprlock.conf" "$HOME/.config/hypr/hyprlock.conf" || fail=1
  if [[ -f "$AURA_DIR/scripts/hyprlock.sh" ]]; then
    check_link "$AURA_DIR/scripts/hyprlock.sh" "$HOME/.local/bin/hyprlock" || fail=1
  fi
  local wd_src="$HYPR_DIR/systemd/user/waydroid-session.service"
  local wd_dest="$HOME/.config/systemd/user/waydroid-session.service"
  if [[ -f "$wd_src" ]]; then
    if [[ -L "$wd_dest" && "$(readlink -f "$wd_dest")" == "$(readlink -f "$wd_src")" ]]; then
      echo "ok  $wd_dest -> $wd_src"
    else
      echo "missing or wrong  $wd_dest (expected -> $wd_src)" >&2
      fail=1
    fi
  fi
  local hlw_timer_src="$HYPR_DIR/systemd/user/hyprlock-watchdog.timer"
  local hlw_timer_dest="$HOME/.config/systemd/user/hyprlock-watchdog.timer"
  if [[ -f "$hlw_timer_src" ]]; then
    if [[ -L "$hlw_timer_dest" && "$(readlink -f "$hlw_timer_dest")" == "$(readlink -f "$hlw_timer_src")" ]]; then
      echo "ok  $hlw_timer_dest -> $hlw_timer_src"
    else
      echo "missing or wrong  $hlw_timer_dest (expected -> $hlw_timer_src)" >&2
      fail=1
    fi
  fi
  local dropin="$HOME/.config/systemd/user/hyprpolkitagent.service.d/override.conf"
  local dropin_src="$HYPR_DIR/systemd/user/hyprpolkitagent.service.d/override.conf"
  if [[ -L "$dropin" && "$(readlink -f "$dropin")" == "$(readlink -f "$dropin_src")" ]]; then
    echo "ok  $dropin -> $dropin_src"
  else
    echo "missing or wrong  $dropin (expected -> $dropin_src)" >&2
    fail=1
  fi
  check_xdg_lua_stub || fail=1
  check_hypr_lua

  if [[ "$fail" -ne 0 ]]; then
    exit 1
  fi
  echo "All Aura hypr links OK"
}

case "$MODE" in
  --check)
    run_check
    ;;
  --help|-h)
    echo "Usage: $0 [--check]"
    echo "  (default) Create symlinks + XDG Lua stub from ~/.config/ags/hypr/"
    echo "  --check   Verify symlinks and Aura Lua compositor entry"
    ;;
  "")
    apply_links
    ;;
  *)
    echo "Unknown option: $MODE (try --help)" >&2
    exit 1
    ;;
esac
