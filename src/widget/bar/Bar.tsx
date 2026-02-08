import { Astal, Gtk, Gdk } from "ags/gtk4"
import config from "../../services/config"
import Clock from "./Clock"
import Battery from "./Battery"
import Workspaces from "./Workspaces"
import SysTray from "./SysTray"
import Media from "./Media"
import Indicators from "./Indicators"

export default function Bar(gdkmonitor: Gdk.Monitor) {
    const { TOP, LEFT, RIGHT } = Astal.WindowAnchor

    return <window
        name={`bar-${gdkmonitor.model}`}
        class="bg-transparent text-white font-bold"
        gdkmonitor={gdkmonitor}
        exclusivity={Astal.Exclusivity.EXCLUSIVE}
        anchor={TOP | LEFT | RIGHT}
        heightRequest={config.bar.height}
    >
        <centerbox css="background-color: #1e1e2e; border-radius: 12px; margin: 8px; padding: 4px;">
            <box hexpand halign={Gtk.Align.START} spacing={12}>
                <Workspaces />
                <Media />
            </box>
            <box>
                <Clock />
            </box>
            <box hexpand halign={Gtk.Align.END} spacing={12}>
                <SysTray />
                <Indicators />
                <Battery />
            </box>
        </centerbox>
    </window>
}
