import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

function labelizeKey(key: string): string {
  return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
}

export function DevopsPane() {
  const { icon, label } = getNavItem("devops")
  const statusQuery = useQuery({
    queryKey: ["devops-status"],
    queryFn: api.getDevopsStatus,
    refetchInterval: 30_000,
  })
  const containersQuery = useQuery({
    queryKey: ["devops-containers"],
    queryFn: api.getDevopsContainers,
    refetchInterval: 30_000,
  })

  const data = statusQuery.data
  const containers = containersQuery.data ?? []

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
            Podman-first container list (Docker only when <code className="text-subtext0">AURA_ALLOW_DOCKER=1</code>
            ). Read-only in this panel.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-sm shrink-0"
          onClick={() => {
            void statusQuery.refetch()
            void containersQuery.refetch()
          }}
          disabled={statusQuery.isFetching || containersQuery.isFetching}
        >
          <span
            className={cn(
              "icon text-base",
              (statusQuery.isFetching || containersQuery.isFetching) && "animate-spin"
            )}
          >
            refresh
          </span>
          Refresh
        </button>
      </div>

      {statusQuery.isLoading ? (
        <div className="skeleton h-32 rounded-xl" />
      ) : statusQuery.isError ? (
        <div className="glass-card p-5 border-red/25 text-sm text-red">
          {statusQuery.error instanceof Error ? statusQuery.error.message : "Load failed"}
        </div>
      ) : data ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {Object.entries(data).map(([key, value]) => (
            <div key={key} className="glass-card p-4">
              <p className="text-xs uppercase text-subtext0">{labelizeKey(key)}</p>
              <p className="text-sm font-mono text-text mt-1 break-all">{String(value)}</p>
            </div>
          ))}
        </div>
      ) : null}

      <div>
        <h3 className="text-sm font-medium text-text mb-2">Containers</h3>
        {containersQuery.isLoading ? (
          <div className="skeleton h-24 rounded-xl" />
        ) : containers.length === 0 ? (
          <p className="text-xs text-subtext1">No containers reported (Podman/Docker unavailable or none running).</p>
        ) : (
          <ul className="flex flex-col gap-2">
            {containers.map((c) => (
              <li key={c.id} className="glass-card p-3 text-xs">
                <p className="font-medium text-text">{c.name}</p>
                <p className="text-subtext0 font-mono mt-1">
                  {c.image} · {c.status}
                </p>
              </li>
            ))}
          </ul>
        )}
      </div>
    </motion.div>
  )
}
