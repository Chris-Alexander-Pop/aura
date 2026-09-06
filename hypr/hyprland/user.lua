-- Machine-specific overrides (was hypr-user.conf)
hl.config({
  -- Native-pixel XWayland on fractional scale (4K @ 1.5) — without this,
  -- X11/Compose apps are stretched and look countable-pixel soft.
  xwayland = {
    force_zero_scaling = true,
  },
  misc = {
    vrr = 0,
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    background_color = 0x000000,
    -- Electron (Cursor) routinely misses Wayland pings for >7.5s during
    -- extension-host startup; default 5 * 1.5s fires a false ANR dialog.
    anr_missed_pings = 20,
  },
  debug = {
    vfr = true,
  },
  decoration = {
    blur = { enabled = false },
    shadow = { enabled = false },
  },
  cursor = {
    -- Software cursors: less NVIDIA wake flicker, more CPU on 4K@1.5
    no_hardware_cursors = true,
  },
})

hl.device({
  name = "tpps/2-elan-trackpoint",
  enabled = false,
})

hl.device({
  name = "logitech-mx-master-3s",
  enabled = true,
  sensitivity = 0.0,
  accel_profile = "flat",
  natural_scroll = false,
  scroll_factor = 1.0,
})

hl.device({
  name = "wacom-pen-and-multitouch-sensor-pen",
  output = "eDP-1",
})

hl.device({
  name = "wacom-pen-and-multitouch-sensor-finger",
  output = "eDP-1",
})
