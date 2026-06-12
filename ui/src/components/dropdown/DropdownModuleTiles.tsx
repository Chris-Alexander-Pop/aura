import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

const TILE_LABELS: Record<string, string> = {
  network: "Network",
  audio: "Audio",
  bluetooth: "Bluetooth",
  battery: "Battery",
  calendar: "Calendar",
  notifications: "Notifications",
  productivity: "Focus",
}

function TileCard({ tileId }: { tileId: string }) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["dropdown-tile", tileId],
    queryFn: () => api.sidebarGetTileData(tileId),
    staleTime: 8000,
    refetchInterval: 12_000,
  })

  const label = TILE_LABELS[tileId] ?? tileId

  const summary = (() => {
    if (isLoading) return "…"
    if (isError || !data) return "Unavailable"
    const d = data.data
    switch (tileId) {
      case "network":
        return (d.active_connection as string) || ((d.wifi_enabled as boolean) ? "Wi‑Fi on" : "Wi‑Fi off")
      case "audio": {
        const sink = (d.default_sink as { name?: string; volume?: number }) ?? {}
        return sink.name ? `${sink.name} · ${Math.round((sink.volume ?? 0) * 100)}%` : "No output"
      }
      case "bluetooth":
        return (d.powered as boolean) ? `${d.connected_count ?? 0} linked` : "Off"
      case "battery": {
        const b = (d.battery as { percent?: number; charging?: boolean }) ?? {}
        return typeof b.percent === "number" ? `${b.percent}%${b.charging ? " ⚡" : ""}` : "—"
      }
      case "calendar": {
        const events = (d.events as unknown[]) ?? []
        return events.length ? `${events.length} upcoming` : "No events"
      }
      case "notifications": {
        const dnd = (d.dnd as { enabled?: boolean }) ?? {}
        return dnd.enabled ? "DND on" : "Inbox"
      }
      case "productivity":
        return (d.focus_mode_enabled as boolean) ? "Focus on" : "Focus off"
      default:
        return JSON.stringify(d).slice(0, 40)
    }
  })()

  const openPanel = () => {
    if (tileId === "calendar") void api.auraToggleWindow("calendar")
    else void api.auraToggleWindow("control-center")
  }

  return (
    <button
      type="button"
      onClick={openPanel}
      className={cn(
        "glass-card flex min-w-[120px] flex-1 flex-col gap-1 rounded-xl border border-surface0/60 p-3 text-left transition-colors hover:border-mauve/30"
      )}
    >
      <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">{label}</span>
      <span className="truncate text-xs text-text">{summary}</span>
    </button>
  )
}

export default function DropdownModuleTiles({ moduleIds }: { moduleIds: string[] }) {
  if (moduleIds.length === 0) return null
  return (
    <div className="flex flex-wrap gap-2">
      {moduleIds.map((id) => (
        <TileCard key={id} tileId={id} />
      ))}
    </div>
  )
}
