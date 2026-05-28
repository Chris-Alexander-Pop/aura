import { useCallback, useEffect, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

const STORAGE_KEY = "aura.control-center.notifications.dndSchedule.v1"

export type DndSchedulePrefs = {
  /** Master switch for automatic quiet hours (stored locally). */
  scheduleEnabled: boolean
  /** Local `HH:mm` — inclusive start of the quiet window. */
  startTime: string
  /** Local `HH:mm` — exclusive end when the window does not cross midnight; see overnight note below. */
  endTime: string
  /** When true, Monday–Friday only (Saturday/Sunday never enter quiet hours). */
  weekdaysOnly: boolean
}

const DEFAULT_PREFS: DndSchedulePrefs = {
  scheduleEnabled: false,
  startTime: "22:00",
  endTime: "07:00",
  weekdaysOnly: true,
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

/** Whether `now` falls inside the configured window (local clock). Overnight windows supported. */
export function isWithinScheduledQuietHours(prefs: DndSchedulePrefs, now = new Date()): boolean {
  if (!prefs.scheduleEnabled) return false
  if (prefs.weekdaysOnly && !isWeekday(now)) return false

  const start = parseHm(prefs.startTime)
  const end = parseHm(prefs.endTime)
  if (!start || !end) return false

  const cur = toMinutes({ h: now.getHours(), m: now.getMinutes() })
  const sm = toMinutes(start)
  const em = toMinutes(end)

  if (sm === em) return false

  if (sm < em) return cur >= sm && cur < em
  return cur >= sm || cur < em
}

function loadPrefs(): DndSchedulePrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULT_PREFS }
    const parsed = JSON.parse(raw) as Partial<DndSchedulePrefs>
    return {
      scheduleEnabled: typeof parsed.scheduleEnabled === "boolean" ? parsed.scheduleEnabled : DEFAULT_PREFS.scheduleEnabled,
      startTime: typeof parsed.startTime === "string" && parseHm(parsed.startTime) ? parsed.startTime : DEFAULT_PREFS.startTime,
      endTime: typeof parsed.endTime === "string" && parseHm(parsed.endTime) ? parsed.endTime : DEFAULT_PREFS.endTime,
      weekdaysOnly: typeof parsed.weekdaysOnly === "boolean" ? parsed.weekdaysOnly : DEFAULT_PREFS.weekdaysOnly,
    }
  } catch {
    return { ...DEFAULT_PREFS }
  }
}

function savePrefs(prefs: DndSchedulePrefs) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs))
  } catch {
    /* quota / privacy mode — prefs stay in memory for the session */
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

export function NotificationsPane() {
  const { icon, label } = getNavItem("notifications")
  const [prefs, setPrefs] = useState<DndSchedulePrefs>(() => loadPrefs())
  const [tick, setTick] = useState(0)

  useEffect(() => {
    savePrefs(prefs)
  }, [prefs])

  const updatePrefs = useCallback((patch: Partial<DndSchedulePrefs>) => {
    setPrefs((prev) => ({ ...prev, ...patch }))
  }, [])

  useEffect(() => {
    const id = window.setInterval(() => setTick((t) => t + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])

  const inQuietHours = useMemo(() => isWithinScheduledQuietHours(prefs), [prefs, tick])

  const overnight =
    parseHm(prefs.startTime) &&
    parseHm(prefs.endTime) &&
    toMinutes(parseHm(prefs.startTime)!) > toMinutes(parseHm(prefs.endTime)!)

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
          Decide when your session should prefer silence. These controls only persist preferences in this browser&apos;s{" "}
          <code className="text-subtext1 text-xs">localStorage</code>
          — they do not talk to mako, dunst, GNOME notifications, or the Aura sidecar yet. Wiring real mute behavior belongs in the compositor stack later;
          here you are shaping the schedule Aura can respect once that bridge exists.
        </p>
      </header>

      <section className="glass-card p-5 flex flex-col gap-5">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h3 className="text-sm font-semibold text-text">Quiet hours schedule</h3>
            <p className="text-xs text-subtext0 mt-1 max-w-prose">
              Pick a daily window using local time. If the end is earlier than the start (for example 22:00 → 07:00), the window crosses midnight.
            </p>
          </div>
          <div
            className={cn(
              "shrink-0 rounded-full px-3 py-1 text-[11px] font-medium border",
              prefs.scheduleEnabled && inQuietHours
                ? "border-peach/40 text-peach bg-peach/10"
                : prefs.scheduleEnabled
                  ? "border-surface1 text-subtext1 bg-surface0/80"
                  : "border-surface1/60 text-subtext0 bg-surface0/40"
            )}
            title="Based on this device's clock and the fields below"
          >
            {prefs.scheduleEnabled ? (inQuietHours ? "Inside window now" : "Outside window now") : "Schedule off"}
          </div>
        </div>

        <ScheduleToggleRow
          label="Enable scheduled quiet hours"
          description="When on, Aura UI can treat this interval as “prefer Do Not Disturb.” Other apps will ignore it until integrated."
          active={prefs.scheduleEnabled}
          onToggle={() => updatePrefs({ scheduleEnabled: !prefs.scheduleEnabled })}
        />

        <div className={cn("flex flex-col gap-4 pt-1 border-t border-surface1/30", !prefs.scheduleEnabled && "opacity-45 pointer-events-none")}>
          <div className="grid grid-cols-2 gap-3">
            <label className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-subtext1">Start</span>
              <input
                type="time"
                className="input"
                value={prefs.startTime}
                onChange={(e) => updatePrefs({ startTime: e.target.value })}
              />
            </label>
            <label className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-subtext1">End</span>
              <input
                type="time"
                className="input"
                value={prefs.endTime}
                onChange={(e) => updatePrefs({ endTime: e.target.value })}
              />
            </label>
          </div>

          {overnight && (
            <p className="text-xs text-subtext1 leading-relaxed flex gap-2">
              <span className="icon text-peach shrink-0 text-lg">schedule</span>
              <span>
                Overnight window: quiet hours stay active after midnight until the end time. Weekend skips apply to entire calendar days (see below).
              </span>
            </p>
          )}

          <ScheduleToggleRow
            label="Weekdays only"
            description="Monday through Friday. Saturdays and Sundays never enter this quiet window, even if times would overlap."
            active={prefs.weekdaysOnly}
            onToggle={() => updatePrefs({ weekdaysOnly: !prefs.weekdaysOnly })}
            disabled={!prefs.scheduleEnabled}
          />
        </div>
      </section>

      <section className="glass-card p-5 flex flex-col gap-3">
        <h3 className="text-sm font-semibold text-text flex items-center gap-2">
          <span className="icon text-subtext1 text-lg">info</span>
          What&apos;s next
        </h3>
        <ul className="text-xs text-subtext0 space-y-2 list-disc pl-4 max-w-prose leading-relaxed">
          <li>Inbox, per-app rules, and history still need a trusted notification feed from the shell.</li>
          <li>This pane intentionally avoids RPC until those streams are stable—no phantom toggles.</li>
          <li>Clearing site data for this origin removes saved quiet-hour prefs.</li>
        </ul>
      </section>
    </motion.div>
  )
}
