#!/usr/bin/env bash
# Install Aura polkit PAM stack (password before fingerprint).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

resolve_aura_dir() {
  if [[ -n "${AURA_DIR:-}" && -f "${AURA_DIR}/hypr/pam/polkit-1" ]]; then
    printf '%s\n' "$AURA_DIR"
    return 0
  fi

  local repo_root
  repo_root="$(cd "$SCRIPT_DIR/.." && pwd)"
  if [[ -f "$repo_root/hypr/pam/polkit-1" ]]; then
    printf '%s\n' "$repo_root"
    return 0
  fi

  local user_home="$HOME"
  if [[ -n "${SUDO_USER:-}" ]]; then
    user_home="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
  fi
  if [[ -n "$user_home" && -f "$user_home/.config/ags/hypr/pam/polkit-1" ]]; then
    printf '%s\n' "$user_home/.config/ags"
    return 0
  fi

  return 1
}

AURA_DIR="$(resolve_aura_dir)" || {
  echo "error: could not find hypr/pam/polkit-1 (set AURA_DIR to your ags config root)" >&2
  exit 1
}

SRC="$AURA_DIR/hypr/pam/polkit-1"
DEST="/etc/pam.d/polkit-1"

if [[ ! -f "$SRC" ]]; then
  echo "error: missing $SRC" >&2
  exit 1
fi

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Installing $DEST requires root:" >&2
  echo "  sudo AURA_DIR=\"$AURA_DIR\" $0" >&2
  exec sudo AURA_DIR="$AURA_DIR" "$0" "$@"
fi

if [[ -f "$DEST" && ! -L "$DEST" ]]; then
  backup="${DEST}.bak.$(date +%Y%m%d%H%M%S)"
  cp -a "$DEST" "$backup"
  echo "Backed up existing config to $backup"
fi

install -Dm644 "$SRC" "$DEST"
echo "Installed $DEST from $SRC (pam_unix before pam_fprintd)"
echo "Test: pkexec true — password should authenticate immediately."
