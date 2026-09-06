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

-- GPU: Intel iGPU only. NVIDIA disabled 2026-08-25 after Xid 79.
-- Do NOT use /dev/dri/by-path/pci-… here: Aquamarine splits on ':' and PCI paths contain ':'.
-- After re-enabling NVIDIA, restore intel-igpu:nvidia-dgpu (colon-free symlinks).
hl.env("AQ_DRM_DEVICES", v.home .. "/.local/share/dri/intel-igpu")
hl.env("AQ_FORCE_LINEAR_BLIT", "0")
hl.env("LIBVA_DRIVER_NAME", "iHD")
hl.env("__GLX_VENDOR_LIBRARY_NAME", "mesa")
hl.env("__EGL_VENDOR_LIBRARY_FILENAMES", "/usr/share/glvnd/egl_vendor.d/50_mesa.json")
hl.env("VK_DRIVER_FILES", "/usr/share/vulkan/icd.d/intel_icd.json")
hl.env("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/intel_icd.json")

hl.env("_JAVA_AWT_WM_NONREPARENTING", "1")
