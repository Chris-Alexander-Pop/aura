import { Gtk } from "ags/gtk4"
import GLib from "gi://GLib"
import Tray from "gi://AstalTray"
import { createState, onMount, onCleanup } from "ags"

/** No bind()/memo on Gtk props — reactive memos stringify as **Accessor { }**. Poll tray.items + plain GObject fields. */

export default function SysTray() {
    const tray = Tray.get_default()
    const [items, setItems] = createState<any[]>([])

    const refresh = () => {
        try {
            const list = tray.items as any[] | undefined
            setItems(list ? [...list] : [])
        } catch {
            setItems([])
        }
    }

    onMount(() => {
        refresh()
        const tick = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 750, () => {
            refresh()
            return true
        })
        let nid = 0
        try {
            nid = tray.connect("notify::items", refresh)
        } catch {
            /* tray may not emit on this build */
        }
        onCleanup(() => {
            GLib.source_remove(tick)
            if (nid) tray.disconnect(nid)
        })
    })

    return (
        <box orientation={Gtk.Orientation.VERTICAL} halign={Gtk.Align.CENTER} spacing={2}>
            {items().map((item: any) => {
                const ag = item.actionGroup
                const dbusPair = ag ? (["dbusmenu", ag] as const) : undefined
                return (
                    <menubutton
                        class="tray-item"
                        // @ts-ignore — Astal tray + dbusmenu
                        popover={undefined}
                        // @ts-ignore
                        actionGroup={dbusPair as any}
                        // @ts-ignore
                        menuModel={item.menuModel}
                    >
                        <Gtk.Image gicon={item.gicon} pixelSize={18} />
                    </menubutton>
                )
            })}
        </box>
    )
}
