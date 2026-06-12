import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { getNavItem } from "../navigation"

export function VaultPane() {
  const { icon, label } = getNavItem("vault")

  const { data: list, isLoading: listLoading } = useQuery({
    queryKey: ["vault-list"],
    queryFn: api.vaultList,
    refetchInterval: 120_000,
  })

  const { data: backup, isLoading: backupLoading } = useQuery({
    queryKey: ["vault-backup"],
    queryFn: api.vaultBackupStatus,
    refetchInterval: 120_000,
  })

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex items-center gap-3">
        <span className="icon text-mauve text-2xl">{icon}</span>
        <h2 className="text-xl font-semibold">{label}</h2>
      </div>
      <p className="text-xs text-subtext1 max-w-prose">
        Read-only vault overview — remotes and backup status from the sidecar.
      </p>

      <section className="glass-card p-4 flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-text">Backup</h3>
        {backupLoading ? (
          <div className="skeleton h-12 rounded-lg" />
        ) : (
          <pre className="text-xs text-subtext0 whitespace-pre-wrap font-mono bg-mantle/60 rounded-lg p-3 border border-surface0/50">
            {JSON.stringify(backup ?? {}, null, 2)}
          </pre>
        )}
      </section>

      <section className="glass-card p-4 flex flex-col gap-2">
        <h3 className="text-sm font-semibold text-text">Remotes</h3>
        {listLoading ? (
          <div className="skeleton h-16 rounded-lg" />
        ) : (
          <pre className="text-xs text-subtext0 whitespace-pre-wrap font-mono bg-mantle/60 rounded-lg p-3 border border-surface0/50 max-h-80 overflow-y-auto">
            {JSON.stringify(list ?? {}, null, 2)}
          </pre>
        )}
      </section>
    </motion.div>
  )
}

export default VaultPane
