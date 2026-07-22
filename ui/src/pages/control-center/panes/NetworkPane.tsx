import { useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import {
  isPasswordRequiredError,
  isSecuredNetwork,
  strengthIcon,
} from "@/lib/network-connect"
import { cn } from "@/lib/utils"
import { NetworkConnectModal } from "./NetworkConnectModal"

export function NetworkPane() {
  const qc = useQueryClient()
  const [connectingToSsid, setConnectingToSsid] = useState<string | null>(null)
  const [connectError, setConnectError] = useState<string | null>(null)
  const [passwordTarget, setPasswordTarget] = useState<{
    ssid: string
    security: string
  } | null>(null)

  const { data: status, isLoading: statusLoading } = useQuery({
    queryKey: ["network-status"],
    queryFn: api.getNetworkStatus,
    refetchInterval: 5000,
  })

  const wifiOn = !!status?.wifi_enabled

  const {
    data: scan,
    isFetching: scanning,
    isError: scanError,
    error: scanErr,
  } = useQuery({
    queryKey: ["wifi-scan"],
    queryFn: api.scanNetworks,
    enabled: wifiOn,
    staleTime: 12_000,
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

  const sorted = useMemo(() => {
    const list = scan ?? []
    return [...list].sort((a, b) => {
      if (a.active !== b.active) return (b.active ? 1 : 0) - (a.active ? 1 : 0)
      return b.strength - a.strength
    })
  }, [scan])

  const invalidateNetwork = async () => {
    await qc.invalidateQueries({ queryKey: ["network-status"] })
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.invalidateQueries({ queryKey: ["network-saved"] })
  }

  const setWifi = async (enabled: boolean) => {
    await api.toggleWifi(enabled)
    await invalidateNetwork()
  }

  const rescan = async () => {
    await qc.invalidateQueries({ queryKey: ["wifi-scan"] })
    await qc.refetchQueries({ queryKey: ["wifi-scan"] })
  }

  const runConnect = async (ssid: string, password?: string) => {
    setConnectingToSsid(ssid)
    setConnectError(null)
    try {
      await api.connectNetwork(ssid, password)
      setPasswordTarget(null)
      await invalidateNetwork()
    } catch (err) {
      if (!password && isPasswordRequiredError(err)) {
        const ap = sorted.find((n) => n.ssid === ssid)
        setPasswordTarget({ ssid, security: ap?.security ?? "Secured" })
        setConnectError(null)
        return
      }
      setConnectError(err instanceof Error ? err.message : "Connection failed")
      throw err
    } finally {
      setConnectingToSsid(null)
    }
  }

  /** Caelestia path: always try saved NM profile / keyring first. */
  const onApActivate = (ap: { ssid: string; security: string; active: boolean }) => {
    if (ap.active) return
    if (!wifiOn) return
    void runConnect(ap.ssid)
  }

  const disconnect = async () => {
    await api.disconnectNetwork()
    await invalidateNetwork()
  }

  const scanErrMsg = scanError && scanErr instanceof Error ? scanErr.message : null

  return (
    <>
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.18 }}
        className="flex flex-col gap-5 p-6 h-full overflow-y-auto min-h-0 relative"
      >
        {/* Shell components not present at `@/components/shell/*` — match ControlCenter pane header */}
        <header className="flex items-start justify-between gap-3">
          <div>
            <div className="flex items-center gap-3">
              <span className="icon text-mauve text-2xl">wifi</span>
              <h2 className="text-xl font-semibold text-text tracking-tight">Network</h2>
            </div>
            <p className="text-xs text-subtext1 mt-1.5 max-w-prose">
              Wi‑Fi status, scan, and connection. Saved networks connect without re-entering a password.
            </p>
          </div>
        </header>

        {keyring && keyring.available && !keyring.unlocked ? (
          <div
            className="rounded-xl border border-yellow/30 bg-yellow/10 px-4 py-3 text-xs text-yellow"
            role="status"
          >
            {keyring.message ??
              "Login keyring is locked — unlock it to use saved WiFi passwords after reboot."}
          </div>
        ) : null}

        {connectError ? (
          <p className="text-xs text-red px-1" role="alert">
            {connectError}
          </p>
        ) : null}

        <section
          className={cn(
            "glass-card rounded-2xl p-5 flex flex-col gap-4 border border-surface0/50",
            "bg-surface0/25 backdrop-blur-xl shadow-lg"
          )}
        >
          {statusLoading ? (
            <div className="skeleton h-24 rounded-xl" />
          ) : (
            <>
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="min-w-0">
                  <p className="text-sm font-semibold text-text truncate">
                    {status?.active_connection?.trim()
                      ? status.active_connection
                      : wifiOn
                        ? "Not connected"
                        : "Wi‑Fi off"}
                  </p>
                  <div className="mt-1 flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-subtext0">
                    <span>Local {status?.local_ip ?? "—"}</span>
                    <span className="text-surface0">·</span>
                    <span>Public {status?.public_ip ?? "—"}</span>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => setWifi(!wifiOn)}
                  className={cn("toggle-chip text-xs shrink-0", wifiOn && "active")}
                >
                  <span className="icon text-base">wifi</span>
                  {wifiOn ? "On" : "Off"}
                </button>
              </div>

              {status?.active_connection?.trim() ? (
                <button
                  type="button"
                  onClick={() => void disconnect()}
                  className="btn-surface text-sm w-full sm:w-auto self-start flex items-center justify-center gap-2 border border-red/25 bg-red/10 text-red hover:bg-red/15"
                >
                  <span className="icon text-base">link_off</span>
                  Disconnect
                </button>
              ) : null}
            </>
          )}
        </section>

        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            disabled={scanning || !wifiOn}
            className={cn(
              "btn-surface text-sm flex items-center gap-2 flex-1 min-w-[10rem] sm:flex-none",
              "border border-surface0/60 bg-surface0/20 backdrop-blur-md"
            )}
            onClick={() => void rescan()}
          >
            <span className={cn("icon text-lg", scanning && "animate-spin")}>wifi_find</span>
            {scanning ? "Scanning…" : "Scan networks"}
          </button>
          {!wifiOn ? (
            <span className="text-[11px] text-subtext0">Turn Wi‑Fi on to scan and connect.</span>
          ) : null}
        </div>

        {scanErrMsg ? (
          <p className="text-xs text-red" role="alert">
            Scan failed: {scanErrMsg}
          </p>
        ) : null}

        <ul className="flex flex-col gap-2.5 pr-0.5">
          {sorted.map((ap) => {
            const secure = isSecuredNetwork(ap.security)
            const isConnecting = connectingToSsid === ap.ssid
            const canInteract = wifiOn && !ap.active
            const isSaved = (saved ?? []).some((s) => s.name === ap.ssid)

            return (
              <li key={`${ap.ssid}-${ap.strength}`}>
                <div
                  className={cn(
                    "glass-card rounded-2xl p-4 flex items-center gap-3 border transition-colors",
                    "bg-surface0/20 backdrop-blur-xl border-surface0/45",
                    ap.active && "border-mauve/50 ring-1 ring-mauve/20",
                    canInteract && "hover:border-mauve/25 cursor-pointer",
                    !canInteract && !ap.active && "opacity-60"
                  )}
                  onClick={() => canInteract && onApActivate(ap)}
                  onKeyDown={(e) => {
                    if (!canInteract) return
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault()
                      onApActivate(ap)
                    }
                  }}
                  role={canInteract ? "button" : undefined}
                  tabIndex={canInteract ? 0 : undefined}
                >
                  <span
                    className={cn(
                      "icon shrink-0 text-[26px] leading-none",
                      ap.active ? "text-mauve" : "text-subtext0"
                    )}
                  >
                    {strengthIcon(ap.strength)}
                  </span>

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <p
                        className={cn(
                          "text-sm font-medium truncate",
                          ap.active ? "text-mauve" : "text-text"
                        )}
                        title={ap.ssid}
                      >
                        {ap.ssid}
                      </p>
                      {secure ? (
                        <span className="icon text-[15px] text-subtext0 shrink-0" title="Secured">
                          lock
                        </span>
                      ) : null}
                      {ap.active ? (
                        <span className="icon text-teal text-base shrink-0" title="Connected">
                          check_circle
                        </span>
                      ) : null}
                    </div>
                    <p className="text-[11px] text-subtext0 mt-0.5">
                      {ap.security || "—"} · {ap.strength}%
                      {isSaved ? " · saved" : ""}
                      {canInteract ? " · tap to connect" : ""}
                    </p>
                  </div>

                  <div className="flex shrink-0 items-center gap-1.5">
                    {ap.active ? (
                      <button
                        type="button"
                        className={cn(
                          "flex h-10 w-10 items-center justify-center rounded-full transition-colors",
                          "bg-mauve/90 text-base hover:brightness-110"
                        )}
                        title="Disconnect"
                        onClick={(e) => {
                          e.stopPropagation()
                          void disconnect()
                        }}
                      >
                        <span className="icon text-[22px]">link_off</span>
                      </button>
                    ) : (
                      <button
                        type="button"
                        disabled={!canInteract || isConnecting}
                        className={cn(
                          "flex h-10 w-10 items-center justify-center rounded-full transition-colors disabled:opacity-40",
                          secure
                            ? "bg-blue/35 text-blue hover:bg-blue/45"
                            : "bg-mauve/35 text-mauve hover:bg-mauve/45"
                        )}
                        title="Connect"
                        onClick={(e) => {
                          e.stopPropagation()
                          onApActivate(ap)
                        }}
                      >
                        {isConnecting ? (
                          <span className="icon animate-spin text-[20px]">progress_activity</span>
                        ) : (
                          <span className="icon text-[22px]">link</span>
                        )}
                      </button>
                    )}
                  </div>
                </div>
              </li>
            )
          })}
        </ul>

        {wifiOn && sorted.length === 0 && !scanning && !scanError ? (
          <p className="text-xs text-subtext1 text-center py-6">
            No networks yet — run a scan or wait for results.
          </p>
        ) : null}

        {(saved?.length ?? 0) > 0 ? (
          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-semibold text-subtext1 px-1">Saved networks</h3>
            <ul className="flex flex-col gap-2">
              {saved!.map((conn) => (
                <li
                  key={conn.uuid}
                  className="glass-card rounded-xl p-3 flex items-center justify-between gap-2 border border-surface0/45"
                >
                  <div className="min-w-0">
                    <p className="text-sm font-medium truncate text-text">{conn.name}</p>
                    <p className="text-[11px] text-subtext0">
                      {conn.autoconnect ? "Auto-connect" : "Manual"}
                    </p>
                  </div>
                  <div className="flex shrink-0 gap-1.5">
                    <button
                      type="button"
                      className="btn-surface text-xs px-3 py-1.5"
                      onClick={() => void runConnect(conn.name)}
                    >
                      Connect
                    </button>
                    <button
                      type="button"
                      className="btn-surface text-xs px-3 py-1.5 text-red border-red/25"
                      onClick={async () => {
                        await api.forgetNetwork({ uuid: conn.uuid, name: conn.name })
                        await invalidateNetwork()
                      }}
                    >
                      Forget
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        ) : null}
      </motion.div>

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

export default NetworkPane
