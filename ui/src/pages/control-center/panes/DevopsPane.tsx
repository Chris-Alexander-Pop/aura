import { useState } from "react"
import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

function labelizeKey(key: string): string {
  return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
}

export function DevopsPane() {
  const { icon, label } = getNavItem("devops")
  const qc = useQueryClient()
  const [logsFor, setLogsFor] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const statusQuery = useQuery({
    queryKey: ["devops-status"],
    queryFn: api.getDevopsStatus,
    refetchInterval: 30_000,
  })
  const containersQuery = useQuery({
    queryKey: ["devops-containers"],
    queryFn: api.getDevopsContainers,
    refetchInterval: 15_000,
  })
  const gitQuery = useQuery({
    queryKey: ["devops-git"],
    queryFn: () => api.getDevopsGitRepos(),
    refetchInterval: 120_000,
  })
  const timersQuery = useQuery({
    queryKey: ["devops-timers"],
    queryFn: api.getDevopsSystemdTimers,
    refetchInterval: 60_000,
  })
  const k8sQuery = useQuery({
    queryKey: ["devops-k8s"],
    queryFn: api.getDevopsKubernetesPods,
    refetchInterval: 60_000,
    retry: false,
  })

  const logsQuery = useQuery({
    queryKey: ["devops-logs", logsFor],
    queryFn: () => api.getDevopsContainerLogs(logsFor!, 80),
    enabled: logsFor != null,
  })

  const invalidateContainers = () => {
    void qc.invalidateQueries({ queryKey: ["devops-containers"] })
    void qc.invalidateQueries({ queryKey: ["devops-status"] })
  }

  const startMut = useMutation({
    mutationFn: (name: string) => api.startDevopsContainer(name),
    onSuccess: invalidateContainers,
    onError: (e) => setActionError(e instanceof Error ? e.message : "Start failed"),
  })
  const stopMut = useMutation({
    mutationFn: (name: string) => api.stopDevopsContainer(name),
    onSuccess: invalidateContainers,
    onError: (e) => setActionError(e instanceof Error ? e.message : "Stop failed"),
  })
  const restartMut = useMutation({
    mutationFn: (name: string) => api.restartDevopsContainer(name),
    onSuccess: invalidateContainers,
    onError: (e) => setActionError(e instanceof Error ? e.message : "Restart failed"),
  })

  const busy =
    startMut.isPending || stopMut.isPending || restartMut.isPending

  const data = statusQuery.data
  const containers = containersQuery.data ?? []

  let k8sPreview = ""
  if (k8sQuery.data?.pods) {
    try {
      const parsed = JSON.parse(k8sQuery.data.pods) as { items?: unknown[] }
      k8sPreview = `${parsed.items?.length ?? 0} pods (kubectl)`
    } catch {
      k8sPreview = "kubectl output available"
    }
  }

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
            Podman-first lifecycle (Docker when <code className="text-subtext0">AURA_ALLOW_DOCKER=1</code>
            ). Start/stop/restart containers; read git repos and systemd timers.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-sm shrink-0"
          onClick={() => {
            void statusQuery.refetch()
            void containersQuery.refetch()
            void gitQuery.refetch()
            void timersQuery.refetch()
          }}
          disabled={statusQuery.isFetching}
        >
          <span className={cn("icon text-base", statusQuery.isFetching && "animate-spin")}>
            refresh
          </span>
          Refresh
        </button>
      </div>

      {actionError ? (
        <p className="text-xs text-red" role="alert">
          {actionError}
        </p>
      ) : null}

      {statusQuery.isLoading ? (
        <div className="skeleton h-32 rounded-xl" />
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
          <p className="text-xs text-subtext1">No containers (Podman/Docker unavailable or none defined).</p>
        ) : (
          <ul className="flex flex-col gap-2">
            {containers.map((c) => (
              <li key={c.id} className="glass-card p-3 text-xs">
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div>
                    <p className="font-medium text-text">{c.name}</p>
                    <p className="text-subtext0 font-mono mt-1">
                      {c.image} · {c.status}
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    <button
                      type="button"
                      className="toggle-chip px-2 py-1 text-[10px]"
                      disabled={busy}
                      onClick={() => {
                        setActionError(null)
                        startMut.mutate(c.name)
                      }}
                    >
                      Start
                    </button>
                    <button
                      type="button"
                      className="toggle-chip px-2 py-1 text-[10px]"
                      disabled={busy}
                      onClick={() => {
                        if (!window.confirm(`Stop container ${c.name}?`)) return
                        setActionError(null)
                        stopMut.mutate(c.name)
                      }}
                    >
                      Stop
                    </button>
                    <button
                      type="button"
                      className="toggle-chip px-2 py-1 text-[10px]"
                      disabled={busy}
                      onClick={() => {
                        setActionError(null)
                        restartMut.mutate(c.name)
                      }}
                    >
                      Restart
                    </button>
                    <button
                      type="button"
                      className={cn(
                        "toggle-chip px-2 py-1 text-[10px]",
                        logsFor === c.name && "active"
                      )}
                      onClick={() => setLogsFor((prev) => (prev === c.name ? null : c.name))}
                    >
                      Logs
                    </button>
                  </div>
                </div>
                {logsFor === c.name ? (
                  <pre className="mt-2 max-h-40 overflow-auto rounded-lg bg-base/80 p-2 font-mono text-[10px] text-subtext1 whitespace-pre-wrap">
                    {logsQuery.isLoading
                      ? "Loading…"
                      : logsQuery.data?.logs || logsQuery.error?.message || "No logs"}
                  </pre>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </div>

      <div>
        <h3 className="text-sm font-medium text-text mb-2">Git repos (~/ scan)</h3>
        {gitQuery.isLoading ? (
          <div className="skeleton h-16 rounded-xl" />
        ) : (gitQuery.data ?? []).length === 0 ? (
          <p className="text-xs text-subtext1">No git repos found in home (depth 3).</p>
        ) : (
          <ul className="flex flex-col gap-1 max-h-40 overflow-y-auto">
            {(gitQuery.data ?? []).slice(0, 12).map((r) => (
              <li key={r.path} className="text-xs text-subtext1 font-mono truncate">
                {r.name} · {r.branch} · {r.status}
              </li>
            ))}
          </ul>
        )}
      </div>

      <div>
        <h3 className="text-sm font-medium text-text mb-2">Systemd timers</h3>
        {timersQuery.isLoading ? (
          <div className="skeleton h-16 rounded-xl" />
        ) : (timersQuery.data ?? []).length === 0 ? (
          <p className="text-xs text-subtext1">No timers or systemctl unavailable.</p>
        ) : (
          <ul className="flex flex-col gap-1 max-h-32 overflow-y-auto">
            {(timersQuery.data ?? []).slice(0, 8).map((t) => (
              <li key={t.name} className="text-xs text-subtext1">
                <span className="text-text">{t.name}</span> · next {t.next_run}
              </li>
            ))}
          </ul>
        )}
      </div>

      {k8sQuery.data ? (
        <div>
          <h3 className="text-sm font-medium text-text mb-2">Kubernetes</h3>
          <p className="text-xs text-subtext1">{k8sPreview}</p>
        </div>
      ) : null}
    </motion.div>
  )
}
