import type { ReactNode } from "react"
import { AnimatePresence, motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { notificationQueryKeys } from "@/lib/ws-invalidation"
import { cn } from "@/lib/utils"

function formatNotifWhen(ts: number): string {
  if (!ts) return ""
  const d = new Date(ts * 1000)
  const now = new Date()
  const sameDay =
    d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate()
  const time = d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" })
  if (sameDay) return time
  const day = d.toLocaleDateString(undefined, { month: "short", day: "numeric" })
  return `${day} · ${time}`
}

function EmptyHint({ children }: { children: ReactNode }) {
  return <p className="px-0.5 py-2 text-[11px] leading-snug text-subtext0">{children}</p>
}

export default function CalendarNotificationHistory() {
  const qc = useQueryClient()

  const listQuery = useQuery({
    queryKey: notificationQueryKeys.list,
    queryFn: () => api.listNotifications(80),
    refetchInterval: 15_000,
  })

  const dismissMut = useMutation({
    mutationFn: (id: number) => api.dismissNotification(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: notificationQueryKeys.list }),
  })

  const clearMut = useMutation({
    mutationFn: () => api.clearAllNotifications(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: notificationQueryKeys.list }),
  })

  const actionMut = useMutation({
    mutationFn: ({ id, action_key }: { id: number; action_key: string }) =>
      api.invokeNotificationAction(id, action_key),
    onSuccess: () => void qc.invalidateQueries({ queryKey: notificationQueryKeys.list }),
  })

  const items = listQuery.data ?? []

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="mb-2 flex shrink-0 items-center justify-between gap-2">
        <div className="min-w-0">
          <p className="text-[10px] font-semibold uppercase tracking-wider text-subtext0">Inbox</p>
          <p className="truncate text-sm font-semibold text-text">
            {items.length === 0 ? "No notifications" : `${items.length} notification${items.length === 1 ? "" : "s"}`}
          </p>
        </div>
        <button
          type="button"
          className="btn-ghost shrink-0 px-2 text-[11px]"
          disabled={items.length === 0 || clearMut.isPending}
          onClick={() => clearMut.mutate()}
          title="Clear all"
        >
          Clear
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {listQuery.isLoading && listQuery.data === undefined ? (
          <EmptyHint>Loading notifications…</EmptyHint>
        ) : listQuery.isError ? (
          <EmptyHint>Could not load notifications — is the sidecar running?</EmptyHint>
        ) : items.length === 0 ? (
          <EmptyHint>No notification history yet. New ones from the bus show up here.</EmptyHint>
        ) : (
          <ul className="flex flex-col gap-1.5 pb-1">
            <AnimatePresence initial={false}>
              {items.map((n) => (
                <motion.li
                  key={n.id}
                  layout
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -4 }}
                  className="group flex items-start gap-2 rounded-xl border border-surface0/35 bg-surface0/25 px-2.5 py-2"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-baseline justify-between gap-2">
                      <p className="truncate text-[10px] font-medium text-mauve">{n.app_name}</p>
                      <p className="shrink-0 text-[10px] text-subtext1">{formatNotifWhen(n.timestamp)}</p>
                    </div>
                    <p className="mt-0.5 text-[13px] font-medium leading-snug text-text">
                      {n.summary || "(no title)"}
                    </p>
                    {n.body ? (
                      <p className="mt-0.5 line-clamp-3 text-[11px] leading-snug text-subtext0">{n.body}</p>
                    ) : null}
                    {n.actions.length > 0 ? (
                      <div className="mt-1.5 flex flex-wrap gap-1">
                        {n.actions.map((a) => (
                          <button
                            key={a.key}
                            type="button"
                            className="btn-ghost px-2 py-0.5 text-[10px]"
                            disabled={actionMut.isPending}
                            onClick={() => actionMut.mutate({ id: n.id, action_key: a.key })}
                          >
                            {a.label}
                          </button>
                        ))}
                      </div>
                    ) : null}
                  </div>
                  <button
                    type="button"
                    className={cn(
                      "icon-btn opacity-0 transition-opacity group-hover:opacity-70 hover:!opacity-100"
                    )}
                    aria-label="Dismiss"
                    onClick={() => dismissMut.mutate(n.id)}
                  >
                    <span className="icon text-sm">close</span>
                  </button>
                </motion.li>
              ))}
            </AnimatePresence>
          </ul>
        )}
      </div>
    </div>
  )
}
