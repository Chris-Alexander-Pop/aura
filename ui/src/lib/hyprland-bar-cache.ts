import type { QueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import type { HyprBarSnapshot } from "@/lib/api-types"

/** Safety-net poll when Hyprland socket events are delayed or unavailable. */
export const HYPRLAND_REFETCH_MS = 4_000

export const hyprlandQueryDefaults = {
  staleTime: 0,
  refetchInterval: HYPRLAND_REFETCH_MS,
} as const

export function applyActiveWorkspace(
  qc: QueryClient,
  id: number,
  name?: string
) {
  qc.setQueryData(["hypr-active-ws"], { id, name: name ?? String(id) })
}

export function applyHyprlandSnapshot(qc: QueryClient, snap: HyprBarSnapshot) {
  qc.setQueryData(["hypr-ws"], snap.workspaces)
  if (snap.active_workspace) {
    qc.setQueryData(["hypr-active-ws"], snap.active_workspace)
  }
  qc.setQueryData(["clients"], snap.clients)
  qc.setQueryData(["hypr-active"], snap.active_window)
}

let snapshotInflight: Promise<void> | null = null

/** One hyprctl round-trip — updates all bar hypr query keys. */
export function scheduleHyprlandSnapshotRefresh(qc: QueryClient) {
  if (snapshotInflight) return snapshotInflight
  snapshotInflight = api
    .hyprlandGetBarSnapshot()
    .then((snap) => applyHyprlandSnapshot(qc, snap))
    .catch(() => {
      /* compositor unavailable */
    })
    .finally(() => {
      snapshotInflight = null
    })
  return snapshotInflight
}

export function onHyprlandWorkspaceActive(
  qc: QueryClient,
  id: number,
  name?: string
) {
  applyActiveWorkspace(qc, id, name)
  void scheduleHyprlandSnapshotRefresh(qc)
}
