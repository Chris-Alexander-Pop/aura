-- general / dwindle / decoration / group / misc / input / devices / cursor / debug
local v = require("hyprland/vars")

hl.config({
  general = {
    layout = "dwindle",
    allow_tearing = false,
    gaps_workspaces = v.workspace_gaps,
    gaps_in = v.window_gaps_in,
    gaps_out = v.window_gaps_out,
    border_size = v.window_border_size,
    col = {
      active_border = v.active_border(),
      inactive_border = v.inactive_border(),
    },
  },
  dwindle = {
    preserve_split = true,
    smart_split = false,
    smart_resizing = true,
  },
  decoration = {
    rounding = v.window_rounding,
    blur = {
      enabled = v.blur_enabled,
      xray = v.blur_xray,
      special = v.blur_special_ws,
      ignore_opacity = true,
      new_optimizations = true,
      popups = v.blur_popups,
      input_methods = v.blur_input_methods,
      size = v.blur_size,
      passes = v.blur_passes,
    },
    shadow = {
      enabled = v.shadow_enabled,
      range = v.shadow_range,
      render_power = v.shadow_render_power,
      color = v.shadow_colour(),
    },
  },
  group = {
    col = {
      border_active = v.active_border(),
      border_inactive = v.inactive_border(),
      border_locked_active = v.active_border(),
      border_locked_inactive = v.inactive_border(),
    },
    groupbar = {
      font_family = "JetBrains Mono NF",
      font_size = 15,
      gradients = true,
      gradient_round_only_edges = false,
      gradient_rounding = 5,
      height = 25,
      indicator_height = 0,
      gaps_in = 3,
      gaps_out = 3,
      text_color = v.rgb("onPrimary"),
      col = {
        active = v.rgba("primary", "d4"),
        inactive = v.rgba("outline", "d4"),
        locked_active = v.rgba("primary", "d4"),
        locked_inactive = v.rgba("secondary", "d4"),
      },
    },
  },
  misc = {
    vrr = 0,
    animate_manual_resizes = false,
    animate_mouse_windowdragging = false,
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    force_default_wallpaper = 0,
    allow_session_lock_restore = true,
    middle_click_paste = false,
    focus_on_activate = true,
    session_lock_xray = true,
    mouse_move_enables_dpms = true,
    key_press_enables_dpms = true,
    background_color = v.rgb("surfaceContainer"),
  },
  input = {
    kb_layout = "us",
    numlock_by_default = false,
    repeat_delay = 250,
    repeat_rate = 35,
    focus_on_close = 1,
    touchpad = {
      natural_scroll = true,
      disable_while_typing = v.touchpad_disable_typing,
      scroll_factor = v.touchpad_scroll_factor,
      clickfinger_behavior = false,
      tap_to_click = false,
    },
  },
  binds = {
    scroll_event_delay = 0,
  },
  cursor = {
    hotspot_padding = 1,
    no_hardware_cursors = false,
  },
  debug = {
    error_position = 1,
    vfr = true,
    disable_logs = false,
  },
  gestures = {
    workspace_swipe_distance = 700,
    workspace_swipe_cancel_ratio = 0.15,
    workspace_swipe_min_speed_to_force = 5,
    workspace_swipe_direction_lock = true,
    workspace_swipe_direction_lock_threshold = 10,
    workspace_swipe_create_new = true,
  },
})

hl.device({
  name = "synaptics-tm3418-002",
  clickfinger_behavior = false,
  middle_button_emulation = false,
  tap_to_click = false,
})

hl.device({
  name = "tpps/2-elan-trackpoint",
  enabled = true,
})
