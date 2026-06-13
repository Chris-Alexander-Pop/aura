import type { QueryClient } from "@tanstack/react-query"
import { connectWs, useWsStore } from "./ws"
import { DROPDOWN_TILE_IDS } from "./dropdown-tiles"

export const notificationQueryKeys = {
  list: ["notifications-list"] as const,
  dnd: ["notification-dnd"] as const,
  rules: ["notification-rules"] as const,
}

function invalidate(qc: QueryClient, queryKey: unknown[]) {
  void qc.invalidateQueries({ queryKey })
}

function invalidateDropdownTiles(qc: QueryClient, tileId?: string) {
  if (tileId) {
    invalidate(qc, ["dropdown-tile", tileId])
    return
  }
  for (const id of DROPDOWN_TILE_IDS) {
    invalidate(qc, ["dropdown-tile", id])
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
      invalidateDropdownTiles(qc, "notifications")
    }),
    useWsStore.getState().on("Dashboard.QuickStatusChanged", () => {
      invalidate(qc, ["dashboard-quick-status"])
    }),
    useWsStore.getState().on("Audio.StateChanged", () => {
      invalidate(qc, ["audio-devices"])
      invalidate(qc, ["audio-streams"])
      invalidateDropdownTiles(qc, "audio")
    }),
    useWsStore.getState().on("Network.StateChanged", () => {
      invalidate(qc, ["net"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateDropdownTiles(qc, "network")
    }),
    useWsStore.getState().on("Bluetooth.StateChanged", () => {
      invalidate(qc, ["bt-ad"])
      invalidate(qc, ["bt-dev"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateDropdownTiles(qc, "bluetooth")
    }),
    useWsStore.getState().on("Calendar.EventsChanged", () => {
      invalidate(qc, ["cal"])
      invalidate(qc, ["calendar-upcoming"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateDropdownTiles(qc, "calendar")
    }),
    useWsStore.getState().on("Power.BatteryState", () => {
      invalidate(qc, ["batt"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateDropdownTiles(qc, "battery")
    }),
    useWsStore.getState().on("Hyprland.StateChanged", () => {
      invalidate(qc, ["hypr-ws"])
      invalidate(qc, ["hypr-active-ws"])
      invalidate(qc, ["clients"])
      invalidate(qc, ["hypr-active"])
    }),
    useWsStore.getState().on("Power.Profile", () => {
      invalidate(qc, ["pwr"])
      invalidate(qc, ["settings", "power-profile"])
      invalidate(qc, ["performance", "power-profile"])
      invalidate(qc, ["dashboard-quick-status"])
      invalidateDropdownTiles(qc, "battery")
    }),
    useWsStore.getState().on("Performance.MetricsChanged", () => {
      invalidate(qc, ["system-stats"])
      invalidate(qc, ["system-stats", "performance-pane"])
      invalidate(qc, ["performance-metrics"])
      invalidate(qc, ["process-top"])
    }),
    useWsStore.getState().on("Productivity.TimerTick", () => {
      invalidateDropdownTiles(qc, "productivity")
    }),
    useWsStore.getState().on("Brightness.StateChanged", () => {
      invalidate(qc, ["brightness"])
      invalidate(qc, ["brightness-monitors"])
    }),
    useWsStore.getState().on("Appearance.NightLightChanged", () => {
      invalidate(qc, ["night-light"])
    }),
    useWsStore.getState().on("Settings.Changed", () => {
      invalidate(qc, ["aura-settings"])
    }),
    useWsStore.getState().on("Tray.Changed", () => {
      invalidate(qc, ["tray-items"])
    }),
  ]
  return () => offs.forEach((off) => off())
}
