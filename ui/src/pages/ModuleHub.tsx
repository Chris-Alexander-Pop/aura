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

function ModuleTile({ tileId }: { tileId: string }) {
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
        "glass-card flex min-w-0 flex-col gap-1 rounded-xl border border-surface0/70 p-3 text-left transition-colors hover:border-mauve/35"
      )}
    >
      <span className="text-[10px] font-semibold uppercase tracking-wide text-mauve">{label}</span>
      <span className="line-clamp-2 text-xs text-text">{body}</span>
    </button>
  )
}

export default function ModuleHub() {
  const { data: settings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 0,
  })

  const atLeftEdge = settings?.settings.module_hub_trigger === "left_edge"

  const modules =
    settings?.settings.dropdown_modules?.length
      ? filterDropdownModules(settings.settings.dropdown_modules)
      : [...DROPDOWN_TILE_IDS]

  return (
    <div
      className="h-full min-h-0 w-full"
      onMouseEnter={() => postPanelHover("moduleHubHover", true)}
      onMouseLeave={() => postPanelHover("moduleHubHover", false)}
    >
      <div
        className={cn(
          "flex h-full min-h-0 flex-col gap-2 overflow-y-auto bg-mantle p-3 text-text shadow-2xl",
          atLeftEdge
            ? "rounded-r-2xl border border-l-0 border-surface0/60"
            : "rounded-b-2xl border border-t-0 border-surface0/60"
        )}
      >
        <header className="shrink-0">
          <h1 className="text-base font-semibold">Modules</h1>
          <p className="text-[11px] text-subtext0">
            Tile hub — configure modules in Control Center → Settings.
          </p>
        </header>
        <div className="grid min-h-0 flex-1 grid-cols-2 gap-2 content-start">
          {modules.map((id) => (
            <ModuleTile key={id} tileId={id} />
          ))}
        </div>
      </div>
    </div>
  )
}
