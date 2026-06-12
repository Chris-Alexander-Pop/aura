import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import api, { type CalendarEvent } from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { getNavItem } from "../navigation"
import { cn } from "@/lib/utils"

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
  const qc = useQueryClient()
  const [title, setTitle] = useState("")
  const [hoursFromNow, setHoursFromNow] = useState(1)
  const [todoTitle, setTodoTitle] = useState("")

  useEffect(() => {
    connectWs()
    const offCal = useWsStore.getState().on("Calendar.EventsChanged", () => {
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
      void qc.invalidateQueries({ queryKey: ["cal"] })
    })
    const offTodos = useWsStore.getState().on("Todos.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["todos"] })
    })
    return () => {
      offCal()
      offTodos()
    }
  }, [qc])

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["calendar-upcoming", "control-center-pane"],
    queryFn: () => api.getCalendarUpcoming(20),
    refetchInterval: 300_000,
  })

  const {
    data: todos,
    isLoading: todosLoading,
    refetch: refetchTodos,
  } = useQuery({
    queryKey: ["todos", "control-center-pane"],
    queryFn: () => api.todosList({ include_completed: false }),
    refetchInterval: 120_000,
  })

  const createMut = useMutation({
    mutationFn: () => {
      const start = Math.floor(Date.now() / 1000) + hoursFromNow * 3600
      const end = start + 3600
      return api.createCalendarEvent({
        title: title.trim() || "New event",
        start,
        end,
        reminder_minutes: 15,
      })
    },
    onSuccess: () => {
      setTitle("")
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
    },
  })

  const deleteMut = useMutation({
    mutationFn: (id: string) => api.deleteCalendarEvent(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] }),
  })

  const createTodoMut = useMutation({
    mutationFn: () =>
      api.todosCreate({
        title: todoTitle.trim() || "New task",
      }),
    onSuccess: () => {
      setTodoTitle("")
      void qc.invalidateQueries({ queryKey: ["todos"] })
    },
  })

  const updateTodoMut = useMutation({
    mutationFn: (opts: { id: string; completed?: boolean; title?: string }) =>
      api.todosUpdate(opts),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["todos"] }),
  })

  const deleteTodoMut = useMutation({
    mutationFn: (id: string) => api.todosDelete(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["todos"] }),
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

      <div className="glass-card p-4 flex flex-col gap-2">
        <p className="text-xs font-medium text-text">Quick add</p>
        <input
          className="rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
          placeholder="Event title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
        />
        <label className="text-xs text-subtext1 flex items-center gap-2">
          Starts in (hours)
          <input
            type="number"
            min={0}
            className="w-16 rounded border border-surface0/80 bg-base/80 px-2 py-1"
            value={hoursFromNow}
            onChange={(e) => setHoursFromNow(Number(e.target.value) || 0)}
          />
        </label>
        <button
          type="button"
          className="btn-surface text-sm self-start"
          disabled={createMut.isPending}
          onClick={() => createMut.mutate()}
        >
          Create event
        </button>
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
      ) : null}

      <div className="glass-card p-4 flex flex-col gap-2">
        <p className="text-xs font-medium text-text">Tasks</p>
        <input
          className="rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
          placeholder="Todo title"
          value={todoTitle}
          onChange={(e) => setTodoTitle(e.target.value)}
        />
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="btn-surface text-sm"
            disabled={createTodoMut.isPending}
            onClick={() => createTodoMut.mutate()}
          >
            Add task
          </button>
          <button
            type="button"
            className="btn-surface text-xs"
            disabled={todosLoading}
            onClick={() => refetchTodos()}
          >
            Refresh
          </button>
        </div>
        {todosLoading ? (
          <div className="skeleton h-10 rounded-xl" />
        ) : (todos ?? []).length === 0 ? (
          <p className="text-xs text-subtext0">No open tasks.</p>
        ) : (
          <ul className="flex flex-col gap-1">
            {(todos ?? []).slice(0, 8).map((t) => (
              <li key={t.id} className="flex items-center gap-2 text-sm">
                <button
                  type="button"
                  className="icon text-base text-subtext0 hover:text-green"
                  title={t.completed ? "Mark incomplete" : "Complete"}
                  onClick={() =>
                    updateTodoMut.mutate({ id: t.id, completed: !t.completed })
                  }
                >
                  {t.completed ? "check_circle" : "radio_button_unchecked"}
                </button>
                <span className={cn("flex-1 truncate", t.completed && "line-through text-subtext0")}>
                  {t.title}
                </span>
                <button
                  type="button"
                  className="text-xs text-red"
                  onClick={() => deleteTodoMut.mutate(t.id)}
                >
                  Delete
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      {!isLoading && !isError && upcoming.length > 0 ? (
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
                  <button
                    type="button"
                    className="text-xs text-red self-start mt-1"
                    onClick={() => deleteMut.mutate(e.id)}
                  >
                    Delete
                  </button>
                </div>
              </li>
            )
          })}
        </ul>
      ) : null}
    </motion.div>
  )
}
