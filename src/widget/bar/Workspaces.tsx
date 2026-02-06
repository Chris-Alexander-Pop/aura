import { Gtk } from "ags/gtk4"
import { For, createState, onMount, createMemo } from "ags"
import hyprland from "../../lib/hyprland"

export default function Workspaces() {
    const [workspaces, setWorkspaces] = createState(hyprland.workspaces)
    const [focusedId, setFocusedId] = createState(hyprland.focusedWorkspaceId)

    onMount(() => {
        // @ts-ignore
        hyprland.connect('workspaces-changed', () => {
            setWorkspaces(hyprland.workspaces)
        })
        // @ts-ignore
        hyprland.connect('focused-workspace-changed', (_, id) => {
            setFocusedId(id)
        })
    })

    // Sort workspaces
    const sortedWorkspaces = createMemo(() => [...workspaces()].sort((a, b) => a.id - b.id))

    return <box class="Workspaces">
        <For each={sortedWorkspaces}>
            {(w: any) => (
                <button
                    onClicked={() => hyprland.messageAsync(`workspace ${w.id}`)}
                    class={createMemo(() => focusedId() === w.id ? "focused" : "")}
                >
                    <label label={String(w.id)} />
                </button>
            )}
        </For>
    </box>
}
