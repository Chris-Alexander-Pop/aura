-- Shared variables + Material scheme loader (scheme/*.conf still driven by AGS/theme).
local M = {}

M.home = os.getenv("HOME") or "/home/user"
M.hypr = M.home .. "/.config/hypr"
M.ags = M.home .. "/.config/ags"
-- Canonical Lua config root (this tree). Scripts / scheme stay under M.hypr.
M.aura_hypr = M.ags .. "/hypr"

-- Ensure scheme/current.conf exists (same as old hyprland.conf bootstrap).
do
  local default = M.hypr .. "/scheme/default.conf"
  local current = M.hypr .. "/scheme/current.conf"
  local f = io.open(current, "r")
  if f then
    f:close()
  else
    os.execute(string.format(
      "cp -L --no-preserve=mode --update=none %q %q",
      default, current
    ))
  end
end

local function load_scheme(path)
  local colors = {}
  local f = io.open(path, "r")
  if not f then return colors end
  for line in f:lines() do
    local k, v = line:match("^%$([%w_]+)%s*=%s*(%S+)")
    if k and v then colors[k] = v end
  end
  f:close()
  return colors
end

M.colors = load_scheme(M.hypr .. "/scheme/current.conf")

function M.hex(name, fallback)
  return M.colors[name] or fallback or "000000"
end

function M.rgba(name, alpha, fallback)
  return string.format("rgba(%s%s)", M.hex(name, fallback), alpha or "ff")
end

function M.rgb(name, fallback)
  return string.format("rgb(%s)", M.hex(name, fallback))
end

-- Apps
M.terminal = "foot"
M.browser = "zen-browser"
M.editor = "code"
M.file_explorer = "thunar"

-- Touchpad / gestures
M.touchpad_disable_typing = true
M.touchpad_scroll_factor = 0.3
M.workspace_swipe_fingers = 4
M.gesture_fingers = 3
M.gesture_fingers_more = 4

-- Blur / shadow (base; hyprland/user.lua may override)
M.blur_enabled = true
M.blur_special_ws = false
M.blur_popups = true
M.blur_input_methods = true
M.blur_size = 8
M.blur_passes = 2
M.blur_xray = false

M.shadow_enabled = true
M.shadow_range = 20
M.shadow_render_power = 3

-- Gaps / window style
M.workspace_gaps = 20
M.window_gaps_in = 10
M.window_gaps_out = 40
M.single_window_gaps_out = 20
M.window_opacity = 0.95
M.window_rounding = 10
M.window_border_size = 3

M.volume_step = 10
M.cursor_theme = "sweet-cursors"
M.cursor_size = 24

function M.shadow_colour()
  return M.rgba("surface", "d4")
end

function M.active_border()
  return M.rgba("primary", "e6")
end

function M.inactive_border()
  return M.rgba("onSurfaceVariant", "11")
end

return M
