local v = require("hyprland/vars")

-- Themes
hl.env("QT_QPA_PLATFORMTHEME", "qt6ct")
hl.env("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1")
hl.env("QT_AUTO_SCREEN_SCALE_FACTOR", "1")
hl.env("XCURSOR_THEME", v.cursor_theme)
hl.env("XCURSOR_SIZE", tostring(v.cursor_size))

-- Toolkit backends
hl.env("GDK_BACKEND", "wayland,x11")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("SDL_VIDEODRIVER", "wayland,x11,windows")
hl.env("CLUTTER_BACKEND", "wayland")
hl.env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
hl.env("MOZ_ENABLE_WAYLAND", "1")

-- XDG
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")

-- GPU: Intel compositor + NVIDIA HDMI scanout.
-- Do NOT use /dev/dri/by-path/pci-… here: Aquamarine splits on ':' and PCI paths contain ':'.
hl.env("AQ_DRM_DEVICES", v.home .. "/.local/share/dri/intel-igpu:" .. v.home .. "/.local/share/dri/nvidia-dgpu")
hl.env("AQ_FORCE_LINEAR_BLIT", "0")
hl.env("LIBVA_DRIVER_NAME", "iHD")

hl.env("_JAVA_AWT_WM_NONREPARENTING", "1")
