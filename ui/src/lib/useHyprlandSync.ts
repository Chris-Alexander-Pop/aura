import { useEffect } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { connectWs, useWsStore } from "@/lib/ws"
import {
  onHyprlandWorkspaceActive,
  scheduleHyprlandSnapshotRefresh,
} from "@/lib/hyprland-bar-cache"

export {
  HYPRLAND_REFETCH_MS,
  hyprlandQueryDefaults,
} from "@/lib/hyprland-bar-cache"

/** Live Hyprland updates for the bar (keyboard workspace switches + socket events). */
export function useHyprlandSync() {
  const qc = useQueryClient()

  useEffect(() => {
    connectWs()

    const onAuraWorkspace = (e: Event) => {
      const { id } = (e as CustomEvent<{ id: number }>).detail
      if (typeof id === "number" && id > 0) onHyprlandWorkspaceActive(qc, id)
    }
    window.addEventListener("aura-hypr-workspace", onAuraWorkspace)

    const offs = [
      useWsStore.getState().on("Hyprland.WorkspaceActive", (params) => {
        const p = params as { id?: number; name?: string }
        if (typeof p.id === "number" && p.id > 0) {
          onHyprlandWorkspaceActive(qc, p.id, p.name)
        }
      }),
      useWsStore.getState().on("Hyprland.StateChanged", () => {
        void scheduleHyprlandSnapshotRefresh(qc)
      }),
    ]

    return () => {
      window.removeEventListener("aura-hypr-workspace", onAuraWorkspace)
      offs.forEach((off) => off())
    }
  }, [qc])
}
