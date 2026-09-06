import { useEffect, useMemo, useRef, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  isPasswordRequiredError,
  isSecuredNetwork,
  REMMINA_DESKTOP_ID,
  strengthIcon,
} from "@/lib/network-connect"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutActionButton,
  FlyoutBanner,
  FlyoutExpandLink,
  FlyoutIconButton,
  FlyoutList,
  FlyoutMeta,
  FlyoutRadioRow,
  FlyoutRow,
  FlyoutRowIcon,
  FlyoutRowLabel,
  FlyoutSectionLabel,
  FlyoutShell,
  FlyoutTitle,
  FlyoutToggleRow,
} from "@/components/bar/flyouts/FlyoutPrimitives"
import { cn } from "@/lib/utils"

type PasswordTarget = { ssid: string; security: string }

export default function NetworkFlyout() {
  const qc = useQueryClient()
  const passwordRef = useRef<HTMLInputElement>(null)
  const [connectingToSsid, setConnectingToSsid] = useState<string | null>(null)
  const [connectError, setConnectError] = useState<string | null>(null)
  const [passwordTarget, setPasswordTarget] = useState<PasswordTarget | null>(null)
  const [password, setPassword] = useState("")
  const [passwordBusy, setPasswordBusy] = useState(false)
  const [passwordError, setPasswordError] = useState<string | null>(null)
  const [vpnBusy, setVpnBusy] = useState(false)
  const [remoteError, setRemoteError] = useState<string | null>(null)

  const { data: net, isLoading: netLoading, isError: netError } = useQuery({
    queryKey: ["net"],
    queryFn: api.getNetworkStatus,
    staleTime: 15_000,
  })

  const wifiOn = !!net?.wifi_enabled

  const {
    data: scan,
    isFetching: scanning,
    isLoading: scanLoading,
  } = useQuery({
    queryKey: ["wifi-scan"],
    queryFn: api.scanNetworks,
    enabled: wifiOn,
    staleTime: 30_000,
  })

  const { data: saved } = useQuery({
    queryKey: ["network-saved"],
    queryFn: api.listSavedNetworks,
    staleTime: 30_000,
  })

  const { data: keyring } = useQuery({
    queryKey: ["keyring-status"],
    queryFn: api.getKeyringStatus,
    staleTime: 60_000,
  })

  const { data: vpnStatus } = useQuery({
    queryKey: ["vpn-status"],
    queryFn: api.getVpnStatus,
    refetchInterval: 4000,
  })

  const { data: vpnProfiles } = useQuery({
    queryKey: ["vpn-profiles"],
    queryFn: api.getVpnProfiles,
    staleTime: 60_000,
  })

  const savedNames = useMemo(() => new Set((saved ?? []).map((s) => s.name)), [saved])

  const sorted = useMemo(() => {
    const list = scan ?? []
    return [...list]
      .sort((a, b) => {
        if (a.active !== b.active) return (b.active ? 1 : 0) - (a.active ? 1 : 0)
        return b.strength - a.strength
      })
      .slice(0, 8)
  }, [scan])

  const [vpnProfileId, setVpnProfileId] = useState("")
  useEffect(() => {
    const id = vpnStatus?.profile_id
    if (id) {
      setVpnProfileId(id)
      return
    }
    if (!vpnProfileId && (vpnProfiles?.length ?? 0) > 0) {
      setVpnProfileId(vpnProfiles![0].id)
    }
  }, [vpnStatus?.profile_id, vpnProfiles, vpnProfileId])

  useEffect(() => {
    if (!passwordTarget) return
    setPassword("")
    setPasswordError(null)
    const t = window.setTimeout(() => passwordRef.current?.focus(), 40)
    return () => window.clearTimeout(t)
  }, [passwordTarget])

  const invalidateNetwork = async () => {
    await qc.invalidateQueries({ queryKey: ["net"] })
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.invalidateQueries({ queryKey: ["network-saved"] })
  }

  const toggleWifi = async (enabled: boolean) => {
    await api.toggleWifi(enabled)
    await invalidateNetwork()
  }

  /** Caelestia path: try NM/keyring first; only then ask for a password. */
  const tryConnect = async (ssid: string, pass?: string) => {
    setConnectingToSsid(ssid)
    setConnectError(null)
    try {
      await api.connectNetwork(ssid, pass)
      setPasswordTarget(null)
      setPassword("")
      await invalidateNetwork()
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Connection failed"
      if (!pass && isPasswordRequiredError(err)) {
        const ap = sorted.find((n) => n.ssid === ssid)
        setPasswordTarget({ ssid, security: ap?.security ?? "Secured" })
        setConnectError(null)
        return
      }
      setConnectError(msg)
      throw err
    } finally {
      setConnectingToSsid(null)
    }
  }

  const onConnectClick = (ap: { ssid: string; security: string; active: boolean }) => {
    if (ap.active) {
      void disconnect()
      return
    }
    void tryConnect(ap.ssid)
  }

  const submitPassword = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!passwordTarget) return
    if (!password.trim()) {
      setPasswordError("Enter the network password.")
      return
    }
    setPasswordBusy(true)
    setPasswordError(null)
    try {
      await tryConnect(passwordTarget.ssid, password)
    } catch (err) {
      setPasswordError(err instanceof Error ? err.message : "Connection failed")
    } finally {
      setPasswordBusy(false)
    }
  }

  const disconnect = async () => {
    await api.disconnectNetwork()
    await invalidateNetwork()
  }

  const rescan = async () => {
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.refetchQueries({ queryKey: ["wifi-scan"] })
  }

  const vpnConnected = vpnStatus?.state === "connected"
  const vpnConnecting = vpnStatus?.state === "connecting" || vpnBusy

  const toggleVpn = async (enabled: boolean) => {
    setVpnBusy(true)
    setConnectError(null)
    try {
      if (enabled) {
        const id = vpnProfileId.trim() || vpnProfiles?.[0]?.id
        if (!id) {
          setConnectError("No VPN profile configured.")
          return
        }
        await api.connectVpn(id)
      } else {
        await api.disconnectVpn()
      }
      await qc.invalidateQueries({ queryKey: ["vpn-status"] })
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : "VPN action failed")
    } finally {
      setVpnBusy(false)
    }
  }

  const openRemmina = async () => {
    setRemoteError(null)
    try {
      await api.launcherRun(REMMINA_DESKTOP_ID)
    } catch (err) {
      setRemoteError(err instanceof Error ? err.message : "Could not open Remmina")
    }
  }

  if (netLoading && net == null) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Network</FlyoutTitle>
        <FlyoutLoading label="Fetching network status…" />
      </FlyoutShell>
    )
  }

  if (netError) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Network</FlyoutTitle>
        <FlyoutEmpty icon="wifi_off" title="Could not load network" detail="Try again from Control Center." />
      </FlyoutShell>
    )
  }

  // Inline password step (never a fixed modal — flyout window is only ~320px wide)
  if (passwordTarget) {
    return (
      <FlyoutShell>
        <button
          type="button"
          className="flex items-center gap-1 text-[11px] text-subtext1 hover:text-text"
          onClick={() => {
            if (!passwordBusy) {
              setPasswordTarget(null)
              setPasswordError(null)
            }
          }}
          disabled={passwordBusy}
        >
          <span className="icon text-sm">arrow_back</span>
          Networks
        </button>
        <FlyoutTitle>
          <span className="flex items-center gap-1.5">
            <span className="icon text-base text-mauve">lock</span>
            {passwordTarget.ssid}
          </span>
        </FlyoutTitle>
        <FlyoutMeta>{passwordTarget.security || "Secured network"}</FlyoutMeta>

        <form onSubmit={(e) => void submitPassword(e)} className="flex flex-col gap-2 pt-1">
          <label className="flex flex-col gap-1">
            <span className="text-[10px] uppercase tracking-wide text-subtext0">Password</span>
            <input
              ref={passwordRef}
              type="password"
              autoComplete="current-password"
              className="w-full rounded-lg border border-surface0/80 bg-base/80 px-2.5 py-2 text-[12px] text-text placeholder:text-subtext0 focus:outline-none focus:ring-2 focus:ring-mauve/50"
              placeholder="Network password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={passwordBusy}
            />
          </label>
          {passwordError ? (
            <FlyoutBanner tone="error">{passwordError}</FlyoutBanner>
          ) : null}
          <div className="flex gap-1.5">
            <button
              type="button"
              className="flex-1 rounded-full bg-surface0/60 py-2 text-[11px] font-medium text-subtext1 hover:bg-surface0 hover:text-text disabled:opacity-40"
              onClick={() => {
                if (!passwordBusy) {
                  setPasswordTarget(null)
                  setPasswordError(null)
                }
              }}
              disabled={passwordBusy}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="flex-1 rounded-full bg-mauve py-2 text-[11px] font-semibold text-base hover:brightness-110 disabled:opacity-40"
              disabled={passwordBusy || !password.trim()}
            >
              {passwordBusy ? "Connecting…" : "Connect"}
            </button>
          </div>
        </form>
      </FlyoutShell>
    )
  }

  const listBusy = wifiOn && sorted.length === 0 && scan == null && scanLoading
  const showCount = wifiOn && sorted.length > 0
  const hasVpnProfiles = (vpnProfiles?.length ?? 0) > 0

  return (
    <FlyoutShell className="gap-2">
      <FlyoutSectionLabel>Wireless</FlyoutSectionLabel>
      <FlyoutTitle>Wi‑Fi {wifiOn ? "enabled" : "disabled"}</FlyoutTitle>

      {keyring && keyring.available && !keyring.unlocked ? (
        <FlyoutBanner>
          {keyring.message ?? "Login keyring is locked — saved Aura passwords unavailable."}
        </FlyoutBanner>
      ) : null}

      {connectError ? <FlyoutBanner tone="error">{connectError}</FlyoutBanner> : null}

      <FlyoutToggleRow label="Enabled" checked={wifiOn} onChange={(v) => void toggleWifi(v)} />

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
            const secure = isSecuredNetwork(ap.security)
            const isSaved = savedNames.has(ap.ssid)

            return (
              <FlyoutRow key={ap.ssid}>
                <FlyoutRowIcon icon={strengthIcon(ap.strength)} active={ap.active} />
                {secure ? (
                  <span className="icon shrink-0 text-[12px] text-subtext0" title="Secured network">
                    lock
                  </span>
                ) : null}
                <FlyoutRowLabel active={ap.active}>
                  {ap.ssid}
                  {isSaved && !ap.active ? (
                    <span className="ml-1 text-[9px] font-normal text-subtext0">saved</span>
                  ) : null}
                </FlyoutRowLabel>
                <FlyoutIconButton
                  icon={ap.active ? "link_off" : "link"}
                  active={ap.active}
                  disabled={!wifiOn}
                  loading={isConnecting}
                  title={ap.active ? "Disconnect" : isSaved ? "Connect (saved)" : "Connect"}
                  onClick={() => onConnectClick(ap)}
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
        onClick={() => void rescan()}
      >
        {scanning ? "Scanning…" : "Rescan networks"}
      </FlyoutActionButton>

      <div className="mt-1 border-t border-surface0/60 pt-2">
        <FlyoutSectionLabel>VPN</FlyoutSectionLabel>
        <FlyoutToggleRow
          label="Enabled"
          checked={vpnConnected}
          disabled={vpnConnecting || (!hasVpnProfiles && !vpnConnected)}
          onChange={(v) => void toggleVpn(v)}
        />
        {vpnConnected && vpnStatus?.profile_id ? (
          <FlyoutMeta>{vpnStatus.profile_id}</FlyoutMeta>
        ) : vpnConnecting ? (
          <FlyoutMeta>Connecting…</FlyoutMeta>
        ) : null}
        {hasVpnProfiles && !vpnConnected ? (
          <div className="mt-1 flex max-h-28 flex-col gap-0.5 overflow-y-auto">
            {vpnProfiles!.map((p) => (
              <FlyoutRadioRow
                key={p.id}
                label={p.display_name || p.name || p.id}
                checked={vpnProfileId === p.id}
                disabled={vpnConnecting}
                onSelect={() => setVpnProfileId(p.id)}
              />
            ))}
          </div>
        ) : null}
        {!hasVpnProfiles ? (
          <FlyoutMeta>Add providers under Control Center → VPN.</FlyoutMeta>
        ) : null}
      </div>

      <div className="border-t border-surface0/60 pt-2">
        <FlyoutSectionLabel>Remote</FlyoutSectionLabel>
        {remoteError ? <FlyoutBanner tone="error">{remoteError}</FlyoutBanner> : null}
        <button
          type="button"
          className={cn(
            "mt-1 flex w-full items-center gap-2 rounded-lg px-1.5 py-1.5 text-left text-[11px]",
            "text-subtext1 transition-colors hover:bg-surface0/70 hover:text-text",
          )}
          onClick={() => void openRemmina()}
        >
          <span className="icon text-base text-subtext0">desktop_windows</span>
          <span className="min-w-0 flex-1 truncate">Remmina</span>
          <span className="icon text-sm text-subtext0">open_in_new</span>
        </button>
      </div>

      <FlyoutExpandLink
        label="Network settings"
        onClick={() => api.openControlCenterPane("network")}
      />
    </FlyoutShell>
  )
}
