import { motion } from "framer-motion"
import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

type UnreadMap = Awaited<ReturnType<typeof api.getUnreadMessages>>

function parseCounts(raw: UnreadMap | undefined): Array<{ channel: string; count: number }> {
  if (!raw || typeof raw !== "object") return []
  return Object.entries(raw).map(([channel, v]) => {
    const n = typeof v === "number" ? v : Number(v)
    return { channel, count: Number.isFinite(n) && n > 0 ? Math.floor(n) : 0 }
  })
}

function formatChannelLabel(key: string): string {
  const s = key.replace(/[_-]+/g, " ").trim()
  if (!s) return key
  return s.replace(/\b\w/g, (c) => c.toUpperCase())
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

function KpiTile({
  label,
  value,
  detail,
  className,
}: {
  label: string
  value: string | number
  detail?: string
  className?: string
}) {
  return (
    <div className={cn("glass-card flex min-w-0 flex-col gap-1 p-3", className)}>
      <p className="text-[10px] font-semibold uppercase tracking-wide text-subtext0">{label}</p>
      <p className="text-2xl font-semibold tabular-nums text-text">{value}</p>
      {detail ? (
        <p className="truncate text-[11px] text-subtext1" title={detail}>
          {detail}
        </p>
      ) : null}
    </div>
  )
}

export function CommunicationPane() {
  const { icon, label } = getNavItem("communication")
  const { data, isLoading, isError, error, isFetching } = useQuery({
    queryKey: ["comm-unread"],
    queryFn: api.getUnreadMessages,
    refetchInterval: 20_000,
  })

  const rows = useMemo(() => {
    const parsed = parseCounts(data).filter((r) => r.count > 0)
    return [...parsed].sort((a, b) => b.count - a.count || a.channel.localeCompare(b.channel))
  }, [data])

  const totalUnread = useMemo(() => rows.reduce((s, r) => s + r.count, 0), [rows])
  const sourceCount = rows.length
  const top = rows[0]

  const topLabel = top ? formatChannelLabel(top.channel) : null

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18 }}
      className="flex h-full flex-col gap-4 overflow-y-auto p-6"
    >
      <div className="flex items-center gap-3">
        <span className="icon text-2xl text-mauve">{icon}</span>
        <div className="min-w-0 flex-1">
          <h2 className="text-xl font-semibold text-text">{label}</h2>
          <p className="mt-0.5 text-xs text-subtext1">
            {isLoading
              ? "Loading unread counts…"
              : isError
                ? "Could not refresh inbox stats."
                : totalUnread === 0
                  ? "Inbox is clear — no unread across connected sources."
                  : `${totalUnread} unread across ${sourceCount} source${sourceCount === 1 ? "" : "s"}.`}
            {isFetching && !isLoading ? <span className="text-subtext0"> · Updating…</span> : null}
          </p>
        </div>
      </div>

      {isError ? (
        <EmptyPanel
          icon="error_outline"
          title="Unread summary unavailable"
          body={
            error instanceof Error
              ? error.message
              : "Sidecar request failed. Check that ags-sidecar is running and Communication.GetUnread is registered."
          }
        />
      ) : null}

      {!isError && isLoading ? (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="glass-card p-3">
              <div className="skeleton mb-2 h-3 w-16 rounded" />
              <div className="skeleton h-8 w-2/3 rounded-lg" />
              <div className="skeleton mt-2 h-3 w-full rounded" />
            </div>
          ))}
        </div>
      ) : null}

      {!isError && !isLoading ? (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
          <KpiTile label="Unread" value={totalUnread} />
          <KpiTile label="Sources" value={sourceCount} detail={sourceCount ? "With new activity" : "None pending"} />
          <KpiTile
            label="Top source"
            value={top ? top.count : "—"}
            detail={topLabel ? `${topLabel}` : "All quiet"}
          />
        </div>
      ) : null}

      {!isError && !isLoading && totalUnread === 0 ? (
        <EmptyPanel
          icon="all_inbox"
          title="You're caught up"
          body="When the sidecar aggregates mail, chat, and other feeds, new unread counts land here with per-source breakdown."
        />
      ) : null}

      {!isError && !isLoading && totalUnread > 0 ? (
        <div className="flex flex-col gap-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-subtext0">By source</p>
          <ul className="flex flex-col gap-2">
            {rows.map(({ channel, count }) => (
              <li
                key={channel}
                className="glass-card flex items-center justify-between gap-3 p-3 transition-colors hover:bg-surface0/40"
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-text">{formatChannelLabel(channel)}</p>
                  <p className="truncate font-mono text-[11px] text-subtext0">{channel}</p>
                </div>
                <span
                  className={cn(
                    "shrink-0 rounded-full px-2.5 py-1 text-xs font-semibold tabular-nums",
                    "bg-mauve/20 text-mauve"
                  )}
                >
                  {count}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </motion.div>
  )
}
