import { useEffect, useRef, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import { AnimatePresence, motion } from "framer-motion"
import api, { type NotificationItemView } from "@/lib/api"
import { notificationQueryKeys } from "@/lib/ws-invalidation"
import { cn } from "@/lib/utils"

const MAX_TOASTS = 4
const TOAST_MS = 6000

type Toast = NotificationItemView & { shownAt: number }

export default function NotificationToasts() {
  const qc = useQueryClient()
  const seenRef = useRef<Set<number>>(new Set())
  const bootstrappedRef = useRef(false)
  const [toasts, setToasts] = useState<Toast[]>([])

  const { data: list } = useQuery({
    queryKey: notificationQueryKeys.list,
    queryFn: () => api.listNotifications(20),
    refetchInterval: 30_000,
  })

  useEffect(() => {
    if (list === undefined) return
    if (!bootstrappedRef.current) {
      for (const n of list) seenRef.current.add(n.id)
      bootstrappedRef.current = true
      return
    }
    const now = Date.now()
    const fresh = list.filter((n) => !seenRef.current.has(n.id))
    if (fresh.length === 0) return
    for (const n of fresh) seenRef.current.add(n.id)
    setToasts((prev) => {
      const added = fresh.map((n) => ({ ...n, shownAt: now }))
      return [...added, ...prev].slice(0, MAX_TOASTS)
    })
  }, [list])

  useEffect(() => {
    if (toasts.length === 0) return
    const id = window.setInterval(() => {
      const cutoff = Date.now() - TOAST_MS
      setToasts((prev) => prev.filter((t) => t.shownAt > cutoff))
    }, 500)
    return () => window.clearInterval(id)
  }, [toasts.length])

  const dismiss = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
    void api.dismissNotification(id).then(() => {
      void qc.invalidateQueries({ queryKey: notificationQueryKeys.list })
    })
  }

  const invokeAction = (id: number, actionKey: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
    void api.invokeNotificationAction(id, actionKey).then(() => {
      void qc.invalidateQueries({ queryKey: notificationQueryKeys.list })
    })
  }

  return (
    <div className="pointer-events-none absolute bottom-2 left-16 z-50 flex w-[min(320px,calc(100vw-5rem))] flex-col gap-2">
      <AnimatePresence mode="popLayout">
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            layout
            initial={{ opacity: 0, x: -16, scale: 0.96 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: -12, scale: 0.97 }}
            className={cn(
              "pointer-events-auto rounded-xl border border-surface0/80 bg-mantle/95 p-3 shadow-xl backdrop-blur-xl"
            )}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <p className="truncate text-[10px] font-semibold uppercase tracking-wide text-mauve">
                  {t.app_name}
                </p>
                <p className="truncate text-sm font-medium text-text">{t.summary || "(notification)"}</p>
                {t.body ? (
                  <p className="mt-0.5 line-clamp-2 text-xs leading-snug text-subtext0">{t.body}</p>
                ) : null}
              </div>
              <button
                type="button"
                className="icon shrink-0 text-lg text-subtext0 hover:text-text"
                aria-label="Dismiss"
                onClick={() => dismiss(t.id)}
              >
                close
              </button>
            </div>
            {t.actions.length > 0 ? (
              <div className="mt-2 flex flex-wrap gap-1">
                {t.actions.map((a) => (
                  <button
                    key={a.key}
                    type="button"
                    className="toggle-chip text-[10px]"
                    onClick={() => invokeAction(t.id, a.key)}
                  >
                    {a.label}
                  </button>
                ))}
              </div>
            ) : null}
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  )
}
