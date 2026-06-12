import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  DROPDOWN_TILE_IDS,
  dropdownTilePane,
  filterDropdownModules,
  isDropdownTileId,
  DROPDOWN_TILE_LABELS,
  sidebarTileSummary,
} from "@/lib/dropdown-tiles"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { useEffect } from "react"

function SidebarTile({ tileId }: { tileId: string }) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["sidebar-tile", tileId],
    queryFn: () => api.sidebarGetTileData(tileId),
    staleTime: 10_000,
    refetchInterval: 15_000,
  })

  const label = isDropdownTileId(tileId) ? DROPDOWN_TILE_LABELS[tileId] : tileId
  const body = sidebarTileSummary(tileId, data, { loading: isLoading, error: isError })

  const onOpen = () => {
    if (!isDropdownTileId(tileId)) return
    if (tileId === "calendar") {
      void api.auraToggleWindow("calendar")
      return
    }
    const pane = dropdownTilePane(tileId)
    if (pane) void api.openControlCenterPane(pane)
    else void api.auraToggleWindow("control-center")
  }

  return (
    <button
      type="button"
      onClick={onOpen}
      className={cn(
        "glass-card flex flex-col gap-2 rounded-2xl border border-surface0/70 p-4 text-left transition-colors hover:border-mauve/35"
      )}
    >
      <span className="text-xs font-semibold uppercase tracking-wide text-mauve">{label}</span>
      <span className="text-sm text-text">{body}</span>
    </button>
  )
}

export default function Sidebar() {
  const queryClient = useQueryClient()

  useEffect(() => {
    connectWs()
    const invalidate = (tile: string) => {
      void queryClient.invalidateQueries({ queryKey: ["sidebar-tile", tile] })
    }
    const offs = [
      useWsStore.getState().on("Network.StateChanged", () => invalidate("network")),
      useWsStore.getState().on("Audio.StateChanged", () => invalidate("audio")),
      useWsStore.getState().on("Bluetooth.StateChanged", () => invalidate("bluetooth")),
      useWsStore.getState().on("Power.BatteryState", () => invalidate("battery")),
      useWsStore.getState().on("Power.Profile", () => invalidate("battery")),
      useWsStore.getState().on("Calendar.EventsChanged", () => invalidate("calendar")),
      useWsStore.getState().on("Notifications.Changed", () => invalidate("notifications")),
      useWsStore.getState().on("Productivity.TimerTick", () => invalidate("productivity")),
    ]
    return () => offs.forEach((off) => off())
  }, [queryClient])

  const { data: settings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
  })

  const modules =
    settings?.settings.dropdown_modules?.length
      ? filterDropdownModules(settings.settings.dropdown_modules)
      : [...DROPDOWN_TILE_IDS]

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto bg-mantle/95 p-4 text-text backdrop-blur-2xl">
      <header>
        <h1 className="text-lg font-semibold">Sidebar</h1>
        <p className="text-xs text-subtext0">
          Tile hub — configure modules in Control Center → Settings.
        </p>
      </header>
      <div className="grid grid-cols-2 gap-3">
        {modules.map((id) => (
          <SidebarTile key={id} tileId={id} />
        ))}
      </div>
    </div>
  )
}
