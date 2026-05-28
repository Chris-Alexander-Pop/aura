/**
 * Shared API response types and safe parsers aligned with sidecar JSON (serde snake_case).
 * @see sidecar/src/services/calendar.rs — CalendarEvent, Calendar
 */

// ── JSON guards ───────────────────────────────────────────────────────────────

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function optionalString(value: unknown): string | undefined {
  if (value === null || value === undefined) return undefined
  return typeof value === "string" ? value : undefined
}

function optionalFiniteInt(value: unknown): number | undefined {
  if (value === null || value === undefined) return undefined
  if (typeof value === "number" && Number.isFinite(value)) return Math.trunc(value)
  return undefined
}

function finiteNumber(value: unknown): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return null
  return value
}

// ── Calendar (matches calendar.rs) ──────────────────────────────────────────

export interface CalendarEvent {
  id: string
  title: string
  /** Instant from sidecar (epoch ms or s — keep consistent with Rust callers). */
  start: number
  end: number
  description: string
  calendar_id?: string
  reminder_minutes?: number
}

export interface Calendar {
  id: string
  name: string
  color: string
}

export function parseCalendarEvent(raw: unknown): CalendarEvent | null {
  if (!isRecord(raw)) return null
  const id = raw.id
  const title = raw.title
  if (typeof id !== "string" || typeof title !== "string") return null
  const start = finiteNumber(raw.start)
  const end = finiteNumber(raw.end)
  if (start === null || end === null) return null
  const description = typeof raw.description === "string" ? raw.description : ""
  const calendar_id = optionalString(raw.calendar_id)
  const reminder_minutes = optionalFiniteInt(raw.reminder_minutes)
  return { id, title, start, end, description, calendar_id, reminder_minutes }
}

export function parseCalendarEvents(raw: unknown): CalendarEvent[] {
  if (!Array.isArray(raw)) return []
  const out: CalendarEvent[] = []
  for (const item of raw) {
    const ev = parseCalendarEvent(item)
    if (ev) out.push(ev)
  }
  return out
}

export function parseCalendar(raw: unknown): Calendar | null {
  if (!isRecord(raw)) return null
  const id = raw.id
  const name = raw.name
  const color = raw.color
  if (typeof id !== "string" || typeof name !== "string" || typeof color !== "string") return null
  return { id, name, color }
}

// ── Performance — unknown aggregate (GetMetrics / evolving sidecar) ─────────

export interface PerformanceMetricsView {
  cpu_percent?: number
  gpu_percent?: number
  memory_percent?: number
  disk_percent?: number
  temperature_c?: number
  /** Preserved object payload when the top-level result is an object */
  raw?: Record<string, unknown>
}

function firstFiniteNumber(obj: Record<string, unknown>, keys: string[]): number | undefined {
  for (const k of keys) {
    const v = obj[k]
    if (typeof v === "number" && Number.isFinite(v)) return v
  }
  return undefined
}

export function adaptPerformanceMetrics(data: unknown): PerformanceMetricsView {
  if (!isRecord(data)) return {}
  return {
    cpu_percent: firstFiniteNumber(data, ["cpu_percent", "cpu", "cpu_usage"]),
    gpu_percent: firstFiniteNumber(data, ["gpu_percent", "gpu", "gpu_usage"]),
    memory_percent: firstFiniteNumber(data, ["memory_percent", "memory", "ram", "ram_percent"]),
    disk_percent: firstFiniteNumber(data, ["disk_percent", "disk", "storage", "storage_percent"]),
    temperature_c: firstFiniteNumber(data, ["temperature_c", "temp", "temperature"]),
    raw: data,
  }
}

// ── Security — unknown aggregate ─────────────────────────────────────────────

export interface SecurityStatusView {
  firewall_enabled?: boolean
  ssh_enabled?: boolean
  encryption_enabled?: boolean
  raw?: Record<string, unknown>
}

function optionalBool(obj: Record<string, unknown>, keys: string[]): boolean | undefined {
  for (const k of keys) {
    const v = obj[k]
    if (typeof v === "boolean") return v
  }
  return undefined
}

export function adaptSecurityStatus(data: unknown): SecurityStatusView {
  if (!isRecord(data)) return {}
  return {
    firewall_enabled: optionalBool(data, ["firewall_enabled", "firewall", "ufw_enabled"]),
    ssh_enabled: optionalBool(data, ["ssh_enabled", "ssh", "sshd_enabled"]),
    encryption_enabled: optionalBool(data, ["encryption_enabled", "luks_enabled", "encrypted"]),
    raw: data,
  }
}

// ── Logs — array of mixed / unknown rows ─────────────────────────────────────

export interface LogEntryView {
  timestamp?: string
  level?: string
  message?: string
  service?: string
  readonly raw: Record<string, unknown>
}

function adaptLogEntry(item: unknown): LogEntryView {
  if (typeof item === "string") {
    return { message: item, raw: {} }
  }
  if (!isRecord(item)) {
    return { raw: {} }
  }
  return {
    timestamp: optionalString(item.timestamp) ?? optionalString(item.time) ?? optionalString(item.date),
    level: optionalString(item.level) ?? optionalString(item.severity) ?? optionalString(item.priority),
    message: optionalString(item.message) ?? optionalString(item.msg) ?? optionalString(item.line),
    service: optionalString(item.service) ?? optionalString(item.unit) ?? optionalString(item.source),
    raw: item,
  }
}

export function adaptLogEntries(data: unknown): LogEntryView[] {
  if (!Array.isArray(data)) return []
  return data.map(adaptLogEntry)
}

// ── Packages — array of unknown update rows ──────────────────────────────────

export interface PackageUpdateView {
  name?: string
  version?: string
  old_version?: string
  repository?: string
  size?: string
  readonly raw: Record<string, unknown>
}

function adaptPackageUpdate(item: unknown): PackageUpdateView {
  if (typeof item === "string") {
    return { name: item, raw: {} }
  }
  if (!isRecord(item)) {
    return { raw: {} }
  }
  return {
    name: optionalString(item.name) ?? optionalString(item.package) ?? optionalString(item.pkg),
    version: optionalString(item.version) ?? optionalString(item.new_version) ?? optionalString(item.to_version),
    old_version: optionalString(item.old_version) ?? optionalString(item.current_version) ?? optionalString(item.from_version),
    repository: optionalString(item.repository) ?? optionalString(item.repo) ?? optionalString(item.source),
    size: optionalString(item.size),
    raw: item,
  }
}

export function adaptPackageUpdates(data: unknown): PackageUpdateView[] {
  if (!Array.isArray(data)) return []
  return data.map(adaptPackageUpdate)
}
