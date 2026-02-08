import { Astal, Gtk, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState } from "ags"
import NavRail from "./NavRail"
import Panes from "./Panes"

export const [activeTab, setActiveTab] = createState("network")
export const [navExpanded, setNavExpanded] = createState(false)

export default function ControlCenter() {
    const anchor = Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.RIGHT

    return <window
        name="control-center"
        anchor={anchor}
        application={App}
        visible={false} // Hidden by default, toggled via App.toggle_window
        margin={10}
        keymode={Astal.Keymode.ON_DEMAND}
        // @ts-ignore
        onKeyPressed={(_, keyval) => {
            if (keyval === Gdk.KEY_Escape) {
                App.toggle_window("control-center")
            }
        }}
    >
        <box class="bg-[#1e1e2e] rounded-xl p-0">
            <NavRail />
            <Panes />
        </box>
    </window>
}
