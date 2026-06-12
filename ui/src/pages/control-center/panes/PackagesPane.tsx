import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import api from "@/lib/api"
import { getNavItem } from "../navigation"

/** Matches the sidecar `Package` JSON shape returned for pending updates. */
export interface PackageUpdateRow {
  name: string
  version: string
  description: string
  installed: boolean
}

function parsePackageUpdates(raw: unknown): PackageUpdateRow[] {
  if (!Array.isArray(raw)) return []
  const rows: PackageUpdateRow[] = []
  for (const item of raw) {
    if (!item || typeof item !== "object") continue
    const o = item as Record<string, unknown>
    if (typeof o.name !== "string" || typeof o.version !== "string") continue
    rows.push({
      name: o.name,
      version: o.version,
      description: typeof o.description === "string" ? o.description : "",
      installed: typeof o.installed === "boolean" ? o.installed : true,
    })
  }
  return rows
}

export function PackagesPane() {
  const { icon, label } = getNavItem("packages")
  const qc = useQueryClient()
  const [searchQ, setSearchQ] = useState("")

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["packages-updates"],
    queryFn: api.getPackageUpdates,
    refetchInterval: 120_000,
    select: parsePackageUpdates,
  })

  const { data: searchHits } = useQuery({
    queryKey: ["packages-search", searchQ],
    queryFn: () => api.searchPackages(searchQ.trim()),
    enabled: searchQ.trim().length >= 2,
  })

  const upgradeMut = useMutation({
    mutationFn: () => api.upgradePackages(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["packages-updates"] }),
  })

  const { data: txHistory } = useQuery({
    queryKey: ["packages-tx-history"],
    queryFn: () => api.getPackageTransactionHistory(15),
    refetchInterval: 300_000,
  })

  const updates = data ?? []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose">
            Pending upgrades reported by the sidecar. Refresh to re-sync with your package manager.
          </p>
        </div>
        <div className="flex gap-2 shrink-0">
          <button
            type="button"
            className="btn-surface text-xs"
            disabled={upgradeMut.isPending}
            onClick={() => upgradeMut.mutate()}
          >
            Upgrade all
          </button>
          <button
            type="button"
            className="btn-surface text-xs"
            onClick={() => refetch()}
            disabled={isFetching}
          >
            <span className="icon text-base">{isFetching ? "hourglass_empty" : "refresh"}</span>
            Refresh
          </button>
        </div>
      </div>

      <input
        className="input text-sm"
        placeholder="Search packages…"
        value={searchQ}
        onChange={(e) => setSearchQ(e.target.value)}
      />
      {(searchHits?.length ?? 0) > 0 ? (
        <ul className="flex flex-col gap-1 text-xs text-subtext1">
          {searchHits!.slice(0, 8).map((p) => (
            <li key={p.name} className="truncate">
              {p.name} — {p.version}
            </li>
          ))}
        </ul>
      ) : null}

      {isLoading ? (
        <div className="flex flex-col gap-2">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="skeleton h-14 rounded-xl" style={{ opacity: 1 - i * 0.14 }} />
          ))}
        </div>
      ) : isError ? (
        <div className="glass-card p-4 border-red/20">
          <p className="text-sm text-red font-medium">Could not load updates</p>
          <p className="text-xs text-subtext0 mt-2">{error instanceof Error ? error.message : "Unknown error"}</p>
        </div>
      ) : updates.length === 0 ? (
        <div className="flex flex-col items-center justify-center gap-4 flex-1 min-h-[240px] px-6 py-10 rounded-2xl border border-surface0/80 bg-surface0/25">
          <div className="rounded-full bg-green/15 p-5">
            <span className="icon text-5xl text-green">verified_user</span>
          </div>
          <div className="text-center max-w-sm">
            <p className="text-lg font-semibold text-text">You're up to date</p>
            <p className="text-sm text-subtext1 mt-2 leading-relaxed">
              No pending package upgrades were returned. Run a database sync and check again if you expected changes.
            </p>
          </div>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          <p className="text-xs text-subtext0">{updates.length} package{updates.length === 1 ? "" : "s"} pending</p>
          <ul className="flex flex-col gap-2 list-none p-0 m-0">
            {updates.map((pkg) => (
              <li key={`${pkg.name}-${pkg.version}`} className="glass-card p-3 flex items-start gap-3">
                <span className="icon text-lg text-mauve shrink-0 mt-0.5">install_mobile</span>
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium text-text truncate">{pkg.name}</p>
                  <p className="text-xs text-subtext0 font-mono truncate">{pkg.version}</p>
                  {pkg.description ? (
                    <p className="text-xs text-subtext1 mt-1 line-clamp-2">{pkg.description}</p>
                  ) : null}
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {(txHistory?.length ?? 0) > 0 ? (
        <section className="glass-card p-4 flex flex-col gap-2">
          <h3 className="text-sm font-semibold text-text">Recent transactions</h3>
          <ul className="flex flex-col gap-1 text-xs text-subtext1 max-h-40 overflow-y-auto">
            {txHistory!.map((tx, i) => (
              <li key={`${tx.ts}-${i}`} className="truncate">
                <span className="text-subtext0">{tx.ts}</span> · {tx.action}:{" "}
                {(tx.packages ?? []).join(", ") || "—"}
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </motion.div>
  )
}
