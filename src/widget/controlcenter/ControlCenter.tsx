import { App, Astal, Gtk, Gdk } from "ags/gtk4"
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
        visible={true} // For now, always visible while testing
        margin={10}
        keymode={Astal.Keymode.ON_DEMAND}
    >
        <box css="background-color: #1e1e2e; border-radius: 12px; padding: 0;">
            <NavRail />
            <Panes />
        </box>
    </window>
}
