import { useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type PowerProfile } from "@/lib/api"
import { FlyoutLoading, FlyoutEmpty } from "@/components/bar/flyouts/FlyoutStates"
import { cn } from "@/lib/utils"

export default function BatteryFlyout() {
  const qc = useQueryClient()
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

  const cycle = async () => {
    const order: PowerProfile[] = ["balanced", "performance", "saver"]
    const cur = prof?.profile ?? "balanced"
    const i = order.indexOf(cur)
    await api.setPowerProfile(order[(i + 1) % order.length])
    await qc.invalidateQueries({ queryKey: ["pwr"] })
  }

  const low = batt && !batt.charging && batt.percent <= 20
  const hasBattery = batt != null && batt.percent >= 0

  const timeLine = (() => {
    if (!batt) return "Battery status unavailable"
    const tr = batt.time_remaining?.trim()
    if (!tr || tr === "Unknown") {
      return batt.charging ? "Time until charged: calculating…" : "Time remaining: calculating…"
    }
    return batt.charging ? `Time until charged: ${tr}` : `Time remaining: ${tr}`
  })()

  const profileLine = `Power profile: ${prof?.profile ?? "balanced"}`

  if (battPending) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Battery</h2>
        <FlyoutLoading label="Reading battery status…" />
      </div>
    )
  }

  if (battErr || batt == null) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Battery</h2>
        <FlyoutEmpty
          icon="battery_unknown"
          title="Battery status unavailable"
          detail="Plug in AC or open Control Center to troubleshoot power info."
        />
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className={cn("pr-2 text-sm font-semibold leading-tight", low ? "text-red" : "text-subtext1")}>
        {hasBattery ? `Remaining: ${batt.percent}%` : "No battery detected"}
      </h2>

      <p className="text-[12px] leading-relaxed text-subtext0">{timeLine}</p>

      <p className="text-[12px] text-subtext1">{profileLine}</p>

      <button
        type="button"
        disabled={profPending}
        className="w-full rounded-xl bg-surface0/90 px-3 py-2.5 text-left text-[12px] font-medium text-subtext1 transition-colors hover:bg-surface1 disabled:opacity-60"
        onClick={() => cycle()}
      >
        {profPending ? (
          <span className="flex items-center gap-2 text-subtext0">
            <span className="icon animate-spin text-xl text-teal">progress_activity</span>
            Loading profile…
          </span>
        ) : (
          <>
            <span className="text-text">{prof?.profile ?? "balanced"}</span>
            <span className="mt-0.5 block text-[10px] font-normal text-subtext0">
              Click to cycle performance / balanced / saver
            </span>
          </>
        )}
      </button>

      <div className="flex justify-center pt-1">
        <span
          className={cn(
            "icon text-5xl leading-none",
            low ? "text-red" : "text-subtext1",
            batt?.charging ? "text-teal" : ""
          )}
        >
          {batt
            ? batt.charging
              ? "battery_charging_full"
              : batt.percent > 50
                ? "battery_5_bar"
                : "battery_2_bar"
            : "battery_unknown"}
        </span>
      </div>
    </div>
  )
}
