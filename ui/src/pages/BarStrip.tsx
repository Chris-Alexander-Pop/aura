/**
 * Always-on vertical strip (#/bar) — replaces GTK Bar.tsx when AURA_GTK_BAR is unset.
 */
import { useEffect, useMemo, useRef, useState } from "react"
import {
  DndContext,
  PointerSensor,
  closestCenter,
  type DragEndEvent,
  useSensor,
  useSensors,
} from "@dnd-kit/core"
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  BAR_SECTION_IDS,
  DEFAULT_BAR_SECTION_ORDER,
  type BarSectionId,
  useBarLayoutStore,
} from "@/components/bar/useBarLayoutStore"
import StatusCluster from "@/components/bar/StatusCluster"
import BarFlyoutLayer from "@/components/bar/BarFlyoutLayer"
import NetworkFlyout from "@/components/bar/flyouts/NetworkFlyout"
import BluetoothFlyout from "@/components/bar/flyouts/BluetoothFlyout"
import BatteryFlyout from "@/components/bar/flyouts/BatteryFlyout"
import WindowsFlyout from "@/components/bar/flyouts/WindowsFlyout"
import { useFlyoutHover, type StatusFlyoutId } from "@/components/bar/useFlyoutHover"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"
import { cn } from "@/lib/utils"

/** Hyprland `workspaces -j` entries — `windows` count when present */
function parseWorkspaces(data: unknown): { id: number; name?: string; windows: number }[] {
  if (!Array.isArray(data)) return []
  return data
    .filter((x): x is Record<string, unknown> => x != null && typeof x === "object")
    .map((x) => ({
      id: Number(x.id),
      name: typeof x.name === "string" ? x.name : undefined,
      windows: typeof x.windows === "number" ? x.windows : 0,
    }))
    .filter((x) => Number.isFinite(x.id))
}

function parseActiveWsId(data: unknown): number | null {
  if (data && typeof data === "object" && "id" in data) {
    const id = Number((data as { id: unknown }).id)
    return Number.isFinite(id) ? id : null
  }
  return null
}

function SortableSection({
  id,
  children,
}: {
  id: BarSectionId
  children: React.ReactNode
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id })
  const style = { transform: CSS.Transform.toString(transform), transition }

  return (
    <div ref={setNodeRef} style={style} className={cn("relative", isDragging && "opacity-60 z-10")}>
      <button
        type="button"
        className="absolute -left-0.5 top-1 z-20 h-4 w-3 cursor-grab text-[8px] text-subtext0 opacity-40 hover:opacity-100"
        {...attributes}
        {...listeners}
        aria-label="Reorder section"
      >
        ⋮
      </button>
      {children}
    </div>
  )
}

function LogoBlock() {
  return (
    <div className="flex justify-center py-1">
      <span className="icon text-xl text-teal">deployed_code</span>
    </div>
  )
}

function WorkspacesBlock() {
  const { data: wsRaw } = useQuery({ queryKey: ["hypr-ws"], queryFn: api.hyprlandGetWorkspaces, refetchInterval: 1500 })
  const { data: activeRaw } = useQuery({
    queryKey: ["hypr-active-ws"],
    queryFn: api.hyprlandGetActiveWorkspace,
    refetchInterval: 1500,
  })

  const list = useMemo(() => parseWorkspaces(wsRaw), [wsRaw])
  const activeId = useMemo(() => parseActiveWsId(activeRaw), [activeRaw])
  const wsById = useMemo(() => new Map(list.map((w) => [w.id, w])), [list])

  const maxShow = 10
  const ids = useMemo(() => {
    const raw = list.map((w) => w.id).filter((id) => id >= 1 && id <= maxShow)
    if (raw.length === 0) return Array.from({ length: 5 }, (_, i) => i + 1)
    return raw.slice(0, maxShow)
  }, [list])

  return (
    <div className="flex flex-col items-center gap-0.5 py-1">
      {ids.map((id) => {
        const w = wsById.get(id)
        const occupied = (w?.windows ?? 0) > 0
        return (
          <button
            key={id}
            type="button"
            title={`Workspace ${id}${occupied ? ` (${w?.windows} windows)` : ""}`}
            onClick={() => api.hyprlandDispatch(`workspace ${id}`)}
            className={cn(
              "h-2 w-2 rounded-full border transition-colors",
              activeId === id
                ? "scale-125 border-teal bg-teal"
                : occupied
                  ? "border-teal/40 bg-overlay1 ring-1 ring-teal/25 hover:bg-overlay2"
                  : "border-surface2 bg-overlay0 hover:bg-overlay1"
            )}
          />
        )
      })}
    </div>
  )
}

function ActiveWindowBlock() {
  const { data: raw } = useQuery({ queryKey: ["hypr-active"], queryFn: api.hyprlandGetActiveWindow, refetchInterval: 1000 })

  const { icon, tip } = useMemo(() => {
    if (!raw || typeof raw !== "object") return { icon: "window", tip: "" }
    const o = raw as Record<string, unknown>
    const cls = String(o.class ?? "")
    const title = String(o.title ?? o.class ?? "")
    return { icon: iconFromHyprClass(cls), tip: title || cls }
  }, [raw])

  if (!tip) return null

  return (
    <div className="flex justify-center py-1" title={tip}>
      <span className="icon text-lg text-subtext1">{icon}</span>
    </div>
  )
}

function MediaBlock() {
  const { data } = useQuery({ queryKey: ["media"], queryFn: api.getMediaNowPlaying, refetchInterval: 2000 })
  if (!data?.playing) return null
  return (
    <div className="px-0.5 py-1 text-center" title={`${data.title} — ${data.artist}`}>
      <span className="icon text-lg text-pink">music_note</span>
    </div>
  )
}

function TrayNoteBlock() {
  return (
    <div className="rounded-md bg-surface0/60 px-0.5 py-1 text-center">
      <p className="text-[7px] leading-tight text-subtext0">StatusNotifier tray not in WebKit yet</p>
    </div>
  )
}

function CalendarPreviewBlock() {
  const { data: evs } = useQuery({ queryKey: ["cal"], queryFn: api.getCalendarEvents, refetchInterval: 60_000 })
  const preview = Array.isArray(evs) ? evs.slice(0, 2) : []

  return (
    <div className="rounded-md bg-surface0/50 px-1 py-1">
      <button type="button" className="mb-1 flex w-full justify-center text-amber" onClick={() => api.auraToggleWindow("calendar")} title="Open calendar">
        <span className="icon">calendar_month</span>
      </button>
      <div className="max-h-12 overflow-hidden text-[8px] leading-snug text-subtext0">
        {preview.length === 0 ? <span>No events</span> : preview.map((e, i) => <div key={i}>{String((e as Record<string, unknown>)?.title ?? "—")}</div>)}
      </div>
    </div>
  )
}

function SystemStatsBlock() {
  const { data: s } = useQuery({ queryKey: ["stats"], queryFn: api.getSystemStats, refetchInterval: 3000 })
  if (!s) return null
  return (
    <div className="rounded-md bg-surface0/40 px-0.5 py-1 font-mono text-[8px] text-subtext0">
      <div>CPU {(s.cpu * 100).toFixed(0)}%</div>
      <div>RAM {(s.ram * 100).toFixed(0)}%</div>
      <div>{s.temp.toFixed(0)}°C</div>
    </div>
  )
}

function WindowHubBlock() {
  return (
    <p className="rounded-md bg-surface0/50 px-1 py-1.5 text-center text-[8px] leading-snug text-subtext0">
      Window list lives in the status stack: hover the{" "}
      <span className="icon align-middle text-[10px] text-subtext1">layers</span> icon next to battery.
    </p>
  )
}

function TaskMgrBlock() {
  const [confirmPid, setConfirmPid] = useState<number | null>(null)
  const { data: rows } = useQuery({ queryKey: ["procs"], queryFn: () => api.processListTop(12), refetchInterval: 3000 })

  const kill = async () => {
    if (confirmPid == null) return
    await api.processKill(confirmPid)
    setConfirmPid(null)
  }

  return (
    <div className="flex flex-col gap-0.5 py-1">
      {(rows ?? []).map((r) => (
        <div key={r.pid} className="flex items-center justify-between gap-0.5 text-[8px]">
          <span className="truncate text-subtext0">{r.name.slice(0, 10)}</span>
          <button type="button" className="text-red/80" onClick={() => setConfirmPid(r.pid)}>
            ×
          </button>
        </div>
      ))}
      {confirmPid != null && (
        <div className="rounded-md bg-red/20 p-1 text-[8px]">
          Kill PID {confirmPid}?
          <div className="mt-1 flex gap-1">
            <button type="button" className="rounded bg-text px-1 text-base" onClick={kill}>
              OK
            </button>
            <button type="button" className="rounded bg-surface2 px-1 text-base" onClick={() => setConfirmPid(null)}>
              No
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

const LAUNCHER_APPS: { id: string; icon: string; label: string }[] = [
  { id: "terminal", icon: "terminal", label: "Term" },
  { id: "browser", icon: "public", label: "Web" },
  { id: "files", icon: "folder", label: "Files" },
  { id: "code", icon: "code", label: "IDE" },
  { id: "music", icon: "music_note", label: "Mu" },
  { id: "discord", icon: "chat", label: "Chat" },
]

function LauncherBlock() {
  return (
    <div className="grid grid-cols-2 gap-1 py-1">
      {LAUNCHER_APPS.map((a) => (
        <button
          key={a.id}
          type="button"
          title={a.label}
          onClick={() => api.appsLaunch(a.id)}
          className="icon-btn flex aspect-square flex-col items-center justify-center rounded-lg bg-surface0/50 p-0.5"
        >
          <span className="icon text-base">{a.icon}</span>
        </button>
      ))}
    </div>
  )
}

function PowerBlock() {
  const [destructive, setDestructive] = useState<null | "reboot" | "poweroff">(null)

  const run = async () => {
    if (destructive === "reboot") await api.sessionReboot()
    if (destructive === "poweroff") await api.sessionPowerOff()
    setDestructive(null)
  }

  return (
    <div className="flex flex-col gap-1 py-1">
      <button type="button" className="icon-btn rounded-lg py-1" title="Lock" onClick={() => api.sessionLock()}>
        <span className="icon text-lg">lock</span>
      </button>
      <button type="button" className="icon-btn rounded-lg py-1" title="Log out" onClick={() => api.sessionLogout()}>
        <span className="icon text-lg">logout</span>
      </button>
      <button type="button" className="icon-btn rounded-lg py-1" title="Suspend" onClick={() => api.sessionSuspend()}>
        <span className="icon text-lg">bedtime</span>
      </button>
      <button type="button" className="icon-btn rounded-lg py-1 text-red/80" title="Reboot" onClick={() => setDestructive("reboot")}>
        <span className="icon text-lg">restart_alt</span>
      </button>
      <button type="button" className="icon-btn rounded-lg py-1 text-red/80" title="Power off" onClick={() => setDestructive("poweroff")}>
        <span className="icon text-lg">power_settings_new</span>
      </button>
      <button type="button" className="icon-btn rounded-lg py-1" title="Control center" onClick={() => api.auraToggleWindow("control-center")}>
        <span className="icon text-lg">tune</span>
      </button>
      {destructive && (
        <div className="rounded-md bg-red/20 p-1 text-[8px]">
          Confirm {destructive}?
          <div className="mt-1 flex gap-1">
            <button type="button" className="rounded bg-text px-1 text-crust" onClick={run}>
              Yes
            </button>
            <button type="button" className="rounded bg-surface2 px-1" onClick={() => setDestructive(null)}>
              No
            </button>
          </div>
        </div>
      )}
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

function MoreBlock() {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="flex flex-col gap-1 py-1">
      <button
        type="button"
        className="rounded-lg bg-surface0/60 py-1.5 text-center text-[10px] text-subtext0 hover:bg-surface1"
        onClick={() => setExpanded((e) => !e)}
        aria-expanded={expanded}
      >
        {expanded ? "Less" : "More…"}
      </button>
      {expanded && (
        <div className="flex flex-col gap-2 border-t border-surface0/50 pt-2">
          <TrayNoteBlock />
          <SystemStatsBlock />
          <WindowHubBlock />
          <TaskMgrBlock />
        </div>
      )}
    </div>
  )
}

/** Prefer store order; ignore unknown ids; empty → baked-in sparse default */
function sanitizeOrder(raw: BarSectionId[]): BarSectionId[] {
  const allowed = new Set<string>(BAR_SECTION_IDS as unknown as string[])
  const seen = new Set<string>()
  const out: BarSectionId[] = []
  for (const id of raw) {
    if (allowed.has(id) && !seen.has(id)) {
      seen.add(id)
      out.push(id)
    }
  }
  if (out.length === 0) return [...DEFAULT_BAR_SECTION_ORDER]
  return out
}

export default function BarStrip() {
  const sectionOrder = useBarLayoutStore((s) => s.sectionOrder)
  const setSectionOrder = useBarLayoutStore((s) => s.setSectionOrder)

  const stripRootRef = useRef<HTMLDivElement>(null)
  const flyout = useFlyoutHover()
  const [anchorCenterY, setAnchorCenterY] = useState<number | null>(null)

  useEffect(() => {
    document.documentElement.classList.add("aura-bar-host")
    return () => document.documentElement.classList.remove("aura-bar-host")
  }, [])

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 6 } }))

  const onDragEnd = (e: DragEndEvent) => {
    const { active, over } = e
    if (!over || active.id === over.id) return
    const ordered = sanitizeOrder(sectionOrder)
    const oldIndex = ordered.indexOf(active.id as BarSectionId)
    const newIndex = ordered.indexOf(over.id as BarSectionId)
    if (oldIndex < 0 || newIndex < 0) return
    setSectionOrder(arrayMove(ordered, oldIndex, newIndex))
  }

  const ordered = useMemo(() => sanitizeOrder(sectionOrder), [sectionOrder])

  const onSegmentEnter = (id: StatusFlyoutId, centerY: number) => {
    setAnchorCenterY(centerY)
    flyout.open(id)
  }

  const onSegmentLeave = () => flyout.scheduleClose()

  const flyoutChild = (id: StatusFlyoutId) => {
    switch (id) {
      case "network":
        return <NetworkFlyout />
      case "bluetooth":
        return <BluetoothFlyout />
      case "battery":
        return <BatteryFlyout />
      case "windows":
        return <WindowsFlyout />
      default:
        return null
    }
  }

  const renderSection = (id: BarSectionId) => {
    switch (id) {
      case "logo":
        return <LogoBlock />
      case "workspaces":
        return <WorkspacesBlock />
      case "activeWindow":
        return <ActiveWindowBlock />
      case "media":
        return <MediaBlock />
      case "trayNote":
        return <TrayNoteBlock />
      case "connectivity":
        return <StatusCluster stripRootRef={stripRootRef} onSegmentEnter={onSegmentEnter} onSegmentLeave={onSegmentLeave} />
      case "calendar":
        return <CalendarPreviewBlock />
      case "systemStats":
        return <SystemStatsBlock />
      case "windowHub":
        return <WindowHubBlock />
      case "taskMgr":
        return <TaskMgrBlock />
      case "launcher":
        return <LauncherBlock />
      case "clock":
        return <ClockBlock />
      case "power":
        return <PowerBlock />
      case "more":
        return <MoreBlock />
      default:
        return null
    }
  }

  return (
    <div className="aura-bar-root relative flex h-full min-h-0 w-full flex-row text-text pointer-events-none">
      <div
        ref={stripRootRef}
        className="relative z-10 flex min-h-0 w-14 shrink-0 flex-col overflow-visible border-r border-surface0/80 bg-mantle/95 backdrop-blur-md pointer-events-auto"
      >
        <div className="scrollbar-thin flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto overflow-x-visible px-1 py-2">
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
            <SortableContext items={ordered} strategy={verticalListSortingStrategy}>
              {ordered.map((id) => (
                <SortableSection key={id} id={id}>
                  <div className="border-b border-surface0/40 pb-1">{renderSection(id)}</div>
                </SortableSection>
              ))}
            </SortableContext>
          </DndContext>
        </div>
        <button
          type="button"
          className="border-t border-surface0 py-1 text-[8px] text-subtext0 hover:text-subtext1"
          onClick={() => useBarLayoutStore.getState().resetOrder()}
        >
          Reset order
        </button>
      </div>

      <BarFlyoutLayer
        active={flyout.active}
        anchorCenterY={anchorCenterY}
        onEnterPanel={flyout.onEnterFlyout}
        onLeavePanel={onSegmentLeave}
      >
        {(fid) => flyoutChild(fid)}
      </BarFlyoutLayer>
    </div>
  )
}
