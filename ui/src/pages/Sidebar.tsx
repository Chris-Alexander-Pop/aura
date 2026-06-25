import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  DROPDOWN_TILE_IDS,
  dropdownTilePane,
  filterDropdownModules,
  isDropdownTileId,
  DROPDOWN_TILE_LABELS,
  sidebarTileSummary,
} from "@/lib/dropdown-tiles"
import { cn } from "@/lib/utils"
import { postPanelHover } from "@/lib/panel-hover"

function SidebarTile({ tileId }: { tileId: string }) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["sidebar-tile", tileId],
    queryFn: () => api.sidebarGetTileData(tileId),
    staleTime: 0,
    refetchInterval: 8_000,
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
  const { data: settings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 0,
  })

  const modules =
    settings?.settings.dropdown_modules?.length
      ? filterDropdownModules(settings.settings.dropdown_modules)
      : [...DROPDOWN_TILE_IDS]

  return (
    <div
      className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto bg-mantle/95 p-4 text-text backdrop-blur-2xl"
      onMouseEnter={() => postPanelHover("sidebarHover", true)}
      onMouseLeave={() => postPanelHover("sidebarHover", false)}
    >
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
