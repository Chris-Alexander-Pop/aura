import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import api from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { getNavItem } from "../navigation"

function labelizeKey(key: string): string {
  return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
}

function formatStatValue(value: unknown): string {
  if (value == null) return "—"
  if (typeof value === "boolean") return value ? "On" : "Off"
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  if (typeof value === "string") return value.trim() === "" ? "—" : value
  return String(value)
}

export function ProductivityPane() {
  const { icon, label } = getNavItem("productivity")
  const qc = useQueryClient()
  const [taskTitle, setTaskTitle] = useState("")

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Productivity.TimerTick", () => {
      void qc.invalidateQueries({ queryKey: ["productivity-stats"] })
      void qc.invalidateQueries({ queryKey: ["productivity-tasks"] })
    })
    return off
  }, [qc])

  const statsQuery = useQuery({
    queryKey: ["productivity-stats"],
    queryFn: api.getProductivityStats,
    refetchInterval: 30_000,
  })

  const tasksQuery = useQuery({
    queryKey: ["productivity-tasks"],
    queryFn: api.getProductivityTasks,
  })

  const createTaskMut = useMutation({
    mutationFn: () => api.createProductivityTask(taskTitle.trim()),
    onSuccess: () => {
      setTaskTitle("")
      void qc.invalidateQueries({ queryKey: ["productivity-tasks"] })
      void qc.invalidateQueries({ queryKey: ["productivity-stats"] })
    },
  })

  const deleteTaskMut = useMutation({
    mutationFn: (id: string) => api.deleteProductivityTask(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["productivity-tasks"] })
      void qc.invalidateQueries({ queryKey: ["productivity-stats"] })
    },
  })

  const focusMut = useMutation({
    mutationFn: (enabled: boolean) => api.setProductivityFocusMode(enabled),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["productivity-stats"] }),
  })

  const stats = statsQuery.data as Record<string, unknown> | undefined
  const focusOn = stats?.focus_mode_enabled === true
  const tasks = tasksQuery.data ?? []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-5 p-6 h-full overflow-y-auto"
    >
      <div>
        <div className="flex items-center gap-3">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-xs text-subtext1 mt-1">Tasks persist in SQLite; timers stay in-memory until restart.</p>
      </div>

      <div className="glass-card p-4 flex flex-wrap items-center gap-3">
        <span className="text-sm text-text">Focus mode (prefs only)</span>
        <button
          type="button"
          className="btn-surface text-xs"
          onClick={() => focusMut.mutate(!focusOn)}
          disabled={focusMut.isPending}
        >
          {focusOn ? "Disable" : "Enable"}
        </button>
      </div>

      {statsQuery.isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {[0, 1, 2].map((i) => (
            <div key={i} className="skeleton h-20 rounded-xl" />
          ))}
        </div>
      ) : stats ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {Object.entries(stats).map(([key, value]) => (
            <div key={key} className="glass-card p-4">
              <p className="text-xs uppercase text-subtext0">{labelizeKey(key)}</p>
              <p className="text-lg font-semibold text-text mt-1">{formatStatValue(value)}</p>
            </div>
          ))}
        </div>
      ) : null}

      <div className="glass-card p-4 flex flex-col gap-3">
        <p className="text-sm font-medium text-text">Tasks</p>
        <div className="flex gap-2">
          <input
            className="flex-1 rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
            placeholder="New task title"
            value={taskTitle}
            onChange={(e) => setTaskTitle(e.target.value)}
          />
          <button
            type="button"
            className="btn-surface text-sm"
            disabled={!taskTitle.trim() || createTaskMut.isPending}
            onClick={() => createTaskMut.mutate()}
          >
            Add
          </button>
        </div>
        <ul className="flex flex-col gap-2">
          {tasks.map((t) => (
            <li key={t.id} className="flex items-center justify-between gap-2 text-sm border border-surface0/50 rounded-lg px-3 py-2">
              <span className={t.completed ? "line-through text-subtext1" : "text-text"}>{t.title}</span>
              <button
                type="button"
                className="text-xs text-red"
                onClick={() => deleteTaskMut.mutate(t.id)}
              >
                Delete
              </button>
            </li>
          ))}
          {tasks.length === 0 && <p className="text-xs text-subtext1">No tasks yet.</p>}
        </ul>
      </div>
    </motion.div>
  )
}
