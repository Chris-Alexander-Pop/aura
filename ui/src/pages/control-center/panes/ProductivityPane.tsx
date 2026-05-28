import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { getNavItem } from "../navigation"

const { icon, label } = getNavItem("productivity")

function humanizeKey(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

function formatStatValue(value: unknown): string {
  if (value == null) return "—"
  if (typeof value === "boolean") return value ? "On" : "Off"
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  if (typeof value === "string") return value.trim() === "" ? "—" : value
  if (Array.isArray(value)) return value.length === 0 ? "None" : `${value.length} items`
  if (typeof value === "object") {
    const keys = Object.keys(value as object)
    if (keys.length === 0) return "—"
    try {
      return JSON.stringify(value)
    } catch {
      return "…"
    }
  }
  return String(value)
}

function isShallowEmptyStats(data: Record<string, unknown>): boolean {
  const keys = Object.keys(data)
  if (keys.length === 0) return true
  return keys.every((k) => {
    const v = data[k]
    if (v == null) return true
    if (typeof v === "string" && v.trim() === "") return true
    if (Array.isArray(v) && v.length === 0) return true
    if (typeof v === "object" && v !== null && !Array.isArray(v) && Object.keys(v).length === 0)
      return true
    return false
  })
}

export function ProductivityPane() {
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["productivity-stats"],
    queryFn: api.getProductivityStats,
    refetchInterval: 30_000,
  })

  const showEmpty =
    !isLoading && !isError && data != null && isShallowEmptyStats(data as Record<string, unknown>)
  const entries =
    data && !isShallowEmptyStats(data as Record<string, unknown>)
      ? Object.entries(data as Record<string, unknown>)
      : []

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
        <p className="text-xs text-subtext1 mt-1 max-w-prose">
          Live stats from the sidecar. When metrics roll in, they appear in the grid below.
        </p>
      </div>

      {isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {[0, 1, 2, 3].map((i) => (
            <div key={i} className="glass-card p-4">
              <div className="skeleton h-3 w-24 rounded mb-3" />
              <div className="skeleton h-8 w-full rounded" />
            </div>
          ))}
        </div>
      ) : isError ? (
        <div className="glass-card p-6 flex flex-col gap-3 items-start">
          <p className="text-sm text-text font-medium">Could not load productivity stats</p>
          <p className="text-xs text-subtext0 max-w-prose">
            {error instanceof Error ? error.message : "Sidecar error"}
          </p>
          <button type="button" className="btn-surface text-sm" onClick={() => refetch()}>
            <span className="icon text-base">refresh</span>
            Retry
          </button>
        </div>
      ) : showEmpty ? (
        <div className="flex flex-1 min-h-[220px] flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-surface0/80 bg-surface0/20 px-6 py-10 text-center">
          <span className="icon text-4xl text-subtext1">analytics</span>
          <div>
            <p className="text-sm font-medium text-text">No productivity metrics yet</p>
            <p className="text-xs text-subtext0 mt-1 max-w-sm mx-auto">
              The dashboard will fill in when `Productivity.GetStats` returns non-empty data from the
              sidecar.
            </p>
          </div>
          <button
            type="button"
            className="btn-surface text-xs"
            onClick={() => refetch()}
            disabled={isFetching}
          >
            <span className="icon text-base">sync</span>
            {isFetching ? "Refreshing…" : "Refresh"}
          </button>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2">
          {entries.map(([key, value]) => (
            <div key={key} className="glass-card p-4 flex flex-col gap-1 min-h-[88px]">
              <p className="text-xs uppercase tracking-wide text-subtext1">{humanizeKey(key)}</p>
              <p className="text-lg font-semibold text-text leading-tight break-words">
                {formatStatValue(value)}
              </p>
            </div>
          ))}
        </div>
      )}
    </motion.div>
  )
}
