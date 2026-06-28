import { Gtk } from "ags/gtk4"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"
import hyprland from "../../lib/hyprland"

const WORKSPACE_MAX_VISIBLE = 10

function pickVisibleWorkspaces(occupiedIds: number[], activeId: number): number[] {
    const occupied = occupiedIds.filter((id) => id > 0).sort((a, b) => a - b)
    const safeActive = activeId > 0 ? activeId : null

    if (occupied.length === 0) {
        return safeActive ? [safeActive] : []
    }

    const activeIsOccupied = safeActive != null && occupied.includes(safeActive)
    const totalTabs = occupied.length + (safeActive && !activeIsOccupied ? 1 : 0)
    if (totalTabs <= WORKSPACE_MAX_VISIBLE) {
        const ids = new Set(occupied)
        if (safeActive) ids.add(safeActive)
        return [...ids].sort((a, b) => a - b)
    }

    const windowSize = WORKSPACE_MAX_VISIBLE
    const maxId = Math.max(occupied[occupied.length - 1]!, safeActive ?? 1)
    const minStart = 1
    const maxStart = Math.max(1, maxId - windowSize + 1)
    const startMin = safeActive ? Math.max(1, safeActive - windowSize + 1) : minStart
    const startMax = safeActive ? safeActive : maxStart

    let bestStart = startMin
    let bestCount = -1
    for (let start = startMin; start <= startMax; start++) {
        const end = start + windowSize - 1
        const count = occupied.filter((id) => id >= start && id <= end).length
        if (count > bestCount || (count === bestCount && start < bestStart)) {
            bestCount = count
            bestStart = start
        }
    }

    const end = bestStart + windowSize - 1
    const ids = new Set(occupied.filter((id) => id >= bestStart && id <= end))
    if (safeActive) ids.add(safeActive)
    return [...ids].sort((a, b) => a - b)
}

function hyprJson(cmd: string): unknown {
    try {
        const [ok, out] = GLib.spawn_command_line_sync(cmd)
        if (!ok || !out) return null
        return JSON.parse(new TextDecoder().decode(out))
    } catch {
        return null
    }
}

function clientsByWorkspace(): Map<number, number> {
    const counts = new Map<number, number>()
    const raw = hyprJson("hyprctl -j clients")
    if (!Array.isArray(raw)) return counts
    for (const item of raw) {
        const ws = item?.workspace
        const id = typeof ws?.id === "number" ? ws.id : Number(ws?.id)
        if (!Number.isFinite(id) || id <= 0) continue
        counts.set(id, (counts.get(id) ?? 0) + 1)
    }
    return counts
}

export default function Workspaces() {
    const [wsStates, setWsStates] = createState<{ id: number; focused: boolean; occupied: boolean }[]>([])

    onMount(() => {
        const update = () => {
            const focused = hyprland.focusedWorkspaceId
            const counts = clientsByWorkspace()
            const occupiedIds = [...counts.entries()]
                .filter(([, n]) => n > 0)
                .map(([id]) => id)
            const visible = pickVisibleWorkspaces(occupiedIds, focused)
            setWsStates(
                visible.map((id) => ({
                    id,
                    focused: focused === id,
                    occupied: (counts.get(id) ?? 0) > 0,
                }))
            )
        }

        // @ts-ignore
        hyprland.connect('workspaces-changed', update)
        // @ts-ignore
        hyprland.connect('focused-workspace-changed', update)
        update()

        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => { update(); return true })
        onCleanup(() => GLib.source_remove(id))
    })

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
