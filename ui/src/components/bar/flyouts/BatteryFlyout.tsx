import { useEffect } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type PowerProfile } from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
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

  useEffect(() => {
    connectWs()
    const offBatt = useWsStore.getState().on("Power.BatteryState", () => {
      void qc.invalidateQueries({ queryKey: ["batt"] })
      void qc.invalidateQueries({ queryKey: ["sidebar-tile", "battery"] })
    })
    const offProf = useWsStore.getState().on("Power.Profile", () => {
      void qc.invalidateQueries({ queryKey: ["pwr"] })
    })
    return () => {
      offBatt()
      offProf()
    }
  }, [qc])

  const { data: tile } = useQuery({
    queryKey: ["sidebar-tile", "battery"],
    queryFn: () => api.sidebarGetTileData("battery"),
    staleTime: 10_000,
  })
  const {
    data: batt,
    isPending: battPending,
    isError: battErr,
  } = useQuery({ queryKey: ["batt"], queryFn: api.getBatteryState, refetchInterval: 8000 })
  const { data: prof, isPending: profPending } = useQuery({
    queryKey: ["pwr"],
    queryFn: api.getPowerProfile,
    refetchInterval: 15_000,
  })

  const profileMut = useMutation({
    mutationFn: (profile: PowerProfile) => api.setPowerProfile(profile),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["pwr"] }),
  })

  const battFromTile = tile && typeof tile === "object" && "percent" in (tile as object)
    ? (tile as { percent?: number; charging?: boolean })
    : null
  const displayBatt = battFromTile?.percent != null && batt
    ? { ...batt, percent: battFromTile.percent ?? batt.percent, charging: battFromTile.charging ?? batt.charging }
    : batt

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

  if (battPending) {
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
        {hasBattery ? `Remaining: ${displayBatt!.percent}%` : "No battery detected"}
      </FlyoutTitle>

      <FlyoutMeta>{timeLine}</FlyoutMeta>

      <FlyoutProfilePicker
        value={currentProfile}
        disabled={profPending || profileMut.isPending}
        onChange={(p) => profileMut.mutate(p)}
      />
    </FlyoutShell>
  )
}
