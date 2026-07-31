-- Machine-specific overrides (was hypr-user.conf)
hl.config({
  misc = {
    vrr = 0,
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    background_color = 0x000000,
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
  name = "wacom-pen-and-multitouch-sensor-pen",
  output = "eDP-1",
})

hl.device({
  name = "wacom-pen-and-multitouch-sensor-finger",
  output = "eDP-1",
})
