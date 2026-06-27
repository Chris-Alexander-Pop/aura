#!/usr/bin/env bash
# Aura-aligned package cleanup — already run 2026-06-26.
# Re-run only if packages were reinstalled: pkexec bash ~/.config/ags/scripts/aura-package-cleanup.sh
set -euo pipefail

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Run with: pkexec bash $0" >&2
  exit 1
fi

run_batch() {
  echo "==> $*"
  pacman -Rs --noconfirm "$@"
}

# Notifications (swaync kept)
run_batch mako dunst

# Tray glue
run_batch network-manager-applet blueberry system-config-printer

# Polkit + launchers
run_batch polkit-gnome polkit-kde-agent rofi wofi waybar

# Misc cruft (skip any not installed)
# Keep libastal-*-git — required by Aura bar (AstalTray, AstalMpris, etc.)
for pkg in materia-gtk-theme file-roller sddm sddm-sugar-candy-git clamav clamtk \
  hyprland-patched-debug gcc14-debug; do
  if pacman -Qi "$pkg" &>/dev/null; then
    run_batch "$pkg"
  else
    echo "==> skip $pkg (not installed)"
  fi
done

echo ""
echo "Orphans (review before removing):"
pacman -Qdtq || true
echo ""
echo "To remove all orphans after review:"
echo "  sudo pacman -Rs \$(pacman -Qdtq)"
echo ""
echo "Done. Re-login recommended."
