import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

type VpnUiState = "disconnected" | "connecting" | "connected" | "error" | string

function stateBadgeClass(state: VpnUiState) {
  switch (state) {
    case "connected":
      return "bg-green/15 text-green border-green/35"
    case "connecting":
      return "bg-yellow/15 text-yellow border-yellow/35"
    case "error":
      return "bg-red/15 text-red border-red/35"
    default:
      return "bg-surface1/80 text-subtext1 border-surface2/60"
  }
}

function readableState(state: VpnUiState) {
  if (!state) return "Unknown"
  return state.charAt(0).toUpperCase() + state.slice(1)
}

export function VpnPane() {
  const qc = useQueryClient()
  const { data: profiles } = useQuery({
    queryKey: ["vpn-profiles"],
    queryFn: api.getVpnProfiles,
    staleTime: 60_000,
  })

  const { data: status, isLoading } = useQuery({
    queryKey: ["vpn-status"],
    queryFn: api.getVpnStatus,
    refetchInterval: 4000,
  })

  const [profileDraft, setProfileDraft] = useState("")
  const [attemptedSubmit, setAttemptedSubmit] = useState(false)

  useEffect(() => {
    const s = status?.state
    const id = status?.profile_id
    if ((s === "connected" || s === "connecting") && id) setProfileDraft(id)
  }, [status?.state, status?.profile_id])

  const connectMut = useMutation({
    mutationFn: (profileId: string) => api.connectVpn(profileId),
    onSuccess: async () => {
      setAttemptedSubmit(false)
      await qc.invalidateQueries({ queryKey: ["vpn-status"] })
      await qc.invalidateQueries({ queryKey: ["vpn-profiles"] })
    },
  })

  const disconnectMut = useMutation({
    mutationFn: api.disconnectVpn,
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["vpn-status"] })
      await qc.invalidateQueries({ queryKey: ["vpn-profiles"] })
    },
  })

  const trimmedProfile = profileDraft.trim()
  const busy = connectMut.isPending || disconnectMut.isPending
  const canConnect =
    trimmedProfile.length > 0 &&
    status?.state !== "connected" &&
    status?.state !== "connecting" &&
    !busy
  const highlightEmptyProfile =
    (status?.state === "disconnected" || status?.state === "error" || status == null) &&
    trimmedProfile.length === 0

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex items-center gap-3">
        <span className="icon text-mauve text-2xl">vpn_key</span>
        <h2 className="text-xl font-semibold">VPN</h2>
      </div>

      <div className="glass-card p-4 flex flex-col gap-3">
        {isLoading ? (
          <div className="skeleton h-24 rounded-lg" />
        ) : (
          <>
            <div className="flex flex-wrap items-center gap-2">
              <span
                className={cn(
                  "text-xs font-medium uppercase tracking-wide px-2.5 py-1 rounded-lg border",
                  stateBadgeClass(status?.state ?? "")
                )}
              >
                {readableState(status?.state ?? "disconnected")}
              </span>
              {status?.profile_id && (
                <span className="text-xs text-subtext0 font-mono truncate max-w-[14rem]" title={status.profile_id}>
                  {status.profile_id}
                </span>
              )}
            </div>
            <p className="text-sm text-subtext1 leading-snug">{status?.message ?? "—"}</p>
          </>
        )}
      </div>

      <div className="glass-card p-4 flex flex-col gap-3">
        <div className="flex flex-col gap-1">
          <label htmlFor="vpn-profile-id" className="text-sm font-medium text-text">
            Profile
          </label>
          {(profiles?.length ?? 0) > 0 ? (
            <select
              id="vpn-profile-id"
              className="input"
              value={profileDraft}
              disabled={busy || status?.state === "connected" || status?.state === "connecting"}
              onChange={(e) => {
                setProfileDraft(e.target.value)
                setAttemptedSubmit(false)
              }}
            >
              <option value="">Select profile…</option>
              {profiles!.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name || p.id}
                  {p.type ? ` (${p.type})` : ""}
                </option>
              ))}
            </select>
          ) : (
            <input
              id="vpn-profile-id"
              type="text"
              autoComplete="off"
              spellCheck={false}
              placeholder="profile id"
              value={profileDraft}
              disabled={busy || status?.state === "connected" || status?.state === "connecting"}
              onChange={(e) => {
                setProfileDraft(e.target.value)
                setAttemptedSubmit(false)
              }}
              className={cn(
                "input",
                highlightEmptyProfile && "border-peach/50 ring-1 ring-peach/25"
              )}
              aria-invalid={attemptedSubmit && trimmedProfile.length === 0 ? true : undefined}
              aria-describedby="vpn-profile-desc"
            />
          )}
          <p className="text-xs text-subtext0 leading-relaxed">
            Pick a configured profile or enter an ID manually.
          </p>
        </div>
        <div id="vpn-profile-desc" className="min-h-[1rem]">
          {attemptedSubmit && trimmedProfile.length === 0 ? (
            <p className="text-xs text-peach" role="status">
              Enter a profile ID to connect.
            </p>
          ) : status?.state === "connected" ? (
            <p className="text-xs text-subtext0">
              Disconnect to switch profiles or edit the profile ID.
            </p>
          ) : null}
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          className="btn-surface text-sm flex-1 min-w-[9rem]"
          disabled={!canConnect}
          onClick={() => {
            if (trimmedProfile.length === 0) {
              setAttemptedSubmit(true)
              return
            }
            connectMut.mutate(trimmedProfile)
          }}
        >
          <span className="icon text-base">link</span>
          {busy && connectMut.isPending ? "Connecting…" : "Connect"}
        </button>
        <button
          type="button"
          className={cn(
            "text-sm flex-1 min-w-[9rem] btn bg-surface0 text-maroon hover:bg-maroon/10 border border-maroon/30"
          )}
          disabled={
            busy ||
            (status?.state !== "connected" && status?.state !== "connecting")
          }
          onClick={() => disconnectMut.mutate()}
        >
          <span className="icon text-base">link_off</span>
          {disconnectMut.isPending ? "Disconnecting…" : "Disconnect"}
        </button>
      </div>

      {(connectMut.isError || disconnectMut.isError) && (
        <p className="text-xs text-red max-w-prose" role="alert">
          {connectMut.error instanceof Error
            ? connectMut.error.message
            : disconnectMut.error instanceof Error
              ? disconnectMut.error.message
              : "Request failed"}
        </p>
      )}

      {(profiles?.length ?? 0) > 0 ? (
        <section className="glass-card p-4 flex flex-col gap-2">
          <h3 className="text-sm font-medium text-subtext1">Configured profiles</h3>
          <ul className="flex flex-col gap-1.5">
            {profiles!.map((p) => (
              <li
                key={p.id}
                className={cn(
                  "flex items-center justify-between gap-2 rounded-lg px-2.5 py-2 text-xs",
                  status?.profile_id === p.id ? "bg-teal/10 text-teal" : "text-subtext0"
                )}
              >
                <span className="min-w-0 truncate font-medium">{p.name || p.id}</span>
                <span className="shrink-0 font-mono text-[10px] uppercase">{p.type ?? "vpn"}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <p className="text-xs text-subtext1 max-w-prose">
        Status refreshes every few seconds. Credentials may be prompted or read from the keyring by the sidecar for
        profiles that require them.
      </p>
    </motion.div>
  )
}
