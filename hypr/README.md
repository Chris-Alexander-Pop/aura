# Aura-owned Hypr config

Canonical Hypr ecosystem files for Aura live under `~/.config/ags/hypr/`. Your main compositor config (monitors, decoration, Caelestia vars, etc.) can stay in `~/.config/hypr/` — source these fragments from there.

## Layout

```
hypr/
  application-style.conf        # Qt Quick style (Arch hyprpolkitagent 0.1.x)
  hyprtoolkit.conf              # hyprtoolkit palette (future/git polkit agent)
  hyprlock.conf
  hyprpolkitagent/hyprpolkitagent.conf
  systemd/user/hyprpolkitagent.service.d/override.conf
  hyprland/
    execs-aura.conf
    aura-keybinds.conf
  pam/
    polkit-1                      # password-before-fingerprint PAM stack
```

## Fresh install

1. Install: `hyprland`, `hyprpolkitagent`, `hyprland-qt-support`, `hyprlock`, `polkit`, `swaync`
2. Link XDG paths + systemd drop-in:

   ```bash
   ./scripts/aura-hypr-link.sh
   systemctl --user restart hyprpolkitagent
   ```

3. Add to live `~/.config/hypr/hyprland.conf` (if not already):

   ```ini
   source = ~/.config/ags/hypr/hyprland/execs-aura.conf
   source = ~/.config/ags/hypr/hyprland/aura-keybinds.conf
   ```

4. **Hybrid setup:** if `execs.conf` already starts `hyprpolkitagent` / `aura-session.sh` / `swaync`, skip sourcing `execs-aura.conf` to avoid duplicates.

5. Verify:

   ```bash
   ./scripts/aura-hypr-link.sh --check
   pkexec true
   ```

## Polkit agent (hyprtoolkit, not Qt)

Arch `extra/hyprpolkitagent` is still **Qt/QML**. Aura builds the **hyprtoolkit-native** agent from git:

```bash
./scripts/build-hypr-polkit.sh      # once (or after Hyprland stack updates)
./scripts/aura-hypr-link.sh         # symlinks configs + systemd override
systemctl --user restart hyprpolkitagent
pkexec true
```

Binary: `hypr/dist/libexec/hyprpolkitagent` (bundled `libhyprtoolkit` under `hypr/dist/lib/`). The systemd drop-in in `hypr/systemd/` points at it — **no Qt**.

Theming (hyprtoolkit build reads these):

| File | Effect |
|------|--------|
| `hypr/hyprtoolkit.conf` | Colors, fonts, rounding |
| `hypr/hyprpolkitagent/hyprpolkitagent.conf` | Window size, `show_details` |

`application-style.conf` is only for the old Qt package; ignore once the Aura build is active.

## Polkit password vs fingerprint

Arch’s default `/etc/pam.d/polkit-1` runs **fprintd before unix**, so the agent cannot use your password until fingerprint times out (~30s). Aura ships a password-first stack:

```bash
sudo ./scripts/aura-pam-install.sh   # backs up existing polkit-1
pkexec true                          # password prompt should work on Enter
```

## uwsm / systemd

```ini
exec-once = systemctl --user enable --now hyprpolkitagent.service
```

No GTK or KDE polkit agents required.

## Keybind RPC

Writes only to `~/.config/ags/hypr/hyprland/aura-keybinds.conf` (override: `AURA_KEYBINDS_PATH`).

## Other surfaces

| Surface | Config |
|---------|--------|
| Lock screen | `hypr/hyprlock.conf` |
| Aura React/GTK shell | `Settings.theme` in control center |

## Migrating from `hyprland_aura.conf`

[`hyprland_aura.conf`](../hyprland_aura.conf) is deprecated — use this directory.
