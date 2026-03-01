import { Gtk } from "ags/gtk4"
import { bind } from "../../lib/utils"
import { colors, fonts } from "../../lib/theme"
import Mpris from "gi://AstalMpris"

export default function Media() {
    const mpris = Mpris.get_default()

    return <box
        halign={Gtk.Align.CENTER}
        visible={bind(mpris, "players").as((p: any[]) => p.length > 0)}
        css="padding: 2px 0;"
    >
        <label
            label="music_note"
            css={`font-family: '${fonts.material}'; font-size: 18px; color: ${colors.mauve};`}
            tooltipText={bind(mpris, "players").as((players: any[]) => {
                const p = players[0]
                if (!p) return "No media"
                return `${p.title || "Unknown"} - ${p.artist || "Unknown"}`
            })}
        />
    </box>
}
