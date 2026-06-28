/**
 * Always-on vertical strip (#/bar) — replaces GTK Bar.tsx when AURA_GTK_BAR is unset.
 */
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  parseHyprActiveWindow,
  parseHyprActiveWorkspace,
  parseHyprClients,
  parseHyprWorkspaces,
  type HyprClient,
} from "@/lib/api-types"
import {
  BAR_SECTION_IDS,
  BAR_STRIP_WIDTH_PX,
  DEFAULT_BAR_SECTION_ORDER,
  HIDDEN_BAR_SECTIONS,
  TOP_BAR_SECTIONS,
  type BarSectionId,
} from "@/components/bar/useBarLayoutStore"
import StatusCluster from "@/components/bar/StatusCluster"
import BarIconButton from "@/components/bar/BarIconButton"
import NotificationToasts from "@/components/notifications/NotificationToasts"
import { type StatusFlyoutId } from "@/components/bar/useFlyoutHover"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"
import { cn } from "@/lib/utils"
import { hyprlandQueryDefaults, useHyprlandSync } from "@/lib/useHyprlandSync"
import { onHyprlandWorkspaceActive } from "@/lib/hyprland-bar-cache"
import { pickVisibleOccupiedWorkspaces } from "@/lib/workspace-visible-range"

/** Post a message to the AGS-registered WebKit message handler. */
function postFlyoutMessage(payload: { open: boolean; panel?: string; y?: number }) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.barFlyout?.postMessage?.(payload)
  } catch { /* non-WebKit context (dev server) */ }
}

function clientWorkspaceId(client: HyprClient): number | null {
  const id = client.workspace?.id
  return typeof id === "number" && Number.isFinite(id) ? id : null
}

function WorkspacesBlock() {
  const qc = useQueryClient()
  const { data: wsRaw, isPending: wsPending } = useQuery({
    queryKey: ["hypr-ws"],
    queryFn: api.hyprlandGetWorkspaces,
    ...hyprlandQueryDefaults,
  })
  const { data: activeRaw, isPending: activePending } = useQuery({
    queryKey: ["hypr-active-ws"],
    queryFn: api.hyprlandGetActiveWorkspace,
    ...hyprlandQueryDefaults,
  })
  const { data: clientsRaw, isPending: clientsPending } = useQuery({
    queryKey: ["clients"],
    queryFn: api.hyprlandGetClients,
    ...hyprlandQueryDefaults,
  })

  const list = useMemo(() => parseHyprWorkspaces(wsRaw), [wsRaw])
  const activeId = useMemo(() => parseHyprActiveWorkspace(activeRaw)?.id ?? null, [activeRaw])
  const clients = useMemo(() => parseHyprClients(clientsRaw), [clientsRaw])
  const clientsByWs = useMemo(() => {
    const byWs = new Map<number, HyprClient[]>()
    for (const client of clients) {
      const id = clientWorkspaceId(client)
      if (id == null) continue
      if (!byWs.has(id)) byWs.set(id, [])
      byWs.get(id)!.push(client)
    }
    return byWs
  }, [clients])

  const ids = useMemo(() => {
    const occupiedIds = [...clientsByWs.entries()]
      .filter(([, wsClients]) => wsClients.length > 0)
      .map(([id]) => id)
    return pickVisibleOccupiedWorkspaces(occupiedIds, activeId)
  }, [clientsByWs, activeId])

  const workspacesLoading =
    (wsPending || activePending || clientsPending) &&
    (list.length === 0 || clientsRaw === undefined)

  if (workspacesLoading) {
    return (
      <div className="flex flex-col items-center gap-0.5 py-0.5" aria-busy="true" aria-label="Loading workspaces">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="h-8 w-8 animate-pulse rounded-full bg-surface1/35" />
        ))}
      </div>
    )
  }

  if (ids.length === 0) return null

  return (
    <div className="flex flex-col items-center gap-0.5 py-0.5">
      {ids.map((id) => {
        const wsClients = clientsByWs.get(id) ?? []
        const windowCount = wsClients.length
        const icons = wsClients.slice(0, 2).map((client) => iconFromHyprClass(client.class))
        return (
          <button
            key={id}
            type="button"
            title={`Workspace ${id} (${windowCount} window${windowCount === 1 ? "" : "s"})`}
            onClick={() => {
              onHyprlandWorkspaceActive(qc, id)
              void api.hyprlandDispatch(`workspace ${id}`)
            }}
            className={cn(
              "group flex h-8 w-8 shrink-0 flex-col items-center justify-center gap-px rounded-full border transition-colors",
              activeId === id
                ? "border-teal bg-teal text-crust"
                : "border-surface2 bg-surface0 text-subtext1 hover:bg-surface1",
            )}
          >
            <span className="text-[8px] font-semibold leading-none">{id}</span>
            <span className="flex items-center justify-center gap-px leading-none">
              {icons.map((icon, index) => (
                <span key={`${icon}-${index}`} className="icon text-[10px] leading-none">
                  {icon}
                </span>
              ))}
            </span>
          </button>
        )
      })}
    </div>
  )
}

/** Compact focused-window indicator (GTK ActiveWindow parity). */
function ActiveWindowBlock() {
  const { data: activeRaw, isPending } = useQuery({
    queryKey: ["hypr-active"],
    queryFn: api.hyprlandGetActiveWindow,
    ...hyprlandQueryDefaults,
  })

  const active = useMemo(() => parseHyprActiveWindow(activeRaw), [activeRaw])

  if (isPending && !active) {
    return (
      <div className="px-0.5 py-0.5" aria-busy="true" aria-label="Loading active window">
        <div className="h-7 w-7 animate-pulse rounded-full bg-surface1/35" />
      </div>
    )
  }

  if (!active) return null

  const title = active.title || active.class || "Window"

  return (
    <div className="flex justify-center px-0.5 py-0.5">
      <BarIconButton
        title={title}
        icon={iconFromHyprClass(active.class)}
        onClick={() => void api.hyprlandDispatch(`focuswindow address:${active.address}`)}
      />
    </div>
  )
}

function MediaBlock() {
  const qc = useQueryClient()
  const { data } = useQuery({
    queryKey: ["media"],
    queryFn: api.getMediaNowPlaying,
    refetchInterval: (query) => (query.state.data?.playing ? 5_000 : 30_000),
  })

  const hasPlayer = Boolean(data?.title || data?.artist || data?.playing || data?.paused)
  if (!hasPlayer) return null

  const toggle = () => {
    void api.mediaPlayPause(data?.player_name ?? undefined).then(() => {
      void qc.invalidateQueries({ queryKey: ["media"] })
    })
  }

  return (
    <div className="flex flex-col items-center gap-0.5 px-0.5 py-1" title={`${data?.title ?? ""} — ${data?.artist ?? ""}`}>
      <BarIconButton
        icon={data?.playing ? "pause" : "play_arrow"}
        className="text-pink hover:bg-pink/15 hover:text-pink"
        onClick={toggle}
        title={data?.playing ? "Pause" : "Play"}
      />
      <div className="flex w-full justify-center gap-1">
        <BarIconButton
          icon="skip_previous"
          size="sm"
          onClick={() => void api.mediaPrevious(data?.player_name ?? undefined).then(() => qc.invalidateQueries({ queryKey: ["media"] }))}
          title="Previous"
        />
        <BarIconButton
          icon="skip_next"
          size="sm"
          onClick={() => void api.mediaNext(data?.player_name ?? undefined).then(() => qc.invalidateQueries({ queryKey: ["media"] }))}
          title="Next"
        />
      </div>
    </div>
  )
}

function PowerBlock() {
  const buttonRef = useRef<HTMLButtonElement>(null)

  return (
    <div className="flex justify-center py-0.5">
      <BarIconButton
        ref={buttonRef}
        icon="power_settings_new"
        title="Power menu"
        className="text-red/85 hover:bg-red/10 hover:text-red"
        onMouseEnter={() => {
          const br = buttonRef.current?.getBoundingClientRect()
          if (br) postFlyoutMessage({ open: true, panel: "power", y: br.top + br.height / 2 })
        }}
        onMouseLeave={() => postFlyoutMessage({ open: false })}
      />
    </div>
  )
}

function ClockBlock() {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const t = window.setInterval(() => setNow(new Date()), 1000)
    return () => window.clearInterval(t)
  }, [])
  const hh = String(now.getHours()).padStart(2, "0")
  const mm = String(now.getMinutes()).padStart(2, "0")

  const openCalendar = () => void api.auraToggleWindow("calendar")

  return (
    <div className="flex w-full flex-col items-center gap-0.5 py-1">
      <BarIconButton
        icon="calendar_month"
        title="Open calendar"
        className="text-teal hover:bg-teal/15 hover:text-teal"
        onClick={openCalendar}
      />
      <button
        type="button"
        title="Open calendar"
        className="flex w-full flex-col items-center font-mono text-[10px] leading-tight text-subtext1 transition-colors hover:text-teal"
        onClick={openCalendar}
      >
        <span>{hh}</span>
        <span>{mm}</span>
      </button>
    </div>
  )
}

function useBarSectionOrder(): BarSectionId[] {
  const { data } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 60_000,
  })
  const raw = data?.settings.bar_section_order
  const base = raw?.length ? raw : DEFAULT_BAR_SECTION_ORDER
  const allowed = new Set<string>(BAR_SECTION_IDS)
  const filtered = base.filter(
    (id): id is BarSectionId => allowed.has(id) && !HIDDEN_BAR_SECTIONS.has(id as BarSectionId)
  )
  return filtered.length > 0 ? filtered : DEFAULT_BAR_SECTION_ORDER
}

function BarSectionGroup({ ids, renderSection }: { ids: BarSectionId[]; renderSection: (id: BarSectionId) => ReactNode }) {
  if (ids.length === 0) return null
  return (
    <>
      {ids.map((id) => (
        <div key={id} className="border-b border-surface0/40 pb-1">
          {renderSection(id)}
        </div>
      ))}
    </>
  )
}

export default function BarStrip() {
  useHyprlandSync()

  useEffect(() => {
    document.documentElement.classList.add("aura-bar-host")
    return () => document.documentElement.classList.remove("aura-bar-host")
  }, [])

  const ordered = useBarSectionOrder()
  const topSections = useMemo(() => ordered.filter((id) => TOP_BAR_SECTIONS.has(id)), [ordered])
  const bottomSections = useMemo(() => ordered.filter((id) => !TOP_BAR_SECTIONS.has(id)), [ordered])

  const onSegmentEnter = (id: StatusFlyoutId, centerY: number) =>
    postFlyoutMessage({ open: true, panel: id, y: centerY })

  const onSegmentLeave = () => postFlyoutMessage({ open: false })

  const renderSection = (id: BarSectionId) => {
    switch (id) {
      case "launcher":
      case "tray":
        return null
      case "workspaces":
        return <WorkspacesBlock />
      case "runningApps":
        return <ActiveWindowBlock />
      case "media":
        return <MediaBlock />
      case "connectivity":
        return <StatusCluster onSegmentEnter={onSegmentEnter} onSegmentLeave={onSegmentLeave} />
      case "calendar":
        return null
      case "clock":
        return <ClockBlock />
      case "power":
        return <PowerBlock />
      default:
        return null
    }
  }

  return (
    <div className="aura-bar-root relative flex h-full min-h-0 w-full flex-row text-text pointer-events-none">
      <NotificationToasts />
      <div
        className="relative z-10 flex min-h-0 shrink-0 flex-col overflow-hidden rounded-r-2xl border border-surface0/80 border-l-transparent bg-mantle pointer-events-auto"
        style={{ width: BAR_STRIP_WIDTH_PX }}
      >
        <div className="flex min-h-0 flex-1 flex-col px-0.5 py-1.5">
          <div className="scrollbar-thin flex min-h-0 shrink-0 flex-col gap-1 overflow-y-auto overflow-x-hidden">
            <BarSectionGroup ids={topSections} renderSection={renderSection} />
          </div>
          <div className="min-h-0 flex-1" aria-hidden />
          <div className="flex shrink-0 flex-col gap-1">
            <BarSectionGroup ids={bottomSections} renderSection={renderSection} />
          </div>
        </div>
      </div>
    </div>
  )
}
