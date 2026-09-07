import type { QueryClient } from "@tanstack/react-query"
import { shouldApplyBrightnessWsEvent } from "./brightness-session"
import { connectWs, useWsStore } from "./ws"
import { DROPDOWN_TILE_IDS } from "./dropdown-tiles"
import { scheduleHyprlandSnapshotRefresh } from "./hyprland-bar-cache"

export const notificationQueryKeys = {
  list: ["notifications-list"] as const,
  dnd: ["notification-dnd"] as const,
  rules: ["notification-rules"] as const,
}

function invalidate(qc: QueryClient, queryKey: unknown[]) {
  void qc.invalidateQueries({ queryKey })
}

function invalidateModuleTiles(qc: QueryClient, tileId?: string) {
  if (tileId) {
    invalidate(qc, ["dropdown-tile", tileId])
    invalidate(qc, ["sidebar-tile", tileId])
    return
  }
  for (const id of DROPDOWN_TILE_IDS) {
    invalidate(qc, ["dropdown-tile", id])
    invalidate(qc, ["sidebar-tile", id])
  }
}

/** Central WS → react-query invalidation for sidecar push events. */
export function registerSidecarInvalidations(qc: QueryClient): () => void {
  connectWs()
  const offs = [
    useWsStore.getState().on("Todos.Changed", () => {
      invalidate(qc, ["todos"])
    }),
    useWsStore.getState().on("Notifications.Changed", () => {
      invalidate(qc, [...notificationQueryKeys.list])
      invalidate(qc, ["dashboard-quick-status"])
      invalidate(qc, [...notificationQueryKeys.dnd])
      invalidateModuleTiles(qc, "notifications")
    }),
    useWsStore.getState().on("Dashboard.QuickStatusChanged", () => {
      invalidate(qc, ["dashboard-quick-status"])
    }),
    useWsStore.getState().on("Audio.StateChanged", () => {
      invalidate(qc, ["audio-devices"])
      invalidate(qc, ["audio-streams"])
      invalidateModuleTiles(qc, "audio")
    }),
    useWsStore.getState().on("Network.StateChanged", () => {
      invalidate(qc, ["net"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateModuleTiles(qc, "network")
    }),
    useWsStore.getState().on("Bluetooth.StateChanged", () => {
      invalidate(qc, ["bt-ad"])
      invalidate(qc, ["bt-dev"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateModuleTiles(qc, "bluetooth")
    }),
    useWsStore.getState().on("Calendar.EventsChanged", () => {
      invalidate(qc, ["cal"])
      invalidate(qc, ["calendar-upcoming"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateModuleTiles(qc, "calendar")
    }),
    useWsStore.getState().on("Power.BatteryState", () => {
      invalidate(qc, ["batt"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateModuleTiles(qc, "battery")
    }),
    useWsStore.getState().on("Hyprland.StateChanged", () => {
      scheduleHyprlandSnapshotRefresh(qc)
    }),
    useWsStore.getState().on("Power.Profile", () => {
      invalidate(qc, ["pwr"])
      invalidate(qc, ["settings", "power-profile"])
      invalidate(qc, ["performance", "power-profile"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateModuleTiles(qc, "battery")
    }),
    useWsStore.getState().on("Performance.MetricsChanged", () => {
      invalidate(qc, ["system-stats"])
      invalidate(qc, ["system-stats", "performance-pane"])
      invalidate(qc, ["performance-metrics"])
      invalidate(qc, ["process-top"])
    }),
    useWsStore.getState().on("Productivity.TimerTick", () => {
      invalidateModuleTiles(qc, "productivity")
    }),
    useWsStore.getState().on("Brightness.StateChanged", (raw) => {
      if (!raw || typeof raw !== "object") return
      const p = raw as { monitor?: string; brightness?: number; applied?: boolean }
      if (typeof p.brightness !== "number" || typeof p.monitor !== "string") return
      if (p.applied !== false && !shouldApplyBrightnessWsEvent()) return
      const row = { brightness: p.brightness, monitor: p.monitor }
      qc.setQueryData(["brightness", p.monitor], row)
      const monitors = qc.getQueryData<{ monitors?: Array<{ monitor: string }> }>([
        "brightness-monitors",
      ])?.monitors
      const soleDisplay = !monitors?.length || monitors.length === 1
      const activeRow = qc.getQueryData<{ monitor?: string }>(["brightness", "active"])
      const activeTarget = activeRow?.monitor ?? monitors?.[0]?.monitor
      if (soleDisplay || p.monitor === activeTarget || activeTarget == null) {
        qc.setQueryData(["brightness", "active"], row)
      }
    }),
    useWsStore.getState().on("Appearance.NightLightChanged", () => {
      invalidate(qc, ["night-light"])
    }),
    useWsStore.getState().on("Settings.Changed", () => {
      invalidate(qc, ["aura-settings"])
      invalidateModuleTiles(qc)
    }),
    useWsStore.getState().on("Tray.Changed", () => {
      invalidate(qc, ["tray-items"])
    }),
  ]
  return () => offs.forEach((off) => off())
}
