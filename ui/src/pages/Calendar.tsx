/**
 * Calendar panel — data & UX choices (see AGENTS.md / roadmap for backend parity):
 *
 * - **Events**: Loaded from `api.getCalendarEvents()` (sidecar `Calendar.GetEvents`). Shown as
 *   indicators on the month grid and as a scrollable agenda beside/below the grid. Query key `["cal"]`
 *   matches `BarStrip` so the shell preview and this panel share React Query cache.
 * - **Tasks**: Loaded from sidecar `Todos.*` (shared with Control Center calendar pane).
 */

import { useState, useEffect, useMemo, useCallback, type ReactNode } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQuery, useQueryClient, useMutation } from "@tanstack/react-query"
import api, { type CalendarEvent } from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
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

const TASKS_STORAGE_KEY = "aura.calendar.tasks" // legacy — no longer used

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

/** Sidecar may use Unix seconds or millis — normalize to ms for Date APIs. */
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

/** Event overlaps local calendar day [startOfDay, endOfDay]. */
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

function formatEventRange(ev: CalendarEvent): string {
  const s = toEpochMs(ev.start)
  const e = toEpochMs(ev.end)
  if (!Number.isFinite(s) || !Number.isFinite(e)) return ""
  const opts: Intl.DateTimeFormatOptions = { hour: "numeric", minute: "2-digit" }
  const ds = new Date(s).toLocaleTimeString(undefined, opts)
  const de = new Date(e).toLocaleTimeString(undefined, opts)
  return `${ds} – ${de}`
}

function loadTasks(): CalendarTask[] {
  return []
}

function saveTasks(_tasks: CalendarTask[]) {
  /* legacy no-op */
}

function EmptyBlock({
  icon,
  title,
  body,
  action,
}: {
  icon: string
  title: string
  body: string
  action?: ReactNode
}) {
  return (
    <div className="glass-card flex flex-col items-center justify-center gap-3 px-5 py-8 text-center border border-surface0/40 border-dashed">
      <span className={cn("icon text-4xl text-subtext1/85")}>{icon}</span>
      <div className="space-y-1 max-w-[260px]">
        <p className="text-sm font-semibold text-text">{title}</p>
        <p className="text-xs leading-relaxed text-subtext0">{body}</p>
      </div>
      {action}
    </div>
  )
}

// ── Calendar grid ─────────────────────────────────────────────────────────────
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
    <div className="grid grid-cols-7 gap-1">
      {WEEKDAYS.map((d) => (
        <div key={d} className="text-center text-[11px] text-subtext0 py-1 font-medium">
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
            whileHover={day ? { scale: 1.06 } : undefined}
            whileTap={day ? { scale: 0.94 } : undefined}
            onClick={() => day != null && onSelectDay(day)}
            className={cn(
              "relative aspect-square flex flex-col items-center justify-center rounded-xl text-sm font-medium transition-colors gap-0.5",
              day ? "cursor-pointer hover:bg-surface1/90" : "pointer-events-none",
              isToday && "ring-2 ring-mauve/80 ring-offset-2 ring-offset-base",
              selected && day && "bg-surface1 shadow-inner",
              !isToday && !selected && day && "text-text"
            )}
          >
            <span
              className={cn(
                "tabular-nums",
                isToday && "text-mauve font-semibold",
                selected && !isToday && "text-text"
              )}
            >
              {day ?? ""}
            </span>
            {day != null && evCount > 0 ? (
              <span className="flex gap-0.5 justify-center" aria-hidden>
                {evCount <= 3
                  ? Array.from({ length: evCount }).map((_, j) => (
                      <span key={j} className="h-1 w-1 rounded-full bg-peach" />
                    ))
                  : (
                      <>
                        <span className="h-1 w-1 rounded-full bg-peach" />
                        <span className="text-[9px] leading-none text-peach font-medium">{evCount}</span>
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

export default function Calendar() {
  const qc = useQueryClient()
  const today = useMemo(() => new Date(), [])
  const [year, setYear] = useState(today.getFullYear())
  const [month, setMonth] = useState(today.getMonth())
  const [selectedDay, setSelectedDay] = useState<number | null>(() =>
    today.getMonth() === new Date().getMonth() && today.getFullYear() === new Date().getFullYear()
      ? today.getDate()
      : null
  )
  const [newTaskText, setNewTaskText] = useState("")

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Todos.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["todos"] })
    })
    return off
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
    [todoItems]
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

  useEffect(() => {
    const now = new Date()
    if (year === now.getFullYear() && month === now.getMonth()) {
      setSelectedDay(now.getDate())
    } else {
      setSelectedDay(null)
    }
  }, [year, month])

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

  const agendaTitle =
    selectedDay != null
      ? `${MONTHS[month]} ${selectedDay}, ${year}`
      : `${MONTHS[month]} ${year}`

  const agendaSubtitle =
    selectedDay != null ? "Events on this day" : "All events this month"

  const addTask = useCallback(() => {
    const text = newTaskText.trim()
    if (!text) return
    createTodoMut.mutate(text)
    setNewTaskText("")
  }, [newTaskText, createTodoMut])

  return (
    <div className="flex flex-col h-full bg-base/80 backdrop-blur-2xl rounded-2xl border border-surface0/60 shadow-2xl overflow-hidden text-text">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-surface0/60 shrink-0">
        <motion.button type="button" whileTap={{ scale: 0.8 }} onClick={prevMonth} className="icon-btn">
          <span className="icon">chevron_left</span>
        </motion.button>
        <motion.h2
          key={`${year}-${month}`}
          initial={{ opacity: 0, y: 6 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-base font-semibold"
        >
          {MONTHS[month]} {year}
        </motion.h2>
        <motion.button type="button" whileTap={{ scale: 0.8 }} onClick={nextMonth} className="icon-btn">
          <span className="icon">chevron_right</span>
        </motion.button>
      </div>

      <div className="flex flex-col lg:flex-row flex-1 min-h-0 gap-0 lg:gap-4 lg:px-5 lg:pt-3 lg:pb-3">
        {/* Month grid */}
        <div className="shrink-0 px-5 pt-3 pb-2 lg:px-0 lg:py-0 lg:w-[min(100%,280px)] lg:border-r lg:border-surface0/50 lg:pr-4">
          <AnimatePresence mode="wait">
            <motion.div
              key={`${year}-${month}`}
              initial={{ opacity: 0, x: 16 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -16 }}
              transition={{ duration: 0.18 }}
            >
              <CalendarGrid
                year={year}
                month={month}
                today={today}
                selectedDay={selectedDay}
                onSelectDay={setSelectedDay}
                eventCountByDay={eventCountByDay}
              />
            </motion.div>
          </AnimatePresence>
          <p className="text-[10px] text-subtext0 mt-2 leading-snug">
            Dots mark days with events from the sidecar. Tap a day to filter the list.
          </p>
        </div>

        {/* Agenda + tasks */}
        <div className="flex flex-col flex-1 min-h-0 min-w-0 border-t lg:border-t-0 border-surface0/60">
          <div className="flex-1 overflow-y-auto px-5 py-4 flex flex-col gap-3 min-h-0">
            <div className="flex items-start justify-between gap-2 shrink-0">
              <div>
                <p className="text-xs text-subtext0 font-medium uppercase tracking-wider">{agendaSubtitle}</p>
                <p className="text-sm font-semibold text-text mt-0.5">{agendaTitle}</p>
              </div>
              <button
                type="button"
                className="btn-ghost text-xs shrink-0"
                disabled={eventsLoading || isFetching}
                onClick={() => refetch()}
              >
                <span className="icon text-base">refresh</span>
                Refresh
              </button>
            </div>

            {eventsLoading && events === undefined ? (
              <EmptyBlock
                icon="hourglass_empty"
                title="Loading events"
                body="Fetching calendar data from Aura sidecar…"
              />
            ) : eventsError ? (
              <EmptyBlock
                icon="cloud_off"
                title="Could not load events"
                body={eventsErr instanceof Error ? eventsErr.message : "Check that ags-sidecar is running and reachable."}
                action={
                  <button type="button" className="btn-ghost text-sm" onClick={() => refetch()}>
                    Try again
                  </button>
                }
              />
            ) : agendaEvents.length === 0 ? (
              <EmptyBlock
                icon="event_busy"
                title={monthEvents.length === 0 ? "No events this month" : "Nothing scheduled"}
                body={
                  monthEvents.length === 0
                    ? "There are no calendar entries for this month yet. Events from the sidecar will appear here and on the grid."
                    : selectedDay != null
                      ? "No events land on this day. Pick another day or another month."
                      : "No events to show."
                }
              />
            ) : (
              <ul className="flex flex-col gap-2">
                <AnimatePresence initial={false}>
                  {agendaEvents.map((ev) => (
                    <motion.li
                      key={ev.id}
                      layout
                      initial={{ opacity: 0, y: 8 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -6 }}
                      className="glass-card px-3 py-2.5 border border-surface0/35"
                    >
                      <p className="text-sm font-medium text-text leading-snug">{ev.title}</p>
                      <p className="text-[11px] text-subtext0 mt-1">{formatEventRange(ev)}</p>
                      {ev.description.trim() ? (
                        <p className="text-xs text-subtext1 mt-2 leading-relaxed line-clamp-3">{ev.description}</p>
                      ) : null}
                    </motion.li>
                  ))}
                </AnimatePresence>
              </ul>
            )}
          </div>

          <div className="border-t border-surface0/60 mx-4 shrink-0" />

          <div className="overflow-y-auto px-5 py-4 flex flex-col gap-2 max-h-[42%] lg:max-h-none lg:flex-initial">
            <p className="text-xs text-subtext0 font-medium uppercase tracking-wider mb-1">Tasks</p>
            <p className="text-[10px] text-subtext1 -mt-1 mb-1 leading-snug">
              Synced via sidecar Todos service.
            </p>
            {tasks.length === 0 ? (
              <EmptyBlock
                icon="task_alt"
                title="No tasks yet"
                body="Quick reminders stay on this device until you add something below."
              />
            ) : (
              <AnimatePresence>
                {tasks.map((todo) => (
                  <motion.div
                    key={todo.id}
                    layout
                    initial={{ opacity: 0, x: -12 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 12 }}
                    className="flex items-center gap-3 glass-card px-3 py-2.5 cursor-pointer border border-surface0/35"
                    onClick={() =>
                      updateTodoMut.mutate({ id: todo.id, completed: !todo.done })
                    }
                  >
                    <motion.span
                      animate={{ scale: todo.done ? [1, 1.2, 1] : 1 }}
                      className={cn("icon text-lg", todo.done ? "text-green" : "text-subtext0")}
                    >
                      {todo.done ? "check_circle" : "radio_button_unchecked"}
                    </motion.span>
                    <span className={cn("text-sm flex-1", todo.done && "line-through text-subtext0")}>
                      {todo.text}
                    </span>
                    <button
                      type="button"
                      className="icon-btn opacity-60 hover:opacity-100"
                      aria-label="Remove task"
                      onClick={(e) => {
                        e.stopPropagation()
                        deleteTodoMut.mutate(todo.id)
                      }}
                    >
                      <span className="icon text-base">close</span>
                    </button>
                  </motion.div>
                ))}
              </AnimatePresence>
            )}
            <div className="flex gap-2 mt-1">
              <input
                type="text"
                value={newTaskText}
                onChange={(e) => setNewTaskText(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") addTask()
                }}
                placeholder="New task…"
                className="flex-1 rounded-lg bg-surface0/50 border border-surface0/60 px-3 py-2 text-sm placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/40"
              />
              <button type="button" className="btn-ghost text-sm shrink-0 px-3" onClick={addTask}>
                <span className="icon text-base">add</span>
                Add
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
