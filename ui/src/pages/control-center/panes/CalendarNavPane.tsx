import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api, { type CalendarEvent } from "@/lib/api"
import { getNavItem } from "../navigation"

/** Sidecar may use seconds or milliseconds for unix timestamps. */
function toMillis(ts: number): number {
  return ts > 1_000_000_000_000 ? ts : ts * 1000
}

function isUpcomingOrOngoing(e: CalendarEvent, now: number): boolean {
  return toMillis(e.end) >= now
}

function formatRange(startMs: number, endMs: number | null): string {
  const opts: Intl.DateTimeFormatOptions = { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" }
  const start = new Intl.DateTimeFormat(undefined, opts).format(new Date(startMs))
  if (endMs == null) return start
  const end = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" }).format(new Date(endMs))
  return `${start} – ${end}`
}

export function CalendarNavPane() {
  const { icon, label } = getNavItem("calendar")
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["calendar-events", "control-center-pane"],
    queryFn: api.getCalendarEvents,
    refetchInterval: 60_000,
  })

  const events = data ?? []
  const now = Date.now()
  const upcoming = events
    .filter((e) => isUpcomingOrOngoing(e, now))
    .sort((a, b) => toMillis(a.start) - toMillis(b.start))

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 mt-1 max-w-prose">Upcoming events from the sidecar calendar store.</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <button
            type="button"
            className="btn-surface text-xs"
            disabled={isFetching}
            onClick={() => refetch()}
            title="Refresh events"
          >
            <span className="icon text-base">refresh</span>
            Refresh
          </button>
          <button
            type="button"
            className="btn-surface text-xs"
            onClick={() => api.auraToggleWindow("calendar")}
            title="Open full calendar window"
          >
            <span className="icon text-base">open_in_new</span>
            Open
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="flex flex-col gap-2">
          <div className="skeleton h-16 rounded-xl" />
          <div className="skeleton h-16 rounded-xl" />
          <div className="skeleton h-16 rounded-xl" />
        </div>
      ) : isError ? (
        <div className="glass-card p-5 border-red/25">
          <p className="text-sm text-red font-medium">Could not load events</p>
          <p className="text-xs text-subtext0 mt-1">{error instanceof Error ? error.message : "Unknown error"}</p>
        </div>
      ) : upcoming.length === 0 ? (
        <div className="glass-card p-8 flex flex-col items-center text-center gap-2 border-dashed border-surface1">
          <span className="icon text-4xl text-subtext1">event_busy</span>
          <p className="text-sm font-medium text-text">No upcoming events</p>
          <p className="text-xs text-subtext0 max-w-sm">
            New events will appear here when the calendar service returns them. Use{" "}
            <span className="text-subtext1">Open</span> for the full calendar view.
          </p>
        </div>
      ) : (
        <ul className="flex flex-col gap-2">
          {upcoming.map((e) => {
            const title = e.title.trim() || "Untitled event"
            const startMs = toMillis(e.start)
            const endMs = toMillis(e.end)
            const when =
              endMs !== startMs ? formatRange(startMs, endMs) : formatRange(startMs, null)

            return (
              <li key={e.id}>
                <div className="glass-card p-4 flex flex-col gap-1">
                  <p className="text-sm font-medium text-text leading-snug">{title}</p>
                  <p className="text-xs text-subtext0 flex items-center gap-1.5">
                    <span className="icon text-sm text-amber">schedule</span>
                    {when}
                  </p>
                  {e.description.trim() ? (
                    <p className="text-xs text-subtext1 mt-1 line-clamp-2">{e.description.trim()}</p>
                  ) : null}
                </div>
              </li>
            )
          })}
        </ul>
      )}
    </motion.div>
  )
}
