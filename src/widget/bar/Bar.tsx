import { Astal, Gtk, Gdk } from "ags/gtk4"
import config from "../../services/config"
import Clock from "./Clock"
import Battery from "./Battery"
import Workspaces from "./Workspaces"

export default function Bar(gdkmonitor: Gdk.Monitor) {
    const { TOP, LEFT, RIGHT } = Astal.WindowAnchor

    return <window
        name={`bar-${gdkmonitor.model}`}
        class="Bar"
        gdkmonitor={gdkmonitor}
        exclusivity={Astal.Exclusivity.EXCLUSIVE}
        anchor={TOP | LEFT | RIGHT}
        heightRequest={config.bar.height}
    >
        <centerbox cssName="centerbox">
            <box hexpand halign={Gtk.Align.START}>
                <Workspaces />
            </box>
            <box>
                <Clock />
            </box>
            <box hexpand halign={Gtk.Align.END}>
                <Battery />
            </box>
        </centerbox>
    </window>
}
