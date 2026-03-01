import app from "ags/gtk4/app"
import Bar from "./src/widget/bar/Bar"
import sidecar from "./src/lib/sidecar"

import ControlCenter from "./src/widget/controlcenter/ControlCenter"
import Launcher from "./src/widget/launcher/Launcher"
import OSD from "./src/widget/osd/OSD"

// Popouts
import AudioPopout from "./src/widget/bar/popouts/AudioPopout"
import NetworkPopout from "./src/widget/bar/popouts/NetworkPopout"
import BluetoothPopout from "./src/widget/bar/popouts/BluetoothPopout"
import BatteryPopout from "./src/widget/bar/popouts/BatteryPopout"

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

        // Singleton windows
        try {
            ControlCenter() 
            Launcher()
            
            // Popout windows
            AudioPopout()
            NetworkPopout()
            BluetoothPopout()
            BatteryPopout()
        } catch (e) {
            console.error("Failed to create singletons:", e)
        }
    },
    requestHandler(request, res) {
        res("ok")
    }
})
