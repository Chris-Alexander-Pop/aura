// Stub — App launcher
// TODO: implement fuzzy search with GTK4 or move into WebView

import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"

export default function Launcher() {
    return <window
        name="launcher"
        class="launcher-window"
        visible={false}
        keymode={Astal.Keymode.ON_DEMAND}
        application={App}
    >
        <box orientation={Gtk.Orientation.VERTICAL} spacing={8} css="padding: 16px;">
            <label label="Launcher — coming soon" />
        </box>
    </window>
}
