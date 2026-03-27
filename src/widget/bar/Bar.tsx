import { Astal, Gtk, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { colors } from "../../lib/theme"

import Logo from "./Logo"
import Workspaces from "./Workspaces"
import ActiveWindow from "./ActiveWindow"
import Clock from "./Clock"
import StatusIcons from "./StatusIcons"
import Power from "./Power"
import SysTray from "./SysTray"
import Media from "./Media"

export default function Bar(gdkmonitor: Gdk.Monitor) {
    const anchor =
        Astal.WindowAnchor.LEFT |
        Astal.WindowAnchor.TOP |
        Astal.WindowAnchor.BOTTOM

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
                padding-top: 12px;
                padding-bottom: 12px;
                padding-left: 4px;
                padding-right: 4px;
                min-width: 52px;
            `}
            orientation={Gtk.Orientation.VERTICAL}
            halign={Gtk.Align.START}
            spacing={4}
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
