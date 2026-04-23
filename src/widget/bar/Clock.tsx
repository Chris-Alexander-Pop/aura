import GtkGi from "gi://Gtk?version=4.0"
import GLib from "gi://GLib"
import { Gtk } from "ags/gtk4"
import { colors, fonts } from "../../lib/theme"

/** Drive labels via Gtk APIs — createState()+label={} does not reliably re-render timers in this shell. */

export default function Clock() {
    const iconCss = `font-family: '${fonts.material}'; font-size: 18px; color: ${colors.m3tertiary};`
    const monoCss = `font-family: '${fonts.mono}', monospace; font-size: 12px; color: ${colors.m3tertiary};`

    return (
        <box
            orientation={Gtk.Orientation.VERTICAL}
            halign={Gtk.Align.CENTER}
            spacing={2}
            css="padding: 4px 0;"
        >
            <label label="calendar_month" css={iconCss} />
            <label
                css={monoCss}
                $={(self: GtkGi.Label) => {
                    const tick = () => {
                        const d = new Date()
                        self.label = `${String(d.getHours()).padStart(2, "0")}\n${String(d.getMinutes()).padStart(2, "0")}`
                    }
                    tick()
                    const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => {
                        tick()
                        return true
                    })
                    self.connect("destroy", () => GLib.source_remove(id))
                }}
            />
        </box>
    )
}
