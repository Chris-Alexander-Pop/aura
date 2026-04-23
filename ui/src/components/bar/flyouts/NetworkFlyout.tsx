import { useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

function strengthIcon(strength: number): string {
  if (strength >= 75) return "wifi"
  if (strength >= 50) return "wifi_2_bar"
  if (strength >= 25) return "wifi_1_bar"
  return "wifi_1_bar"
}

export default function NetworkFlyout() {
  const qc = useQueryClient()
  const [connectingToSsid, setConnectingToSsid] = useState<string | null>(null)

  const { data: net } = useQuery({ queryKey: ["net"], queryFn: api.getNetworkStatus, refetchInterval: 8000 })
  const { data: scan, isFetching: scanning } = useQuery({
    queryKey: ["wifi-scan"],
    queryFn: api.scanNetworks,
    staleTime: 15_000,
  })

  const sorted = useMemo(() => {
    const list = scan ?? []
    return [...list].sort((a, b) => {
      if (a.active !== b.active) return (b.active ? 1 : 0) - (a.active ? 1 : 0)
      return b.strength - a.strength
    }).slice(0, 8)
  }, [scan])

  const wifiOn = !!net?.wifi_enabled

  const toggleWifi = async (enabled: boolean) => {
    await api.toggleWifi(enabled)
    await qc.invalidateQueries({ queryKey: ["net"] })
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
  }

  const connect = async (ssid: string) => {
    setConnectingToSsid(ssid)
    try {
      await api.connectNetwork(ssid)
      await qc.invalidateQueries({ queryKey: ["net"] })
      await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    } finally {
      setConnectingToSsid(null)
    }
  }

  const disconnect = async () => {
    await api.disconnectNetwork()
    await qc.invalidateQueries({ queryKey: ["net"] })
  }

  const rescan = async () => {
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.refetchQueries({ queryKey: ["wifi-scan"] })
  }

  const availableCount = sorted.length

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">
        Wifi {wifiOn ? "enabled" : "disabled"}
      </h2>

      <label className="flex cursor-pointer items-center justify-between gap-3 rounded-xl bg-surface0/90 px-3 py-2.5">
        <span className="text-[12px] font-medium text-text">Enabled</span>
        <input
          type="checkbox"
          className="h-4 w-4 accent-teal"
          checked={wifiOn}
          onChange={(e) => toggleWifi(e.target.checked)}
        />
      </label>

      <p className="px-0.5 text-[11px] leading-snug text-subtext0">
        {availableCount} network{availableCount === 1 ? "" : "s"} available
      </p>

      <ul className="flex max-h-56 flex-col gap-1.5 overflow-y-auto pr-0.5">
        {sorted.map((ap) => {
          const isConnecting = connectingToSsid === ap.ssid
          const isSecure = ap.security && ap.security !== "none"

          return (
            <li
              key={ap.ssid}
              className="flex items-center gap-2 rounded-xl bg-surface0/90 px-2 py-2"
            >
              <span
                className={cn(
                  "icon shrink-0 text-[22px] leading-none",
                  ap.active ? "text-teal" : "text-subtext0"
                )}
              >
                {strengthIcon(ap.strength)}
              </span>
              {isSecure ? (
                <span className="icon shrink-0 text-[14px] text-subtext0" title="Secured network">
                  lock
                </span>
              ) : null}
              <span
                className={cn(
                  "min-w-0 flex-1 truncate text-[13px] leading-tight",
                  ap.active ? "font-semibold text-teal" : "text-subtext1"
                )}
              >
                {ap.ssid}
              </span>

              <button
                type="button"
                disabled={isConnecting || !wifiOn}
                title={ap.active ? "Disconnect" : "Connect"}
                className={cn(
                  "flex h-10 w-10 shrink-0 items-center justify-center rounded-full transition-colors disabled:opacity-40",
                  ap.active
                    ? "bg-teal text-crust"
                    : "bg-teal/90 text-crust hover:bg-teal"
                )}
                onClick={() => (ap.active ? disconnect() : connect(ap.ssid))}
              >
                {isConnecting ? (
                  <span className="icon animate-spin text-[20px] text-crust">progress_activity</span>
                ) : (
                  <span className="icon text-[22px]">{ap.active ? "link_off" : "link"}</span>
                )}
              </button>
            </li>
          )
        })}
      </ul>

      <button
        type="button"
        disabled={scanning || !wifiOn}
        className="flex w-full items-center justify-center gap-2 rounded-full bg-blue/30 py-3 text-[12px] font-semibold text-blue hover:bg-blue/40 disabled:opacity-40"
        onClick={() => rescan()}
      >
        <span className={cn("icon text-lg", scanning && "animate-spin")}>wifi_find</span>
        {scanning ? "Scanning…" : "Rescan networks"}
      </button>

      <button
        type="button"
        className="rounded-lg py-2 text-center text-[11px] text-subtext0 underline-offset-2 hover:text-subtext1 hover:underline"
        onClick={() => api.auraToggleWindow("control-center")}
      >
        Wi‑Fi settings
      </button>
    </div>
  )
}
