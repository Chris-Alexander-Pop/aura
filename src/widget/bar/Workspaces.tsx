import { Gtk } from "ags/gtk4"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"
import hyprland from "../../lib/hyprland"

export default function Workspaces() {
    const [wsStates, setWsStates] = createState<{ id: number; focused: boolean; occupied: boolean }[]>([])

    onMount(() => {
        const update = () => {
            const ws = hyprland.workspaces || []
            const focused = hyprland.focusedWorkspaceId
            const occupiedIds = new Set(ws.map((w: any) => w.id))
            const maxId = Math.max(5, ...(ws.map((w: any) => w.id || 0)))

            const result = []
            for (let i = 1; i <= Math.min(maxId, 10); i++) {
                result.push({ id: i, focused: focused === i, occupied: occupiedIds.has(i) })
            }
            setWsStates(result)
        }

        // @ts-ignore
        hyprland.connect('workspaces-changed', update)
        // @ts-ignore
        hyprland.connect('focused-workspace-changed', update)
        update()

        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => { update(); return true })
        onCleanup(() => GLib.source_remove(id))
    })

    // Build class names for each dot
    function wsClass(ws: { focused: boolean; occupied: boolean }): string {
        let c = "ws-btn"
        if (ws.focused) c += " focused"
        else if (ws.occupied) c += " occupied"
        return c
    }

    return <box
        orientation={Gtk.Orientation.VERTICAL}
        halign={Gtk.Align.CENTER}
        spacing={0}
    >
        {wsStates().map(ws => (
            <button
                class={wsClass(ws)}
                onClicked={() => hyprland.messageAsync(`dispatch workspace ${ws.id}`)}
                tooltipText={`Workspace ${ws.id}`}
            />
        ))}
    </box>
}
