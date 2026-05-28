import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { getNavItem } from "../navigation"

function labelizeKey(key: string): string {
  return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
}

function isEffectivelyEmpty(value: unknown): boolean {
  if (value == null) return true
  if (typeof value === "string") return value.trim() === ""
  if (Array.isArray(value)) return value.length === 0
  if (typeof value === "object") return Object.keys(value as object).length === 0
  return false
}

function hasRenderableDevopsData(data: Record<string, unknown> | undefined): boolean {
  if (!data) return false
  return Object.entries(data).some(([, v]) => !isEffectivelyEmpty(v))
}

function formatDevopsValue(value: unknown): string {
  if (value === null || value === undefined) return "—"
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return String(value)
  }
  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

export function DevopsPane() {
  const { icon, label } = getNavItem("devops")
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["devops-status"],
    queryFn: api.getDevopsStatus,
    refetchInterval: 30_000,
  })

  const showDashboard = hasRenderableDevopsData(data)

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-5 p-6 h-full overflow-y-auto"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose">
            Workload and tooling status from the sidecar. Refreshes automatically.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-sm shrink-0"
          onClick={() => refetch()}
          disabled={isFetching}
        >
          <span className="icon text-base">refresh</span>
          {isFetching ? "Refreshing…" : "Refresh"}
        </button>
      </div>

      {isLoading ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {[0, 1, 2, 3].map((i) => (
            <div key={i} className="skeleton h-24 rounded-xl" style={{ opacity: 1 - i * 0.12 }} />
          ))}
        </div>
      ) : isError ? (
        <div className="glass-card p-5 border border-red/30 bg-red/5">
          <p className="text-sm font-medium text-text">Could not load DevOps status</p>
          <p className="text-xs text-subtext0 mt-2">{error instanceof Error ? error.message : "Unknown error"}</p>
          <p className="text-xs text-subtext1 mt-3 max-w-prose">
            Ensure the sidecar is running and exposes <code className="text-subtext0">Devops.GetStatus</code>.
          </p>
        </div>
      ) : showDashboard ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {Object.entries(data!).map(([key, value]) => (
            <div key={key} className="glass-card p-4 flex flex-col gap-1 min-w-0">
              <p className="text-xs font-medium uppercase tracking-wide text-subtext0">{labelizeKey(key)}</p>
              <p className="text-sm text-text break-words whitespace-pre-wrap font-mono leading-snug">
                {formatDevopsValue(value)}
              </p>
            </div>
          ))}
        </div>
      ) : (
        <div className="glass-card p-8 flex flex-col items-center text-center gap-4 border border-surface0/60">
          <span className="icon text-5xl text-subtext0">inventory_2</span>
          <div>
            <p className="text-sm font-medium text-text">No DevOps signals yet</p>
            <p className="text-xs text-subtext1 mt-2 max-w-md mx-auto leading-relaxed">
              The sidecar returned no non-empty fields. When <code className="text-subtext0">getDevopsStatus</code>{" "}
              includes containers, clusters, repos, or health checks, they will show here as cards.
            </p>
          </div>
        </div>
      )}
    </motion.div>
  )
}
