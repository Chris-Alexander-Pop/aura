import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api, { type VaultBackupStatusView, type VaultListView } from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

function formatTimestamp(ts: number | null): string {
  if (ts == null || !Number.isFinite(ts)) return "Never"
  const ms = ts > 1_000_000_000_000 ? ts : ts * 1000
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(ms))
}

function backupStateClass(state: string): string {
  switch (state) {
    case "running":
      return "bg-blue/15 text-blue"
    case "error":
      return "bg-red/15 text-red"
    case "idle":
      return "bg-green/15 text-green"
    default:
      return "bg-surface0 text-subtext1"
  }
}

function BackupSection({ backup }: { backup: VaultBackupStatusView | undefined }) {
  if (!backup) {
    return <p className="text-xs text-subtext0">No backup status returned.</p>
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <span
          className={cn(
            "text-[10px] uppercase px-2 py-0.5 rounded-full font-semibold",
            backupStateClass(backup.state)
          )}
        >
          {backup.state}
        </span>
        {backup.in_progress ? (
          <span className="text-[10px] text-blue">In progress</span>
        ) : null}
      </div>
      <dl className="grid gap-2 sm:grid-cols-2 text-xs">
        <div>
          <dt className="text-subtext0 uppercase tracking-wide text-[10px]">Engine</dt>
          <dd className="text-text mt-0.5">{backup.engine ?? "Not configured"}</dd>
        </div>
        <div>
          <dt className="text-subtext0 uppercase tracking-wide text-[10px]">Last success</dt>
          <dd className="text-text mt-0.5">{formatTimestamp(backup.last_success_at)}</dd>
        </div>
      </dl>
      {backup.last_error ? (
        <p className="text-xs text-red border border-red/25 rounded-lg px-3 py-2">{backup.last_error}</p>
      ) : backup.state === "idle" && backup.engine == null ? (
        <p className="text-xs text-subtext0">
          Set <code className="text-subtext1">AURA_VAULT_RESTIC_REPO</code> to a restic repository path
          (local dir or <code className="text-subtext1">rclone:remote:path</code>) and install restic to
          probe snapshot status.
        </p>
      ) : null}
    </div>
  )
}

function RemotesSection({ list }: { list: VaultListView | undefined }) {
  if (!list) {
    return <p className="text-xs text-subtext0">No remote list returned.</p>
  }

  if (!list.rclone_available) {
    return (
      <div className="flex flex-col items-center gap-2 py-6 text-center">
        <span className="icon text-4xl text-subtext1">cloud_off</span>
        <p className="text-sm font-medium text-text">rclone not available</p>
        <p className="text-xs text-subtext0 max-w-sm">
          Install rclone and configure remotes to see vault destinations here.
        </p>
      </div>
    )
  }

  if (list.remotes.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-6 text-center">
        <span className="icon text-4xl text-subtext1">folder_off</span>
        <p className="text-sm font-medium text-text">No remotes configured</p>
        <p className="text-xs text-subtext0 max-w-sm">
          Run <code className="text-subtext1">rclone config</code> to add cloud remotes.
        </p>
      </div>
    )
  }

  return (
    <ul className="flex flex-col gap-2 list-none m-0 p-0">
      {list.remotes.map((remote) => (
        <li
          key={remote.name}
          className="flex items-center justify-between gap-3 rounded-lg border border-surface0/50 bg-surface0/30 px-3 py-2"
        >
          <div className="min-w-0">
            <p className="text-sm font-medium text-text truncate">{remote.name}</p>
            {remote.remote_type ? (
              <p className="text-[11px] text-subtext0 font-mono">{remote.remote_type}</p>
            ) : null}
          </div>
          <span className="icon text-lg text-subtext0 shrink-0">cloud</span>
        </li>
      ))}
    </ul>
  )
}

export function VaultPane() {
  const { icon, label } = getNavItem("vault")

  const {
    data: list,
    isLoading: listLoading,
    isError: listError,
    error: listErr,
    refetch: refetchList,
    isFetching: listFetching,
  } = useQuery({
    queryKey: ["vault-list"],
    queryFn: api.vaultList,
    refetchInterval: 120_000,
  })

  const {
    data: backup,
    isLoading: backupLoading,
    isError: backupError,
    error: backupErr,
    refetch: refetchBackup,
    isFetching: backupFetching,
  } = useQuery({
    queryKey: ["vault-backup"],
    queryFn: api.vaultBackupStatus,
    refetchInterval: 120_000,
  })

  const fetching = listFetching || backupFetching

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose mt-1">
            Read-only vault overview — rclone remotes and backup status from the sidecar.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-xs shrink-0"
          disabled={fetching}
          onClick={() => {
            void refetchList()
            void refetchBackup()
          }}
        >
          <span className={cn("icon text-base", fetching && "animate-spin")}>refresh</span>
          Refresh
        </button>
      </div>

      <section className="glass-card p-4 flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-text">Backup</h3>
        {backupLoading ? (
          <div className="skeleton h-16 rounded-lg" />
        ) : backupError ? (
          <p className="text-xs text-red">
            {backupErr instanceof Error ? backupErr.message : "Could not load backup status."}
          </p>
        ) : (
          <BackupSection backup={backup} />
        )}
      </section>

      <section className="glass-card p-4 flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-text">Remotes</h3>
        {listLoading ? (
          <div className="skeleton h-16 rounded-lg" />
        ) : listError ? (
          <p className="text-xs text-red">
            {listErr instanceof Error ? listErr.message : "Could not load remotes."}
          </p>
        ) : (
          <RemotesSection list={list} />
        )}
      </section>
    </motion.div>
  )
}

export default VaultPane
