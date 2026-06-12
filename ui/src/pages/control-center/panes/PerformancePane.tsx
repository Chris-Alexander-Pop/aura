import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo, useState } from "react"
import api, { type PowerProfile } from "@/lib/api"
import { cn } from "@/lib/utils"

const POWER_LABELS: Record<PowerProfile, string> = {
  performance: "Performance",
  balanced: "Balanced",
  saver: "Power saver",
}

// ── Safe numeric / record guards (no @/lib/api-types in tree) ─────────────────

function num(v: unknown): number | undefined {
  if (typeof v === "number" && Number.isFinite(v)) return v
  if (typeof v === "string" && v.trim() !== "") {
    const x = Number(v)
    if (Number.isFinite(x)) return x
  }
  return undefined
}

function str(v: unknown): string | undefined {
  return typeof v === "string" ? v : undefined
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return v !== null && typeof v === "object" && !Array.isArray(v)
}

function parseSystemStats(raw: unknown): {
  cpuPct: number
  ramPct: number
  tempC: number | null
  gpuPct: number | null
  storagePct: number | null
} | null {
  if (!isRecord(raw)) return null
  const cpu = num(raw.cpu)
  const ram = num(raw.ram)
  if (cpu == null || ram == null) return null
  const cpuPct = cpu <= 1 ? cpu * 100 : cpu
  const ramPct = ram <= 1 ? ram * 100 : ram
  const t = num(raw.temp)
  const tempC = t != null && Number.isFinite(t) ? t : null
  const gpu = num(raw.gpu)
  const gpuPct = gpu != null ? (gpu <= 1 ? gpu * 100 : gpu) : null
  const storage = num(raw.storage)
  const storagePct = storage != null ? (storage <= 1 ? storage * 100 : storage) : null
  return { cpuPct, ramPct, tempC, gpuPct, storagePct }
}

type CoreRow = { id: number; usage: number; mhz?: number; governor?: string }

function parseCores(data: Record<string, unknown>): CoreRow[] {
  const raw = data.cores ?? data.cpu_cores
  if (!Array.isArray(raw)) return []
  const out: CoreRow[] = []
  for (const item of raw) {
    if (!isRecord(item)) continue
    const id = num(item.core_id) ?? out.length
    const usage = num(item.usage_percent) ?? num(item.cpu_usage) ?? num(item.usage) ?? 0
    const mhz = num(item.frequency_mhz)
    const governor = str(item.governor)
    const row: CoreRow = { id, usage: Math.min(Math.max(usage, 0), 100) }
    if (mhz != null) row.mhz = mhz
    if (governor != null) row.governor = governor
    out.push(row)
  }
  return out
}

type MemDetail = { usedMb: number; totalMb: number; swapUsedMb?: number; swapTotalMb?: number }

function parseMemoryDetail(data: Record<string, unknown>): MemDetail | null {
  const nested = data.memory ?? data.memory_stats
  const src = nested && isRecord(nested) ? nested : data
  const totalMb = num(src.total_mb)
  const usedMb = num(src.used_mb)
  if (totalMb == null || usedMb == null || totalMb <= 0) return null
  const swapU = num(src.swap_used_mb)
  const swapT = num(src.swap_total_mb)
  const d: MemDetail = { usedMb, totalMb }
  if (swapU != null) d.swapUsedMb = swapU
  if (swapT != null) d.swapTotalMb = swapT
  return d
}

type GpuRow = { name: string; util: number; memUsed?: number; memTotal?: number; temp?: number }

function parseGpus(data: Record<string, unknown>): GpuRow[] {
  const raw = data.gpus ?? data.gpu_stats ?? data.gpu
  const list: unknown[] = Array.isArray(raw) ? raw : raw !== undefined && raw !== null ? [raw] : []
  const out: GpuRow[] = []
  for (const item of list) {
    if (!isRecord(item)) continue
    const name = str(item.name) ?? "GPU"
    const util = num(item.utilization_percent) ?? num(item.utilization) ?? num(item.util) ?? 0
    const row: GpuRow = { name, util: Math.min(Math.max(util, 0), 100) }
    const mu = num(item.memory_used_mb)
    const mt = num(item.memory_total_mb)
    const te = num(item.temperature_c) ?? num(item.temp)
    if (mu != null) row.memUsed = mu
    if (mt != null) row.memTotal = mt
    if (te != null) row.temp = te
    out.push(row)
  }
  return out
}

type DiskRow = { device: string; mount: string; usedGb: number; totalGb: number }

function parseDisks(data: Record<string, unknown>): DiskRow[] {
  const raw = data.disks ?? data.disk_stats
  if (!Array.isArray(raw)) return []
  const out: DiskRow[] = []
  for (const item of raw) {
    if (!isRecord(item)) continue
    const device = str(item.device) ?? "—"
    const mount = str(item.mount_point) ?? str(item.mount) ?? "—"
    const usedGb = num(item.used_gb) ?? 0
    const totalGb = num(item.total_gb) ?? 0
    if (totalGb > 0) out.push({ device, mount, usedGb, totalGb })
  }
  return out
}

type NetRow = { iface: string; rx: number; tx: number }

function parseNetwork(data: Record<string, unknown>): NetRow[] {
  const raw = data.network ?? data.network_stats ?? data.interfaces
  if (!Array.isArray(raw)) return []
  const out: NetRow[] = []
  for (const item of raw) {
    if (!isRecord(item)) continue
    const iface = str(item.interface) ?? str(item.iface) ?? "—"
    const rx = num(item.rx_bytes_per_sec) ?? num(item.rx) ?? 0
    const tx = num(item.tx_bytes_per_sec) ?? num(item.tx) ?? 0
    out.push({ iface, rx, tx })
  }
  return out
}

function parsePerformanceRecord(raw: unknown): Record<string, unknown> | null {
  if (isRecord(raw)) return raw
  if (Array.isArray(raw)) return { cores: raw }
  return null
}

function useRollingHistory(max: number, value: number | null | undefined) {
  const [hist, setHist] = useState<number[]>([])
  useEffect(() => {
    if (value == null || !Number.isFinite(value)) return
    setHist((h) => [...h.slice(-(max - 1)), value])
  }, [max, value])
  return hist
}

function Sparkline({
  values,
  colorClass,
}: {
  values: number[]
  colorClass: string
}) {
  const w = 128
  const h = 40
  const pad = 2
  if (values.length < 2) {
    return <div className="h-10 w-full rounded-md bg-surface0/40 border border-surface1/50" />
  }
  const pts = values.map((v, i) => {
    const x = pad + (i / (values.length - 1)) * (w - pad * 2)
    const clamped = Math.min(Math.max(v, 0), 100)
    const y = pad + (h - pad * 2) * (1 - clamped / 100)
    return `${x},${y}`
  })
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className="w-full h-10"
      preserveAspectRatio="none"
      aria-hidden
    >
      <polyline
        fill="none"
        stroke="currentColor"
        strokeWidth="1.25"
        strokeLinejoin="round"
        strokeLinecap="round"
        points={pts.join(" ")}
        className={colorClass}
      />
    </svg>
  )
}

function MiniGauge({
  label,
  valuePct,
  sub,
  gradient,
}: {
  label: string
  valuePct: number
  sub: string
  gradient: string
}) {
  const pct = Math.min(Math.max(valuePct, 0), 100)
  return (
    <div className="flex flex-col gap-2 min-w-0">
      <div className="flex justify-between text-[11px] text-subtext0">
        <span>{label}</span>
        <span className="text-text tabular-nums">{sub}</span>
      </div>
      <div className="progress-bar">
        <motion.div
          className={cn("progress-fill bg-gradient-to-r", gradient)}
          initial={{ width: 0 }}
          animate={{ width: `${pct}%` }}
          transition={{ duration: 0.35 }}
        />
      </div>
    </div>
  )
}

export function PerformancePane() {
  const qc = useQueryClient()

  const statsQuery = useQuery({
    queryKey: ["system-stats", "performance-pane"],
    queryFn: api.getSystemStats,
    refetchInterval: 3500,
  })

  const metricsQuery = useQuery({
    queryKey: ["performance-metrics"],
    queryFn: api.getPerformanceMetrics,
    refetchInterval: 5000,
    retry: 1,
  })

  const processesQuery = useQuery({
    queryKey: ["process-top"],
    queryFn: () => api.processListTop(12),
    refetchInterval: 4000,
  })

  const powerProfileQuery = useQuery({
    queryKey: ["performance", "power-profile"],
    queryFn: api.getPowerProfile,
    refetchInterval: 15_000,
  })

  const powerMut = useMutation({
    mutationFn: (profile: PowerProfile) => api.setPowerProfile(profile),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["performance", "power-profile"] }),
  })

  const killMut = useMutation({
    mutationFn: (pid: number) => api.processKill(pid),
    onSuccess: () => void processesQuery.refetch(),
  })

  const parsedStats = useMemo(() => parseSystemStats(statsQuery.data ?? null), [statsQuery.data])

  const perfRec = useMemo(
    () => parsePerformanceRecord(metricsQuery.data ?? null),
    [metricsQuery.data]
  )

  const cores = useMemo(() => (perfRec ? parseCores(perfRec) : []), [perfRec])
  const memDetail = useMemo(() => (perfRec ? parseMemoryDetail(perfRec) : null), [perfRec])
  const gpus = useMemo(() => (perfRec ? parseGpus(perfRec) : []), [perfRec])
  const disks = useMemo(() => (perfRec ? parseDisks(perfRec) : []).slice(0, 5), [perfRec])
  const nets = useMemo(() => (perfRec ? parseNetwork(perfRec) : []).slice(0, 4), [perfRec])

  const cpuHist = useRollingHistory(32, parsedStats?.cpuPct)
  const ramHist = useRollingHistory(32, parsedStats?.ramPct)

  const tempBarPct = parsedStats?.tempC != null && parsedStats.tempC > 0
    ? Math.min((parsedStats.tempC / 95) * 100, 100)
    : 0

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-5 p-6 h-full overflow-y-auto"
    >
      <div>
        <div className="flex items-center gap-3 mb-1">
          <span className="icon text-mauve text-2xl">speed</span>
          <h2 className="text-xl font-semibold text-text">Performance</h2>
        </div>
        <p className="text-xs text-subtext1 max-w-prose">
          Live system snapshot from the sidecar — overview plus detailed metrics when available.
        </p>
      </div>

      {metricsQuery.isError && (
        <div className="rounded-xl border border-yellow/30 bg-yellow/10 px-3 py-2 text-xs text-subtext0">
          Extended performance metrics are unavailable (RPC may be missing upstream). System overview
          below still updates.
        </div>
      )}

      <section className="glass-card p-4 flex flex-col gap-3">
        <h3 className="text-sm font-medium text-text flex items-center gap-2">
          <span className="icon text-base text-peach">bolt</span>
          Power profile
        </h3>
        {powerProfileQuery.isLoading ? (
          <div className="skeleton h-8 rounded-lg" />
        ) : powerProfileQuery.isError ? (
          <p className="text-xs text-red">
            {powerProfileQuery.error instanceof Error
              ? powerProfileQuery.error.message
              : "Could not load power profile"}
          </p>
        ) : (
          <div className="flex flex-wrap gap-2">
            {(Object.keys(POWER_LABELS) as PowerProfile[]).map((p) => (
              <button
                key={p}
                type="button"
                disabled={powerMut.isPending}
                onClick={() => powerMut.mutate(p)}
                className={cn(
                  "toggle-chip text-xs",
                  powerProfileQuery.data?.profile === p && "active"
                )}
              >
                {POWER_LABELS[p]}
              </button>
            ))}
          </div>
        )}
        {powerMut.isError && (
          <p className="text-xs text-red">
            {powerMut.error instanceof Error ? powerMut.error.message : "Could not set profile"}
          </p>
        )}
      </section>

      <section className="glass-card p-4 flex flex-col gap-4">
        <h3 className="text-sm font-medium text-text flex items-center gap-2">
          <span className="icon text-base text-blue">monitor_heart</span>
          System overview
        </h3>
        {statsQuery.isLoading && !parsedStats ? (
          <div className="skeleton h-24 rounded-lg" />
        ) : !parsedStats ? (
          <p className="text-sm text-subtext0">Could not read system stats.</p>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="flex flex-col gap-2 min-w-0">
              <MiniGauge
                label="CPU"
                valuePct={parsedStats.cpuPct}
                sub={`${parsedStats.cpuPct.toFixed(0)}%`}
                gradient="from-blue to-sapphire"
              />
              <Sparkline values={cpuHist} colorClass="text-blue" />
            </div>
            <div className="flex flex-col gap-2 min-w-0">
              <MiniGauge
                label="RAM"
                valuePct={parsedStats.ramPct}
                sub={`${parsedStats.ramPct.toFixed(0)}%`}
                gradient="from-mauve to-pink"
              />
              <Sparkline values={ramHist} colorClass="text-mauve" />
            </div>
            <div className="flex flex-col gap-2 min-w-0">
              <MiniGauge
                label="Temp"
                valuePct={tempBarPct}
                sub={
                  parsedStats.tempC != null && parsedStats.tempC > 0
                    ? `${parsedStats.tempC.toFixed(0)}°C`
                    : "—"
                }
                gradient="from-peach to-maroon"
              />
              <div className="h-10 rounded-md bg-surface0/30 border border-surface1/40 flex items-center justify-center text-[10px] text-subtext1">
                vs 95°C reference
              </div>
            </div>
          </div>
        )}

        {parsedStats?.gpuPct != null && (
          <MiniGauge
            label="GPU (system aggregate)"
            valuePct={parsedStats.gpuPct}
            sub={`${parsedStats.gpuPct.toFixed(0)}%`}
            gradient="from-teal to-green"
          />
        )}
        {parsedStats?.storagePct != null && (
          <MiniGauge
            label="Storage (block devices)"
            valuePct={parsedStats.storagePct}
            sub={`${parsedStats.storagePct.toFixed(0)}%`}
            gradient="from-subtext0 to-text"
          />
        )}
      </section>

      {cores.length > 0 && (
        <section className="glass-card p-4 flex flex-col gap-3">
          <h3 className="text-sm font-medium text-text">CPU cores</h3>
          <div className="flex flex-wrap gap-1.5">
            {cores.map((c) => (
              <div
                key={c.id}
                className="flex flex-col items-center gap-1 w-8"
                title={
                  c.governor
                    ? `Core ${c.id} · ${c.usage.toFixed(0)}% · ${c.mhz ?? "—"} MHz · ${c.governor}`
                    : `Core ${c.id} · ${c.usage.toFixed(0)}%`
                }
              >
                <div className="h-14 w-3 rounded-full bg-surface0 overflow-hidden flex flex-col justify-end">
                  <motion.div
                    className="w-full bg-gradient-to-t from-blue to-sapphire rounded-full"
                    initial={{ height: 0 }}
                    animate={{ height: `${Math.min(Math.max(c.usage, 0), 100)}%` }}
                    transition={{ duration: 0.25 }}
                  />
                </div>
                <span className="text-[9px] text-subtext1 tabular-nums">{c.id}</span>
              </div>
            ))}
          </div>
        </section>
      )}

      {memDetail && (
        <section className="glass-card p-4 flex flex-col gap-3">
          <h3 className="text-sm font-medium text-text">Memory detail</h3>
          <MiniGauge
            label="RAM"
            valuePct={(memDetail.usedMb / memDetail.totalMb) * 100}
            sub={`${memDetail.usedMb} / ${memDetail.totalMb} MiB`}
            gradient="from-mauve to-pink"
          />
          {memDetail.swapTotalMb != null &&
            memDetail.swapTotalMb > 0 &&
            memDetail.swapUsedMb != null && (
              <MiniGauge
                label="Swap"
                valuePct={(memDetail.swapUsedMb / memDetail.swapTotalMb) * 100}
                sub={`${memDetail.swapUsedMb} / ${memDetail.swapTotalMb} MiB`}
                gradient="from-subtext0/80 to-subtext1/80"
              />
            )}
        </section>
      )}

      {gpus.length > 0 && (
        <section className="glass-card p-4 flex flex-col gap-3">
          <h3 className="text-sm font-medium text-text">GPUs</h3>
          <div className="flex flex-col gap-3">
            {gpus.map((g) => (
              <div key={g.name} className="flex flex-col gap-2">
                <div className="flex justify-between text-xs text-subtext0">
                  <span className="truncate font-medium text-text">{g.name}</span>
                  <span className="tabular-nums">
                    {g.temp != null ? `${g.temp.toFixed(0)}°C · ` : ""}
                    {g.util.toFixed(0)}%
                  </span>
                </div>
                <div className="progress-bar">
                  <motion.div
                    className="progress-fill bg-gradient-to-r from-teal to-green"
                    initial={{ width: 0 }}
                    animate={{ width: `${Math.min(Math.max(g.util, 0), 100)}%` }}
                    transition={{ duration: 0.35 }}
                  />
                </div>
                {g.memTotal != null && g.memUsed != null && (
                  <p className="text-[10px] text-subtext1 tabular-nums">
                    VRAM {g.memUsed} / {g.memTotal} MiB
                  </p>
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      {disks.length > 0 && (
        <section className="glass-card p-4 flex flex-col gap-3">
          <h3 className="text-sm font-medium text-text">Mounts</h3>
          <div className="flex flex-col gap-3">
            {disks.map((d) => {
              const pct = d.totalGb > 0 ? (d.usedGb / d.totalGb) * 100 : 0
              return (
                <div key={`${d.device}-${d.mount}`} className="flex flex-col gap-1.5">
                  <div className="flex justify-between text-[11px] text-subtext0 gap-2 min-w-0">
                    <span className="truncate" title={d.mount}>
                      {d.mount}
                    </span>
                    <span className="tabular-nums shrink-0">
                      {d.usedGb.toFixed(1)} / {d.totalGb.toFixed(1)} GB
                    </span>
                  </div>
                  <div className="progress-bar">
                    <motion.div
                      className="progress-fill bg-gradient-to-r from-sapphire to-blue"
                      initial={{ width: 0 }}
                      animate={{ width: `${Math.min(Math.max(pct, 0), 100)}%` }}
                      transition={{ duration: 0.35 }}
                    />
                  </div>
                </div>
              )
            })}
          </div>
        </section>
      )}

      {nets.length > 0 && (
        <section className="glass-card p-4 flex flex-col gap-3">
          <h3 className="text-sm font-medium text-text">Interfaces</h3>
          <ul className="text-xs text-subtext0 flex flex-col gap-2">
            {nets.map((n) => (
              <li key={n.iface} className="flex justify-between gap-2">
                <span className="font-medium text-text">{n.iface}</span>
                <span className="tabular-nums text-[11px]">
                  RX {n.rx.toLocaleString()} · TX {n.tx.toLocaleString()}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="glass-card p-4 flex flex-col gap-3">
        <h3 className="text-sm font-medium text-text">Top processes</h3>
        {processesQuery.isLoading ? (
          <p className="text-xs text-subtext0">Loading…</p>
        ) : processesQuery.isError ? (
          <p className="text-xs text-red">
            {processesQuery.error instanceof Error
              ? processesQuery.error.message
              : "Could not load processes"}
          </p>
        ) : (processesQuery.data?.length ?? 0) === 0 ? (
          <p className="text-xs text-subtext0">No process data</p>
        ) : (
          <ul className="flex flex-col gap-1 text-xs">
            {processesQuery.data!.map((p) => (
              <li key={p.pid} className="flex items-center justify-between gap-2">
                <span className="truncate text-text">{p.name}</span>
                <span className="shrink-0 tabular-nums text-subtext1">
                  {p.cpu.toFixed(1)}% · {p.pid}
                </span>
                <button
                  type="button"
                  className="toggle-chip text-[10px] text-red"
                  title="Kill process"
                  disabled={killMut.isPending}
                  onClick={() => killMut.mutate(p.pid)}
                >
                  {killMut.isPending ? "…" : "kill"}
                </button>
              </li>
            ))}
          </ul>
        )}
        {killMut.isError && (
          <p className="text-xs text-red">
            {killMut.error instanceof Error ? killMut.error.message : "Kill failed"}
          </p>
        )}
      </section>
    </motion.div>
  )
}
