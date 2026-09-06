import { motion } from "framer-motion"
import { useMemo } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
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

const APP_ICONS: Record<string, string> = {
  telegram: "send",
  discord: "forum",
  signal: "lock",
  whatsapp: "chat",
  slack: "tag",
}

export function CommunicationPane() {
  const { icon, label } = getNavItem("communication")
  const { data, isLoading, isError, error, isFetching } = useQuery({
    queryKey: ["comm-unread"],
    queryFn: api.getUnreadMessages,
    refetchInterval: 20_000,
  })
  const appsQuery = useQuery({
    queryKey: ["comm-apps"],
    queryFn: api.getCommunicationApps,
    refetchInterval: 120_000,
  })

  const launchMut = useMutation({
    mutationFn: (app: string) => api.launchCommunicationApp(app),
  })

  const rows = useMemo(() => {
    const parsed = parseCounts(data).filter((r) => r.count > 0)
    return [...parsed].sort((a, b) => b.count - a.count || a.channel.localeCompare(b.channel))
  }, [data])

  const totalUnread = useMemo(() => rows.reduce((s, r) => s + r.count, 0), [rows])
  const apps = appsQuery.data ?? []

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
          <p className="mt-0.5 text-xs text-subtext1 max-w-prose">
            Launch installed clients here. Unified inbox unread counts come from your separate comm app via{" "}
            <code className="text-subtext0">Communication.ImportUnread</code> (see{" "}
            <code className="text-subtext0">docs/integrations/communication-hub.md</code>).
          </p>
        </div>
      </div>

      <section className="glass-card p-4">
        <h3 className="text-sm font-semibold text-text mb-2">Quick launch</h3>
        {appsQuery.isLoading ? (
          <div className="skeleton h-12 rounded-lg" />
        ) : apps.length === 0 ? (
          <p className="text-xs text-subtext0">
            No supported messaging apps on PATH (telegram, discord, signal, whatsapp, slack).
          </p>
        ) : (
          <div className="flex flex-wrap gap-2">
            {apps.map((app) => (
              <button
                key={app}
                type="button"
                className="toggle-chip text-xs capitalize"
                disabled={launchMut.isPending}
                onClick={() => launchMut.mutate(app)}
              >
                <span className="icon text-base">{APP_ICONS[app] ?? "chat"}</span>
                {app}
              </button>
            ))}
          </div>
        )}
        {launchMut.isError ? (
          <p className="mt-2 text-xs text-red">
            {launchMut.error instanceof Error ? launchMut.error.message : "Launch failed"}
          </p>
        ) : null}
      </section>

      {isError ? (
        <p className="text-xs text-red">
          {error instanceof Error ? error.message : "Could not load unread counts"}
        </p>
      ) : null}

      {!isError && !isLoading && totalUnread === 0 ? (
        <div className="glass-card flex flex-col items-center gap-2 px-6 py-10 text-center">
          <span className="icon text-5xl text-subtext1">all_inbox</span>
          <p className="text-sm font-semibold text-text">Unified inbox pending</p>
          <p className="text-xs text-subtext0 max-w-md">
            When your comm hub pushes unread counts through the sidecar, they appear here. Until then, use quick
            launch above.
          </p>
        </div>
      ) : null}

      {!isError && !isLoading && totalUnread > 0 ? (
        <div className="flex flex-col gap-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-subtext0">
            Unread · {totalUnread}
            {isFetching ? " · updating…" : ""}
          </p>
          <ul className="flex flex-col gap-2">
            {rows.map(({ channel, count }) => (
              <li
                key={channel}
                className="glass-card flex items-center justify-between gap-3 p-3"
              >
                <p className="text-sm font-medium text-text">{formatChannelLabel(channel)}</p>
                <span className="rounded-full bg-mauve/20 px-2.5 py-1 text-xs font-semibold text-mauve">
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
