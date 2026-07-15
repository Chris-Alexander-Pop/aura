import type { QueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import type { HyprBarSnapshot } from "@/lib/api-types"

/** Safety-net poll when Hyprland socket events are delayed or unavailable. */
export const HYPRLAND_REFETCH_MS = 4_000

export const hyprlandQueryDefaults = {
  staleTime: 0,
  refetchInterval: HYPRLAND_REFETCH_MS,
} as const

/** Optimistically set active workspace on the focused (or named) monitor. */
export function patchMonitorsActiveWorkspace(
  raw: unknown,
  id: number,
  name: string,
  monitorName?: string | null,
): unknown {
  if (!Array.isArray(raw)) return raw

  const ws = { id, name }
  let matched = false
  const next = raw.map((item) => {
    if (!item || typeof item !== "object") return item
    const m = item as Record<string, unknown>
    const rowName = typeof m.name === "string" ? m.name : ""
    const focused = m.focused === true
    const shouldPatch = monitorName ? rowName === monitorName : focused
    if (!shouldPatch) return item
    matched = true
    return { ...m, active_workspace: ws, activeWorkspace: ws }
  })

  if (matched) return next

  // Single-monitor setups sometimes omit `focused` in cached snapshots.
  if (!monitorName && next.length === 1) {
    const only = next[0]
    if (only && typeof only === "object") {
      return [{ ...(only as Record<string, unknown>), active_workspace: ws, activeWorkspace: ws }]
    }
  }

  return raw
}

export function applyActiveWorkspace(
  qc: QueryClient,
  id: number,
  name?: string,
  monitorName?: string | null,
) {
  const wsName = name ?? String(id)
  qc.setQueryData(["hypr-active-ws"], { id, name: wsName })
  qc.setQueryData(["hypr-monitors"], (old: unknown) =>
    patchMonitorsActiveWorkspace(old, id, wsName, monitorName),
  )
}

export function applyHyprlandSnapshot(qc: QueryClient, snap: HyprBarSnapshot) {
  qc.setQueryData(["hypr-ws"], snap.workspaces)
  if (snap.active_workspace) {
    qc.setQueryData(["hypr-active-ws"], snap.active_workspace)
  }
  qc.setQueryData(["clients"], snap.clients)
  qc.setQueryData(["hypr-active"], snap.active_window)
  if (snap.monitors) {
    qc.setQueryData(["hypr-monitors"], snap.monitors)
  }
}

let snapshotInflight: Promise<void> | null = null
let snapshotQueued = false

/** One hyprctl round-trip — updates all bar hypr query keys. */
export function scheduleHyprlandSnapshotRefresh(qc: QueryClient) {
  if (snapshotInflight) {
    snapshotQueued = true
    return snapshotInflight
  }
  snapshotInflight = api
    .hyprlandGetBarSnapshot()
    .then((snap) => applyHyprlandSnapshot(qc, snap))
    .catch(() => {
      /* compositor unavailable */
    })
    .finally(() => {
      snapshotInflight = null
      if (snapshotQueued) {
        snapshotQueued = false
        void scheduleHyprlandSnapshotRefresh(qc)
      }
    })
  return snapshotInflight
}

export function onHyprlandWorkspaceActive(
  qc: QueryClient,
  id: number,
  name?: string,
  monitorName?: string | null,
) {
  applyActiveWorkspace(qc, id, name, monitorName)
  void scheduleHyprlandSnapshotRefresh(qc)
}
