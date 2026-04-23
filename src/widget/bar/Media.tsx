import { Gtk } from "ags/gtk4"
import { colors, fonts } from "../../lib/theme"
import Mpris from "gi://AstalMpris"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"

export default function Media() {
    const mpris = Mpris.get_default()
    // Both must be plain values — bind().as() memos stringified on some Gtk props (e.g. visible) as "Accessor { }"
    const [tip, setTip] = createState("")
    const [hasPlayers, setHasPlayers] = createState(false)

    onMount(() => {
        const refresh = () => {
            const players = mpris.players || []
            setHasPlayers(players.length > 0)
            const p = players[0]
            setTip(p ? `${p.title || "Unknown"} — ${p.artist || "Unknown"}` : "")
        }
        refresh()
        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1200, () => {
            refresh()
            return true
        })
        onCleanup(() => GLib.source_remove(id))
    })

    return <box
        halign={Gtk.Align.CENTER}
        visible={hasPlayers()}
        css="padding: 2px 0;"
    >
        <label
            label="music_note"
            css={`font-family: '${fonts.material}'; font-size: 18px; color: ${colors.mauve};`}
            tooltipText={tip()}
        />
    </box>
}
