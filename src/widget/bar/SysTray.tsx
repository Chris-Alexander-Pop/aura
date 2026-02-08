import { Gtk, Gdk } from "ags/gtk4"
import { bind } from "../../lib/utils"
import { createMemo } from "ags"
import Tray from "gi://AstalTray"

export default function SysTray() {
    const tray = Tray.get_default()

    // @ts-ignore
    return <box class="SysTray">
        {bind(tray, "items").as(items => items.map(item => (
            <menubutton
                tooltipMarkup={bind(item, "tooltipMarkup")}
                // @ts-ignore
                popover={undefined}
                // @ts-ignore
                actionGroup={bind(item, "actionGroup").as(ag => ["dbusmenu", ag])}
                menuModel={bind(item, "menuModel")}>
                <Gtk.Image gicon={bind(item, "gicon")} />
            </menubutton>
        )))}
    </box>
}
