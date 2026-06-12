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

function invalidatePackageQueries(qc: ReturnType<typeof useQueryClient>) {
  void qc.invalidateQueries({ queryKey: ["packages-updates"] })
  void qc.invalidateQueries({ queryKey: ["packages-tx-history"] })
  void qc.invalidateQueries({ queryKey: ["packages-search"] })
}

export function PackagesPane() {
  const { icon, label } = getNavItem("packages")
  const qc = useQueryClient()
  const [searchQ, setSearchQ] = useState("")
  const [actionError, setActionError] = useState<string | null>(null)

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["packages-updates"],
    queryFn: api.getPackageUpdates,
    refetchInterval: 120_000,
    select: parsePackageUpdates,
  })

  const trimmedSearch = searchQ.trim()
  const {
    data: searchHits,
    isFetching: searchFetching,
    isError: searchError,
    error: searchErr,
  } = useQuery({
    queryKey: ["packages-search", trimmedSearch],
    queryFn: () => api.searchPackages(trimmedSearch),
    enabled: trimmedSearch.length >= 2,
  })

  const upgradeMut = useMutation({
    mutationFn: () => api.upgradePackages(),
    onSuccess: () => {
      setActionError(null)
      invalidatePackageQueries(qc)
    },
    onError: (e) => setActionError(e instanceof Error ? e.message : "Upgrade failed"),
  })

  const installMut = useMutation({
    mutationFn: (name: string) => api.installPackage(name),
    onSuccess: () => {
      setActionError(null)
      invalidatePackageQueries(qc)
    },
    onError: (e) => setActionError(e instanceof Error ? e.message : "Install failed"),
  })

  const removeMut = useMutation({
    mutationFn: (name: string) => api.removePackage(name),
    onSuccess: () => {
      setActionError(null)
      invalidatePackageQueries(qc)
    },
    onError: (e) => setActionError(e instanceof Error ? e.message : "Remove failed"),
  })

  const {
    data: txHistory,
    isLoading: txLoading,
    isError: txError,
    error: txErr,
  } = useQuery({
    queryKey: ["packages-tx-history"],
    queryFn: () => api.getPackageTransactionHistory(15),
    refetchInterval: 300_000,
  })

  const updates = data ?? []
  const packageBusy = upgradeMut.isPending || installMut.isPending || removeMut.isPending

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
            Search, install, remove, and review pending upgrades. Transaction history is logged by the sidecar.
          </p>
        </div>
        <div className="flex gap-2 shrink-0">
          <button
            type="button"
            className="btn-surface text-xs"
            disabled={packageBusy}
            onClick={() => upgradeMut.mutate()}
          >
            {upgradeMut.isPending ? "Upgrading…" : "Upgrade all"}
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

      {actionError ? (
        <div className="glass-card p-3 border-red/20 text-xs text-red">{actionError}</div>
      ) : null}

      <section className="glass-card p-4 flex flex-col gap-3">
        <h3 className="text-sm font-semibold text-text">Search packages</h3>
        <input
          className="input text-sm"
          placeholder="Search packages (min 2 chars)…"
          value={searchQ}
          onChange={(e) => setSearchQ(e.target.value)}
        />
        {trimmedSearch.length >= 2 && searchFetching ? (
          <p className="text-xs text-subtext0">Searching…</p>
        ) : null}
        {searchError ? (
          <p className="text-xs text-red">
            {searchErr instanceof Error ? searchErr.message : "Search failed"}
          </p>
        ) : null}
        {(searchHits?.length ?? 0) > 0 ? (
          <ul className="flex flex-col gap-2 text-xs list-none p-0 m-0">
            {searchHits!.slice(0, 12).map((p) => (
              <li
                key={p.name}
                className="flex items-center justify-between gap-2 border-b border-surface0/40 pb-2 last:border-0"
              >
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium text-text truncate">{p.name}</p>
                  <p className="text-subtext0 truncate">{p.version}</p>
                  {p.description ? (
                    <p className="text-subtext1 mt-0.5 line-clamp-1">{p.description}</p>
                  ) : null}
                </div>
                <button
                  type="button"
                  className="toggle-chip text-[10px] shrink-0"
                  disabled={packageBusy}
                  onClick={() => installMut.mutate(p.name)}
                >
                  {installMut.isPending ? "…" : "Install"}
                </button>
              </li>
            ))}
          </ul>
        ) : trimmedSearch.length >= 2 && !searchFetching && !searchError ? (
          <p className="text-xs text-subtext0">No matches</p>
        ) : null}
      </section>

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
        <div className="flex flex-col items-center justify-center gap-4 flex-1 min-h-[180px] px-6 py-8 rounded-2xl border border-surface0/80 bg-surface0/25">
          <div className="rounded-full bg-green/15 p-5">
            <span className="icon text-5xl text-green">verified_user</span>
          </div>
          <div className="text-center max-w-sm">
            <p className="text-lg font-semibold text-text">You're up to date</p>
            <p className="text-sm text-subtext1 mt-2 leading-relaxed">
              No pending package upgrades were returned.
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
                <button
                  type="button"
                  className="toggle-chip text-[10px] text-red shrink-0"
                  disabled={packageBusy}
                  title="Remove installed package"
                  onClick={() => removeMut.mutate(pkg.name)}
                >
                  {removeMut.isPending ? "…" : "Remove"}
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      <section className="glass-card p-4 flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-text">Recent transactions</h3>
        {txLoading ? (
          <div className="skeleton h-16 rounded-lg" />
        ) : txError ? (
          <p className="text-xs text-red">
            {txErr instanceof Error ? txErr.message : "Could not load transaction history"}
          </p>
        ) : (txHistory?.length ?? 0) === 0 ? (
          <p className="text-xs text-subtext0">No logged transactions yet</p>
        ) : (
          <ul className="flex flex-col gap-1 text-xs text-subtext1 max-h-40 overflow-y-auto list-none p-0 m-0">
            {txHistory!.map((tx, i) => (
              <li key={`${tx.ts}-${i}`} className="truncate">
                <span className="text-subtext0">{tx.ts}</span> · {tx.action}:{" "}
                {(tx.packages ?? []).join(", ") || "—"}
              </li>
            ))}
          </ul>
        )}
      </section>
    </motion.div>
  )
}
