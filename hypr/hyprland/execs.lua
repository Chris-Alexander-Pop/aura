local v = require("hyprland/vars")

hl.on("hyprland.start", function()
  -- HDMI cold-plug then Aura session (order matters).
  hl.exec_cmd("bash -c '" .. v.hypr .. "/scripts/reinit-hdmi-boot.sh; " .. v.ags .. "/scripts/aura-session.sh'")

  -- Keyring: systemd --user gnome-keyring-daemon.service owns secrets.
  -- Do not also `--start` here — that races PAM and logs "Secret Service was already initialized".
  hl.exec_cmd("hypridle")
  hl.exec_cmd("dbus-update-activation-environment --all")
  hl.exec_cmd("sleep 1 && dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP")

  -- Waydroid: persistent user session (minigbm_gbm_mesa + Intel renderD128).
  -- Do not bare `waydroid session start` from here — the process must stay alive.
  hl.exec_cmd("systemctl --user start waydroid-session.service")
  hl.exec_cmd(v.hypr .. "/scripts/waydroid-hypr-size-sync.sh")
  hl.exec_cmd(v.hypr .. "/scripts/watch-monitor-events.sh")

  hl.exec_cmd("wl-paste --type text --watch cliphist store")
  hl.exec_cmd("wl-paste --type image --watch cliphist store")
  hl.exec_cmd("trash-empty 30")

  hl.exec_cmd("hyprctl setcursor " .. v.cursor_theme .. " " .. tostring(v.cursor_size))
  hl.exec_cmd("gsettings set org.gnome.desktop.interface cursor-theme '" .. v.cursor_theme .. "'")
  hl.exec_cmd("gsettings set org.gnome.desktop.interface cursor-size " .. tostring(v.cursor_size))

  hl.exec_cmd("mpris-proxy")
  hl.exec_cmd("rquickshare")

  -- From hypr-user.conf / aura execs
  hl.exec_cmd("valent --gapplication-service")
  hl.exec_cmd("systemctl --user enable --now hyprpolkitagent.service")
  hl.exec_cmd("swaync")
  hl.exec_cmd(v.ags .. "/scripts/aura-brightness-floor.sh")
end)
