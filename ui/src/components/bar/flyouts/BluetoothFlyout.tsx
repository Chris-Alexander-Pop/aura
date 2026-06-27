import { useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutActionButton,
  FlyoutExpandLink,
  FlyoutIconButton,
  FlyoutList,
  FlyoutMeta,
  FlyoutRow,
  FlyoutRowIcon,
  FlyoutRowLabel,
  FlyoutShell,
  FlyoutTitle,
  FlyoutToggleRow,
} from "@/components/bar/flyouts/FlyoutPrimitives"

type Adapter = Awaited<ReturnType<typeof api.getBluetoothAdapters>>[number]
type Device = Awaited<ReturnType<typeof api.getBluetoothDevices>>[number]

function adapterSummary(adapters: Adapter[] | undefined): string {
  if (!adapters?.length) return "No adapter"
  const a = adapters[0]
  if (!a.powered) return "off"
  if (a.discovering) return "discovering"
  return "on"
}

export default function BluetoothFlyout() {
  const qc = useQueryClient()
  const [busyAddr, setBusyAddr] = useState<string | null>(null)

  const {
    data: adapters,
    isLoading: adLoading,
    isError: adError,
  } = useQuery({
    queryKey: ["bt-ad"],
    queryFn: api.getBluetoothAdapters,
    staleTime: 15_000,
    placeholderData: () => qc.getQueryData<Adapter[]>(["bt-ad"]),
  })

  const {
    data: devices,
    isLoading: devLoading,
    isError: devError,
  } = useQuery({
    queryKey: ["bt-dev"],
    queryFn: api.getBluetoothDevices,
    staleTime: 15_000,
    placeholderData: () => qc.getQueryData<Device[]>(["bt-dev"]),
  })

  const list = [...(devices ?? [])].sort(
    (a, b) => Number(b.connected) - Number(a.connected) || Number(b.paired) - Number(a.paired)
  )
  const shown = list.slice(0, 6)
  const connected = list.filter((d) => d.connected).length
  const powered = adapters?.some((a) => a.powered)

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

  const toggleAdapterPower = async (enabled: boolean) => {
    if (!adapters?.length) return
    await Promise.all(adapters.map((a) => api.setBluetoothAdapterPower(a.path, enabled)))
    await qc.invalidateQueries({ queryKey: ["bt-ad"] })
    await qc.invalidateQueries({ queryKey: ["bt-dev"] })
  }

  if (adLoading && adapters == null) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Bluetooth</FlyoutTitle>
        <FlyoutLoading label="Fetching Bluetooth adapters…" />
      </FlyoutShell>
    )
  }

  if (adError || devError) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Bluetooth</FlyoutTitle>
        <FlyoutEmpty
          icon="bluetooth_disabled"
          title="Could not load Bluetooth"
          detail="Check that BlueZ is running."
        />
      </FlyoutShell>
    )
  }

  if (!adapters?.length) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Bluetooth</FlyoutTitle>
        <FlyoutEmpty
          icon="settings_bluetooth"
          title="No adapter found"
          detail="No Bluetooth controller reported."
        />
      </FlyoutShell>
    )
  }

  return (
    <FlyoutShell>
      <FlyoutTitle>Bluetooth {adapterSummary(adapters)}</FlyoutTitle>

      <FlyoutToggleRow
        label="Enabled"
        checked={!!powered}
        onChange={toggleAdapterPower}
      />

      <FlyoutMeta>{devLoading && devices == null ? "Loading devices…" : countLine}</FlyoutMeta>

      {devLoading && devices == null ? (
        <FlyoutLoading label="Fetching paired devices…" />
      ) : shown.length === 0 ? (
        <FlyoutEmpty
          icon="bluetooth_searching"
          title="No devices yet"
          detail={powered ? "Scan below to discover hardware." : "Turn Bluetooth on first."}
        />
      ) : (
        <FlyoutList>
          {shown.map((d) => {
            const loading = busyAddr === d.address
            return (
              <FlyoutRow key={d.address}>
                <FlyoutRowIcon icon="bluetooth" active={d.connected} />
                <FlyoutRowLabel active={d.connected}>{d.name || d.address}</FlyoutRowLabel>
                <FlyoutIconButton
                  icon={d.connected ? "link_off" : "link"}
                  active={d.connected}
                  disabled={!powered}
                  loading={loading}
                  title={d.connected ? "Disconnect" : "Connect"}
                  onClick={() => toggleConn(d.address, d.connected)}
                />
              </FlyoutRow>
            )
          })}
        </FlyoutList>
      )}

      <FlyoutActionButton
        icon="bluetooth_searching"
        disabled={!powered}
        variant="primary"
        onClick={() => scan()}
      >
        Scan for devices
      </FlyoutActionButton>

      <FlyoutExpandLink
        label="Bluetooth settings"
        onClick={() => api.openControlCenterPane("bluetooth")}
      />
    </FlyoutShell>
  )
}
