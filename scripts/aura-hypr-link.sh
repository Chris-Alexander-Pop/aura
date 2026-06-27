#!/usr/bin/env bash
# Link Aura-owned Hypr ecosystem configs from ~/.config/ags/hypr/ into XDG paths
# that hyprpolkitagent, hyprtoolkit, hyprlock, and Qt Quick style read by default.
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

check_hypr_sources() {
  local conf="${HYPRLAND_CONFIG:-$HOME/.config/hypr/hyprland.conf}"
  if [[ ! -f "$conf" ]]; then
    echo "warn  $conf not found — add source lines manually (see hypr/README.md)" >&2
    return 0
  fi
  local execs="$HYPR_DIR/hyprland/execs-aura.conf"
  local keybinds="$HYPR_DIR/hyprland/aura-keybinds.conf"
  if grep -qF "$execs" "$conf" || grep -qF "hypr/hyprland/execs-aura.conf" "$conf"; then
    echo "ok  hyprland.conf sources execs-aura.conf"
  elif grep -rqF "hyprpolkitagent" "$HOME/.config/hypr/hyprland/" 2>/dev/null \
    && grep -rqF "aura-session.sh" "$HOME/.config/hypr/hyprland/" 2>/dev/null; then
    echo "ok  Aura autostart present in ~/.config/hypr/hyprland/ (execs-aura.conf not sourced — OK for hybrid setups)"
  else
    echo "warn  hyprland.conf does not source execs-aura.conf and autostart not found in hyprland/" >&2
    fail=1
  fi
  if grep -qF "$keybinds" "$conf" || grep -qF "hypr/hyprland/aura-keybinds.conf" "$conf"; then
    echo "ok  hyprland.conf sources aura-keybinds.conf"
  else
    echo "warn  hyprland.conf does not source aura-keybinds.conf" >&2
    fail=1
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

apply_links() {
  if [[ ! -d "$HYPR_DIR" ]]; then
    echo "error: $HYPR_DIR not found — is AURA_DIR correct?" >&2
    exit 1
  fi

  link_file "$HYPR_DIR/hyprtoolkit.conf" "$HOME/.config/hypr/hyprtoolkit.conf"
  link_file "$HYPR_DIR/application-style.conf" "$HOME/.config/hypr/application-style.conf"
  link_dir "$HYPR_DIR/hyprpolkitagent" "$HOME/.config/hyprpolkitagent"
  link_file "$HYPR_DIR/hyprlock.conf" "$HOME/.config/hypr/hyprlock.conf"
  install_systemd_dropin

  echo "Linked Aura hypr configs from $HYPR_DIR"
  echo
  echo "Add to your live Hyprland config if not already present:"
  echo "  source = ~/.config/ags/hypr/hyprland/execs-aura.conf"
  echo "  source = ~/.config/ags/hypr/hyprland/aura-keybinds.conf"
  echo
  echo "Then: systemctl --user restart hyprpolkitagent && pkexec true"
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
  local dropin="$HOME/.config/systemd/user/hyprpolkitagent.service.d/override.conf"
  local dropin_src="$HYPR_DIR/systemd/user/hyprpolkitagent.service.d/override.conf"
  if [[ -L "$dropin" && "$(readlink -f "$dropin")" == "$(readlink -f "$dropin_src")" ]]; then
    echo "ok  $dropin -> $dropin_src"
  else
    echo "missing or wrong  $dropin (expected -> $dropin_src)" >&2
    fail=1
  fi
  check_hypr_sources

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
    echo "  (default) Create symlinks from ~/.config/ags/hypr/ to XDG paths"
    echo "  --check   Verify symlinks and hyprland.conf source lines"
    ;;
  "")
    apply_links
    ;;
  *)
    echo "Unknown option: $MODE (try --help)" >&2
    exit 1
    ;;
esac
