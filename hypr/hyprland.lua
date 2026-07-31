-- Aura Hyprland Lua config (canonical).
-- Loaded via ~/.config/hypr/hyprland.lua (thin XDG stub).
-- Docs: https://wiki.hypr.land/Configuring/Start/
--       ~/.config/ags/hypr/README.md

require("hyprland/vars")
require("hyprland/monitors")
require("hyprland/env")
require("hyprland/settings")
require("hyprland/animations")
require("hyprland/gestures")
require("hyprland/execs")
require("hyprland/rules")
require("hyprland/keybinds")
require("hyprland/user")
-- Optional Keybinds.* RPC overrides (generated); missing file is fine.
pcall(require, "hyprland/aura-overrides")
