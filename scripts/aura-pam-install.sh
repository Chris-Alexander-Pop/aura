#!/usr/bin/env bash
# Install Aura PAM stacks:
#   polkit-1     — password before fingerprint (privilege prompts)
#   login-auth   — password-only auth snippet (shared)
#   login        — TTY/console session login (password only)
#   sddm         — graphical greeter login (password only)
#   hyprlock     — lock screen (password or fingerprint)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PAM_FILES=(
  login-auth
  login
  sddm
  hyprlock
  polkit-1
)

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

install_pam_file() {
  local name="$1"
  local src="$AURA_DIR/hypr/pam/$name"
  local dest="/etc/pam.d/$name"

  if [[ "$name" == "login-auth" ]]; then
    dest="/etc/pam.d/aura-login-auth"
  fi

  if [[ ! -f "$src" ]]; then
    echo "error: missing $src" >&2
    exit 1
  fi

  if [[ -f "$dest" && ! -L "$dest" ]]; then
    local backup="${dest}.bak.$(date +%Y%m%d%H%M%S)"
    cp -a "$dest" "$backup"
    echo "Backed up $dest -> $backup"
  fi

  install -Dm644 "$src" "$dest"
  echo "Installed $dest"
}

AURA_DIR="$(resolve_aura_dir)" || {
  echo "error: could not find hypr/pam/ (set AURA_DIR to your ags config root)" >&2
  exit 1
}

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Installing /etc/pam.d/* requires root:" >&2
  echo "  sudo AURA_DIR=\"$AURA_DIR\" $0" >&2
  exec sudo AURA_DIR="$AURA_DIR" "$0" "$@"
fi

for name in "${PAM_FILES[@]}"; do
  install_pam_file "$name"
done

cat <<'EOF'

Aura PAM installed:
  aura-login-auth + login/sddm — password only at session login
  polkit-1 + hyprlock          — password or fingerprint (hyprlock: PAM password + native fprintd)

Test polkit:  pkexec true
Test lock:    hyprlock (fingerprint should still work)
Next login:   password required (TTY or SDDM); sudo still accepts fingerprint via system-auth
EOF
