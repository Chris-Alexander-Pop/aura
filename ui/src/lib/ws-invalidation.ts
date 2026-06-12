import type { QueryClient } from "@tanstack/react-query"
import { connectWs, useWsStore } from "./ws"

/** Central WS → react-query invalidation for sidecar push events. */
export function registerSidecarInvalidations(qc: QueryClient): () => void {
  connectWs()
  const offs = [
    useWsStore.getState().on("Todos.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["todos"] })
    }),
    useWsStore.getState().on("Notifications.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["notifications-list"] })
      void qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
    }),
    useWsStore.getState().on("Dashboard.QuickStatusChanged", () => {
      void qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
    }),
    useWsStore.getState().on("Audio.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["audio-devices"] })
      void qc.invalidateQueries({ queryKey: ["audio-streams"] })
    }),
  ]
  return () => offs.forEach((off) => off())
}
