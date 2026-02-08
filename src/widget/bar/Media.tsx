import { Gtk } from "ags/gtk4"
import { bind } from "../../lib/utils"
import Mpris from "gi://AstalMpris"

export default function Media() {
    const mpris = Mpris.get_default()

    return <box class="Media" visible={bind(mpris, "players").as(p => p.length > 0)}>
        {/* @ts-ignore */}
        {bind(mpris, "players").as(players => {
            const player = players[0]
            if (!player) return <box />

            return <box spacing={8} orientation={Gtk.Orientation.HORIZONTAL}>
                <label
                    label={bind(player, "title").as(t => t || "Unknown")}
                    maxWidthChars={20}
                    ellipsize={3} // truncating
                />
                <label label="-" />
                <label
                    label={bind(player, "artist").as(a => a || "Unknown")}
                    maxWidthChars={20}
                    ellipsize={3}
                    css="color: #a6adc8;"
                />
            </box>
        })}
    </box>
}
