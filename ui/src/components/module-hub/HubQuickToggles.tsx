import { useCallback, useEffect, useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type PowerProfile } from "@/lib/api"
import type { DashboardQuickStatusView, DndPrefsView } from "@/lib/api-types"
import { notificationQueryKeys } from "@/lib/ws-invalidation"
import { applyThemeToDocument, isDarkTheme } from "@/lib/applyTheme"
import { isWithinScheduledQuietHours } from "@/pages/control-center/panes/NotificationsPane"
import { HubQuickToggle } from "./HubQuickToggle"

const POWER_ABBR: Record<PowerProfile, string> = {
  performance: "perf",
  balanced: "bal",
  saver: "save",
}

const DEFAULT_DND: DndPrefsView = {
  enabled: false,
  schedule_enabled: false,
  start_time: "22:00",
  end_time: "07:00",
  weekdays_only: true,
}

export function HubQuickToggles({
  quick,
  quickLoading,
  quickError,
}: {
  quick: DashboardQuickStatusView | undefined
  quickLoading: boolean
  quickError: boolean
}) {
  const qc = useQueryClient()
  const { data: dndPrefs } = useQuery({
    queryKey: [...notificationQueryKeys.dnd],
    queryFn: api.getNotificationDnd,
    refetchInterval: 60_000,
  })
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

  const [, setPrefsTick] = useState(0)
  useEffect(() => {
    const id = window.setInterval(() => setPrefsTick((n) => n + 1), 60_000)
    return () => window.clearInterval(id)
  }, [])

  const dndSchedulePrefs = (dndPrefs ?? DEFAULT_DND) as DndPrefsView
  const inScheduledQuietHours = isWithinScheduledQuietHours(dndSchedulePrefs, new Date())

  const btFromQuick = quick?.bluetooth
  const btPowered = btFromQuick?.powered ?? false
  const btConnCount = btFromQuick?.connected_count ?? 0
  const adapterCount = (btFromQuick as { adapter_count?: number } | undefined)?.adapter_count

  const btLabel = useMemo(() => {
    if (!btFromQuick || adapterCount === 0) return "BT — none"
    if (!btPowered) return "BT off"
    if (btConnCount === 1) return "BT linked"
    if (btConnCount > 1) return `${btConnCount} linked`
    return "BT on"
  }, [btFromQuick, adapterCount, btPowered, btConnCount])

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

  const toggleDnd = useCallback(async () => {
    const next = { ...dndSchedulePrefs, enabled: !dndSchedulePrefs.enabled }
    await api.setNotificationDnd(next)
    await qc.invalidateQueries({ queryKey: [...notificationQueryKeys.dnd] })
    await qc.invalidateQueries({ queryKey: ["dashboard-quick-status"] })
  }, [dndSchedulePrefs, qc])

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

  const batt = quick?.battery
  const net = quick?.network
  const lowBatt = !!(batt && !batt.charging && batt.percent <= 20)
  const profile = (quick?.power_profile ?? "balanced") as PowerProfile
  const themeDark = isDarkTheme(auraSettings?.settings.theme)

  const battIcon =
    batt == null || batt.percent < 0
      ? "battery_unknown"
      : batt.charging
        ? "battery_charging_full"
        : batt.percent > 50
          ? "battery_5_bar"
          : "battery_2_bar"

  const battLabel =
    quickLoading && !quick ? "…" : batt ? `${batt.percent}% · ${POWER_ABBR[profile]}` : "Batt"

  const dndChipLabel = dndSchedulePrefs.enabled
    ? "DnD on"
    : !dndSchedulePrefs.schedule_enabled
      ? "DnD off"
      : inScheduledQuietHours
        ? "Quiet hrs"
        : "DnD idle"

  const netLabel =
    quickLoading && !quick ? "…" : quickError ? "Wi‑Fi n/a" : (net?.active_connection ?? "Wi-Fi")

  return (
    <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
      <HubQuickToggle
        icon="wifi"
        label={netLabel}
        active={net?.wifi_enabled}
        disabled={quickLoading && !quick}
        onClick={() => void toggleWifi()}
      />
      <HubQuickToggle
        icon={
          btPowered
            ? btConnCount > 0
              ? "bluetooth_connected"
              : "bluetooth"
            : "bluetooth_disabled"
        }
        label={btLabel}
        active={btPowered}
        disabled={adapterCount === 0}
        title="Bluetooth adapter power"
        onClick={() => void toggleBluetooth()}
      />
      <HubQuickToggle
        icon="do_not_disturb_on"
        label={dndChipLabel}
        active={dndSchedulePrefs.enabled || inScheduledQuietHours}
        title="Toggle DND"
        onClick={() => void toggleDnd()}
      />
      <HubQuickToggle
        icon={themeDark ? "dark_mode" : "light_mode"}
        label={themeDark ? "Dark" : "Light"}
        active={themeDark}
        title="Toggle light/dark theme"
        onClick={() => void toggleTheme()}
      />
      <HubQuickToggle
        icon="bedtime"
        label={nightLight?.enabled ? "Night on" : "Night off"}
        active={nightLight?.enabled}
        title="Night light"
        onClick={() => void toggleNightLight()}
      />
      <HubQuickToggle
        icon={battIcon}
        label={battLabel}
        title={`${profile}${lowBatt ? " · low battery" : ""} · tap to cycle power profile`}
        disabled={quickLoading && !quick}
        onClick={() => void cyclePowerProfile()}
      />
    </div>
  )
}
