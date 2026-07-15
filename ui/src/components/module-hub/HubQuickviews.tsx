import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import type { CalendarEvent } from "@/lib/api-types"
import { cn } from "@/lib/utils"

function toEpochMs(t: number): number {
  if (!Number.isFinite(t)) return NaN
  return Math.abs(t) > 1e12 ? t : t * 1000
}

function formatEventRange(ev: CalendarEvent): string {
  const s = toEpochMs(ev.start)
  const e = toEpochMs(ev.end)
  if (!Number.isFinite(s) || !Number.isFinite(e)) return ""
  const opts: Intl.DateTimeFormatOptions = { hour: "numeric", minute: "2-digit" }
  const ds = new Date(s).toLocaleTimeString(undefined, opts)
  const de = new Date(e).toLocaleTimeString(undefined, opts)
  return `${ds} – ${de}`
}

function formatEventDay(ev: CalendarEvent): string {
  const s = toEpochMs(ev.start)
  if (!Number.isFinite(s)) return ""
  return new Date(s).toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" })
}

function CalendarQuickview() {
  const { data: events, isLoading, isError } = useQuery({
    queryKey: ["cal-upcoming", 3],
    queryFn: () => api.getCalendarUpcoming(3),
    refetchInterval: 60_000,
  })

  const list = (events ?? []).slice(0, 3)

  return (
    <button
      type="button"
      className="glass-card flex min-h-0 min-w-0 flex-col gap-1.5 rounded-xl border border-surface0/60 p-3 text-left transition-colors hover:border-mauve/30"
      title="Open calendar"
      onClick={() => void api.auraToggleWindow("calendar")}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-[10px] font-semibold uppercase tracking-wide text-mauve">Calendar</span>
        <span className="icon text-sm text-subtext0">event</span>
      </div>
      {isLoading && !events ? (
        <p className="text-xs text-subtext0">Loading…</p>
      ) : isError ? (
        <p className="text-xs text-red">Unavailable</p>
      ) : list.length === 0 ? (
        <p className="text-xs text-subtext0">No upcoming events</p>
      ) : (
        <ul className="flex min-h-0 flex-col gap-1.5">
          {list.map((ev) => (
            <li key={ev.id} className="min-w-0">
              <p className="truncate text-xs font-medium text-text">{ev.title}</p>
              <p className="truncate text-[10px] text-subtext0">
                {formatEventDay(ev)} · {formatEventRange(ev)}
              </p>
            </li>
          ))}
        </ul>
      )}
    </button>
  )
}

function DevopsQuickview() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["devops-status"],
    queryFn: api.getDevopsStatus,
    refetchInterval: 30_000,
  })

  const toolMissing = data?.tool_missing === true
  const runtime = typeof data?.container_runtime === "string" ? data.container_runtime : null
  const count = typeof data?.container_count === "number" ? data.container_count : null
  const dirty =
    typeof data?.git_dirty_count === "number"
      ? data.git_dirty_count
      : data?.git_dirty_hint === true
        ? 1
        : 0

  const healthy = !toolMissing && runtime != null && runtime !== "none"
  const statusLabel = isLoading && !data
    ? "Checking…"
    : isError
      ? "Unavailable"
      : toolMissing
        ? "Tools missing"
        : healthy
          ? `${runtime} · ${count ?? 0} containers`
          : "No runtime"

  return (
    <button
      type="button"
      className="glass-card flex min-h-0 min-w-0 flex-col gap-1.5 rounded-xl border border-surface0/60 p-3 text-left transition-colors hover:border-mauve/30"
      title="Open DevOps in Control Center"
      onClick={() => void api.openControlCenterPane("devops")}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-[10px] font-semibold uppercase tracking-wide text-mauve">DevOps</span>
        <span
          className={cn(
            "h-2 w-2 shrink-0 rounded-full",
            isError || toolMissing ? "bg-red" : healthy ? "bg-green" : "bg-yellow"
          )}
          aria-hidden
        />
      </div>
      <p className="line-clamp-2 text-xs text-text">{statusLabel}</p>
      {!isLoading && !isError && dirty > 0 ? (
        <p className="text-[10px] text-peach">{dirty} dirty repo{dirty === 1 ? "" : "s"}</p>
      ) : null}
    </button>
  )
}

function CommsQuickviewStub() {
  return (
    <div
      className="glass-card flex min-h-0 min-w-0 flex-col gap-1.5 rounded-xl border border-surface0/40 p-3 opacity-60"
      title="Communication hub deferred"
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">
          Communication
        </span>
        <span className="icon text-sm text-subtext0">forum</span>
      </div>
      <p className="text-xs text-subtext0">Deferred — separate project</p>
    </div>
  )
}

export function HubQuickviews() {
  return (
    <section className="flex flex-col gap-1.5">
      <h2 className="px-0.5 text-[10px] font-semibold uppercase tracking-wide text-subtext1">
        Quickviews
      </h2>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
        <CalendarQuickview />
        <DevopsQuickview />
        <CommsQuickviewStub />
      </div>
    </section>
  )
}
