import { useCallback, useEffect, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type PowerProfile } from "@/lib/api"
import { connectWs } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { isWithinScheduledQuietHours, type DndSchedulePrefs } from "@/pages/control-center/panes/NotificationsPane"

/** Same key as NotificationsPane — local scheduled quiet-hours only (not OS DND). */
const DND_SCHEDULE_STORAGE = "aura.control-center.notifications.dndSchedule.v1"

function parseHm(value: string): { h: number; m: number } | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value.trim())
  if (!match) return null
  const h = Number(match[1])
  const m = Number(match[2])
  if (h > 23 || m > 59 || Number.isNaN(h) || Number.isNaN(m)) return null
  return { h, m }
}

function loadDndSchedulePrefs(): DndSchedulePrefs {
  const defaults: DndSchedulePrefs = {
    scheduleEnabled: false,
    startTime: "22:00",
    endTime: "07:00",
    weekdaysOnly: true,
  }
  try {
    const raw = localStorage.getItem(DND_SCHEDULE_STORAGE)
    if (!raw) return defaults
    const parsed = JSON.parse(raw) as Partial<DndSchedulePrefs>
    return {
      scheduleEnabled:
        typeof parsed.scheduleEnabled === "boolean" ? parsed.scheduleEnabled : defaults.scheduleEnabled,
      startTime:
        typeof parsed.startTime === "string" && parseHm(parsed.startTime) ? parsed.startTime : defaults.startTime,
      endTime:
        typeof parsed.endTime === "string" && parseHm(parsed.endTime) ? parsed.endTime : defaults.endTime,
      weekdaysOnly: typeof parsed.weekdaysOnly === "boolean" ? parsed.weekdaysOnly : defaults.weekdaysOnly,
    }
  } catch {
    return defaults
  }
}

async function bluetoothSetAdapterPower(adapter_path: string, powered: boolean) {
  const res = await fetch("/api/Bluetooth.SetAdapterPower", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ adapter_path, powered }),
  })
  const json: { ok?: boolean; error?: string } = await res.json()
  if (!json.ok) throw new Error(json.error ?? "Bluetooth.SetAdapterPower failed")
}

const POWER_ABBR: Record<PowerProfile, string> = {
  performance: "perf",
  balanced: "bal",
  saver: "save",
}

// ── Quick toggle button ───────────────────────────────────────────────────────
function QuickToggle({
  icon,
  label,
  active,
  onClick,
  disabled,
  title,
}: {
  icon: string
  label: string
  active?: boolean
  onClick?: () => void
  disabled?: boolean
  title?: string
}) {
  return (
    <motion.button
      type="button"
      whileHover={disabled ? undefined : { scale: 1.04 }}
      whileTap={disabled ? undefined : { scale: 0.95 }}
      onClick={disabled ? undefined : onClick}
      title={title}
      disabled={disabled}
      className={cn(
        "toggle-chip flex-1 flex-col gap-1 py-3 text-center",
        active && !disabled && "active",
        disabled && "opacity-55 cursor-default pointer-events-none"
      )}
    >
      <span className="icon text-2xl">{icon}</span>
      <span className="text-[11px] leading-tight line-clamp-2">{label}</span>
    </motion.button>
  )
}

// ── System stats mini bar ─────────────────────────────────────────────────────
function MiniStats() {
  const { data } = useQuery({
    queryKey: ["system-stats"],
    queryFn: api.getSystemStats,
    refetchInterval: 4000,
  })

  const stats = [
    { label: "CPU", value: data?.cpu ?? 0, color: "from-blue to-sapphire" },
    { label: "RAM", value: data?.ram ?? 0, color: "from-mauve to-pink" },
    { label: "Temp", value: data?.temp ? data.temp / 100 : 0, color: "from-peach to-maroon" },
  ]

  return (
    <div className="flex gap-3">
      {stats.map((s) => (
        <div key={s.label} className="flex flex-col gap-1 flex-1">
          <div className="flex justify-between text-[11px] text-subtext0">
            <span>{s.label}</span>
            <span>
              {s.label === "Temp"
                ? `${data?.temp?.toFixed(0) ?? "—"}°`
                : `${s.value.toFixed(0)}%`}
            </span>
          </div>
          <div className="progress-bar">
            <motion.div
              className={`progress-fill bg-gradient-to-r ${s.color}`}
              initial={{ width: 0 }}
              animate={{ width: `${Math.min(s.value, 100)}%` }}
              transition={{ duration: 0.5 }}
            />
          </div>
        </div>
      ))}
    </div>
  )
}

export default function Dropdown() {
  const qc = useQueryClient()
  const { data: quick } = useQuery({
    queryKey: ["dashboard-quick-status"],
    queryFn: api.dashboardGetQuickStatus,
    refetchInterval: 8000,
  })
  const { data: adapters } = useQuery({ queryKey: ["bt-ad"], queryFn: api.getBluetoothAdapters, refetchInterval: 8000 })
  const { data: devices } = useQuery({ queryKey: ["bt-dev"], queryFn: api.getBluetoothDevices, refetchInterval: 8000 })

  const [prefsTick, setPrefsTick] = useState(0)
  useEffect(() => {
    const id = window.setInterval(() => setPrefsTick((n) => n + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])
  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === DND_SCHEDULE_STORAGE) setPrefsTick((n) => n + 1)
    }
    window.addEventListener("storage", onStorage)
    return () => window.removeEventListener("storage", onStorage)
  }, [])
  useEffect(() => {
    const onVis = () => {
      if (document.visibilityState === "visible") setPrefsTick((n) => n + 1)
    }
    document.addEventListener("visibilitychange", onVis)
    return () => document.removeEventListener("visibilitychange", onVis)
  }, [])
  const dndSchedulePrefs = useMemo(() => loadDndSchedulePrefs(), [prefsTick])
  const inScheduledQuietHours = isWithinScheduledQuietHours(dndSchedulePrefs)

  const btPowered = adapters?.some((a) => a.powered)
  const connectedDevices = devices?.filter((d) => d.connected) ?? []
  const btLabel = useMemo(() => {
    const n = connectedDevices.length
    if (!adapters?.length) return "BT — none"
    if (!btPowered) return "BT off"
    if (n === 1) return connectedDevices[0].name.trim() || connectedDevices[0].address
    if (n > 1) return `${n} linked`
    return "BT on"
  }, [adapters?.length, btPowered, connectedDevices])

  const toggleBluetooth = useCallback(async () => {
    if (!adapters?.length) return
    const wantOn = !adapters.some((a) => a.powered)
    try {
      await Promise.all(adapters.map((a) => bluetoothSetAdapterPower(a.path, wantOn)))
    } finally {
      await qc.invalidateQueries({ queryKey: ["bt-ad"] })
      await qc.invalidateQueries({ queryKey: ["bt-dev"] })
    }
  }, [adapters, qc])

  const cyclePowerProfile = useCallback(async () => {
    const order: PowerProfile[] = ["balanced", "performance", "saver"]
    const cur = (quick?.power_profile ?? "balanced") as PowerProfile
    const i = Math.max(0, order.indexOf(cur))
    await api.setPowerProfile(order[(i + 1) % order.length])
    await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
  }, [quick?.power_profile, qc])

  useEffect(() => {
    connectWs()
  }, [])

  const batt = quick?.battery
  const net = quick?.network
  const lowBatt = !!(batt && !batt.charging && batt.percent <= 20)
  const battIcon =
    batt == null || batt.percent < 0
      ? "battery_unknown"
      : batt.charging
        ? "battery_charging_full"
        : batt.percent > 50
          ? "battery_5_bar"
          : "battery_2_bar"

  const profile = (quick?.power_profile ?? "balanced") as PowerProfile
  const battLabel = batt ? `${batt.percent}% · ${POWER_ABBR[profile]}` : "Batt"

  const dndChipLabel =
    !dndSchedulePrefs.scheduleEnabled ? "DnD · sched off" : inScheduledQuietHours ? "Quiet hrs" : "DnD idle"

  return (
    <motion.div
      initial={{ opacity: 0, y: -16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: "spring", stiffness: 400, damping: 30 }}
      className="flex flex-col gap-3 h-full bg-mantle/90 backdrop-blur-2xl border border-surface0/60 rounded-2xl shadow-2xl p-4 overflow-hidden"
    >
      {/* Top row: quick toggles */}
      <div className="flex gap-2">
        <QuickToggle
          icon="wifi"
          label={net?.active_connection ?? "Wi-Fi"}
          active={net?.wifi_enabled}
          onClick={() => api.toggleWifi(!net?.wifi_enabled)}
        />
        <QuickToggle
          icon={
            adapters?.length
              ? btPowered
                ? connectedDevices.length > 0
                  ? "bluetooth_connected"
                  : "bluetooth"
                : "bluetooth_disabled"
              : "bluetooth_disabled"
          }
          label={btLabel}
          active={btPowered}
          disabled={!adapters?.length}
          title={
            adapters?.length
              ? "Bluetooth adapter power · devices from Bluetooth.GetDevices"
              : "No adapter reported · pair in Control Center Bluetooth"
          }
          onClick={() => void toggleBluetooth()}
        />
        <QuickToggle
          icon="do_not_disturb_on"
          label={dndChipLabel}
          active={inScheduledQuietHours}
          disabled
          title="Shows Control Center scheduled quiet-hours only — not wired to your notification daemon/OS DND. Edit under Notifications pane."
        />
        <QuickToggle
          icon="bedtime"
          label="Night · n/a"
          active={false}
          disabled
          title="No night-light / blue-filter integration yet (e.g. wlsunset)."
        />
        <QuickToggle
          icon={battIcon}
          label={battLabel}
          title={`${profile}${lowBatt ? " · low battery" : ""} · tap to cycle power profile`}
          onClick={() => void cyclePowerProfile()}
        />
        <QuickToggle icon="screenshot_monitor" label="Screen" />
      </div>

      {/* Stats bar */}
      <div className="glass-card px-4 py-3">
        <MiniStats />
      </div>
    </motion.div>
  )
}
