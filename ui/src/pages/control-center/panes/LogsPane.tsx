import { useMemo, useState } from "react"
import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

type SeverityBucket = "err" | "warn" | "info" | "debug" | "other"

type LogRow = {
  id: string
  display: string
  bucket: SeverityBucket
}

const FILTERS: Array<{ id: SeverityBucket | "all"; label: string }> = [
  { id: "all", label: "All" },
  { id: "err", label: "Errors" },
  { id: "warn", label: "Warnings" },
  { id: "info", label: "Info" },
  { id: "debug", label: "Debug" },
]

function normalizeLevelToken(raw: string): SeverityBucket {
  const s = raw.trim().toLowerCase()
  if (["emerg", "alert", "crit", "critical", "fatal", "error", "err"].includes(s)) return "err"
  if (["warning", "warn"].includes(s)) return "warn"
  if (["notice", "info"].includes(s)) return "info"
  if (s === "debug") return "debug"
  return "other"
}

function bucketFromLine(text: string): SeverityBucket {
  const upper = text.toUpperCase()
  if (/\b(ERR|ERROR|CRIT|CRITICAL|FATAL|EMERG|ALERT)\b/.test(upper)) return "err"
  if (/\b(WARN|WARNING)\b/.test(upper)) return "warn"
  if (/\b(DEBUG)\b/.test(upper)) return "debug"
  if (/\b(INFO|NOTICE)\b/.test(upper)) return "info"
  return "other"
}

function pushLines(target: LogRow[], lines: string[], keyPrefix: string) {
  lines.forEach((line, j) => {
    const trimmed = line.trim()
    if (!trimmed) return
    target.push({
      id: `${keyPrefix}:${j}`,
      display: trimmed,
      bucket: bucketFromLine(trimmed),
    })
  })
}

function coerceLogs(raw: unknown): LogRow[] {
  if (!Array.isArray(raw)) return []

  const rows: LogRow[] = []
  raw.forEach((item, i) => {
    const prefix = `${i}`

    if (typeof item === "string") {
      pushLines(rows, item.split("\n"), prefix)
      return
    }

    if (!item || typeof item !== "object") return
    const o = item as Record<string, unknown>

    if (typeof o.logs === "string") {
      pushLines(rows, o.logs.split("\n"), `${prefix}:logs`)
      return
    }

    if (typeof o.message === "string") {
      const levelRaw = typeof o.level === "string" ? o.level : ""
      const bucket = levelRaw ? normalizeLevelToken(levelRaw) : bucketFromLine(o.message)
      const ts = typeof o.timestamp === "string" ? o.timestamp : ""
      const svc = typeof o.service === "string" ? o.service : ""
      const head = [ts, svc].filter(Boolean).join(" · ")
      const display = head ? `${head} — ${o.message}` : o.message
      rows.push({ id: `${prefix}:msg`, display, bucket })
      return
    }

    try {
      rows.push({
        id: `${prefix}:json`,
        display: JSON.stringify(item),
        bucket: "other",
      })
    } catch {
      rows.push({ id: `${prefix}:str`, display: String(item), bucket: "other" })
    }
  })

  return rows
}

function bucketColor(bucket: SeverityBucket): string {
  switch (bucket) {
    case "err":
      return "text-red"
    case "warn":
      return "text-peach"
    case "info":
      return "text-blue"
    case "debug":
      return "text-subtext0"
    default:
      return "text-subtext1"
  }
}

export function LogsPane() {
  const { icon, label } = getNavItem("logs")
  const [filter, setFilter] = useState<SeverityBucket | "all">("all")

  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["logs"],
    queryFn: () => api.getLogs({ lines: 200 }),
    refetchInterval: 30_000,
  })

  const rows = useMemo(() => coerceLogs(data ?? []), [data])

  const visible = useMemo(
    () => (filter === "all" ? rows : rows.filter((r) => r.bucket === filter)),
    [rows, filter]
  )

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-4 p-6 h-full min-h-0"
    >
      <div className="flex flex-wrap items-start justify-between gap-3 shrink-0">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose">
            Recent entries from the sidecar. Filter by inferred severity.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-xs shrink-0"
          onClick={() => refetch()}
          disabled={isFetching}
        >
          <span className={cn("icon text-base", isFetching && "animate-spin")}>refresh</span>
          Refresh
        </button>
      </div>

      <div className="flex flex-wrap gap-2 shrink-0">
        {FILTERS.map((f) => {
          const count =
            f.id === "all" ? rows.length : rows.filter((r) => r.bucket === f.id).length
          return (
            <button
              key={f.id}
              type="button"
              onClick={() => setFilter(f.id)}
              className={cn("toggle-chip text-xs", filter === f.id && "active")}
            >
              {f.label}
              <span className="text-subtext0 ml-1 tabular-nums">({count})</span>
            </button>
          )
        })}
      </div>

      {isError && (
        <div className="glass-card border-red/30 p-4 text-sm text-red shrink-0">
          {error instanceof Error ? error.message : "Could not load logs."}
        </div>
      )}

      <div className="flex-1 min-h-0 glass-card overflow-hidden flex flex-col">
        {isLoading ? (
          <div className="p-4 space-y-2 overflow-y-auto">
            {[...Array(8)].map((_, i) => (
              <div key={i} className="skeleton h-8 rounded-md" style={{ opacity: 1 - i * 0.08 }} />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-3 py-16 px-6 text-center flex-1">
            <span className="icon text-5xl text-subtext0/40">inventory_2</span>
            <div>
              <p className="text-sm font-medium text-text">
                {rows.length === 0 ? "No log entries yet" : "No entries match this filter"}
              </p>
              <p className="text-xs text-subtext1 mt-1 max-w-xs">
                {rows.length === 0
                  ? "When the sidecar returns log lines, they will appear here."
                  : "Try another filter or refresh to fetch new lines."}
              </p>
            </div>
          </div>
        ) : (
          <ul className="overflow-y-auto flex-1 min-h-0 divide-y divide-surface0/50">
            {visible.map((row) => (
              <li key={row.id} className="px-3 py-2 font-mono text-[11px] leading-relaxed">
                <span className={cn("select-none mr-2 uppercase text-[10px]", bucketColor(row.bucket))}>
                  {row.bucket}
                </span>
                <span className="text-subtext0 break-words">{row.display}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </motion.div>
  )
}
