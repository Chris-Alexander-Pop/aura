/**
 * Calendar panel — month-first layout for the 480px right-edge window.
 * Events: `Calendar.GetEvents` (+ Google sync). Tasks: `Todos.*`.
 */

import { useState, useEffect, useMemo, useCallback, type ReactNode } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQuery, useQueryClient, useMutation } from "@tanstack/react-query"
import api, { type CalendarEvent } from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { postPanelHover } from "@/lib/panel-hover"

const WEEKDAYS = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
]

interface CalendarTask {
  id: string
  done: boolean
  text: string
}

function getDaysInMonth(year: number, month: number) {
  return new Date(year, month + 1, 0).getDate()
}

function getFirstDayOfMonth(year: number, month: number) {
  return new Date(year, month, 1).getDay()
}

function toEpochMs(t: number): number {
  if (!Number.isFinite(t)) return NaN
  return Math.abs(t) > 1e12 ? t : t * 1000
}

function startOfLocalDay(year: number, month: number, day: number): number {
  return new Date(year, month, day, 0, 0, 0, 0).getTime()
}

function endOfLocalDay(year: number, month: number, day: number): number {
  return new Date(year, month, day, 23, 59, 59, 999).getTime()
}

function eventTouchesDay(ev: CalendarEvent, year: number, month: number, day: number): boolean {
  const s = toEpochMs(ev.start)
  const e = toEpochMs(ev.end)
  if (!Number.isFinite(s) || !Number.isFinite(e)) return false
  const d0 = startOfLocalDay(year, month, day)
  const d1 = endOfLocalDay(year, month, day)
  return s <= d1 && e >= d0
}

function eventsForMonth(events: CalendarEvent[], year: number, month: number): CalendarEvent[] {
  const start = startOfLocalDay(year, month, 1)
  const last = getDaysInMonth(year, month)
  const end = endOfLocalDay(year, month, last)
  return events.filter((ev) => {
    const s = toEpochMs(ev.start)
    const e = toEpochMs(ev.end)
    if (!Number.isFinite(s) || !Number.isFinite(e)) return false
    return s <= end && e >= start
  })
}

function formatEventWhen(ev: CalendarEvent, showDate: boolean): string {
  const s = toEpochMs(ev.start)
  const e = toEpochMs(ev.end)
  if (!Number.isFinite(s) || !Number.isFinite(e)) return ""
  const timeOpts: Intl.DateTimeFormatOptions = { hour: "numeric", minute: "2-digit" }
  const ds = new Date(s).toLocaleTimeString(undefined, timeOpts)
  const de = new Date(e).toLocaleTimeString(undefined, timeOpts)
  if (!showDate) return `${ds} – ${de}`
  const dateOpts: Intl.DateTimeFormatOptions = { weekday: "short", month: "short", day: "numeric" }
  const day = new Date(s).toLocaleDateString(undefined, dateOpts)
  return `${day} · ${ds} – ${de}`
}

function EmptyHint({ children }: { children: ReactNode }) {
  return <p className="px-0.5 py-2 text-[11px] leading-snug text-subtext0">{children}</p>
}

function CalendarGrid({
  year,
  month,
  today,
  selectedDay,
  onSelectDay,
  eventCountByDay,
}: {
  year: number
  month: number
  today: Date
  selectedDay: number | null
  onSelectDay: (day: number) => void
  eventCountByDay: Map<number, number>
}) {
  const days = getDaysInMonth(year, month)
  const firstDay = getFirstDayOfMonth(year, month)
  const cells = [...Array(firstDay).fill(null), ...Array.from({ length: days }, (_, i) => i + 1)] as (
    | number
    | null
  )[]

  return (
    <div className="grid grid-cols-7 gap-0.5">
      {WEEKDAYS.map((d) => (
        <div key={d} className="py-1 text-center text-[10px] font-semibold uppercase tracking-wide text-subtext0">
          {d}
        </div>
      ))}
      {cells.map((day, i) => {
        const isToday =
          day !== null &&
          today.getDate() === day &&
          today.getMonth() === month &&
          today.getFullYear() === year
        const selected = day !== null && selectedDay === day
        const evCount = day != null ? eventCountByDay.get(day) ?? 0 : 0
        return (
          <motion.button
            type="button"
            key={i}
            whileHover={day ? { scale: 1.04 } : undefined}
            whileTap={day ? { scale: 0.96 } : undefined}
            onClick={() => day != null && onSelectDay(day)}
            className={cn(
              "relative flex aspect-square flex-col items-center justify-center gap-0.5 rounded-lg text-sm font-medium transition-colors",
              day ? "cursor-pointer hover:bg-surface0/80" : "pointer-events-none",
              isToday && !selected && "ring-1 ring-mauve/70",
              selected && day && "bg-mauve/25 text-text",
              !selected && day && "text-text",
            )}
          >
            <span className={cn("tabular-nums text-[13px]", isToday && "font-semibold text-mauve")}>
              {day ?? ""}
            </span>
            {day != null && evCount > 0 ? (
              <span className="flex justify-center gap-0.5" aria-hidden>
                {evCount <= 3
                  ? Array.from({ length: Math.min(evCount, 3) }).map((_, j) => (
                      <span key={j} className="h-1 w-1 rounded-full bg-peach" />
                    ))
                  : (
                      <>
                        <span className="h-1 w-1 rounded-full bg-peach" />
                        <span className="text-[8px] font-medium leading-none text-peach">{evCount}</span>
                      </>
                    )}
              </span>
            ) : (
              <span className="h-1" aria-hidden />
            )}
          </motion.button>
        )
      })}
    </div>
  )
}

function GoogleAccountStrip() {
  const qc = useQueryClient()
  const { data: status } = useQuery({
    queryKey: ["calendar-google-auth"],
    queryFn: api.getGoogleCalendarAuthStatus,
    refetchInterval: (q) => (q.state.data?.pending ? 2000 : 15_000),
  })
  const { data: calendars } = useQuery({
    queryKey: ["calendars"],
    queryFn: api.getCalendars,
    enabled: !!status?.connected,
    staleTime: 60_000,
  })

  const startAuth = useMutation({
    mutationFn: () => api.googleCalendarStartAuth(),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["calendar-google-auth"] })
    },
  })

  const disconnect = useMutation({
    mutationFn: () => api.googleCalendarDisconnect(),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["calendar-google-auth"] })
      void qc.invalidateQueries({ queryKey: ["calendars"] })
      void qc.invalidateQueries({ queryKey: ["cal"] })
    },
  })

  const sync = useMutation({
    mutationFn: () => api.syncCalendars(),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["cal"] })
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
      void qc.invalidateQueries({ queryKey: ["calendar-google-auth"] })
      void qc.invalidateQueries({ queryKey: ["calendars"] })
    },
  })

  const connected = !!status?.connected
  const pending = !!status?.pending
  const busy = startAuth.isPending || sync.isPending || disconnect.isPending || pending

  useEffect(() => {
    if (connected && !status?.last_sync && !sync.isPending && !sync.isSuccess) {
      sync.mutate()
    }
    // Intentionally only when connection flips on without a prior sync.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connected, status?.last_sync])

  return (
    <div className="flex shrink-0 flex-col gap-1.5 border-b border-surface0/60 px-4 py-2.5">
      <div className="flex items-center gap-2">
        <span className="icon text-base text-subtext0">cloud_sync</span>
        <div className="min-w-0 flex-1">
          <p className="truncate text-[11px] font-medium text-text">
            {connected
              ? status?.email
                ? `Google · ${status.email}`
                : "Google connected"
              : pending
                ? "Waiting for Google sign-in…"
                : "Google Calendar"}
          </p>
          <p className="truncate text-[10px] text-subtext0">
            {connected
              ? status?.last_sync
                ? `Last sync ${new Date(status.last_sync * 1000).toLocaleString([], { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" })}`
                : "Connected — sync to pull events"
              : pending
                ? "Complete sign-in in your browser"
                : status?.configured === false
                  ? "Set AURA_GOOGLE_OAUTH_CLIENT_ID to enable"
                  : "Connect to sync events"}
          </p>
        </div>
        {connected ? (
          <>
            <button
              type="button"
              className="btn-ghost shrink-0 px-2 text-[11px]"
              disabled={busy}
              onClick={() => sync.mutate()}
              title="Sync from Google"
            >
              <span className={cn("icon text-sm", sync.isPending && "animate-spin")}>
                {sync.isPending ? "progress_activity" : "sync"}
              </span>
            </button>
            <button
              type="button"
              className="btn-ghost shrink-0 px-2 text-[11px] text-subtext0"
              disabled={busy}
              onClick={() => disconnect.mutate()}
              title="Disconnect Google"
            >
              <span className="icon text-sm">logout</span>
            </button>
          </>
        ) : (
          <button
            type="button"
            className="btn-ghost shrink-0 px-2.5 text-[11px]"
            disabled={busy || status?.configured === false}
            onClick={() => startAuth.mutate()}
          >
            {pending || startAuth.isPending ? "Waiting…" : "Connect"}
          </button>
        )}
      </div>
      {connected && calendars && calendars.length > 0 ? (
        <p className="truncate pl-6 text-[10px] text-subtext1">
          {calendars.map((c) => c.name).join(" · ")}
        </p>
      ) : null}
      {status?.error ? <p className="pl-6 text-[10px] text-red">{status.error}</p> : null}
      {startAuth.isError ? (
        <p className="pl-6 text-[10px] text-red">
          {startAuth.error instanceof Error ? startAuth.error.message : "Auth failed"}
        </p>
      ) : null}
      {sync.isError ? (
        <p className="pl-6 text-[10px] text-red">
          {sync.error instanceof Error ? sync.error.message : "Sync failed"}
        </p>
      ) : null}
    </div>
  )
}

export default function Calendar() {
  const qc = useQueryClient()
  const [nowTick, setNowTick] = useState(0)
  const today = useMemo(() => new Date(), [nowTick])

  useEffect(() => {
    const id = window.setInterval(() => setNowTick((n) => n + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])

  const [year, setYear] = useState(() => new Date().getFullYear())
  const [month, setMonth] = useState(() => new Date().getMonth())
  const [selectedDay, setSelectedDay] = useState<number | null>(() => new Date().getDate())
  const [newTaskText, setNewTaskText] = useState("")
  const [newEventTitle, setNewEventTitle] = useState("")
  const [showAddEvent, setShowAddEvent] = useState(false)

  useEffect(() => {
    connectWs()
    const offTodos = useWsStore.getState().on("Todos.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["todos"] })
    })
    const offCal = useWsStore.getState().on("Calendar.EventsChanged", () => {
      void qc.invalidateQueries({ queryKey: ["cal"] })
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
      void qc.invalidateQueries({ queryKey: ["calendars"] })
      void qc.invalidateQueries({ queryKey: ["calendar-google-auth"] })
    })
    return () => {
      offTodos()
      offCal()
    }
  }, [qc])

  const { data: todoItems } = useQuery({
    queryKey: ["todos", "calendar"],
    queryFn: () => api.todosList({ include_completed: true }),
    refetchInterval: 120_000,
  })

  const tasks: CalendarTask[] = useMemo(
    () =>
      (todoItems ?? []).map((t) => ({
        id: t.id,
        text: t.title,
        done: t.completed,
      })),
    [todoItems],
  )

  const createTodoMut = useMutation({
    mutationFn: (title: string) => api.todosCreate({ title }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["todos"] }),
  })

  const updateTodoMut = useMutation({
    mutationFn: (opts: { id: string; completed: boolean }) =>
      api.todosUpdate({ id: opts.id, completed: opts.completed }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["todos"] }),
  })

  const deleteTodoMut = useMutation({
    mutationFn: (id: string) => api.todosDelete(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["todos"] }),
  })

  const {
    data: events,
    isLoading: eventsLoading,
    isError: eventsError,
    error: eventsErr,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ["cal"],
    queryFn: () => api.fetchCalendarEvents(),
    refetchInterval: 60_000,
  })

  const createEventMut = useMutation({
    mutationFn: (opts: { title: string; start: number; end: number }) =>
      api.createCalendarEvent({ ...opts, reminder_minutes: 15 }),
    onSuccess: () => {
      setNewEventTitle("")
      setShowAddEvent(false)
      void qc.invalidateQueries({ queryKey: ["cal"] })
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
    },
  })

  const deleteEventMut = useMutation({
    mutationFn: (id: string) => api.deleteCalendarEvent(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["cal"] })
      void qc.invalidateQueries({ queryKey: ["calendar-upcoming"] })
    },
  })

  const eventList = events ?? []
  const monthEvents = useMemo(() => eventsForMonth(eventList, year, month), [eventList, year, month])

  const eventCountByDay = useMemo(() => {
    const m = new Map<number, number>()
    const dim = getDaysInMonth(year, month)
    for (let d = 1; d <= dim; d++) {
      const n = monthEvents.filter((ev) => eventTouchesDay(ev, year, month, d)).length
      if (n > 0) m.set(d, n)
    }
    return m
  }, [monthEvents, year, month])

  const agendaEvents = useMemo(() => {
    let list: CalendarEvent[]
    if (selectedDay != null) {
      list = monthEvents.filter((ev) => eventTouchesDay(ev, year, month, selectedDay))
    } else {
      list = [...monthEvents]
    }
    return list.sort((a, b) => toEpochMs(a.start) - toEpochMs(b.start))
  }, [monthEvents, selectedDay, year, month])

  const goToday = () => {
    const n = new Date()
    setYear(n.getFullYear())
    setMonth(n.getMonth())
    setSelectedDay(n.getDate())
  }

  const prevMonth = () => {
    if (month === 0) {
      setMonth(11)
      setYear((y) => y - 1)
    } else setMonth((m) => m - 1)
  }
  const nextMonth = () => {
    if (month === 11) {
      setMonth(0)
      setYear((y) => y + 1)
    } else setMonth((m) => m + 1)
  }

  const onSelectDay = (day: number) => {
    setSelectedDay((prev) => (prev === day ? null : day))
  }

  const agendaTitle =
    selectedDay != null ? `${MONTHS[month]} ${selectedDay}, ${year}` : `${MONTHS[month]} ${year}`

  const agendaSubtitle = selectedDay != null ? "This day" : "This month"

  const addTask = useCallback(() => {
    const text = newTaskText.trim()
    if (!text) return
    createTodoMut.mutate(text)
    setNewTaskText("")
  }, [newTaskText, createTodoMut])

  const addEvent = () => {
    const title = newEventTitle.trim() || "New event"
    const day = selectedDay ?? today.getDate()
    const useYear = selectedDay != null ? year : today.getFullYear()
    const useMonth = selectedDay != null ? month : today.getMonth()
    const startDate = new Date(useYear, useMonth, day, 10, 0, 0, 0)
    const start = Math.floor(startDate.getTime() / 1000)
    const end = start + 3600
    createEventMut.mutate({ title, start, end })
  }

  return (
    <div
      className="flex h-full flex-col overflow-hidden rounded-2xl border border-surface0/60 bg-mantle text-text shadow-2xl"
      onMouseEnter={() => postPanelHover("calendarHover", true)}
      onMouseLeave={() => postPanelHover("calendarHover", false)}
    >
      <GoogleAccountStrip />

      <div className="flex shrink-0 items-center gap-1 border-b border-surface0/60 px-3 py-2.5">
        <motion.button type="button" whileTap={{ scale: 0.85 }} onClick={prevMonth} className="icon-btn">
          <span className="icon">chevron_left</span>
        </motion.button>
        <motion.h2
          key={`${year}-${month}`}
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          className="min-w-0 flex-1 truncate text-center text-[15px] font-semibold"
        >
          {MONTHS[month]} {year}
        </motion.h2>
        <motion.button type="button" whileTap={{ scale: 0.85 }} onClick={nextMonth} className="icon-btn">
          <span className="icon">chevron_right</span>
        </motion.button>
        <button type="button" className="btn-ghost ml-0.5 shrink-0 px-2 text-[11px]" onClick={goToday}>
          Today
        </button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        <div className="shrink-0 px-3 pt-3 pb-2">
          <AnimatePresence mode="wait">
            <motion.div
              key={`${year}-${month}`}
              initial={{ opacity: 0, x: 12 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -12 }}
              transition={{ duration: 0.15 }}
            >
              <CalendarGrid
                year={year}
                month={month}
                today={today}
                selectedDay={selectedDay}
                onSelectDay={onSelectDay}
                eventCountByDay={eventCountByDay}
              />
            </motion.div>
          </AnimatePresence>
        </div>

        <div className="flex min-h-0 flex-1 flex-col border-t border-surface0/50">
          <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-3 py-3">
            <div className="mb-2 flex shrink-0 items-center justify-between gap-2">
              <div className="min-w-0">
                <p className="text-[10px] font-semibold uppercase tracking-wider text-subtext0">
                  {agendaSubtitle}
                </p>
                <p className="truncate text-sm font-semibold text-text">{agendaTitle}</p>
              </div>
              <div className="flex shrink-0 items-center gap-1">
                <button
                  type="button"
                  className="btn-ghost px-2 text-[11px]"
                  title="Add event"
                  onClick={() => setShowAddEvent((v) => !v)}
                >
                  <span className="icon text-sm">add</span>
                </button>
                <button
                  type="button"
                  className="btn-ghost px-2 text-[11px]"
                  disabled={eventsLoading || isFetching}
                  onClick={() => void refetch()}
                  title="Refresh"
                >
                  <span className={cn("icon text-sm", isFetching && "animate-spin")}>refresh</span>
                </button>
              </div>
            </div>

            {showAddEvent ? (
              <div className="mb-2 flex gap-1.5">
                <input
                  type="text"
                  value={newEventTitle}
                  onChange={(e) => setNewEventTitle(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") addEvent()
                  }}
                  placeholder={
                    selectedDay != null
                      ? `Event on ${MONTHS[month]} ${selectedDay}…`
                      : "New event (today 10:00)…"
                  }
                  className="flex-1 rounded-lg border border-surface0/60 bg-surface0/50 px-2.5 py-1.5 text-[12px] placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/40"
                />
                <button
                  type="button"
                  className="btn-ghost shrink-0 px-2.5 text-[11px]"
                  disabled={createEventMut.isPending}
                  onClick={addEvent}
                >
                  Add
                </button>
              </div>
            ) : null}

            {eventsLoading && events === undefined ? (
              <EmptyHint>Loading events…</EmptyHint>
            ) : eventsError ? (
              <EmptyHint>
                {eventsErr instanceof Error ? eventsErr.message : "Could not load events."}{" "}
                <button type="button" className="text-mauve underline" onClick={() => void refetch()}>
                  Retry
                </button>
              </EmptyHint>
            ) : agendaEvents.length === 0 ? (
              <EmptyHint>
                {monthEvents.length === 0
                  ? "No events this month. Connect Google or add one."
                  : selectedDay != null
                    ? "Nothing on this day — tap again to show the whole month."
                    : "No events to show."}
              </EmptyHint>
            ) : (
              <ul className="flex flex-col gap-1.5">
                <AnimatePresence initial={false}>
                  {agendaEvents.map((ev) => (
                    <motion.li
                      key={ev.id}
                      layout
                      initial={{ opacity: 0, y: 6 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -4 }}
                      className="group flex items-start gap-2 rounded-xl border border-surface0/35 bg-surface0/25 px-2.5 py-2"
                    >
                      <div className="min-w-0 flex-1">
                        <p className="text-[13px] font-medium leading-snug text-text">{ev.title}</p>
                        <p className="mt-0.5 text-[10px] text-subtext0">
                          {formatEventWhen(ev, selectedDay == null)}
                        </p>
                      </div>
                      <button
                        type="button"
                        className="icon-btn opacity-0 transition-opacity group-hover:opacity-70 hover:!opacity-100"
                        aria-label="Delete event"
                        onClick={() => deleteEventMut.mutate(ev.id)}
                      >
                        <span className="icon text-sm">close</span>
                      </button>
                    </motion.li>
                  ))}
                </AnimatePresence>
              </ul>
            )}
          </div>

          <div className="shrink-0 border-t border-surface0/50 px-3 py-2.5">
            <p className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-subtext0">Tasks</p>
            {tasks.length === 0 ? (
              <EmptyHint>No tasks yet — add one below.</EmptyHint>
            ) : (
              <div className="mb-2 flex max-h-28 flex-col gap-1 overflow-y-auto">
                <AnimatePresence>
                  {tasks.map((todo) => (
                    <motion.div
                      key={todo.id}
                      layout
                      initial={{ opacity: 0, x: -8 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 8 }}
                      className="flex cursor-pointer items-center gap-2 rounded-lg border border-surface0/30 bg-surface0/20 px-2 py-1.5"
                      onClick={() => updateTodoMut.mutate({ id: todo.id, completed: !todo.done })}
                    >
                      <span className={cn("icon text-base", todo.done ? "text-green" : "text-subtext0")}>
                        {todo.done ? "check_circle" : "radio_button_unchecked"}
                      </span>
                      <span className={cn("flex-1 truncate text-[12px]", todo.done && "text-subtext0 line-through")}>
                        {todo.text}
                      </span>
                      <button
                        type="button"
                        className="icon-btn opacity-50 hover:opacity-100"
                        aria-label="Remove task"
                        onClick={(e) => {
                          e.stopPropagation()
                          deleteTodoMut.mutate(todo.id)
                        }}
                      >
                        <span className="icon text-sm">close</span>
                      </button>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
            )}
            <div className="flex gap-1.5">
              <input
                type="text"
                value={newTaskText}
                onChange={(e) => setNewTaskText(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") addTask()
                }}
                placeholder="New task…"
                className="flex-1 rounded-lg border border-surface0/60 bg-surface0/50 px-2.5 py-1.5 text-[12px] placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/40"
              />
              <button type="button" className="btn-ghost shrink-0 px-2.5 text-[11px]" onClick={addTask}>
                Add
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
