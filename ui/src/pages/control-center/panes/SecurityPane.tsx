import type { ReactNode } from "react"
import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import api, { type SecurityStatusView } from "@/lib/api"
import { getNavItem } from "../navigation"

const FIREWALL_KEYS = new Set(["firewall_enabled", "firewall", "ufw_enabled"])
const SSH_KEYS = new Set(["ssh_enabled", "ssh", "sshd_enabled"])
const ENCRYPTION_KEYS = new Set(["encryption_enabled", "luks_enabled", "encrypted"])

const KEYS_IN_SUMMARY = new Set<string>([
  ...FIREWALL_KEYS,
  ...SSH_KEYS,
  ...ENCRYPTION_KEYS,
])

function humanizeKey(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

/** Renders primitives and collections; nested objects fall back to JSON. */
function formatUnknownValue(value: unknown): ReactNode {
  if (value === null || value === undefined) return "—"
  if (typeof value === "boolean") return value ? "Yes" : "No"
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  if (typeof value === "string") return value.length > 0 ? value : "—"
  if (Array.isArray(value)) {
    if (value.length === 0) return "—"
    const primitivesOnly = value.every(
      (x) => x === null || ["string", "number", "boolean"].includes(typeof x)
    )
    if (primitivesOnly) return value.map((x) => (x === null ? "null" : String(x))).join(", ")
    return (
      <pre className="mt-1 text-xs text-subtext0 whitespace-pre-wrap break-all font-mono bg-mantle/60 rounded-lg p-2 max-h-44 overflow-y-auto border border-surface0/50">
        {JSON.stringify(value, null, 2)}
      </pre>
    )
  }
  if (typeof value === "object") {
    return (
      <pre className="mt-1 text-xs text-subtext0 whitespace-pre-wrap break-all font-mono bg-mantle/60 rounded-lg p-2 max-h-44 overflow-y-auto border border-surface0/50">
        {JSON.stringify(value, null, 2)}
      </pre>
    )
  }
  return String(value)
}

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return v !== null && typeof v === "object" && !Array.isArray(v)
}

function Row({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-4 text-sm">
      <span className="text-subtext0 shrink-0">{label}</span>
      <div className="text-text text-right font-medium break-all min-w-0">{value}</div>
    </div>
  )
}

function SectionCard({ title, icon, children, footer }: { title: string; icon: string; children: ReactNode; footer?: ReactNode }) {
  return (
    <div className="glass-card p-4 flex flex-col gap-3">
      <div className="flex items-center gap-2 border-b border-surface0/50 pb-2">
        <span className="icon text-mauve text-lg">{icon}</span>
        <h3 className="text-sm font-semibold text-text">{title}</h3>
      </div>
      <div className="flex flex-col gap-2">{children}</div>
      {footer ? <div className="pt-1 border-t border-surface0/40">{footer}</div> : null}
    </div>
  )
}

function boolPhrase(v: boolean | undefined): ReactNode {
  if (v === undefined) return <span className="text-subtext1">Unknown</span>
  return v ? <span className="text-green">On</span> : <span className="text-subtext0">Off</span>
}

function renderExtraSection(key: string, value: unknown): ReactNode {
  if (isPlainObject(value)) {
    const entries = Object.entries(value)
    if (entries.length === 0) {
      return <p className="text-xs text-subtext0">Empty</p>
    }
    return entries.map(([k, v]) => <Row key={k} label={humanizeKey(k)} value={formatUnknownValue(v)} />)
  }
  return <Row label="Value" value={formatUnknownValue(value)} />
}

export function SecurityPane() {
  const { icon, label } = getNavItem("security")
  const qc = useQueryClient()

  const { data, isLoading, isError, error } = useQuery({
    queryKey: ["control-center", "security-status"],
    queryFn: api.getSecurityStatus,
    refetchInterval: 60_000,
  })

  const fingerprintsQuery = useQuery({
    queryKey: ["control-center", "fingerprints"],
    queryFn: api.listFingerprints,
    refetchInterval: 120_000,
  })

  const clamMut = useMutation({
    mutationFn: () => api.runClamScan(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["control-center", "security-status"] }),
  })

  const firewallEnableMut = useMutation({
    mutationFn: () => api.enableFirewall(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["control-center", "security-status"] }),
  })

  const firewallDisableMut = useMutation({
    mutationFn: () => api.disableFirewall(),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["control-center", "security-status"] }),
  })

  const lockMut = useMutation({
    mutationFn: () => api.sessionLock(),
  })

  const raw = pickRaw(data)
  const actionBusy =
    clamMut.isPending || firewallEnableMut.isPending || firewallDisableMut.isPending || lockMut.isPending
  const actionError =
    (clamMut.error ?? firewallEnableMut.error ?? firewallDisableMut.error ?? lockMut.error) instanceof Error
      ? (clamMut.error ?? firewallEnableMut.error ?? firewallDisableMut.error ?? lockMut.error)?.message
      : null

  const extraKeys =
    raw != null
      ? Object.keys(raw)
          .filter((k) => !KEYS_IN_SUMMARY.has(k) && k !== "raw")
          .sort((a, b) => a.localeCompare(b))
      : []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div>
        <div className="flex items-center gap-3 mb-1">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-xs text-subtext1 max-w-prose leading-relaxed">
          Live snapshot from <code className="text-subtext0">Security.GetStatus</code>. Firewall, session lock, AV scan, and fingerprint enrollment status.
        </p>
      </div>

      {isLoading ? (
        <div className="flex flex-col gap-4">
          <div className="skeleton h-32 rounded-xl" />
          <div className="skeleton h-24 rounded-xl" />
        </div>
      ) : isError ? (
        <p className="text-sm text-red">{error instanceof Error ? error.message : "Failed to load security status"}</p>
      ) : (
        <div className="flex flex-col gap-4">
          <SectionCard title="Protections overview" icon="shield" footer={<p className="text-[10px] text-subtext1">Firewall / SSH / encryption flags when present</p>}>
            <Row label="Firewall" value={boolPhrase(data?.firewall_enabled)} />
            <Row label="SSH server" value={boolPhrase(data?.ssh_enabled)} />
            <Row label="Encryption (disk)" value={boolPhrase(data?.encryption_enabled)} />
          </SectionCard>

          <SectionCard title="Fingerprints" icon="fingerprint" footer={<p className="text-[10px] text-subtext1">Security.ListFingerprints</p>}>
            {fingerprintsQuery.isLoading ? (
              <div className="skeleton h-10 rounded-lg" />
            ) : fingerprintsQuery.isError ? (
              <p className="text-xs text-red">
                {fingerprintsQuery.error instanceof Error
                  ? fingerprintsQuery.error.message
                  : "Could not list fingerprints"}
              </p>
            ) : (fingerprintsQuery.data?.length ?? 0) === 0 ? (
              <p className="text-xs text-subtext0">No enrolled fingerprints or fprintd unavailable</p>
            ) : (
              <ul className="flex flex-col gap-1 text-xs text-subtext1">
                {fingerprintsQuery.data!.map((fp) => (
                  <li key={fp.name}>
                    {fp.name}
                    {fp.finger ? ` · ${fp.finger}` : ""}
                  </li>
                ))}
              </ul>
            )}
          </SectionCard>

          {extraKeys.length === 0 ? (
            <p className="text-xs text-subtext0">No extra fields beyond the overview — sidecar aggregate may grow over time.</p>
          ) : (
            extraKeys.map((key) => (
              <SectionCard key={key} title={humanizeKey(key)} icon="tune">
                {renderExtraSection(key, raw![key])}
              </SectionCard>
            ))
          )}
          <div className="flex flex-wrap gap-2 pt-2">
            <button
              type="button"
              className="btn-surface text-xs"
              disabled={actionBusy}
              onClick={() => lockMut.mutate()}
            >
              {lockMut.isPending ? "Locking…" : "Lock session"}
            </button>
            <button
              type="button"
              className="btn-surface text-xs"
              disabled={actionBusy}
              onClick={() => clamMut.mutate()}
            >
              {clamMut.isPending ? "Scanning…" : "Run ClamAV scan"}
            </button>
            <button
              type="button"
              className="btn-surface text-xs"
              disabled={actionBusy}
              onClick={() => firewallEnableMut.mutate()}
            >
              {firewallEnableMut.isPending ? "Enabling…" : "Enable firewall"}
            </button>
            <button
              type="button"
              className="btn-surface text-xs"
              disabled={actionBusy}
              onClick={() => firewallDisableMut.mutate()}
            >
              {firewallDisableMut.isPending ? "Disabling…" : "Disable firewall"}
            </button>
          </div>
          {actionError ? <p className="text-xs text-red">{actionError}</p> : null}
          {lockMut.isSuccess && !lockMut.isPending ? (
            <p className="text-xs text-green">Lock dispatched via Session.Lock</p>
          ) : null}
        </div>
      )}
    </motion.div>
  )
}

function pickRaw(view: SecurityStatusView | undefined): Record<string, unknown> | undefined {
  const r = view?.raw
  return r != null && isPlainObject(r) ? r : undefined
}
