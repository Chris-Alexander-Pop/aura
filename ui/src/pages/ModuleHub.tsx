import { useMemo } from "react"
import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  DROPDOWN_TILE_IDS,
  filterDropdownModules,
} from "@/lib/dropdown-tiles"
import { cn } from "@/lib/utils"
import { postPanelHover } from "@/lib/panel-hover"
import { HubQuickToggles } from "@/components/module-hub/HubQuickToggles"
import { HubDashboard } from "@/components/module-hub/HubDashboard"
import { HubQuickviews } from "@/components/module-hub/HubQuickviews"
import HubModuleTiles from "@/components/module-hub/HubModuleTiles"

export default function ModuleHub() {
  const { data: settings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 0,
  })

  const {
    data: quick,
    isLoading: quickLoading,
    isError: quickError,
  } = useQuery({
    queryKey: ["dashboard-quick-status"],
    queryFn: api.dashboardGetQuickStatus,
    refetchInterval: 8000,
  })

  const atLeftEdge = settings?.settings.module_hub_trigger === "left_edge"

  const moduleIds = useMemo(() => {
    const fromQuick = quick?.dropdown_modules
    if (fromQuick && fromQuick.length > 0) return filterDropdownModules(fromQuick)
    const fromSettings = settings?.settings.dropdown_modules
    if (fromSettings && fromSettings.length > 0) return filterDropdownModules(fromSettings)
    return [...DROPDOWN_TILE_IDS]
  }, [quick?.dropdown_modules, settings?.settings.dropdown_modules])

  return (
    <div
      className="h-full min-h-0 w-full"
      onMouseEnter={() => postPanelHover("moduleHubHover", true)}
      onMouseLeave={() => postPanelHover("moduleHubHover", false)}
    >
      <motion.div
        initial={{ opacity: 0, y: -12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ type: "spring", stiffness: 400, damping: 30 }}
        className={cn(
          "scrollbar-thin flex h-full min-h-0 flex-col gap-3 overflow-y-auto overflow-x-hidden bg-mantle p-3 text-text shadow-2xl",
          atLeftEdge
            ? "rounded-r-2xl border border-l-0 border-surface0/60"
            : "rounded-b-2xl border border-t-0 border-surface0/60"
        )}
      >
        <header className="shrink-0">
          <h1 className="text-base font-semibold">Tile hub</h1>
          <p className="text-[11px] text-subtext0">
            Mission control — modules in Control Center → Settings.
          </p>
        </header>

        <HubQuickToggles
          quick={quick}
          quickLoading={quickLoading}
          quickError={quickError}
        />

        <HubDashboard quick={quick} />

        <HubQuickviews />

        <HubModuleTiles moduleIds={moduleIds} />
      </motion.div>
    </div>
  )
}
