import app from "ags/gtk4/app"
import Bar from "./src/widget/bar/Bar"
import sidecar from "./src/lib/sidecar"
import OSD from "./src/widget/osd/OSD"
import Launcher from "./src/widget/launcher/Launcher"

// WebView overlay windows (React UI served from sidecar at localhost:9080)
import ControlCenterWindow from "./src/widget/webview/ControlCenterWindow"
import SidebarWindow from "./src/widget/webview/SidebarWindow"
import DropdownWindow from "./src/widget/webview/DropdownWindow"
import CalendarWindow from "./src/widget/webview/CalendarWindow"

// Initialize sidecar access (used by bar/osd via sidecar.ts)
// @ts-ignore
globalThis.sidecar = sidecar

app.start({
    css: "./style/style.css",
    main() {
        const monitors = app.monitors || []

        for (const monitor of monitors) {
            try {
                Bar(monitor)
                OSD(monitor)
            } catch (e) {
                console.error("Failed to create per-monitor windows:", e)
            }
        }

        // Singleton windows
        try {
            Launcher()

            // WebKit overlay panels
            ControlCenterWindow()
            SidebarWindow()
            DropdownWindow()
            CalendarWindow()
        } catch (e) {
            console.error("Failed to create singleton windows:", e)
        }
    },
    requestHandler(request: string, res: (r: string) => void) {
        res("ok")
    }
})
