-- Default autostart for clones.
-- Host extras: copy execs-local.lua.example → execs-local.lua (gitignored)
-- and export `before` / `after` callbacks (see example).

local v = require("hyprland/vars")

local host = {}
do
  local ok, mod = pcall(require, "hyprland/execs-local")
  if ok and type(mod) == "table" then
    host = mod
  end
end

hl.on("hyprland.start", function()
  if type(host.before) == "function" then
    host.before(v)
  end

  hl.exec_cmd(v.ags .. "/scripts/aura-session.sh")

  -- Keyring: systemd --user gnome-keyring-daemon.service owns secrets.
  -- Do not also `--start` here — that races PAM and logs "Secret Service was already initialized".
  hl.exec_cmd("hypridle")
  hl.exec_cmd("dbus-update-activation-environment --all")
  hl.exec_cmd("sleep 1 && dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP")

  hl.exec_cmd("wl-paste --type text --watch cliphist store")
  hl.exec_cmd("wl-paste --type image --watch cliphist store")

  hl.exec_cmd("hyprctl setcursor " .. v.cursor_theme .. " " .. tostring(v.cursor_size))
  hl.exec_cmd("gsettings set org.gnome.desktop.interface cursor-theme '" .. v.cursor_theme .. "'")
  hl.exec_cmd("gsettings set org.gnome.desktop.interface cursor-size " .. tostring(v.cursor_size))

  hl.exec_cmd("mpris-proxy")
  hl.exec_cmd("systemctl --user enable --now hyprpolkitagent.service")
  hl.exec_cmd("swaync")
  hl.exec_cmd(v.ags .. "/scripts/aura-brightness-floor.sh")

  if type(host.after) == "function" then
    host.after(v)
  end
end)
