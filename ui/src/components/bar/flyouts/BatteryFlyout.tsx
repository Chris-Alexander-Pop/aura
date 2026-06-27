import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useMemo } from "react"
import api, { type BatteryState, type PowerProfile } from "@/lib/api"
import { FlyoutLoading, FlyoutEmpty } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutMeta,
  FlyoutProfilePicker,
  FlyoutShell,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"
import { cn } from "@/lib/utils"

export default function BatteryFlyout() {
  const qc = useQueryClient()

  const { data: tile } = useQuery({
    queryKey: ["sidebar-tile", "battery"],
    queryFn: () => api.sidebarGetTileData("battery"),
    staleTime: 30_000,
  })

  const tileBatt = useMemo((): BatteryState | undefined => {
    if (!tile || typeof tile !== "object" || !("percent" in tile)) return undefined
    const t = tile as { percent?: number; charging?: boolean; time_remaining?: string }
    if (t.percent == null) return undefined
    return {
      percent: t.percent,
      charging: !!t.charging,
      time_remaining: t.time_remaining ?? "Unknown",
    }
  }, [tile])

  const {
    data: batt,
    isLoading: battLoading,
    isError: battErr,
  } = useQuery({
    queryKey: ["batt"],
    queryFn: api.getBatteryState,
    staleTime: 30_000,
    placeholderData: () =>
      tileBatt ?? qc.getQueryData<BatteryState>(["batt"]),
  })

  const { data: prof, isLoading: profLoading } = useQuery({
    queryKey: ["pwr"],
    queryFn: api.getPowerProfile,
    staleTime: 30_000,
    placeholderData: () => qc.getQueryData<{ profile: PowerProfile }>(["pwr"]),
  })

  const profileMut = useMutation({
    mutationFn: (profile: PowerProfile) => api.setPowerProfile(profile),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["pwr"] }),
  })

  const displayBatt = batt
  const low = displayBatt && !displayBatt.charging && displayBatt.percent <= 20
  const hasBattery = displayBatt != null && displayBatt.percent >= 0
  const currentProfile = (prof?.profile ?? "balanced") as PowerProfile

  const timeLine = (() => {
    if (!displayBatt) return "Battery status unavailable"
    const tr = displayBatt.time_remaining?.trim()
    if (!tr || tr === "Unknown") {
      return displayBatt.charging ? "Time until charged: calculating…" : "Time remaining: calculating…"
    }
    return displayBatt.charging ? `Until charged: ${tr}` : `Remaining: ${tr}`
  })()

  if (battLoading && batt == null) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Battery</FlyoutTitle>
        <FlyoutLoading label="Reading battery status…" />
      </FlyoutShell>
    )
  }

  if (battErr || batt == null) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Battery</FlyoutTitle>
        <FlyoutEmpty
          icon="battery_unknown"
          title="Battery status unavailable"
          detail="Open Control Center to troubleshoot."
        />
      </FlyoutShell>
    )
  }

  return (
    <FlyoutShell>
      <FlyoutTitle className={cn(low && "text-red")}>
        {hasBattery ? `Remaining: ${displayBatt.percent}%` : "No battery detected"}
      </FlyoutTitle>

      <FlyoutMeta>{timeLine}</FlyoutMeta>

      <FlyoutProfilePicker
        value={currentProfile}
        disabled={profLoading || profileMut.isPending}
        onChange={(p) => profileMut.mutate(p)}
      />
    </FlyoutShell>
  )
}
