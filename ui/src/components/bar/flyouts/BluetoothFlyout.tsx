import { useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

function adapterSummary(
  adapters: Awaited<ReturnType<typeof api.getBluetoothAdapters>> | undefined
): string {
  if (!adapters?.length) return "No adapter"
  const a = adapters[0]
  if (!a.powered) return "off"
  if (a.discovering) return "discovering"
  return "on"
}

export default function BluetoothFlyout() {
  const qc = useQueryClient()
  const [busyAddr, setBusyAddr] = useState<string | null>(null)

  const { data: adapters } = useQuery({ queryKey: ["bt-ad"], queryFn: api.getBluetoothAdapters, refetchInterval: 6000 })
  const { data: devices } = useQuery({ queryKey: ["bt-dev"], queryFn: api.getBluetoothDevices, refetchInterval: 6000 })

  const list = [...(devices ?? [])].sort(
    (a, b) => Number(b.connected) - Number(a.connected) || Number(b.paired) - Number(a.paired)
  )
  const shown = list.slice(0, 6)
  const connected = list.filter((d) => d.connected).length
  const powered = adapters?.some((a) => a.powered)
  const discovering = adapters?.some((a) => a.discovering)

  const countLine = (() => {
    const n = list.length
    let s = `${n} device${n === 1 ? "" : "s"} available`
    if (connected > 0) s += ` (${connected} connected)`
    return s
  })()

  const scan = async () => {
    await api.scanBluetooth()
    await qc.invalidateQueries({ queryKey: ["bt-dev"] })
  }

  const toggleConn = async (address: string, connectedNow: boolean) => {
    setBusyAddr(address)
    try {
      if (connectedNow) await api.disconnectDevice(address)
      else await api.connectDevice(address)
      await qc.invalidateQueries({ queryKey: ["bt-dev"] })
    } finally {
      setBusyAddr(null)
    }
  }

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">
        Bluetooth {adapterSummary(adapters)}
      </h2>

      <div className="flex flex-wrap gap-2 px-0.5">
        <span
          className={cn(
            "rounded-full px-2.5 py-1 text-[10px] font-medium uppercase tracking-wide",
            powered ? "bg-teal/20 text-teal" : "bg-surface1 text-subtext0"
          )}
        >
          {powered ? "Powered" : "Off"}
        </span>
        {discovering ? (
          <span className="rounded-full bg-blue/20 px-2.5 py-1 text-[10px] font-medium uppercase tracking-wide text-blue">
            Discovering
          </span>
        ) : null}
      </div>

      <p className="px-0.5 text-[11px] leading-snug text-subtext0">{countLine}</p>

      <ul className="flex max-h-48 flex-col gap-1.5 overflow-y-auto pr-0.5">
        {shown.map((d) => {
          const loading = busyAddr === d.address
          return (
            <li key={d.address} className="flex items-center gap-2 rounded-xl bg-surface0/90 px-2.5 py-2">
              <span className="icon shrink-0 text-[22px] text-subtext1">bluetooth</span>
              <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-subtext1">{d.name || d.address}</span>
              <button
                type="button"
                disabled={loading || !powered}
                className={cn(
                  "min-w-[5.5rem] shrink-0 rounded-full px-3 py-1.5 text-[11px] font-semibold transition-colors disabled:opacity-40",
                  d.connected ? "bg-teal/25 text-teal hover:bg-teal/35" : "bg-teal/90 text-crust hover:bg-teal"
                )}
                onClick={() => toggleConn(d.address, d.connected)}
              >
                {loading ? "…" : d.connected ? "Disconnect" : "Connect"}
              </button>
            </li>
          )
        })}
      </ul>

      <button
        type="button"
        className="w-full rounded-full bg-blue/25 py-3 text-[12px] font-semibold text-blue hover:bg-blue/35 disabled:opacity-40"
        disabled={!powered}
        onClick={() => scan()}
      >
        Scan for devices
      </button>

      <p className="px-0.5 text-[10px] leading-snug text-subtext0">
        Enable Bluetooth in system settings if toggles are unavailable here.
      </p>
    </div>
  )
}
