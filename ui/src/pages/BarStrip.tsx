/**
 * Always-on vertical strip (#/bar) — replaces GTK Bar.tsx when AURA_GTK_BAR is unset.
 */
import { useEffect, useMemo, useRef, useState } from "react"
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
  DEFAULT_BAR_SECTION_ORDER,
  type BarSectionId,
} from "@/components/bar/useBarLayoutStore"
import StatusCluster from "@/components/bar/StatusCluster"
import { type StatusFlyoutId } from "@/components/bar/useFlyoutHover"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"

const HYPRLAND_REFETCH_MS = 60_000

/** Post a message to the AGS-registered WebKit message handler. */
function postFlyoutMessage(payload: { open: boolean; panel?: string; y?: number }) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.barFlyout?.postMessage?.(payload)
  } catch { /* non-WebKit context (dev server) */ }
}

function useHyprlandInvalidation() {
  const qc = useQueryClient()
  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Hyprland.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["hypr-ws"] })
      void qc.invalidateQueries({ queryKey: ["hypr-active-ws"] })
      void qc.invalidateQueries({ queryKey: ["clients"] })
      void qc.invalidateQueries({ queryKey: ["hypr-active"] })
    })
    return off
  }, [qc])
}

function clientWorkspaceId(client: HyprClient): number | null {
  const id = client.workspace?.id
  return typeof id === "number" && Number.isFinite(id) ? id : null
}

function WorkspacesBlock() {
  useHyprlandInvalidation()

  const { data: wsRaw, isPending: wsPending } = useQuery({
    queryKey: ["hypr-ws"],
    queryFn: api.hyprlandGetWorkspaces,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })
  const { data: activeRaw, isPending: activePending } = useQuery({
    queryKey: ["hypr-active-ws"],
    queryFn: api.hyprlandGetActiveWorkspace,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })
  const { data: clientsRaw, isPending: clientsPending } = useQuery({
    queryKey: ["clients"],
    queryFn: api.hyprlandGetClients,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })

  const list = useMemo(() => parseHyprWorkspaces(wsRaw), [wsRaw])
  const activeId = useMemo(() => parseHyprActiveWorkspace(activeRaw)?.id ?? null, [activeRaw])
  const wsById = useMemo(() => new Map(list.map((w) => [w.id, w])), [list])
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
    const safeActive = activeId && activeId > 0 ? activeId : 1
    const offset = Math.floor((safeActive - 1) / 5) * 5
    return Array.from({ length: 5 }, (_, i) => offset + i + 1)
  }, [activeId])

  const workspacesLoading = (wsPending || activePending) && list.length === 0

  if (workspacesLoading) {
    return (
      <div className="flex flex-col items-center gap-1 py-1" aria-busy="true" aria-label="Loading workspaces">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="h-11 w-full animate-pulse rounded-2xl bg-surface1/35" />
        ))}
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center gap-1 py-1">
      {ids.map((id) => {
        const w = wsById.get(id)
        const wsClients = clientsByWs.get(id) ?? []
        const occupied = wsClients.length > 0 || (w?.windows ?? 0) > 0
        const icons = wsClients.slice(0, 3).map((client) => iconFromHyprClass(client.class))
        return (
          <button
            key={id}
            type="button"
            title={`Workspace ${id}${occupied ? ` (${w?.windows} windows)` : ""}`}
            onClick={() => void api.hyprlandDispatch(`workspace ${id}`)}
            className={cn(
              "group flex min-h-11 w-full flex-col items-center justify-center gap-0.5 rounded-2xl border px-1 py-1 transition-colors",
              activeId === id
                ? "border-teal bg-teal text-crust"
                : occupied
                  ? "border-surface2 bg-surface0 text-subtext1 hover:bg-surface1"
                  : "border-transparent text-overlay0 hover:bg-surface0/70 hover:text-subtext0"
            )}
          >
            <span className="text-[10px] font-semibold leading-none">{id}</span>
            {icons.length > 0 ? (
              <span className="flex max-w-full flex-wrap items-center justify-center gap-0.5">
                {icons.map((icon, index) => (
                  <span key={`${icon}-${index}`} className="icon text-[13px] leading-none">
                    {icon}
                  </span>
                ))}
              </span>
            ) : (
              <span className="h-3" />
            )}
          </button>
        )
      })}
    </div>
  )
}

function RunningAppsBlock() {
  useHyprlandInvalidation()

  const { data: clientsRaw, isPending: clientsPending } = useQuery({
    queryKey: ["clients"],
    queryFn: api.hyprlandGetClients,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })
  const { data: activeRaw } = useQuery({
    queryKey: ["hypr-active"],
    queryFn: api.hyprlandGetActiveWindow,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })

  const activeAddr = parseHyprActiveWindow(activeRaw)?.address ?? ""

  const apps = useMemo(() => {
    const byClass = new Map<string, HyprClient>()
    for (const client of parseHyprClients(clientsRaw)) {
      const address = client.address
      if (!address) continue
      const cls = client.class || "Window"
      const key = cls.toLowerCase()
      if (!byClass.has(key) || address === activeAddr) byClass.set(key, client)
    }
    return [...byClass.values()].slice(0, 7)
  }, [activeAddr, clientsRaw])

  if (clientsPending && apps.length === 0) {
    return (
      <div className="flex flex-col items-center gap-1 px-0.5 py-1" aria-busy="true" aria-label="Loading apps">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-9 w-full animate-pulse rounded-xl bg-surface1/35" />
        ))}
      </div>
    )
  }

  if (apps.length === 0) {
    return (
      <div className="mx-0.5 flex flex-col items-center justify-center rounded-xl border border-surface1/45 bg-surface0/35 px-1 py-3 text-center">
        <span className="icon text-lg leading-none text-overlay0">widgets</span>
        <p className="mt-1.5 text-[9px] leading-tight text-subtext0">No running apps</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center gap-0.5 py-1">
      {apps.map((client) => {
        const addr = client.address
        const cls = client.class
        const title = client.title || cls || "Window"
        const active = addr === activeAddr

        return (
          <button
            key={addr || cls}
            type="button"
            title={title}
            className={cn(
              "flex h-9 w-full items-center justify-center rounded-xl transition-colors",
              active ? "bg-surface1 text-text" : "text-subtext1 hover:bg-surface1/70 hover:text-text"
            )}
            onClick={() => addr && void api.hyprlandDispatch(`focuswindow address:${addr}`)}
          >
            <span className="icon text-[21px]">{iconFromHyprClass(cls)}</span>
          </button>
        )
      })}
    </div>
  )
}

function MediaBlock() {
  const qc = useQueryClient()
  const { data } = useQuery({
    queryKey: ["media"],
    queryFn: api.getMediaNowPlaying,
    refetchInterval: 30_000,
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
      <button
        type="button"
        className="flex h-8 w-full items-center justify-center rounded-lg text-pink hover:bg-surface1/80"
        onClick={toggle}
        title={data?.playing ? "Pause" : "Play"}
      >
        <span className="icon text-lg">{data?.playing ? "pause" : "play_arrow"}</span>
      </button>
      <div className="flex w-full gap-0.5">
        <button
          type="button"
          className="flex flex-1 items-center justify-center rounded-md text-subtext0 hover:bg-surface1/70"
          onClick={() => void api.mediaPrevious(data?.player_name ?? undefined).then(() => qc.invalidateQueries({ queryKey: ["media"] }))}
          title="Previous"
        >
          <span className="icon text-base">skip_previous</span>
        </button>
        <button
          type="button"
          className="flex flex-1 items-center justify-center rounded-md text-subtext0 hover:bg-surface1/70"
          onClick={() => void api.mediaNext(data?.player_name ?? undefined).then(() => qc.invalidateQueries({ queryKey: ["media"] }))}
          title="Next"
        >
          <span className="icon text-base">skip_next</span>
        </button>
      </div>
    </div>
  )
}

function CalendarPreviewBlock() {
  const qc = useQueryClient()
  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Calendar.EventsChanged", () => {
      void qc.invalidateQueries({ queryKey: ["cal"] })
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
    })
    return off
  }, [qc])

  const { data: evs, isPending: calPending } = useQuery({
    queryKey: ["cal"],
    queryFn: () => api.fetchCalendarEvents(),
    refetchInterval: 300_000,
  })
  const preview = Array.isArray(evs) ? evs.slice(0, 2) : []

  return (
    <div className="rounded-md border border-surface1/35 bg-surface0/50 px-1 py-1">
      <button type="button" className="mb-1 flex w-full justify-center text-amber" onClick={() => api.auraToggleWindow("calendar")} title="Open calendar">
        <span className="icon">calendar_month</span>
      </button>
      <div className="max-h-12 overflow-hidden text-[8px] leading-snug text-subtext0">
        {calPending && preview.length === 0 ? (
          <div className="space-y-1 px-0.5 py-1" aria-busy="true">
            <div className="h-2.5 animate-pulse rounded bg-surface1/40" />
            <div className="h-2.5 w-[85%] max-w-full animate-pulse rounded bg-surface1/30" />
          </div>
        ) : preview.length === 0 ? (
          <div className="flex flex-col items-center gap-1 rounded-md border border-surface1/30 bg-mantle/40 px-1 py-2 text-center">
            <span className="icon text-sm text-overlay0">event_busy</span>
            <span>No events soon</span>
          </div>
        ) : (
          preview.map((e, i) => <div key={i}>{String((e as { title?: unknown }).title ?? "—")}</div>)
        )}
      </div>
    </div>
  )
}

function PowerBlock() {
  const buttonRef = useRef<HTMLButtonElement>(null)

  return (
    <div className="flex justify-center py-1">
      <button
        ref={buttonRef}
        type="button"
        className="flex h-10 w-full items-center justify-center rounded-xl text-red/85 transition-colors hover:bg-red/10 hover:text-red"
        title="Power menu"
        onMouseEnter={() => {
          const br = buttonRef.current?.getBoundingClientRect()
          if (br) postFlyoutMessage({ open: true, panel: "power", y: br.top + br.height / 2 })
        }}
        onMouseLeave={() => postFlyoutMessage({ open: false })}
      >
        <span className="icon text-[22px]">power_settings_new</span>
      </button>
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

  return (
    <div className="flex flex-col items-center py-1 font-mono text-[10px] text-subtext1">
      <span className="icon mb-0.5 text-lg text-teal">calendar_month</span>
      <span>{hh}</span>
      <span>{mm}</span>
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
  if (!raw?.length) return DEFAULT_BAR_SECTION_ORDER
  const allowed = new Set<string>(BAR_SECTION_IDS)
  const filtered = raw.filter((id): id is BarSectionId => allowed.has(id))
  return filtered.length > 0 ? filtered : DEFAULT_BAR_SECTION_ORDER
}

export default function BarStrip() {
  useEffect(() => {
    document.documentElement.classList.add("aura-bar-host")
    return () => document.documentElement.classList.remove("aura-bar-host")
  }, [])

  const ordered = useBarSectionOrder()

  const onSegmentEnter = (id: StatusFlyoutId, centerY: number) =>
    postFlyoutMessage({ open: true, panel: id, y: centerY })

  const onSegmentLeave = () => postFlyoutMessage({ open: false })

  const renderSection = (id: BarSectionId) => {
    switch (id) {
      case "workspaces":
        return <WorkspacesBlock />
      case "runningApps":
        return <RunningAppsBlock />
      case "media":
        return <MediaBlock />
      case "connectivity":
        return <StatusCluster onSegmentEnter={onSegmentEnter} onSegmentLeave={onSegmentLeave} />
      case "calendar":
        return <CalendarPreviewBlock />
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
      <div className="relative z-10 flex min-h-0 w-14 shrink-0 flex-col overflow-hidden border-r border-surface0/80 bg-mantle pointer-events-auto">
        <div className="scrollbar-thin flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overflow-x-hidden px-1 py-2">
          {ordered.map((id) => (
            <div key={id} className="border-b border-surface0/40 pb-1">
              {renderSection(id)}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
