import { useCallback, useEffect, useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type PowerProfile } from "@/lib/api"
import { cn, tempToBarPercent, usageToPercent } from "@/lib/utils"
import DropdownModuleTiles from "@/components/dropdown/DropdownModuleTiles"
import { filterDropdownModules } from "@/lib/dropdown-tiles"
import { postPanelHover } from "@/lib/panel-hover"
import { notificationQueryKeys } from "@/lib/ws-invalidation"
import { applyThemeToDocument, isDarkTheme } from "@/lib/applyTheme"
import type { DndPrefsView } from "@/lib/api-types"
import { isWithinScheduledQuietHours } from "@/pages/control-center/panes/NotificationsPane"

const POWER_ABBR: Record<PowerProfile, string> = {
  performance: "perf",
  balanced: "bal",
  saver: "save",
}

const WEATHER_REFETCH_MS = 300_000

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
        "toggle-chip flex-1 min-w-0 flex-col gap-0.5 py-1.5 px-2 text-center",
        active && !disabled && "active",
        disabled && "opacity-55 cursor-default pointer-events-none"
      )}
    >
      <span className="icon text-lg">{icon}</span>
      <span className="text-[10px] leading-tight line-clamp-2">{label}</span>
    </motion.button>
  )
}

function DashboardHeader() {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000)
    return () => window.clearInterval(id)
  }, [])

  const { data: weather, isLoading, isError } = useQuery({
    queryKey: ["weather"],
    queryFn: api.getWeather,
    staleTime: 120_000,
    refetchInterval: WEATHER_REFETCH_MS,
    retry: 0,
  })

  const timeStr = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
  const dateStr = now.toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" })

  const weatherLine = isLoading
    ? "Weather…"
    : isError || !weather
      ? "Weather unavailable"
      : `${weather.temp} · ${weather.description}`

  return (
    <div className="flex items-center justify-between gap-3 px-1">
      <button
        type="button"
        className="flex flex-col text-left transition-opacity hover:opacity-80"
        title="Open calendar"
        onClick={() => void api.auraToggleWindow("calendar")}
      >
        <span className="text-lg font-semibold tabular-nums">{timeStr}</span>
        <span className="text-[11px] text-subtext0">{dateStr}</span>
      </button>
      <p className="max-w-[55%] truncate text-right text-[11px] text-subtext0" title={weatherLine}>
        {weatherLine}
      </p>
    </div>
  )
}

// ── System stats mini bar ─────────────────────────────────────────────────────
function MiniStats() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["system-stats"],
    queryFn: api.getSystemStats,
    // WebKit layer-shell windows often stay document.hidden; keep polling anyway.
    refetchInterval: 2000,
    refetchIntervalInBackground: true,
    staleTime: 0,
  })

  if (isLoading && !data) {
    return <p className="text-[11px] text-subtext0">Loading resources…</p>
  }
  if (isError && !data) {
    return <p className="text-[11px] text-red">Resources unavailable</p>
  }

  const cpu = usageToPercent(data?.cpu ?? 0)
  const ram = usageToPercent(data?.ram ?? 0)
  const tempC = data?.temp ?? 0
  const stats = [
    { label: "CPU", value: cpu, text: `${cpu.toFixed(0)}%`, color: "from-blue to-sapphire" },
    { label: "RAM", value: ram, text: `${ram.toFixed(0)}%`, color: "from-mauve to-pink" },
    {
      label: "Temp",
      value: tempToBarPercent(tempC),
      text: Number.isFinite(tempC) && tempC > 0 ? `${tempC.toFixed(0)}°` : "—",
      color: "from-peach to-maroon",
    },
  ]

  return (
    <div className="flex gap-3">
      {stats.map((s) => (
        <div key={s.label} className="flex flex-col gap-1 flex-1">
          <div className="flex justify-between text-[11px] text-subtext0">
            <span>{s.label}</span>
            <span>{s.text}</span>
          </div>
          <div className="progress-bar">
            <motion.div
              className={`progress-fill bg-gradient-to-r ${s.color}`}
              initial={{ width: 0 }}
              animate={{ width: `${s.value}%` }}
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
  const {
    data: quick,
    isLoading: quickLoading,
    isError: quickError,
  } = useQuery({
    queryKey: ["dashboard-quick-status"],
    queryFn: api.dashboardGetQuickStatus,
    refetchInterval: 8000,
  })
  const { data: dndPrefs } = useQuery({
    queryKey: [...notificationQueryKeys.dnd],
    queryFn: api.getNotificationDnd,
    refetchInterval: 60_000,
  })

  const [prefsTick, setPrefsTick] = useState(0)
  useEffect(() => {
    const id = window.setInterval(() => setPrefsTick((n) => n + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])
  const dndSchedulePrefs = (dndPrefs ?? { enabled: false, schedule_enabled: false, start_time: "22:00", end_time: "07:00", weekdays_only: true }) as DndPrefsView
  const inScheduledQuietHours = isWithinScheduledQuietHours(dndSchedulePrefs, new Date())

  const btFromQuick = quick?.bluetooth
  const btPowered = btFromQuick?.powered ?? false
  const btConnCount = btFromQuick?.connected_count ?? 0
  const btLabel = useMemo(() => {
    if (!btFromQuick || (btFromQuick as { adapter_count?: number }).adapter_count === 0) {
      return "BT — none"
    }
    if (!btPowered) return "BT off"
    if (btConnCount === 1) return "BT linked"
    if (btConnCount > 1) return `${btConnCount} linked`
    return "BT on"
  }, [btFromQuick, btPowered, btConnCount])

  const toggleBluetooth = useCallback(async () => {
    const adapters = await api.getBluetoothAdapters()
    if (!adapters.length) return
    const wantOn = !adapters.some((a) => a.powered)
    await Promise.all(adapters.map((a) => api.setBluetoothAdapterPower(a.path, wantOn)))
    await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
    await qc.invalidateQueries({ queryKey: ["bt-ad"] })
  }, [qc])

  const toggleWifi = useCallback(async () => {
    await api.toggleWifi(!quick?.network?.wifi_enabled)
    await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
  }, [qc, quick?.network?.wifi_enabled])

  const cyclePowerProfile = useCallback(async () => {
    const order: PowerProfile[] = ["balanced", "performance", "saver"]
    const cur = (quick?.power_profile ?? "balanced") as PowerProfile
    const i = Math.max(0, order.indexOf(cur))
    try {
      await api.setPowerProfile(order[(i + 1) % order.length])
      await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
    } catch (err) {
      console.warn("power profile change failed:", err)
    }
  }, [quick?.power_profile, qc])

  const moduleIds = useMemo(
    () => filterDropdownModules(quick?.dropdown_modules ?? []),
    [quick?.dropdown_modules]
  )

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
  const battLabel =
    quickLoading && !quick ? "…" : batt ? `${batt.percent}% · ${POWER_ABBR[profile]}` : "Batt"

  const dndChipLabel =
    dndSchedulePrefs.enabled
      ? "DnD on"
      : !dndSchedulePrefs.schedule_enabled
        ? "DnD off"
        : inScheduledQuietHours
          ? "Quiet hrs"
          : "DnD idle"

  const toggleDnd = useCallback(async () => {
    const next = { ...dndSchedulePrefs, enabled: !dndSchedulePrefs.enabled }
    await api.setNotificationDnd(next)
    await qc.invalidateQueries({ queryKey: [...notificationQueryKeys.dnd] })
    await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
  }, [dndSchedulePrefs, qc])

  const takeScreenshot = useCallback(async () => {
    await api.captureScreenshot({ mode: "region", output: "clipboard" })
  }, [])

  const { data: auraSettings } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 60_000,
  })
  const { data: nightLight } = useQuery({
    queryKey: ["night-light"],
    queryFn: api.getNightLight,
    refetchInterval: 30_000,
  })

  const toggleTheme = useCallback(async () => {
    const current = auraSettings?.settings.theme ?? "dark"
    const next = isDarkTheme(current) ? "light" : "dark"
    await api.setAppearanceTheme(next)
    applyThemeToDocument(next)
    await qc.invalidateQueries({ queryKey: ["aura-settings"] })
  }, [auraSettings?.settings.theme, qc])

  const toggleNightLight = useCallback(async () => {
    const enabled = !(nightLight?.enabled ?? false)
    await api.setNightLight({ enabled })
    await qc.invalidateQueries({ queryKey: ["night-light"] })
  }, [nightLight?.enabled, qc])

  const themeDark = isDarkTheme(auraSettings?.settings.theme)

  const netLabel =
    quickLoading && !quick ? "…" : quickError ? "Wi‑Fi n/a" : (net?.active_connection ?? "Wi-Fi")

  return (
    <div
      className="h-full min-h-0 w-full"
      onMouseEnter={() => postPanelHover("dropdownHover", true)}
      onMouseLeave={() => postPanelHover("dropdownHover", false)}
    >
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: "spring", stiffness: 400, damping: 30 }}
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-2xl border border-surface0/60 bg-mantle p-3 text-text shadow-2xl"
    >
      <DashboardHeader />

      <div className="scrollbar-thin mt-2 flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto overflow-x-hidden">
        {/* Quick toggles */}
        <div className="grid grid-cols-4 gap-1.5">
          <QuickToggle
            icon="wifi"
            label={netLabel}
            active={net?.wifi_enabled}
            disabled={quickLoading && !quick}
            onClick={() => void toggleWifi()}
          />
          <QuickToggle
            icon={
              btPowered
                ? btConnCount > 0
                  ? "bluetooth_connected"
                  : "bluetooth"
                : "bluetooth_disabled"
            }
            label={btLabel}
            active={btPowered}
            disabled={(btFromQuick as { adapter_count?: number } | undefined)?.adapter_count === 0}
            title="Bluetooth adapter power"
            onClick={() => void toggleBluetooth()}
          />
          <QuickToggle
            icon="do_not_disturb_on"
            label={dndChipLabel}
            active={dndSchedulePrefs.enabled || inScheduledQuietHours}
            title="Toggle sidecar DND prefs"
            onClick={() => void toggleDnd()}
          />
          <QuickToggle
            icon={themeDark ? "dark_mode" : "light_mode"}
            label={themeDark ? "Dark" : "Light"}
            active={themeDark}
            title="Toggle light/dark theme (React panels)"
            onClick={() => void toggleTheme()}
          />
          <QuickToggle
            icon="bedtime"
            label={nightLight?.enabled ? "Night on" : "Night off"}
            active={nightLight?.enabled}
            title="Blue-light filter (wlsunset / gammastep via sidecar)"
            onClick={() => void toggleNightLight()}
          />
          <QuickToggle
            icon={battIcon}
            label={battLabel}
            title={`${profile}${lowBatt ? " · low battery" : ""} · tap to cycle power profile`}
            disabled={quickLoading && !quick}
            onClick={() => void cyclePowerProfile()}
          />
          <QuickToggle icon="screenshot_monitor" label="Screen" onClick={() => void takeScreenshot()} />
        </div>

        {quick?.next_event?.title ? (
          <button
            type="button"
            className="truncate px-0.5 text-left text-[11px] text-subtext0 hover:text-text"
            title="Open calendar"
            onClick={() => void api.auraToggleWindow("calendar")}
          >
            Next: {quick.next_event.title}
          </button>
        ) : null}

        <DropdownModuleTiles moduleIds={moduleIds} />

        {/* Stats bar */}
        <div className="glass-card shrink-0 px-3 py-2">
          <MiniStats />
        </div>
      </div>
    </motion.div>
    </div>
  )
}
