import { useCallback, useRef, type Ref, type RefObject } from "react"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import BarIconButton from "@/components/bar/BarIconButton"
import type { StatusFlyoutId } from "./useFlyoutHover"

type Props = {
  onSegmentEnter: (id: StatusFlyoutId, anchorCenterY: number) => void
  onSegmentLeave: () => void
}

export default function StatusCluster({ onSegmentEnter, onSegmentLeave }: Props) {
  const netRef = useRef<HTMLButtonElement>(null)
  const btRef = useRef<HTMLButtonElement>(null)
  const audioRef = useRef<HTMLButtonElement>(null)
  const brightRef = useRef<HTMLButtonElement>(null)
  const battRef = useRef<HTMLButtonElement>(null)
  const winRef = useRef<HTMLButtonElement>(null)

  const { data: net } = useQuery({ queryKey: ["net"], queryFn: api.getNetworkStatus, refetchInterval: 5000 })
  const { data: audio } = useQuery({ queryKey: ["audio-devices"], queryFn: api.getAudioDevices, refetchInterval: 5000 })
  const { data: batt } = useQuery({ queryKey: ["batt"], queryFn: api.getBatteryState, refetchInterval: 8000 })
  const { data: brightness } = useQuery({
    queryKey: ["brightness", "active"],
    queryFn: () => api.getBrightness("active"),
    refetchInterval: 8000,
  })
  const { data: adapters } = useQuery({ queryKey: ["bt-ad"], queryFn: api.getBluetoothAdapters, refetchInterval: 8000 })
  const { data: devices } = useQuery({ queryKey: ["bt-dev"], queryFn: api.getBluetoothDevices, refetchInterval: 8000 })

  const btOn = adapters?.some((a) => a.powered)
  const btConn = devices?.some((d) => d.connected)
  const defaultSink = audio?.sinks.find((s) => s.is_default) ?? audio?.sinks[0]
  const sinkMuted = !!defaultSink?.muted
  const sinkLow = defaultSink && !sinkMuted && defaultSink.volume <= 0.05
  const brightPct = Math.round((brightness?.brightness ?? 0.5) * 100)
  const brightLow = brightPct <= 25

  const anchorFor = useCallback(
    (id: StatusFlyoutId, el: HTMLElement | null) => {
      if (!el) return
      const er = el.getBoundingClientRect()
      onSegmentEnter(id, er.top + er.height / 2)
    },
    [onSegmentEnter]
  )

  const segment = (
    id: StatusFlyoutId,
    ref: RefObject<HTMLButtonElement | null>,
    title: string,
    icon: string,
    highlight?: boolean
  ) => (
    <div key={id} className="flex w-full justify-center py-0.5">
      <BarIconButton
        ref={ref as Ref<HTMLButtonElement>}
        title={title}
        icon={icon}
        active={highlight}
        onMouseEnter={() => anchorFor(id, ref.current)}
        onMouseLeave={onSegmentLeave}
      />
    </div>
  )

  return (
    <div className="flex flex-col items-center py-0.5">
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
        "audio",
        audioRef,
        defaultSink
          ? sinkMuted
            ? "Muted"
            : `${Math.round(defaultSink.volume * 100)}% volume`
          : "Audio",
        sinkMuted ? "volume_off" : sinkLow ? "volume_mute" : "volume_up",
        !sinkMuted && !sinkLow
      )}
      {segment(
        "brightness",
        brightRef,
        `${brightPct}% brightness`,
        brightLow ? "brightness_4" : brightPct >= 75 ? "brightness_7" : "brightness_6",
        !brightLow
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
