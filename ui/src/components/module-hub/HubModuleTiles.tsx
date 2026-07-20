import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import {
  DROPDOWN_TILE_LABELS,
  dropdownTilePane,
  isDropdownTileId,
  sidebarTileSummary,
  type DropdownTileId,
} from "@/lib/dropdown-tiles"

/** Tiles that get a larger card in the hub grid. */
const LARGE_TILES = new Set<DropdownTileId>(["calendar", "notifications"])

function TileCard({ tileId, large }: { tileId: string; large?: boolean }) {
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
        "flex min-w-0 flex-col gap-0.5 rounded-xl bg-surface0/35 p-2.5 text-left transition-colors hover:bg-surface0/55",
        large && "sm:col-span-2",
        (!isDropdownTileId(tileId) || isError) && "opacity-60"
      )}
    >
      <span className="text-[10px] font-semibold uppercase tracking-wide text-mauve">{label}</span>
      <span
        className={cn(
          "text-xs text-text",
          large ? "line-clamp-3" : "truncate",
          isError && "text-red"
        )}
      >
        {summary}
      </span>
    </button>
  )
}

export default function HubModuleTiles({
  moduleIds,
  showSectionHeader = true,
}: {
  moduleIds: string[]
  showSectionHeader?: boolean
}) {
  if (moduleIds.length === 0) {
    return showSectionHeader ? (
      <p className="px-0.5 text-[11px] text-subtext0">
        No modules enabled — configure in Control Center → Settings.
      </p>
    ) : null
  }

  return (
    <section className="flex flex-col gap-1.5">
      {showSectionHeader ? (
        <h2 className="px-0.5 text-[10px] font-semibold uppercase tracking-wide text-subtext1">
          Modules
        </h2>
      ) : null}
      <div className="grid grid-cols-2 gap-1.5">
        {moduleIds.map((id) => (
          <TileCard
            key={id}
            tileId={id}
            large={isDropdownTileId(id) && LARGE_TILES.has(id)}
          />
        ))}
      </div>
    </section>
  )
}
