import { Astal, Gtk, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { colors, fonts } from "../../lib/theme"

import Logo from "./Logo"
import Workspaces from "./Workspaces"
import ActiveWindow from "./ActiveWindow"
import SysTray from "./SysTray"
import Media from "./Media"
import Clock from "./Clock"
import StatusIcons from "./StatusIcons"
import Power from "./Power"

export default function Bar(gdkmonitor: Gdk.Monitor) {
    const anchor = Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM

    return <window
        name={`bar-${gdkmonitor.model}`}
        class="bar-window"
        gdkmonitor={gdkmonitor}
        visible={true}
        exclusivity={Astal.Exclusivity.EXCLUSIVE}
        anchor={anchor}
        application={App}
    >
        <box
            css={`
                background-color: ${colors.m3surfaceContainer};
                padding-top: 15px;
                padding-bottom: 15px;
                padding-left: 4px;
                padding-right: 4px;
                min-width: 50px;
            `}
            orientation={Gtk.Orientation.VERTICAL}
            halign={Gtk.Align.START}
            spacing={6}
        >
            <Logo />
            <Workspaces />
            <box vexpand={true} />
            <ActiveWindow />
            <Media />
            <box vexpand={true} />
            <SysTray />
            <Clock />
            <StatusIcons />
            <Power />
        </box>
    </window>
}
