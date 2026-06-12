import { useCallback, useEffect, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type DndPrefsView } from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

const STORAGE_KEY = "aura.control-center.notifications.dndSchedule.v1"

/** @deprecated Use DndPrefsView from api — kept for Dropdown.tsx localStorage shape */
export type DndSchedulePrefs = {
  scheduleEnabled: boolean
  startTime: string
  endTime: string
  weekdaysOnly: boolean
}

const DEFAULT_DND: DndPrefsView = {
  enabled: false,
  schedule_enabled: false,
  start_time: "22:00",
  end_time: "07:00",
  weekdays_only: true,
}

function parseHm(value: string): { h: number; m: number } | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value.trim())
  if (!match) return null
  const h = Number(match[1])
  const m = Number(match[2])
  if (h > 23 || m > 59 || Number.isNaN(h) || Number.isNaN(m)) return null
  return { h, m }
}

function toMinutes({ h, m }: { h: number; m: number }): number {
  return h * 60 + m
}

function isWeekday(d: Date): boolean {
  const day = d.getDay()
  return day >= 1 && day <= 5
}

function normalizeDndPrefs(prefs: DndPrefsView | DndSchedulePrefs): DndPrefsView {
  if ("schedule_enabled" in prefs) return prefs
  return {
    enabled: prefs.scheduleEnabled,
    schedule_enabled: prefs.scheduleEnabled,
    start_time: prefs.startTime,
    end_time: prefs.endTime,
    weekdays_only: prefs.weekdaysOnly,
  }
}

export function isWithinScheduledQuietHours(prefs: DndPrefsView | DndSchedulePrefs, now = new Date()): boolean {
  const p = normalizeDndPrefs(prefs)
  if (!p.schedule_enabled) return false
  if (p.weekdays_only && !isWeekday(now)) return false

  const start = parseHm(p.start_time)
  const end = parseHm(p.end_time)
  if (!start || !end) return false

  const cur = toMinutes({ h: now.getHours(), m: now.getMinutes() })
  const sm = toMinutes(start)
  const em = toMinutes(end)

  if (sm === em) return false
  if (sm < em) return cur >= sm && cur < em
  return cur >= sm || cur < em
}

function loadLocalDndFallback(): DndPrefsView {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULT_DND }
    const parsed = JSON.parse(raw) as Record<string, unknown>
    return {
      enabled: parsed.scheduleEnabled === true,
      schedule_enabled: parsed.scheduleEnabled === true,
      start_time:
        typeof parsed.startTime === "string" && parseHm(parsed.startTime)
          ? parsed.startTime
          : DEFAULT_DND.start_time,
      end_time:
        typeof parsed.endTime === "string" && parseHm(parsed.endTime)
          ? parsed.endTime
          : DEFAULT_DND.end_time,
      weekdays_only: parsed.weekdaysOnly !== false,
    }
  } catch {
    return { ...DEFAULT_DND }
  }
}

function ScheduleToggleRow({
  label,
  description,
  active,
  onToggle,
  disabled,
}: {
  label: string
  description: string
  active: boolean
  onToggle: () => void
  disabled?: boolean
}) {
  return (
    <div className={cn("flex items-start justify-between gap-4", disabled && "opacity-50 pointer-events-none")}>
      <div className="min-w-0">
        <p className="text-sm font-medium text-text">{label}</p>
        <p className="text-xs text-subtext0 leading-relaxed mt-1 max-w-prose">{description}</p>
      </div>
      <button
        type="button"
        onClick={onToggle}
        className={cn("toggle-chip shrink-0 text-xs mt-0.5", active && "active")}
        aria-pressed={active}
      >
        <span className="icon text-base">{active ? "toggle_on" : "toggle_off"}</span>
        {active ? "On" : "Off"}
      </button>
    </div>
  )
}

function formatTime(ts: number): string {
  if (!ts) return ""
  const d = new Date(ts * 1000)
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" })
}

export function NotificationsPane() {
  const { icon, label } = getNavItem("notifications")
  const qc = useQueryClient()
  const [tick, setTick] = useState(0)
  const [dndLocal, setDndLocal] = useState<DndPrefsView | null>(null)

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Notifications.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["notifications-list"] })
    })
    return off
  }, [qc])

  useEffect(() => {
    const id = window.setInterval(() => setTick((t) => t + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])

  const listQuery = useQuery({
    queryKey: ["notifications-list"],
    queryFn: () => api.listNotifications(80),
    refetchInterval: 15_000,
  })

  const dndQuery = useQuery({
    queryKey: ["notifications-dnd"],
    queryFn: async () => {
      try {
        return await api.getNotificationDnd()
      } catch {
        return loadLocalDndFallback()
      }
    },
  })

  const prefs = dndLocal ?? dndQuery.data ?? DEFAULT_DND

  const saveDnd = useMutation({
    mutationFn: (next: DndPrefsView) => api.setNotificationDnd(next),
    onSuccess: (_data, next) => {
      qc.setQueryData(["notifications-dnd"], next)
      setDndLocal(null)
    },
    onError: () => {
      try {
        localStorage.setItem(
          STORAGE_KEY,
          JSON.stringify({
            scheduleEnabled: prefs.schedule_enabled,
            startTime: prefs.start_time,
            endTime: prefs.end_time,
            weekdaysOnly: prefs.weekdays_only,
          })
        )
      } catch {
        /* ignore */
      }
    },
  })

  const updatePrefs = useCallback(
    (patch: Partial<DndPrefsView>) => {
      const next = { ...prefs, ...patch }
      setDndLocal(next)
      saveDnd.mutate(next)
    },
    [prefs, saveDnd]
  )

  const dismissMut = useMutation({
    mutationFn: (id: number) => api.dismissNotification(id),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["notifications-list"] }),
  })

  const clearMut = useMutation({
    mutationFn: () => api.clearAllNotifications(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["notifications-list"] }),
  })

  const rulesQuery = useQuery({
    queryKey: ["notifications-rules"],
    queryFn: api.getNotificationRules,
  })

  const rulesMut = useMutation({
    mutationFn: (muted_apps: string[]) => api.setNotificationRules({ muted_apps }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["notifications-rules"] }),
  })

  const actionMut = useMutation({
    mutationFn: ({ id, action_key }: { id: number; action_key: string }) =>
      api.invokeNotificationAction(id, action_key),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["notifications-list"] }),
  })

  const mutedApps = rulesQuery.data?.muted_apps ?? []

  const inQuietHours = useMemo(() => isWithinScheduledQuietHours(prefs), [prefs, tick])

  const overnight =
    parseHm(prefs.start_time) &&
    parseHm(prefs.end_time) &&
    toMinutes(parseHm(prefs.start_time)!) > toMinutes(parseHm(prefs.end_time)!)

  const items = listQuery.data ?? []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-6 p-6 h-full overflow-y-auto"
    >
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-sm text-subtext0 leading-relaxed max-w-prose">
          Inbox history from the Freedesktop notification bus (any compliant daemon). Quiet-hour preferences sync to
          the sidecar; if the sidecar is offline, schedule falls back to browser storage.
        </p>
      </header>

      <section className="glass-card p-5 flex flex-col gap-4">
        <div className="flex items-center justify-between gap-3">
          <h3 className="text-sm font-semibold text-text">Inbox</h3>
          <button
            type="button"
            className="text-xs text-subtext1 hover:text-text px-2 py-1 rounded-lg border border-surface1/60"
            disabled={items.length === 0 || clearMut.isPending}
            onClick={() => clearMut.mutate()}
          >
            Clear all
          </button>
        </div>
        {listQuery.isLoading && <p className="text-xs text-subtext0">Loading…</p>}
        {listQuery.isError && (
          <p className="text-xs text-peach">Could not load notifications — is ags-sidecar running?</p>
        )}
        {!listQuery.isLoading && items.length === 0 && (
          <p className="text-xs text-subtext0">No notifications captured yet. Try `notify-send` with a daemon running.</p>
        )}
        <ul className="flex flex-col gap-2 max-h-64 overflow-y-auto">
          {items.map((n) => (
            <li
              key={n.id}
              className="flex items-start justify-between gap-3 rounded-xl border border-surface0/70 bg-base/50 px-3 py-2.5"
            >
              <div className="min-w-0">
                <p className="text-xs text-mauve font-medium truncate">{n.app_name}</p>
                <p className="text-sm text-text font-medium truncate">{n.summary || "(no title)"}</p>
                {n.body ? <p className="text-xs text-subtext0 line-clamp-2 mt-0.5">{n.body}</p> : null}
                {n.actions.length > 0 ? (
                  <div className="mt-2 flex flex-wrap gap-1">
                    {n.actions.map((a) => (
                      <button
                        key={a.key}
                        type="button"
                        className="toggle-chip text-[10px]"
                        disabled={actionMut.isPending}
                        onClick={() => actionMut.mutate({ id: n.id, action_key: a.key })}
                      >
                        {a.label}
                      </button>
                    ))}
                  </div>
                ) : null}
                <p className="text-[10px] text-subtext1 mt-1">{formatTime(n.timestamp)}</p>
              </div>
              <button
                type="button"
                className="shrink-0 icon text-subtext0 hover:text-text text-lg"
                aria-label="Dismiss"
                onClick={() => dismissMut.mutate(n.id)}
              >
                close
              </button>
            </li>
          ))}
        </ul>
      </section>

      <section className="glass-card p-5 flex flex-col gap-3">
        <h3 className="text-sm font-semibold text-text">App rules</h3>
        <p className="text-xs text-subtext0">Muted apps are suppressed in the inbox UI.</p>
        {rulesQuery.isLoading ? (
          <p className="text-xs text-subtext0">Loading rules…</p>
        ) : (
          <>
            <div className="flex flex-wrap gap-1">
              {mutedApps.length === 0 ? (
                <span className="text-xs text-subtext0">No muted apps</span>
              ) : (
                mutedApps.map((app) => (
                  <button
                    key={app}
                    type="button"
                    className="toggle-chip text-[10px] active"
                    onClick={() =>
                      rulesMut.mutate(mutedApps.filter((a) => a !== app))
                    }
                  >
                    {app} ×
                  </button>
                ))
              )}
            </div>
            <form
              className="flex gap-2"
              onSubmit={(e) => {
                e.preventDefault()
                const fd = new FormData(e.currentTarget)
                const app = String(fd.get("app") ?? "").trim()
                if (!app || mutedApps.includes(app)) return
                rulesMut.mutate([...mutedApps, app])
                e.currentTarget.reset()
              }}
            >
              <input name="app" className="input flex-1 text-sm" placeholder="App name to mute" />
              <button type="submit" className="btn-surface text-xs" disabled={rulesMut.isPending}>
                Mute app
              </button>
            </form>
          </>
        )}
      </section>

      <section className="glass-card p-5 flex flex-col gap-5">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h3 className="text-sm font-semibold text-text">Quiet hours schedule</h3>
            <p className="text-xs text-subtext0 mt-1 max-w-prose">
              Aura preference (stored in sidecar). Does not force OS DND on all apps until daemon integration lands.
            </p>
          </div>
          <div
            className={cn(
              "shrink-0 rounded-full px-3 py-1 text-[11px] font-medium border",
              prefs.schedule_enabled && inQuietHours
                ? "border-peach/40 text-peach bg-peach/10"
                : prefs.schedule_enabled
                  ? "border-surface1 text-subtext1 bg-surface0/80"
                  : "border-surface1/60 text-subtext0 bg-surface0/40"
            )}
          >
            {prefs.schedule_enabled ? (inQuietHours ? "Inside window now" : "Outside window now") : "Schedule off"}
          </div>
        </div>

        <ScheduleToggleRow
          label="Do not disturb (manual)"
          description="When on, Aura treats notifications as suppressed in UI."
          active={prefs.enabled}
          onToggle={() => updatePrefs({ enabled: !prefs.enabled })}
        />

        <ScheduleToggleRow
          label="Enable scheduled quiet hours"
          description="Prefer silence during the configured local-time window."
          active={prefs.schedule_enabled}
          onToggle={() => updatePrefs({ schedule_enabled: !prefs.schedule_enabled })}
        />

        <div
          className={cn(
            "flex flex-col gap-4 pt-1 border-t border-surface1/30",
            !prefs.schedule_enabled && "opacity-45 pointer-events-none"
          )}
        >
          <div className="grid grid-cols-2 gap-3">
            <label className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-subtext1">Start</span>
              <input
                type="time"
                className="input"
                value={prefs.start_time}
                onChange={(e) => updatePrefs({ start_time: e.target.value })}
              />
            </label>
            <label className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-subtext1">End</span>
              <input
                type="time"
                className="input"
                value={prefs.end_time}
                onChange={(e) => updatePrefs({ end_time: e.target.value })}
              />
            </label>
          </div>

          {overnight && (
            <p className="text-xs text-subtext1 leading-relaxed flex gap-2">
              <span className="icon text-peach shrink-0 text-lg">schedule</span>
              <span>Overnight window crosses midnight until the end time.</span>
            </p>
          )}

          <ScheduleToggleRow
            label="Weekdays only"
            description="Monday through Friday only."
            active={prefs.weekdays_only}
            onToggle={() => updatePrefs({ weekdays_only: !prefs.weekdays_only })}
            disabled={!prefs.schedule_enabled}
          />
        </div>
      </section>
    </motion.div>
  )
}
