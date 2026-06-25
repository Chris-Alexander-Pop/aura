import { useEffect, useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutActionButton,
  FlyoutBanner,
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
import { NetworkConnectModal } from "@/pages/control-center/panes/NetworkConnectModal"

function strengthIcon(strength: number): string {
  if (strength >= 75) return "wifi"
  if (strength >= 50) return "wifi_2_bar"
  if (strength >= 25) return "wifi_1_bar"
  return "wifi_1_bar"
}

function isSecured(security: string): boolean {
  const s = security.trim().toLowerCase()
  return s.length > 0 && s !== "none" && s !== "open"
}

export default function NetworkFlyout() {
  const qc = useQueryClient()
  const [connectingToSsid, setConnectingToSsid] = useState<string | null>(null)
  const [connectError, setConnectError] = useState<string | null>(null)
  const [passwordTarget, setPasswordTarget] = useState<{ ssid: string; security: string } | null>(
    null
  )

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Network.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["net"] })
      void qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    })
    return off
  }, [qc])

  const { data: net, isPending: netPending, isError: netError } = useQuery({
    queryKey: ["net"],
    queryFn: api.getNetworkStatus,
    refetchInterval: 8000,
  })
  const { data: scan, isFetching: scanning, isPending: scanPending } = useQuery({
    queryKey: ["wifi-scan"],
    queryFn: api.scanNetworks,
    staleTime: 15_000,
  })
  const { data: keyring } = useQuery({
    queryKey: ["keyring-status"],
    queryFn: api.getKeyringStatus,
    staleTime: 60_000,
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

  const runConnect = async (ssid: string, password?: string) => {
    setConnectingToSsid(ssid)
    setConnectError(null)
    try {
      await api.connectNetwork(ssid, password)
      await qc.invalidateQueries({ queryKey: ["net"] })
      await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Connection failed"
      setConnectError(msg)
      throw err
    } finally {
      setConnectingToSsid(null)
    }
  }

  const onConnectClick = (ap: { ssid: string; security: string; active: boolean }, active: boolean) => {
    if (active) {
      void disconnect()
      return
    }
    if (isSecured(ap.security)) {
      setPasswordTarget({ ssid: ap.ssid, security: ap.security })
      return
    }
    void runConnect(ap.ssid)
  }

  const disconnect = async () => {
    await api.disconnectNetwork()
    await qc.invalidateQueries({ queryKey: ["net"] })
  }

  const rescan = async () => {
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.refetchQueries({ queryKey: ["wifi-scan"] })
  }

  if (netPending) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Wi‑Fi</FlyoutTitle>
        <FlyoutLoading label="Fetching network status…" />
      </FlyoutShell>
    )
  }

  if (netError) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Wi‑Fi</FlyoutTitle>
        <FlyoutEmpty icon="wifi_off" title="Could not load Wi‑Fi" detail="Try again from Control Center." />
      </FlyoutShell>
    )
  }

  const listBusy = wifiOn && sorted.length === 0 && (scanning || scanPending)
  const showCount = wifiOn && sorted.length > 0

  return (
    <>
      <FlyoutShell>
        <FlyoutTitle>Wi‑Fi {wifiOn ? "enabled" : "disabled"}</FlyoutTitle>

        {keyring && keyring.available && !keyring.unlocked ? (
          <FlyoutBanner>
            {keyring.message ?? "Login keyring is locked — saved passwords unavailable."}
          </FlyoutBanner>
        ) : null}

        {connectError ? <FlyoutBanner tone="error">{connectError}</FlyoutBanner> : null}

        <FlyoutToggleRow label="Enabled" checked={wifiOn} onChange={toggleWifi} />

        {showCount ? (
          <FlyoutMeta>
            {sorted.length} network{sorted.length === 1 ? "" : "s"} available
          </FlyoutMeta>
        ) : null}

        {listBusy ? <FlyoutLoading label="Scanning for networks…" /> : null}
        {!wifiOn ? (
          <FlyoutEmpty icon="wifi_off" title="Wi‑Fi is off" detail="Enable Wi‑Fi to scan and connect." />
        ) : null}
        {wifiOn && !listBusy && sorted.length === 0 ? (
          <FlyoutEmpty
            icon="wifi_find"
            title="No networks found"
            detail="Move closer to your router or rescan."
          />
        ) : null}

        {wifiOn && sorted.length > 0 ? (
          <FlyoutList>
            {sorted.map((ap) => {
              const isConnecting = connectingToSsid === ap.ssid
              const secure = isSecured(ap.security)

              return (
                <FlyoutRow key={ap.ssid}>
                  <FlyoutRowIcon icon={strengthIcon(ap.strength)} active={ap.active} />
                  {secure ? (
                    <span className="icon shrink-0 text-[12px] text-subtext0" title="Secured network">
                      lock
                    </span>
                  ) : null}
                  <FlyoutRowLabel active={ap.active}>{ap.ssid}</FlyoutRowLabel>
                  <FlyoutIconButton
                    icon={ap.active ? "link_off" : "link"}
                    active={ap.active}
                    disabled={!wifiOn}
                    loading={isConnecting}
                    title={ap.active ? "Disconnect" : secure ? "Enter password" : "Connect"}
                    onClick={() => onConnectClick(ap, ap.active)}
                  />
                </FlyoutRow>
              )
            })}
          </FlyoutList>
        ) : null}

        <FlyoutActionButton
          icon="wifi_find"
          loading={scanning}
          disabled={scanning || !wifiOn}
          variant="primary"
          onClick={() => rescan()}
        >
          {scanning ? "Scanning…" : "Rescan networks"}
        </FlyoutActionButton>

        <FlyoutExpandLink
          label="Wi‑Fi settings"
          onClick={() => api.openControlCenterPane("network")}
        />
      </FlyoutShell>

      <NetworkConnectModal
        open={passwordTarget != null}
        ssid={passwordTarget?.ssid ?? ""}
        securityLabel={passwordTarget?.security ?? ""}
        onClose={() => setPasswordTarget(null)}
        onConnect={async (password) => {
          if (!passwordTarget) return
          await runConnect(passwordTarget.ssid, password)
        }}
      />
    </>
  )
}
