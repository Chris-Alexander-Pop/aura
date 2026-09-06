import { Gtk } from "ags/gtk4"
import { colors, fonts } from "../../lib/theme"

export default function Logo() {
    return <box halign={Gtk.Align.CENTER} css="padding: 4px 0;">
        <label
            label="deployed_code"
            css={`font-family: '${fonts.material}'; font-size: 22px; color: ${colors.m3tertiary};`}
        />
    </box>
}
