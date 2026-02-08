import app from "ags/gtk4/app"
import Bar from "./src/widget/bar/Bar"
import sidecar from "./src/lib/sidecar"

import ControlCenter from "./src/widget/controlcenter/ControlCenter"

// Initialize sidecar access
// @ts-ignore
globalThis.sidecar = sidecar

app.start({
    css: "./style/style.css",
    main() {
        const monitors = app.monitors || []
        
        for (const monitor of monitors) {
            try {
                const win = Bar(monitor)
                // In Astal/Gnim, creating the window attaches it to the application
            } catch (e) {
                console.error("Failed to create Bar for monitor:", e)
            }
        }
        // Initialize Control Center (single instance for now)
        try {
            ControlCenter() 
        } catch (e) {
            console.error("Failed to create ControlCenter:", e)
        }
    },
    requestHandler(request, res) {
        res("ok")
    }
})
