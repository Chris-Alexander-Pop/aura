import { useState } from "react"
import { motion } from "framer-motion"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

type Adapter = Awaited<ReturnType<typeof api.getBluetoothAdapters>>[number]
type Device = Awaited<ReturnType<typeof api.getBluetoothDevices>>[number]

function emptyMessage(adapters: Adapter[] | undefined, powered: boolean): { title: string; body: string; icon: string } | null {
  if (adapters != null && adapters.length === 0) {
    return {
      icon: "settings_bluetooth",
      title: "No adapter found",
      body: "This system did not report a Bluetooth controller. Plug in a USB adapter or check firmware.",
    }
  }
  if (adapters?.length && !powered) {
    return {
      icon: "bluetooth_disabled",
      title: "Bluetooth is off",
      body: "Turn Bluetooth on in system settings, then return here to scan and manage devices.",
    }
  }
  return null
}

function sortDevices(list: Device[]): Device[] {
  return [...list].sort(
    (a, b) =>
      Number(b.connected) - Number(a.connected) ||
      Number(b.paired) - Number(a.paired) ||
      (a.name || a.address).localeCompare(b.name || b.address)
  )
}

function EmptyPanel({ icon, title, body }: { icon: string; title: string; body: string }) {
  return (
    <div className="glass-card flex flex-col items-center justify-center gap-3 px-6 py-12 text-center">
      <span className="icon text-5xl text-subtext1/90">{icon}</span>
      <div className="space-y-1.5 max-w-sm">
        <p className="text-sm font-semibold text-text">{title}</p>
        <p className="text-xs leading-relaxed text-subtext0">{body}</p>
      </div>
    </div>
  )
}

export function BluetoothPane() {
  const qc = useQueryClient()
  const [busyAddr, setBusyAddr] = useState<string | null>(null)
  const [scanBusy, setScanBusy] = useState(false)

  const adaptersQuery = useQuery({
    queryKey: ["bt-ad"],
    queryFn: api.getBluetoothAdapters,
    refetchInterval: 6000,
  })
  const devicesQuery = useQuery({
    queryKey: ["bt-dev"],
    queryFn: api.getBluetoothDevices,
    refetchInterval: 6000,
  })

  const adapters = adaptersQuery.data
  const devices = devicesQuery.data

  const powered = adapters?.some((a) => a.powered) ?? false
  const discovering = adapters?.some((a) => a.discovering) ?? false
  const blockedEmpty = emptyMessage(adapters, powered)

  const sortedDevices = sortDevices(devices ?? [])
  const connectedCount = sortedDevices.filter((d) => d.connected).length

  const scan = async () => {
    setScanBusy(true)
    try {
      await api.scanBluetooth()
      await qc.invalidateQueries({ queryKey: ["bt-ad"] })
      await qc.invalidateQueries({ queryKey: ["bt-dev"] })
    } finally {
      setScanBusy(false)
    }
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

  const loading = adaptersQuery.isLoading || devicesQuery.isLoading
  const fetchErr = adaptersQuery.isError || devicesQuery.isError
  const errObj = adaptersQuery.error ?? devicesQuery.error

  const subtitle = loading
    ? "Loading devices…"
    : sortedDevices.length > 0
      ? `${sortedDevices.length} device${sortedDevices.length === 1 ? "" : "s"}` +
        (connectedCount > 0 ? ` · ${connectedCount} connected` : "")
      : powered
        ? "No devices yet — scan to discover nearby hardware."
        : null

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18 }}
      className="flex h-full flex-col gap-4 overflow-y-auto p-6"
    >
      <div>
        <div className="flex items-center gap-3">
          <span className="icon text-2xl text-mauve">bluetooth</span>
          <div className="min-w-0 flex-1">
            <h2 className="text-xl font-semibold text-text">Bluetooth</h2>
            {subtitle ? <p className="mt-0.5 text-xs text-subtext1">{subtitle}</p> : null}
          </div>
        </div>
      </div>

      {fetchErr ? (
        <EmptyPanel
          icon="error_outline"
          title="Could not load Bluetooth"
          body={errObj instanceof Error ? errObj.message : "Sidecar request failed. Is ags-sidecar running?"}
        />
      ) : null}

      {!fetchErr && (
        <div className="glass-card flex flex-col gap-3 p-4">
          {adaptersQuery.isLoading ? (
            <div className="flex flex-col gap-2">
              <div className="skeleton h-10 rounded-lg" />
              <div className="skeleton h-6 w-2/3 rounded-lg" />
            </div>
          ) : adapters?.length ? (
            <>
              <div className="flex flex-wrap items-center gap-2">
                <span
                  className={cn(
                    "rounded-full px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide",
                    powered ? "bg-teal/20 text-teal" : "bg-surface1 text-subtext0"
                  )}
                >
                  {powered ? "Powered" : "Off"}
                </span>
                {discovering ? (
                  <span className="rounded-full bg-blue/20 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-blue">
                    Discovering
                  </span>
                ) : null}
                <span className="text-[11px] text-subtext0">
                  {adapters.length === 1 ? adapters[0].name : `${adapters.length} adapters`}
                </span>
              </div>
              {adapters.length > 1 ? (
                <ul className="flex flex-col gap-1.5 border-t border-surface0/50 pt-3">
                  {adapters.map((a) => (
                    <li key={a.path} className="flex items-center justify-between gap-2 text-xs">
                      <span className="min-w-0 truncate font-medium text-subtext1">{a.name}</span>
                      <span className="shrink-0 text-subtext0">{a.powered ? "On" : "Off"}</span>
                    </li>
                  ))}
                </ul>
              ) : null}
            </>
          ) : (
            <p className="text-sm text-subtext0">Checking for adapters…</p>
          )}
        </div>
      )}

      {!fetchErr && !loading && blockedEmpty ? (
        <EmptyPanel icon={blockedEmpty.icon} title={blockedEmpty.title} body={blockedEmpty.body} />
      ) : null}

      {!fetchErr && powered ? (
        <>
          <button
            type="button"
            className="btn-surface text-sm disabled:opacity-40"
            disabled={scanBusy}
            onClick={() => void scan()}
          >
            <span className={cn("icon text-base", scanBusy && "animate-pulse")}>radar</span>
            {scanBusy ? "Scanning…" : "Scan for devices"}
          </button>

          {devicesQuery.isLoading ? (
            <div className="flex flex-col gap-2">
              {[1, 2, 3].map((i) => (
                <div key={i} className="skeleton h-[4.25rem] rounded-xl" />
              ))}
            </div>
          ) : sortedDevices.length === 0 ? (
            <EmptyPanel
              icon="devices_other"
              title="No Bluetooth devices"
              body="Run a scan to find keyboards, headsets, and other gear. Pair unfamiliar devices in system settings first if they do not appear."
            />
          ) : (
            <div className="flex flex-col gap-2">
              {sortedDevices.map((d) => {
                const loadingRow = busyAddr === d.address
                return (
                  <div
                    key={d.address}
                    className={cn(
                      "glass-card flex items-center gap-3 p-3 transition-colors",
                      d.connected && "border-teal/35 bg-teal/5"
                    )}
                  >
                    <div
                      className={cn(
                        "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl",
                        d.connected ? "bg-teal/20 text-teal" : "bg-surface1 text-subtext1"
                      )}
                    >
                      <span className="icon text-[22px]">bluetooth</span>
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-1.5">
                        <p className="truncate text-sm font-semibold text-text">{d.name || "Unknown device"}</p>
                        {d.connected ? (
                          <span className="rounded bg-teal/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-teal">
                            Connected
                          </span>
                        ) : null}
                        {d.paired && !d.connected ? (
                          <span className="rounded bg-surface2 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-subtext0">
                            Paired
                          </span>
                        ) : null}
                      </div>
                      <p className="truncate font-mono text-[11px] text-subtext0">{d.address}</p>
                      {d.battery_percentage != null ? (
                        <p className="mt-0.5 text-[11px] text-subtext1">
                          Battery ~{d.battery_percentage}%
                        </p>
                      ) : null}
                    </div>
                    <button
                      type="button"
                      disabled={loadingRow || !powered}
                      className={cn(
                        "min-w-[6.5rem] shrink-0 rounded-full px-3 py-2 text-[11px] font-semibold transition-colors disabled:opacity-40",
                        d.connected
                          ? "bg-surface2 text-subtext1 hover:bg-surface1"
                          : "bg-teal/90 text-crust hover:bg-teal"
                      )}
                      onClick={() => void toggleConn(d.address, d.connected)}
                    >
                      {loadingRow ? "…" : d.connected ? "Disconnect" : "Connect"}
                    </button>
                  </div>
                )
              })}
            </div>
          )}
        </>
      ) : null}

      {!fetchErr && powered && sortedDevices.length > 0 ? (
        <p className="text-xs leading-relaxed text-subtext1 max-w-prose">
          Trust and pairing details may still require system Bluetooth settings for some devices.
        </p>
      ) : null}
    </motion.div>
  )
}

export default BluetoothPane
