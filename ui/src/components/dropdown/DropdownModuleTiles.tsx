import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import {
  DROPDOWN_TILE_LABELS,
  dropdownTilePane,
  isDropdownTileId,
  sidebarTileSummary,
} from "@/lib/dropdown-tiles"

function TileCard({ tileId }: { tileId: string }) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["dropdown-tile", tileId],
    queryFn: () => api.sidebarGetTileData(tileId),
    staleTime: 0,
    refetchInterval: 8_000,
    enabled: isDropdownTileId(tileId),
  })

  const label = isDropdownTileId(tileId) ? DROPDOWN_TILE_LABELS[tileId] : tileId
  const summary = sidebarTileSummary(tileId, data, { loading: isLoading, error: isError })

  const openPanel = () => {
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
      onClick={openPanel}
      disabled={!isDropdownTileId(tileId)}
      className={cn(
        "glass-card flex min-w-[120px] flex-1 flex-col gap-1 rounded-xl border border-surface0/60 p-3 text-left transition-colors hover:border-mauve/30",
        (!isDropdownTileId(tileId) || isError) && "opacity-60"
      )}
    >
      <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">{label}</span>
      <span className={cn("truncate text-xs text-text", isError && "text-red")}>{summary}</span>
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
