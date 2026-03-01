import { Gtk } from "ags/gtk4"
import { bind } from "../../lib/utils"
import Tray from "gi://AstalTray"

export default function SysTray() {
    const tray = Tray.get_default()

    // @ts-ignore
    return <box
        orientation={Gtk.Orientation.VERTICAL}
        halign={Gtk.Align.CENTER}
        spacing={2}
    >
        {bind(tray, "items").as(items => items.map(item => (
            <menubutton
                class="tray-item"
                tooltipMarkup={bind(item, "tooltipMarkup")}
                // @ts-ignore
                popover={undefined}
                // @ts-ignore
                actionGroup={bind(item, "actionGroup").as(ag => ["dbusmenu", ag])}
                menuModel={bind(item, "menuModel")}>
                <Gtk.Image gicon={bind(item, "gicon")} pixelSize={18} />
            </menubutton>
        )))}
    </box>
}
