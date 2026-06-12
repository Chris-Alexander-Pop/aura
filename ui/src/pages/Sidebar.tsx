import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { useEffect } from "react"

const TILE_IDS = [
  "network",
  "audio",
  "bluetooth",
  "battery",
  "calendar",
  "notifications",
  "productivity",
] as const

function SidebarTile({ tileId }: { tileId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ["sidebar-tile", tileId],
    queryFn: () => api.sidebarGetTileData(tileId),
    staleTime: 10_000,
    refetchInterval: 15_000,
  })

  const title = tileId.charAt(0).toUpperCase() + tileId.slice(1)
  const body =
    isLoading || !data
      ? "Loading…"
      : tileId === "network"
        ? String((data.data.active_connection as string) ?? "—")
        : tileId === "battery"
          ? `${(data.data.battery as { percent?: number })?.percent ?? "—"}%`
          : tileId === "calendar"
            ? `${((data.data.events as unknown[]) ?? []).length} events`
            : "Open panel"

  const onOpen = () => {
    if (tileId === "calendar") void api.auraToggleWindow("calendar")
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
      <span className="text-xs font-semibold uppercase tracking-wide text-mauve">{title}</span>
      <span className="text-sm text-text">{body}</span>
    </button>
  )
}

export default function Sidebar() {
  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Network.StateChanged", () => {})
    return off
  }, [])

  const { data: settings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
  })

  const modules =
    settings?.settings.dropdown_modules?.length
      ? settings.settings.dropdown_modules.filter((m) =>
          (TILE_IDS as readonly string[]).includes(m)
        )
      : [...TILE_IDS]

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto bg-mantle/95 p-4 text-text backdrop-blur-2xl">
      <header>
        <h1 className="text-lg font-semibold">Sidebar</h1>
        <p className="text-xs text-subtext0">Tile hub — configure modules in Control Center → Settings.</p>
      </header>
      <div className="grid grid-cols-2 gap-3">
        {modules.map((id) => (
          <SidebarTile key={id} tileId={id} />
        ))}
      </div>
    </div>
  )
}
