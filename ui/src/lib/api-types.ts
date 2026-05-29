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

// ── Notifications (matches notifications.rs) ───────────────────────────────

export interface NotificationActionView {
  key: string
  label: string
}

export interface NotificationItemView {
  id: number
  server_id?: number
  app_name: string
  summary: string
  body: string
  icon?: string
  urgency: number
  timestamp: number
  actions: NotificationActionView[]
  closed?: boolean
}

export interface DndPrefsView {
  enabled: boolean
  schedule_enabled: boolean
  start_time: string
  end_time: string
  weekdays_only: boolean
}

export interface AppNotificationRulesView {
  muted_apps: string[]
}

function adaptNotificationItem(item: unknown): NotificationItemView | null {
  if (!isRecord(item)) return null
  const id = item.id
  if (typeof id !== "number" || !Number.isFinite(id)) return null
  const actions: NotificationActionView[] = []
  if (Array.isArray(item.actions)) {
    for (const a of item.actions) {
      if (!isRecord(a)) continue
      const key = optionalString(a.key)
      const label = optionalString(a.label)
      if (key && label) actions.push({ key, label })
    }
  }
  return {
    id,
    server_id: optionalFiniteInt(item.server_id),
    app_name: optionalString(item.app_name) ?? "unknown",
    summary: optionalString(item.summary) ?? "",
    body: optionalString(item.body) ?? "",
    icon: optionalString(item.icon),
    urgency: typeof item.urgency === "number" ? item.urgency : 1,
    timestamp: typeof item.timestamp === "number" ? item.timestamp : 0,
    actions,
    closed: item.closed === true,
  }
}

export function adaptNotificationList(data: unknown): NotificationItemView[] {
  if (!Array.isArray(data)) return []
  return data.map(adaptNotificationItem).filter((x): x is NotificationItemView => x != null)
}

// ── Hyprland (matches sidecar Hypr* types, snake_case on wire) ───────────────

export interface HyprWorkspaceRef {
  id: number
  name?: string
}

export interface HyprWorkspace {
  id: number
  name: string
  windows: number
}

export interface HyprClient {
  address: string
  title: string
  class: string
  workspace: HyprWorkspaceRef
  floating: boolean
}

export interface HyprActiveWindow {
  address: string
  title: string
  class: string
  workspace: HyprWorkspaceRef
  floating: boolean
}

export interface HyprActiveWorkspace {
  id: number
  name: string
}

export interface HyprMonitor {
  name: string
  id: number
  active_workspace: HyprWorkspaceRef
}

export interface MediaNowPlaying {
  playing: boolean
  paused?: boolean
  stopped?: boolean
  title: string
  artist: string
  player_name?: string | null
}

function hyprClassName(item: Record<string, unknown>): string {
  const c = item.class ?? item.class_name
  return typeof c === "string" ? c : ""
}

export function parseHyprWorkspaces(raw: unknown): HyprWorkspace[] {
  if (!Array.isArray(raw)) return []
  return raw
    .filter((x): x is Record<string, unknown> => isRecord(x))
    .map((x) => ({
      id: Number(x.id),
      name: typeof x.name === "string" ? x.name : String(x.id ?? ""),
      windows: typeof x.windows === "number" ? x.windows : 0,
    }))
    .filter((x) => Number.isFinite(x.id))
}

export function parseHyprClients(raw: unknown): HyprClient[] {
  if (!Array.isArray(raw)) return []
  return raw
    .filter((x): x is Record<string, unknown> => isRecord(x))
    .map((x) => {
      const ws = isRecord(x.workspace) ? x.workspace : {}
      const id = Number(ws.id)
      return {
        address: typeof x.address === "string" ? x.address : "",
        title: typeof x.title === "string" ? x.title : "",
        class: hyprClassName(x),
        workspace: {
          id: Number.isFinite(id) ? id : -1,
          name: typeof ws.name === "string" ? ws.name : undefined,
        },
        floating: x.floating === true,
      }
    })
    .filter((x) => x.address.length > 0)
}

export function parseHyprActiveWorkspace(raw: unknown): HyprActiveWorkspace | null {
  if (!isRecord(raw)) return null
  const id = Number(raw.id)
  if (!Number.isFinite(id)) return null
  return {
    id,
    name: typeof raw.name === "string" ? raw.name : String(id),
  }
}

export function parseHyprActiveWindow(raw: unknown): HyprActiveWindow | null {
  if (!isRecord(raw)) return null
  const address = typeof raw.address === "string" ? raw.address : ""
  if (!address) return null
  const ws = isRecord(raw.workspace) ? raw.workspace : {}
  const id = Number(ws.id)
  return {
    address,
    title: typeof raw.title === "string" ? raw.title : "",
    class: hyprClassName(raw),
    workspace: {
      id: Number.isFinite(id) ? id : -1,
      name: typeof ws.name === "string" ? ws.name : undefined,
    },
    floating: raw.floating === true,
  }
}

export function adaptDndPrefs(data: unknown): DndPrefsView {
  if (!isRecord(data)) {
    return {
      enabled: false,
      schedule_enabled: false,
      start_time: "22:00",
      end_time: "07:00",
      weekdays_only: true,
    }
  }
  return {
    enabled: data.enabled === true,
    schedule_enabled: data.schedule_enabled === true,
    start_time: optionalString(data.start_time) ?? "22:00",
    end_time: optionalString(data.end_time) ?? "07:00",
    weekdays_only: data.weekdays_only !== false,
  }
}
