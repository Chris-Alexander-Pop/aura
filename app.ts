import app from "ags/gtk4/app"
import Bar from "./src/widget/bar/Bar"
import sidecar from "./src/lib/sidecar"

import ControlCenter from "./src/widget/controlcenter/ControlCenter"
import Launcher from "./src/widget/launcher/Launcher"
import OSD from "./src/widget/osd/OSD"

// Initialize sidecar access
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
                console.error("Failed to create windows for monitor:", e)
            }
        }
        // Initialize Control Center (single instance for now)
        try {
            ControlCenter() 
            Launcher()
        } catch (e) {
            console.error("Failed to create singletons:", e)
        }
    },
    requestHandler(request, res) {
        res("ok")
    }
})
