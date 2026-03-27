// Stub — OSD renders brightness/volume overlays
// TODO: implement with WebView or GTK4 Revealer animation

import { Astal, Gtk, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"

export default function OSD(gdkmonitor: Gdk.Monitor) {
    const anchor =
        Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT

    return <window
        name={`osd-${gdkmonitor.model}`}
        class="osd-window"
        gdkmonitor={gdkmonitor}
        visible={false}
        anchor={anchor}
        margin={20}
        application={App}
    >
        <box class="osd-container" orientation={Gtk.Orientation.VERTICAL} spacing={8}>
            {/* Volume / Brightness indicators go here */}
            <label label="OSD" />
        </box>
    </window>
}
