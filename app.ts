import app from "ags/gtk4/app"
import GLib from "gi://GLib"
import Bar from "./src/widget/bar/Bar"
import sidecar from "./src/lib/sidecar"
import OSD, { refreshOsdFromSidecar } from "./src/widget/osd/OSD"
import LauncherWindow from "./src/widget/webview/LauncherWindow"
import SidebarWindow from "./src/widget/webview/SidebarWindow"

// WebView overlay windows (React UI served from sidecar at localhost:9080)
import ControlCenterWindow from "./src/widget/webview/ControlCenterWindow"
import DropdownWindow from "./src/widget/webview/DropdownWindow"
import CalendarWindow from "./src/widget/webview/CalendarWindow"

import { mountPanelEdgeTriggers } from "./src/widget/triggers/PanelEdgeTriggers"
import BarWebViewWindow from "./src/widget/webview/BarWebViewWindow"

/** Legacy GTK vertical bar — set `AURA_GTK_BAR=1` to restore it instead of the React/WebKit strip */
const USE_GTK_BAR = GLib.getenv("AURA_GTK_BAR") === "1"

// Initialize sidecar (bar/osd use stdin/stdout path)
// @ts-ignore
globalThis.sidecar = sidecar

app.start({
    css: "./style/style.css",
    main() {
        const monitors = app.monitors || []
        console.error(`[aura] main: ${monitors.length} monitor(s), mounting edge triggers`)

        for (const monitor of monitors) {
            try {
                if (USE_GTK_BAR) {
                    Bar(monitor)
                } else {
                    BarWebViewWindow(monitor)
                }
            } catch (e) {
                console.error("Failed to create bar for monitor:", e)
            }

            try {
                mountPanelEdgeTriggers(monitor)
            } catch (e) {
                console.error("Failed to create edge triggers for monitor:", e)
            }

            try {
                OSD(monitor)
            } catch (e) {
                console.error("Failed to create OSD for monitor:", e)
            }
        }

        // Singleton windows
        try {
            LauncherWindow()
            SidebarWindow()

            // WebKit overlay panels
            ControlCenterWindow()
            DropdownWindow()
            CalendarWindow()
        } catch (e) {
            console.error("Failed to create singleton windows:", e)
        }
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
                if (name) {
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
