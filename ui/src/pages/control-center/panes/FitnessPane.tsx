import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

type FitnessGoal = {
  id: string
  goal_type: string
  target: number
  current: number
}

type SummaryEntry = { label: string; value: string }

function isFitnessGoal(x: unknown): x is FitnessGoal {
  if (x == null || typeof x !== "object" || Array.isArray(x)) return false
  const r = x as Record<string, unknown>
  return (
    typeof r.id === "string" &&
    typeof r.goal_type === "string" &&
    typeof r.target === "number" &&
    typeof r.current === "number"
  )
}

function formatLabel(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

function formatStatValue(v: unknown): string {
  if (typeof v === "number" && Number.isFinite(v)) {
    return Number.isInteger(v) ? String(v) : v.toFixed(1).replace(/\.0$/, "")
  }
  if (typeof v === "boolean") return v ? "Yes" : "No"
  if (v == null) return "—"
  return String(v)
}

/** Sidecar may return a goal array or an object with optional `goals` plus summary fields. */
function normalizeFitnessStats(raw: unknown): { goals: FitnessGoal[]; summary: SummaryEntry[] } {
  const goals: FitnessGoal[] = []

  if (Array.isArray(raw)) {
    for (const item of raw) {
      if (isFitnessGoal(item)) goals.push(item)
    }
    return { goals, summary: [] }
  }

  if (raw && typeof raw === "object") {
    const o = raw as Record<string, unknown>
    const g = o.goals
    if (Array.isArray(g)) {
      for (const item of g) {
        if (isFitnessGoal(item)) goals.push(item)
      }
    }
    const summary = Object.entries(o)
      .filter(([k, v]) => k !== "goals" && v != null && typeof v !== "object")
      .map(([k, v]) => ({ label: formatLabel(k), value: formatStatValue(v) }))
    return { goals, summary }
  }

  return { goals, summary: [] }
}

function EmptyPanel({ icon, title, body }: { icon: string; title: string; body: string }) {
  return (
    <div className="glass-card flex flex-col items-center justify-center gap-3 px-6 py-12 text-center">
      <span className="icon text-5xl text-subtext1/90">{icon}</span>
      <div className="max-w-sm space-y-1.5">
        <p className="text-sm font-semibold text-text">{title}</p>
        <p className="text-xs leading-relaxed text-subtext0">{body}</p>
      </div>
    </div>
  )
}

function goalProgress(g: FitnessGoal): number {
  if (g.target <= 0) return 0
  return Math.min(100, (g.current / g.target) * 100)
}

const GOAL_TYPES = [
  { id: "steps", label: "Steps" },
  { id: "workout_minutes", label: "Workout minutes" },
  { id: "active_calories", label: "Active calories" },
] as const

export function FitnessPane() {
  const { icon, label } = getNavItem("fitness")
  const qc = useQueryClient()
  const [goalType, setGoalType] = useState<(typeof GOAL_TYPES)[number]["id"]>("steps")
  const [goalTarget, setGoalTarget] = useState("10000")

  const { data: raw, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["fitness-stats"],
    queryFn: async () => {
      const res = await api.getFitnessStats()
      return res as unknown
    },
    refetchInterval: 30_000,
  })

  const createGoalMut = useMutation({
    mutationFn: () => {
      const target = Number(goalTarget)
      if (!Number.isFinite(target) || target <= 0) {
        throw new Error("Target must be a positive number")
      }
      return api.fitnessSetGoal(goalType, target)
    },
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["fitness-stats"] }),
  })

  const { goals, summary } = normalizeFitnessStats(raw)
  const hasDashboard = goals.length > 0 || summary.length > 0
  const showEmpty = !isLoading && !isError && !hasDashboard

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18 }}
      className="flex h-full flex-col gap-4 overflow-y-auto p-6"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex items-center gap-3">
          <span className="icon text-2xl text-mauve">{icon}</span>
          <div className="min-w-0">
            <h2 className="text-xl font-semibold text-text">{label}</h2>
            <p className="mt-0.5 text-xs text-subtext1">
              {isLoading ? "Loading goals…" : `${goals.length} goal${goals.length === 1 ? "" : "s"} · sidecar SQLite`}
            </p>
          </div>
        </div>
        <button
          type="button"
          className="btn-surface text-xs disabled:opacity-40"
          disabled={isFetching}
          onClick={() => void refetch()}
        >
          <span className={cn("icon text-base", isFetching && "animate-pulse")}>refresh</span>
          Refresh
        </button>
      </div>

      {isError ? (
        <EmptyPanel
          icon="error_outline"
          title="Could not load fitness data"
          body={error instanceof Error ? error.message : "Sidecar request failed. Is ags-sidecar running?"}
        />
      ) : null}

      {!isError && isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2">
          <div className="skeleton h-28 rounded-xl" />
          <div className="skeleton h-28 rounded-xl" />
        </div>
      ) : null}

      <div className="glass-card flex flex-col gap-3 p-4">
        <p className="text-xs font-semibold uppercase tracking-wide text-subtext0">Add goal</p>
        <div className="flex flex-wrap gap-2">
          <select
            className="rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
            value={goalType}
            onChange={(e) => setGoalType(e.target.value as (typeof GOAL_TYPES)[number]["id"])}
          >
            {GOAL_TYPES.map((t) => (
              <option key={t.id} value={t.id}>
                {t.label}
              </option>
            ))}
          </select>
          <input
            type="number"
            min={1}
            className="w-28 rounded-xl border border-surface0/80 bg-base/80 px-3 py-2 text-sm"
            value={goalTarget}
            onChange={(e) => setGoalTarget(e.target.value)}
          />
          <button
            type="button"
            className="btn-surface text-sm"
            disabled={createGoalMut.isPending}
            onClick={() => createGoalMut.mutate()}
          >
            Create
          </button>
        </div>
        {createGoalMut.isError ? (
          <p className="text-xs text-red">
            {createGoalMut.error instanceof Error ? createGoalMut.error.message : "Create failed"}
          </p>
        ) : null}
      </div>

      {!isError && !isLoading && showEmpty ? (
        <EmptyPanel
          icon="route"
          title="No fitness goals yet"
          body="Create a goal above or sync a device when Fitness.SyncDevice integrations land."
        />
      ) : null}

      {!isError && !isLoading && hasDashboard ? (
        <div className="flex flex-col gap-4">
          {goals.length > 0 ? (
            <div>
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-subtext0">Goals</h3>
              <div className="grid gap-3 sm:grid-cols-2">
                {goals.map((g) => {
                  const pct = goalProgress(g)
                  const typeLabel = formatLabel(g.goal_type)
                  return (
                    <div key={g.id} className="glass-card flex flex-col gap-3 p-4">
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0">
                          <p className="truncate text-sm font-semibold text-text">{typeLabel}</p>
                          <p className="mt-0.5 font-mono text-[11px] text-subtext0">{g.id}</p>
                        </div>
                        <span className="shrink-0 rounded-full bg-teal/15 px-2 py-0.5 text-[10px] font-semibold text-teal">
                          {pct.toFixed(0)}%
                        </span>
                      </div>
                      <div className="space-y-1">
                        <div className="flex justify-between text-xs text-subtext1">
                          <span>Progress</span>
                          <span>
                            {formatStatValue(g.current)} / {formatStatValue(g.target)}
                          </span>
                        </div>
                        <div className="h-2 overflow-hidden rounded-full bg-surface1">
                          <div
                            className="h-full rounded-full bg-teal transition-[width] duration-500"
                            style={{ width: `${pct}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          ) : null}

          {summary.length > 0 ? (
            <div>
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-subtext0">Snapshot</h3>
              <div className="glass-card grid gap-2 p-3 sm:grid-cols-2">
                {summary.map((s) => (
                  <div
                    key={s.label}
                    className="flex flex-col gap-0.5 rounded-lg border border-surface0/50 bg-surface0/30 px-3 py-2"
                  >
                    <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext0">
                      {s.label}
                    </span>
                    <span className="text-sm font-medium text-text">{s.value}</span>
                  </div>
                ))}
              </div>
            </div>
          ) : null}
        </div>
      ) : null}
    </motion.div>
  )
}

export default FitnessPane
