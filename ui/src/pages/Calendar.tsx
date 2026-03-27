import { useState, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs } from "@/lib/ws"
import { cn } from "@/lib/utils"

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"]

function getDaysInMonth(year: number, month: number) {
  return new Date(year, month + 1, 0).getDate()
}

function getFirstDayOfMonth(year: number, month: number) {
  return new Date(year, month, 1).getDay()
}

// ── Calendar grid ─────────────────────────────────────────────────────────────
function CalendarGrid({
  year,
  month,
  today,
}: {
  year: number
  month: number
  today: Date
}) {
  const days = getDaysInMonth(year, month)
  const firstDay = getFirstDayOfMonth(year, month)
  const cells = [...Array(firstDay).fill(null), ...Array.from({ length: days }, (_, i) => i + 1)]

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
        return (
          <motion.button
            key={i}
            whileHover={day ? { scale: 1.15 } : undefined}
            whileTap={day ? { scale: 0.9 } : undefined}
            className={cn(
              "aspect-square flex items-center justify-center rounded-full text-sm font-medium transition-colors",
              day ? "cursor-pointer hover:bg-surface1" : "pointer-events-none",
              isToday && "bg-mauve text-crust hover:bg-mauve/90",
              !isToday && day && "text-text"
            )}
          >
            {day ?? ""}
          </motion.button>
        )
      })}
    </div>
  )
}

// ── Stub todo item ────────────────────────────────────────────────────────────
const STUB_TODOS = [
  { id: 1, done: false, text: "Review sidecar calendar service" },
  { id: 2, done: true,  text: "Set up WebKit hybrid architecture" },
  { id: 3, done: false, text: "Wire Google Calendar sync" },
  { id: 4, done: false, text: "Implement ReclaimAI integration" },
]

export default function Calendar() {
  const today = new Date()
  const [year, setYear] = useState(today.getFullYear())
  const [month, setMonth] = useState(today.getMonth())
  const [todos, setTodos] = useState(STUB_TODOS)

  useEffect(() => { connectWs() }, [])

  const prevMonth = () => {
    if (month === 0) { setMonth(11); setYear(y => y - 1) }
    else setMonth(m => m - 1)
  }
  const nextMonth = () => {
    if (month === 11) { setMonth(0); setYear(y => y + 1) }
    else setMonth(m => m + 1)
  }

  return (
    <div className="flex flex-col h-full bg-base/80 backdrop-blur-2xl rounded-2xl border border-surface0/60 shadow-2xl overflow-hidden text-text">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-surface0/60">
        <motion.button whileTap={{ scale: 0.8 }} onClick={prevMonth} className="icon-btn">
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
        <motion.button whileTap={{ scale: 0.8 }} onClick={nextMonth} className="icon-btn">
          <span className="icon">chevron_right</span>
        </motion.button>
      </div>

      {/* Calendar grid */}
      <div className="px-5 pt-3 pb-2">
        <AnimatePresence mode="wait">
          <motion.div
            key={`${year}-${month}`}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: 0.18 }}
          >
            <CalendarGrid year={year} month={month} today={today} />
          </motion.div>
        </AnimatePresence>
      </div>

      {/* Divider */}
      <div className="border-t border-surface0/60 mx-4" />

      {/* Todo subpanel */}
      <div className="flex-1 overflow-y-auto px-5 py-4 flex flex-col gap-2">
        <p className="text-xs text-subtext0 font-medium uppercase tracking-wider mb-1">Tasks</p>
        <AnimatePresence>
          {todos.map((todo) => (
            <motion.div
              key={todo.id}
              layout
              initial={{ opacity: 0, x: -12 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 12 }}
              className="flex items-center gap-3 glass-card px-3 py-2.5 cursor-pointer"
              onClick={() =>
                setTodos((ts) =>
                  ts.map((t) => (t.id === todo.id ? { ...t, done: !t.done } : t))
                )
              }
            >
              <motion.span
                animate={{ scale: todo.done ? [1, 1.3, 1] : 1 }}
                className={cn("icon text-lg", todo.done ? "text-green" : "text-subtext0")}
              >
                {todo.done ? "check_circle" : "radio_button_unchecked"}
              </motion.span>
              <span className={cn("text-sm flex-1", todo.done && "line-through text-subtext0")}>
                {todo.text}
              </span>
            </motion.div>
          ))}
        </AnimatePresence>
        <button className="btn-ghost text-sm mt-1 self-start">
          <span className="icon text-base">add</span>
          Add task
        </button>
      </div>
    </div>
  )
}
