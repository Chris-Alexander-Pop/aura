import { useCallback, useRef, type Ref, type RefObject } from "react"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import type { StatusFlyoutId } from "./useFlyoutHover"

type Props = {
  stripRootRef: RefObject<HTMLElement | null>
  onSegmentEnter: (id: StatusFlyoutId, anchorCenterY: number) => void
  onSegmentLeave: () => void
}

export default function StatusCluster({ stripRootRef, onSegmentEnter, onSegmentLeave }: Props) {
  const netRef = useRef<HTMLButtonElement>(null)
  const btRef = useRef<HTMLButtonElement>(null)
  const battRef = useRef<HTMLButtonElement>(null)
  const winRef = useRef<HTMLButtonElement>(null)

  const { data: net } = useQuery({ queryKey: ["net"], queryFn: api.getNetworkStatus, refetchInterval: 5000 })
  const { data: batt } = useQuery({ queryKey: ["batt"], queryFn: api.getBatteryState, refetchInterval: 8000 })
  const { data: adapters } = useQuery({ queryKey: ["bt-ad"], queryFn: api.getBluetoothAdapters, refetchInterval: 8000 })
  const { data: devices } = useQuery({ queryKey: ["bt-dev"], queryFn: api.getBluetoothDevices, refetchInterval: 8000 })

  const btOn = adapters?.some((a) => a.powered)
  const btConn = devices?.some((d) => d.connected)

  const anchorFor = useCallback(
    (id: StatusFlyoutId, el: HTMLElement | null) => {
      const root = stripRootRef.current
      if (!el || !root) return
      const er = el.getBoundingClientRect()
      const rr = root.getBoundingClientRect()
      const center = er.top - rr.top + er.height / 2
      onSegmentEnter(id, center)
    },
    [stripRootRef, onSegmentEnter]
  )

  const segment = (
    id: StatusFlyoutId,
    ref: RefObject<HTMLButtonElement | null>,
    title: string,
    icon: string,
    highlight?: boolean
  ) => (
    <button
      ref={ref as Ref<HTMLButtonElement>}
      type="button"
      title={title}
      className={cn(
        "flex h-10 w-full shrink-0 items-center justify-center rounded-xl transition-colors",
        highlight ? "bg-teal/15 text-teal" : "text-subtext1 hover:bg-surface1/80"
      )}
      onMouseEnter={() => anchorFor(id, ref.current)}
      onMouseLeave={onSegmentLeave}
    >
      <span className="icon text-xl">{icon}</span>
    </button>
  )

  return (
    <div className="flex flex-col items-center gap-0.5 rounded-full bg-surface0/70 px-1 py-1.5">
      {segment(
        "network",
        netRef,
        net?.wifi_enabled ? net?.active_connection ?? "Wi‑Fi" : "Wi‑Fi off",
        net?.wifi_enabled ? "wifi" : "wifi_off",
        !!net?.wifi_enabled
      )}
      {segment(
        "bluetooth",
        btRef,
        btOn ? (btConn ? "Bluetooth connected" : "Bluetooth on") : "Bluetooth off",
        btOn ? (btConn ? "bluetooth_connected" : "bluetooth") : "bluetooth_disabled",
        !!btConn
      )}
      {segment(
        "battery",
        battRef,
        batt ? `${batt.percent}%${batt.charging ? ", charging" : ""}` : "Battery",
        batt ? (batt.charging ? "battery_charging_full" : "battery_5_bar") : "battery_unknown",
        !!(batt && batt.percent <= 20 && !batt.charging)
      )}
      {segment("windows", winRef, "Windows & workspaces — hover for list", "layers", false)}
    </div>
  )
}
