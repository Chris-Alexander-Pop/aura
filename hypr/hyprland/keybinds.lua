-- Keybinds (final Aura / user overrides applied here — no separate override dance).
local v = require("hyprland/vars")
local home = v.home
local ags = v.ags
local hypr = v.hypr
local media = ags .. "/scripts/aura-media-key.sh"
local rotate = hypr .. "/scripts/set-monitor-rotation.sh"

local function exec(cmd) return hl.dsp.exec_cmd(cmd) end
local L = { locked = true }
local R = { release = true }
local E = { repeating = true }
local LE = { locked = true, repeating = true }
local M = { mouse = true }

-- Launcher
hl.bind("SUPER + SUPER_L", exec("vicinae toggle"), R)
hl.bind("SUPER + SUPER_R", exec("vicinae toggle"), R)
hl.bind("SUPER + A", exec("ags request toggle launcher"))
hl.bind("SUPER + SPACE", exec("ags request toggle launcher"))

-- Media transport
hl.bind("XF86AudioPlay", exec("playerctl play-pause"), L)
hl.bind("XF86AudioPause", exec("playerctl play-pause"), L)
hl.bind("CTRL + SUPER + equal", exec("playerctl next"), L)
hl.bind("XF86AudioNext", exec("playerctl next"), L)
hl.bind("CTRL + SUPER + minus", exec("playerctl previous"), L)
hl.bind("XF86AudioPrev", exec("playerctl previous"), L)
hl.bind("XF86AudioStop", exec("playerctl stop"), L)

-- Aura restart
hl.bind("CTRL + SUPER + SHIFT + R", exec(ags .. "/scripts/aura-restart.sh"), R)
hl.bind("CTRL + SUPER + ALT + R", exec(ags .. "/scripts/aura-restart.sh"), R)
hl.bind("SUPER + SHIFT + R", exec(ags .. "/scripts/aura-restart.sh"))

-- Workspaces 1-10 / groups (was wsaction.zsh — classic `hyprctl dispatch workspace N`
-- is broken on Lua-config Hyprland; dispatch via hl.dsp instead).
-- SUPER+SHIFT also binds shifted keysyms: with resolve_binds_by_sym off, digits
-- match; if a build consumes Shift into the keysym, exclam/at/... still fire.
local shift_keys = {
  "exclam",
  "at",
  "numbersign",
  "dollar",
  "percent",
  "asciicircum",
  "ampersand",
  "asterisk",
  "parenleft",
  "parenright",
}

local function slot_workspace(slot, to_group)
  local active = hl.get_active_workspace()
  local id = (active and active.id) or 1
  if to_group then
    return (slot == 10) and 10 or ((slot - 1) * 10 + 1)
  end
  return math.floor((id - 1) / 10) * 10 + slot
end

for i = 1, 10 do
  local key = tostring(i % 10)
  local shift_key = shift_keys[i]

  hl.bind("SUPER + " .. key, function()
    hl.dispatch(hl.dsp.focus({ workspace = slot_workspace(i, false) }))
  end)
  hl.bind("CTRL + SUPER + " .. key, function()
    hl.dispatch(hl.dsp.focus({ workspace = slot_workspace(i, true) }))
  end)
  hl.bind("SUPER + ALT + " .. key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, false) }))
  end)
  hl.bind("SUPER + SHIFT + " .. key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, false) }))
  end)
  hl.bind("SUPER + SHIFT + " .. shift_key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, false) }))
  end)
  hl.bind("CTRL + SUPER + ALT + " .. key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, true) }))
  end)
  hl.bind("CTRL + SUPER + SHIFT + " .. key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, true) }))
  end)
  hl.bind("CTRL + SUPER + SHIFT + " .. shift_key, function()
    hl.dispatch(hl.dsp.window.move({ workspace = slot_workspace(i, true) }))
  end)
end

hl.bind("CTRL + SUPER + left", hl.dsp.focus({ workspace = "-1" }), E)
hl.bind("CTRL + SUPER + right", hl.dsp.focus({ workspace = "+1" }), E)
hl.bind("SUPER + Page_Up", hl.dsp.focus({ workspace = "-1" }), E)
hl.bind("SUPER + Page_Down", hl.dsp.focus({ workspace = "+1" }), E)

hl.bind("SUPER + S", hl.dsp.workspace.toggle_special("special"))

hl.bind("SUPER + ALT + Page_Up", hl.dsp.window.move({ workspace = "-1" }), E)
hl.bind("SUPER + ALT + Page_Down", hl.dsp.window.move({ workspace = "+1" }), E)
hl.bind("CTRL + SUPER + SHIFT + right", hl.dsp.window.move({ workspace = "+1" }), E)
hl.bind("CTRL + SUPER + SHIFT + left", hl.dsp.window.move({ workspace = "-1" }), E)
hl.bind("CTRL + SUPER + SHIFT + up", hl.dsp.window.move({ workspace = "special:special" }))
hl.bind("CTRL + SUPER + SHIFT + down", hl.dsp.window.move({ workspace = "e+0" }))
hl.bind("SUPER + ALT + S", hl.dsp.window.move({ workspace = "special:special" }))

-- Groups
hl.bind("ALT + Tab", hl.dsp.window.cycle_next(), E)
hl.bind("SHIFT + ALT + Tab", hl.dsp.window.cycle_next({ next = false }), E)
hl.bind("CTRL + ALT + Tab", hl.dsp.group.next(), E)
hl.bind("CTRL + SHIFT + ALT + Tab", hl.dsp.group.prev(), E)
hl.bind("SUPER + comma", hl.dsp.group.toggle())
hl.bind("SUPER + U", hl.dsp.window.move({ out_of_group = true }))
hl.bind("SUPER + SHIFT + comma", hl.dsp.group.lock_active({ action = "toggle" }))

-- Focus / move
hl.bind("SUPER + left", hl.dsp.focus({ direction = "left" }))
hl.bind("SUPER + right", hl.dsp.focus({ direction = "right" }))
hl.bind("SUPER + up", hl.dsp.focus({ direction = "up" }))
hl.bind("SUPER + down", hl.dsp.focus({ direction = "down" }))
hl.bind("SUPER + SHIFT + left", hl.dsp.window.move({ direction = "left" }))
hl.bind("SUPER + SHIFT + right", hl.dsp.window.move({ direction = "right" }))
hl.bind("SUPER + SHIFT + up", hl.dsp.window.move({ direction = "up" }))
hl.bind("SUPER + SHIFT + down", hl.dsp.window.move({ direction = "down" }))
hl.bind("SUPER + minus", hl.dsp.layout("splitratio -0.1"), E)
hl.bind("SUPER + equal", hl.dsp.layout("splitratio +0.1"), E)

-- Mouse move/resize
hl.bind("SUPER + mouse:272", hl.dsp.window.drag(), M)
hl.bind("SUPER + mouse:273", hl.dsp.window.resize(), M)
hl.bind("SUPER + Z", hl.dsp.window.drag(), M)
hl.bind("SUPER + X", hl.dsp.window.resize(), M)

hl.bind("CTRL + SUPER + backslash", hl.dsp.window.center())
hl.bind("CTRL + SUPER + ALT + backslash", function()
  hl.dispatch(hl.dsp.window.resize({ x = "55%", y = "70%" }))
  hl.dispatch(hl.dsp.window.center())
end)

-- Super+P → Aura control center (overrides old pin bind)
hl.bind("SUPER + P", exec("ags request toggle control-center"))
hl.bind("SUPER + F", hl.dsp.window.fullscreen({ mode = "fullscreen" }))
hl.bind("SUPER + ALT + F", hl.dsp.window.fullscreen({ mode = "maximized" }))
hl.bind("SUPER + ALT + SPACE", hl.dsp.window.float({ action = "toggle" }))
hl.bind("SUPER + Q", hl.dsp.window.close())

-- Special toggles
hl.bind("CTRL + SHIFT + Escape", exec("ags request toggle module-hub"))
hl.bind("SUPER + M", exec("ags request toggle media-popup"))
hl.bind("SUPER + D", hl.dsp.workspace.toggle_special("communication"))
hl.bind("SUPER + R", exec("ags request toggle calendar"))

-- Apps
hl.bind("SUPER + T", exec("app2unit -- " .. v.terminal))
hl.bind("SUPER + W", exec("app2unit -- " .. v.browser))
hl.bind("SUPER + C", exec("app2unit -- " .. v.editor))
hl.bind("SUPER + G", exec("app2unit -- github-desktop"))
hl.bind("SUPER + E", exec("app2unit -- " .. v.file_explorer))
hl.bind("CTRL + ALT + Escape", exec("app2unit -- qps"))
hl.bind("CTRL + ALT + V", exec("app2unit -- pavucontrol"))

-- Screenshots / picker
hl.bind("Print", exec(ags .. "/scripts/full-screenshot.sh"), L)
hl.bind("SUPER + SHIFT + S", exec(ags .. "/scripts/region-screenshot.sh"))
hl.bind("SUPER + SHIFT + ALT + S", exec(ags .. "/scripts/region-screenshot.sh"))
hl.bind("SUPER + ALT + R", exec('notify-send -u low Aura "Screen recording needs wf-recorder + sidecar Capture RPC"'))
hl.bind("CTRL + ALT + R", exec('notify-send -u low Aura "Screen recording needs wf-recorder + sidecar Capture RPC"'))
hl.bind("SUPER + SHIFT + ALT + R", exec('notify-send -u low Aura "Screen recording needs wf-recorder + sidecar Capture RPC"'))
hl.bind("SUPER + SHIFT + C", exec("hyprpicker -a"))

-- Volume / brightness via Aura OSD (overrides raw wpctl binds)
hl.bind("XF86AudioRaiseVolume", exec(media .. " volume-up"), LE)
hl.bind("XF86AudioLowerVolume", exec(media .. " volume-down"), LE)
hl.bind("XF86AudioMute", exec(media .. " mute"), L)
hl.bind("XF86AudioMicMute", exec(media .. " mic-mute"), L)
hl.bind("SUPER + SHIFT + M", exec(media .. " mute"), L)
hl.bind("XF86MonBrightnessUp", exec(media .. " brightness-up"), LE)
hl.bind("XF86MonBrightnessDown", exec(media .. " brightness-down"), LE)
hl.bind("F6", exec(media .. " brightness-up"))
hl.bind("F5", exec(media .. " brightness-down"))

hl.bind("SUPER + SHIFT + L", exec("systemctl suspend-then-hibernate"))

-- Clipboard
hl.bind("SUPER + V", exec(ags .. "/scripts/clipboard-picker.sh"))
hl.bind("SUPER + ALT + V", exec(ags .. "/scripts/clipboard-picker.sh -d"))
hl.bind("SUPER + period", exec("notify-send -u low 'Emoji' 'No picker configured yet'"))
hl.bind("CTRL + SHIFT + ALT + V", exec('sleep 0.5s && ydotool type -d 1 "$(cliphist list | head -1 | cliphist decode)"'), L)

hl.bind("SUPER + ALT + F12", exec(
  [[notify-send -u low -i dialog-information-symbolic 'Test notification' "Here's a really long message to test truncation and wrapping\nYou can middle click or flick this notification to dismiss it!" -a 'Shell' -A "Test1=I got it!" -A "Test2=Another action"]]
), L)

hl.bind("SUPER + slash", exec("ags request toggle control-center"))
hl.bind("SUPER + SHIFT + slash", exec("ags request toggle control-center"))

-- Disable middle click paste
hl.bind("mouse:274", exec(":"))

-- Screen rotation (hypr-user.conf mapping wins)
hl.bind("CTRL + ALT + SUPER + Up", exec(rotate .. " 0"))
hl.bind("CTRL + ALT + SUPER + Right", exec(rotate .. " 3"))
hl.bind("CTRL + ALT + SUPER + Down", exec(rotate .. " 2"))
hl.bind("CTRL + ALT + SUPER + Left", exec(rotate .. " 1"))
hl.bind("CTRL + ALT + SUPER + H", exec(hypr .. "/scripts/rehome-workspaces-to-internal.sh --disable-externals"))

-- Pointer workspace scroll (default submap / universal)
hl.bind("SUPER + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
hl.bind("SUPER + mouse_up", hl.dsp.focus({ workspace = "e-1" }))
hl.bind("CTRL + SUPER + mouse_down", hl.dsp.focus({ workspace = "e+10" }))
hl.bind("CTRL + SUPER + mouse_up", hl.dsp.focus({ workspace = "e-10" }))
hl.bind("SUPER + ALT + mouse_down", hl.dsp.window.move({ workspace = "+1" }))
hl.bind("SUPER + ALT + mouse_up", hl.dsp.window.move({ workspace = "-1" }))

-- Aura panels
hl.bind("SUPER + SHIFT + B", exec("ags request toggle dropdown"))
hl.bind("SUPER + SHIFT + D", exec("ags request toggle calendar"))

-- User extras
hl.bind("SUPER + SHIFT + T", exec(home .. "/.local/bin/toggle_touchscreen.sh"))
hl.bind("XF86Launch2", exec("xournalpp"))
