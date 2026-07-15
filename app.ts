import app from "ags/gtk4/app"
import GLib from "gi://GLib"
import sidecar from "./src/lib/sidecar"
import { refreshOsdFromSidecar } from "./src/widget/osd/OSD"
import LauncherWindow from "./src/widget/webview/LauncherWindow"
import ModuleHubWindow from "./src/widget/webview/ModuleHubWindow"

// WebView overlay windows (React UI served from sidecar at localhost:9080)
import ControlCenterWindow from "./src/widget/webview/ControlCenterWindow"
import DropdownWindow from "./src/widget/webview/DropdownWindow"
import CalendarWindow from "./src/widget/webview/CalendarWindow"
import MediaPopupWindow from "./src/widget/webview/MediaPopupWindow"

import { initAuraSettingsShell } from "./src/lib/aura-settings-shell"
import { initMonitorShell } from "./src/lib/monitor-shell"
import { initShellVisibility } from "./src/lib/shell-visibility"
import { isHoverPanel, toggleHoverPanel, MODULE_HUB_ENABLED } from "./src/lib/panel-hover"

const DEBUG_EDGE = GLib.getenv("AURA_DEBUG_EDGE_TRIGGERS") === "1"

// Initialize sidecar (bar/osd use stdin/stdout path)
// @ts-ignore
globalThis.sidecar = sidecar

app.start({
    css: "./style/style.css",
    main() {
        const monitors = app.monitors || []
        if (DEBUG_EDGE) {
            console.error(`[aura] main: ${monitors.length} monitor(s), mounting edge triggers`)
        }

        for (const monitor of monitors) {
            if (DEBUG_EDGE) {
                console.error(`[aura] monitor: ${String(monitor.model ?? "unknown")}`)
            }
        }

        initMonitorShell()

        // Singleton windows
        try {
            LauncherWindow()
            // Tile hub UI parked — re-enable MODULE_HUB_ENABLED in panel-hover.ts when ready
            if (MODULE_HUB_ENABLED) ModuleHubWindow()

            // WebKit overlay panels
            ControlCenterWindow()
            DropdownWindow()
            CalendarWindow()
            MediaPopupWindow()
        } catch (e) {
            console.error("Failed to create singleton windows:", e)
        }

        initAuraSettingsShell()
        initShellVisibility()
    },

    // Handle: ags request <command>…  (argv[] from DBus; may include leading "ags", "request")
    // Hyprland: exec, ags request toggle control-center
    requestHandler(argv: string[], res: (r: string) => void) {
        const raw = argv.filter((s) => s.length > 0)
        const reqAt = raw.indexOf("request")
        const parts = reqAt !== -1 && reqAt + 1 < raw.length ? raw.slice(reqAt + 1) : raw

        switch (parts[0]) {
            case "toggle": {
                const name = parts[1]
                if (name === "module-hub" && !MODULE_HUB_ENABLED) {
                    res("module-hub disabled")
                    break
                }
                if (name) {
                    if (isHoverPanel(name)) {
                        const visible = toggleHoverPanel(
                            name,
                            undefined,
                            name === "module-hub" ? { moduleHubTop: true } : undefined
                        )
                        res(`toggled ${name} → ${visible}`)
                        break
                    }
                    const win = app.get_window(name)
                    if (win) {
                        win.visible = !win.visible
                        res(`toggled ${name} → ${win.visible}`)
                    } else {
                        res(`window not found: ${name}`)
                    }
                } else {
                    res("usage: toggle <window-name>")
                }
                break
            }
            case "show": {
                const win = app.get_window(parts[1])
                if (win) { win.visible = true; res("ok") }
                else res("not found")
                break
            }
            case "hide": {
                const win = app.get_window(parts[1])
                if (win) { win.visible = false; res("ok") }
                else res("not found")
                break
            }
            case "osd": {
                const target = (parts[1] ?? "all") as "volume" | "brightness" | "mic" | "all"
                const allowed = new Set(["volume", "brightness", "mic", "all"])
                if (!allowed.has(target)) {
                    res("usage: osd [volume|brightness|mic|all]")
                    break
                }
                refreshOsdFromSidecar(target).then(() => res(`osd ${target}`)).catch((e) => {
                    res(`osd error: ${e}`)
                })
                break
            }
            default:
                res("ok")
        }
    }
})
