local v = require("hyprland/vars")

hl.gesture({
  fingers = v.workspace_swipe_fingers,
  direction = "horizontal",
  action = "workspace",
})

hl.gesture({
  fingers = v.gesture_fingers,
  direction = "up",
  action = "special",
  workspace_name = "special",
})

hl.gesture({
  fingers = v.gesture_fingers,
  direction = "down",
  action = function()
    hl.dispatch(hl.dsp.workspace.toggle_special("special"))
  end,
})

hl.gesture({
  fingers = v.gesture_fingers_more,
  direction = "down",
  action = function()
    hl.exec_cmd("systemctl suspend-then-hibernate")
  end,
})
