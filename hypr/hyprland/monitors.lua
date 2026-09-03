-- Per-monitor profiles.
-- Machine-specific EDID descriptions live in gitignored monitors-local.lua
-- (copy monitors-local.lua.example and edit). If that file is present it
-- owns the layout; this module then adds nothing.

if pcall(require, "hyprland/monitors-local") then
  return
end

-- Public fallback for clones without a local override.
hl.monitor({
  output = "",
  mode = "preferred",
  position = "auto",
  scale = 1,
})
