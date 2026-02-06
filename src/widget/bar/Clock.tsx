import GLib from "gi://GLib"
import { createState, onMount, onCleanup } from "ags"

export default function Clock() {
    const [time, setTime] = createState("")

    onMount(() => {
        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => {
            setTime(GLib.DateTime.new_now_local().format("%H:%M")!)
            return true
        })
        onCleanup(() => GLib.source_remove(id))
    })

    return <label
        class="Clock"
        label={time()}
    />
}
