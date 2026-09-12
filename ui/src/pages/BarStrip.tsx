/**
 * Always-on vertical strip (#/bar) — replaces GTK Bar.tsx when AURA_GTK_BAR is unset.
 */
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react"
import { LayoutGroup, motion } from "framer-motion"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  parseHyprActiveWindow,
  parseHyprActiveWorkspace,
  parseHyprClients,
  parseHyprMonitors,
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
import SpecialWorkspaceGlyph, { isGrokSpecialWorkspace } from "@/components/bar/SpecialWorkspaceGlyph"
import { cn } from "@/lib/utils"
import { hyprlandQueryDefaults, useHyprlandSync } from "@/lib/useHyprlandSync"
import { onHyprlandWorkspaceActive, scheduleHyprlandSnapshotRefresh } from "@/lib/hyprland-bar-cache"
import { collectOccupiedWorkspaceIds, pickVisibleWorkspaces } from "@/lib/workspace-visible-range"
import { useBarMonitorName } from "@/lib/useBarMonitor"
import {
  specialCoverByWorkspaceId,
  specialWorkspaceCovers,
  specialWorkspaceLabel,
  specialWorkspaceToggleArg,
  type SpecialWorkspaceCover,
} from "@/lib/special-workspace"

/** Post a message to the AGS-registered WebKit message handler. */
function postFlyoutMessage(payload: { open: boolean; panel?: string; y?: number }) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.barFlyout?.postMessage?.(payload)
  } catch { /* non-WebKit context (dev server) */ }
}

function clientWorkspaceId(client: HyprClient): number | null {
  const id = client.workspace?.id
  return typeof id === "number" && Number.isFinite(id) && id > 0 ? id : null
}

const WS_PILL_SPRING = { type: "spring" as const, stiffness: 520, damping: 38, mass: 0.65 }

function WorkspacesBlock() {
  const qc = useQueryClient()
  const barMonitorName = useBarMonitorName()
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
  const { data: monitorsRaw, isPending: monitorsPending } = useQuery({
    queryKey: ["hypr-monitors"],
    queryFn: api.hyprlandGetMonitors,
    ...hyprlandQueryDefaults,
  })

  const list = useMemo(() => parseHyprWorkspaces(wsRaw), [wsRaw])
  const globalActiveId = useMemo(() => parseHyprActiveWorkspace(activeRaw)?.id ?? null, [activeRaw])
  const monitors = useMemo(() => parseHyprMonitors(monitorsRaw), [monitorsRaw])
  const barMonitor = useMemo(
    () => (barMonitorName ? monitors.find((m) => m.name === barMonitorName) ?? null : null),
    [barMonitorName, monitors],
  )
  const monitorActiveId = useMemo(() => {
    // Prefer live `hypr-active-ws` on the focused monitor so Super+# updates
    // the pill from Hyprland.WorkspaceActive before the monitors snapshot.
    if (barMonitor?.focused && globalActiveId != null) return globalActiveId
    const id = barMonitor?.active_workspace?.id
    return typeof id === "number" && id > 0 ? id : globalActiveId
  }, [barMonitor, globalActiveId])
  const specialCovers = useMemo(() => specialCoverByWorkspaceId(specialWorkspaceCovers(monitors)), [monitors])
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

  const visibleWorkspaces = useMemo(() => {
    const occupiedIds = collectOccupiedWorkspaceIds(clientsByWs.keys(), list)
    const visible = pickVisibleWorkspaces(occupiedIds, monitorActiveId).map((id) => ({
      id,
      wsClients: clientsByWs.get(id) ?? [],
      hyprWindows: list.find((ws) => ws.id === id)?.windows ?? 0,
    }))
    for (const wsId of specialCovers.keys()) {
      if (!visible.some((row) => row.id === wsId)) {
        visible.push({
          id: wsId,
          wsClients: clientsByWs.get(wsId) ?? [],
          hyprWindows: list.find((ws) => ws.id === wsId)?.windows ?? 0,
        })
      }
    }
    visible.sort((a, b) => a.id - b.id)
    return visible
  }, [clientsByWs, list, monitorActiveId, specialCovers])

  const hyprPending = wsPending || activePending || clientsPending || monitorsPending
  const workspacesLoading = hyprPending && visibleWorkspaces.length === 0

  if (workspacesLoading) {
    return (
      <div className="flex flex-col items-center gap-2.5 py-1" aria-busy="true" aria-label="Loading workspaces">
        <div className="h-8 w-8 animate-pulse rounded-full bg-surface1/35" />
      </div>
    )
  }

  if (visibleWorkspaces.length === 0) return null

  return (
    <LayoutGroup id="bar-workspaces">
      <div className="flex flex-col items-center gap-2.5 py-1">
        {visibleWorkspaces.map(({ id, wsClients, hyprWindows }) => {
          const windowCount = wsClients.length
          const occupied = windowCount > 0 || hyprWindows > 0
          const icons = wsClients.slice(0, 2).map((client) => iconFromHyprClass(client.class))
          const isActive = monitorActiveId === id
          const specialCover: SpecialWorkspaceCover | undefined = specialCovers.get(id)
          const isCovered = specialCover != null
          const openSpecial = specialCover?.special
          const specialLabel = openSpecial ? specialWorkspaceLabel(openSpecial.name ?? "") : ""
          const specialName = openSpecial?.name ?? ""
          const grokCover = isCovered && isGrokSpecialWorkspace(specialName)
          const specialTitle = openSpecial?.name ?? specialLabel
          const coverMonitor = specialCover?.monitorName
          const isRemoteCover =
            isCovered && coverMonitor != null && barMonitorName != null && coverMonitor !== barMonitorName
          return (
            <motion.button
              key={id}
              type="button"
              initial={false}
              title={
                isCovered
                  ? `Special workspace “${specialLabel}” covering workspace ${id}${coverMonitor ? ` on ${coverMonitor}` : ""} — click to close`
                  : occupied
                    ? `Workspace ${id} (${windowCount || hyprWindows} window${(windowCount || hyprWindows) === 1 ? "" : "s"})`
                    : `Workspace ${id}`
              }
              onClick={() => {
                if (isCovered && openSpecial) {
                  void api.hyprlandDispatch(
                    `togglespecialworkspace ${specialWorkspaceToggleArg(openSpecial.name ?? specialLabel)}`,
                  )
                  void scheduleHyprlandSnapshotRefresh(qc)
                  return
                }
                onHyprlandWorkspaceActive(qc, id, undefined, barMonitorName)
                void api.hyprlandDispatch(`workspace ${id}`)
              }}
              whileTap={{ scale: 0.9 }}
              transition={WS_PILL_SPRING}
              className={cn(
                "relative group flex h-8 w-8 min-h-8 min-w-8 shrink-0 flex-col items-center justify-center gap-px overflow-hidden rounded-full border",
                isActive && !isCovered
                  ? "border-teal bg-teal text-crust"
                  : isCovered
                    ? "border-transparent bg-transparent"
                    : occupied
                      ? "border-surface2 bg-surface0 text-subtext1 hover:bg-surface1"
                      : "border-transparent text-overlay0 hover:bg-surface0/70 hover:text-subtext0",
              )}
            >
              {isActive ? (
                <motion.span
                  layoutId={barMonitorName ? `bar-ws-active-pill-${barMonitorName}` : "bar-ws-active-pill"}
                  initial={false}
                  className={cn(
                    "absolute inset-0 rounded-full border border-teal bg-teal shadow-[0_0_14px_rgb(var(--c-teal)/0.45)]",
                    isCovered ? "z-[15]" : "z-0",
                  )}
                  transition={WS_PILL_SPRING}
                />
              ) : null}
              {isCovered ? (
                <motion.span
                  layoutId={barMonitorName ? `bar-ws-special-${barMonitorName}` : "bar-ws-special"}
                  initial={false}
                  className={cn(
                    "absolute inset-0 z-20 overflow-hidden rounded-full text-crust shadow-[0_0_14px_rgb(var(--c-mauve)/0.35)]",
                    grokCover
                      ? isRemoteCover
                        ? "border-2 border-teal"
                        : "border-2 border-mauve"
                      : isRemoteCover
                        ? "border-2 border-teal bg-mauve shadow-[0_0_16px_rgb(var(--c-teal)/0.5)]"
                        : "border-2 border-mauve bg-mauve",
                  )}
                  transition={WS_PILL_SPRING}
                  aria-label={`Special workspace ${specialTitle} covering workspace ${id}`}
                >
                  <span
                    className={cn(
                      "absolute left-0.5 top-0.5 z-10 text-[7px] font-bold leading-none",
                      grokCover ? "text-white/80" : "text-crust/70",
                    )}
                  >
                    {id}
                  </span>
                  <span className="flex h-full w-full items-center justify-center">
                    <SpecialWorkspaceGlyph name={specialName} />
                  </span>
                </motion.span>
              ) : null}
              <span
                className={cn(
                  "relative z-10 flex flex-col items-center justify-center gap-px",
                  isCovered && "opacity-0",
                  isActive && !isCovered && "text-crust",
                )}
              >
                {occupied && icons.length > 0 ? (
                  <>
                    <span className="text-[8px] font-semibold leading-none">{id}</span>
                    <span className="flex max-w-full items-center justify-center gap-px overflow-hidden leading-none">
                      {icons.map((icon, index) => (
                        <span key={`${icon}-${index}`} className="icon text-[10px] leading-none">
                          {icon}
                        </span>
                      ))}
                    </span>
                  </>
                ) : (
                  <span className="text-[11px] font-semibold leading-none">{id}</span>
                )}
              </span>
            </motion.button>
          )
        })}
      </div>
    </LayoutGroup>
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
    <div className="flex flex-col gap-1">
      {ids.map((id) => (
        <div key={id}>{renderSection(id)}</div>
      ))}
    </div>
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
          <div className="flex shrink-0 flex-col gap-1">
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
