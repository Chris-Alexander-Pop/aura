import { Gtk } from "ags/gtk4"
import { colors, fonts } from "../../lib/theme"

export default function Power() {
    return <box halign={Gtk.Align.CENTER}>
        <button class="power-btn"
            onClicked={() => console.log("Power — session menu (Phase 2)")}
        >
            <label
                label="power_settings_new"
                css={`font-family: '${fonts.material}'; font-size: 18px; color: ${colors.m3error}; font-weight: bold;`}
            />
        </button>
    </box>
}
