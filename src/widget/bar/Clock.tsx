import { Gtk } from "ags/gtk4"
import GLib from "gi://GLib"
import { createState, onMount, onCleanup } from "ags"
import { colors, fonts } from "../../lib/theme"

export default function Clock() {
    const [hours, setHours] = createState("00")
    const [minutes, setMinutes] = createState("00")

    onMount(() => {
        const update = () => {
            const now = GLib.DateTime.new_now_local()
            setHours(now.format("%H") || "00")
            setMinutes(now.format("%M") || "00")
        }
        update()
        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => { update(); return true })
        onCleanup(() => GLib.source_remove(id))
    })

    return <box
        orientation={Gtk.Orientation.VERTICAL}
        halign={Gtk.Align.CENTER}
        spacing={2}
        css="padding: 4px 0;"
    >
        <label
            label="calendar_month"
            css={`font-family: '${fonts.material}'; font-size: 18px; color: ${colors.m3tertiary};`}
        />
        <label
            label={hours()}
            css={`font-family: '${fonts.mono}', monospace; font-size: 12px; color: ${colors.m3tertiary};`}
        />
        <label
            label={minutes()}
            css={`font-family: '${fonts.mono}', monospace; font-size: 12px; color: ${colors.m3tertiary};`}
        />
    </box>
}
