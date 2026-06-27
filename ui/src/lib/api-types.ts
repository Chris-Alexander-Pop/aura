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

/** Batched bar state from `Hyprland.GetBarSnapshot`. */
export interface HyprBarSnapshot {
  workspaces: HyprWorkspace[]
  active_workspace: HyprActiveWorkspace | null
  clients: HyprClient[]
  active_window: HyprActiveWindow | null
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

// ── Aura settings (matches settings.rs) ─────────────────────────────────────

export type ModuleHubTriggerMode = "left_edge" | "top_third" | "none"

export interface AuraSettingsView {
  bar_section_order: string[]
  cc_enabled_panes: string[]
  theme: string
  dropdown_modules: string[]
  module_hub_trigger: ModuleHubTriggerMode
  hide_shell_on_fullscreen: boolean
}

const MODULE_HUB_TRIGGER_MODES = new Set<ModuleHubTriggerMode>([
  "left_edge",
  "top_third",
  "none",
])

export function parseAuraSettings(raw: unknown): AuraSettingsView | null {
  if (!isRecord(raw)) return null
  const bar = raw.bar_section_order
  const cc = raw.cc_enabled_panes
  const theme = raw.theme
  const dropdown = raw.dropdown_modules
  if (!Array.isArray(bar) || !Array.isArray(cc) || typeof theme !== "string") return null
  if (!Array.isArray(dropdown)) return null
  if (!bar.every((x) => typeof x === "string") || !cc.every((x) => typeof x === "string")) {
    return null
  }
  if (!dropdown.every((x) => typeof x === "string")) return null
  const triggerRaw = raw.module_hub_trigger
  const module_hub_trigger =
    typeof triggerRaw === "string" && MODULE_HUB_TRIGGER_MODES.has(triggerRaw as ModuleHubTriggerMode)
      ? (triggerRaw as ModuleHubTriggerMode)
      : "left_edge"
  const hideRaw = raw.hide_shell_on_fullscreen
  const hide_shell_on_fullscreen = typeof hideRaw === "boolean" ? hideRaw : true
  return {
    bar_section_order: bar as string[],
    cc_enabled_panes: cc as string[],
    theme,
    dropdown_modules: dropdown as string[],
    module_hub_trigger,
    hide_shell_on_fullscreen,
  }
}

// ── Launcher (matches launcher.rs) ───────────────────────────────────────────

export interface LauncherAppView {
  id: string
  name: string
  pinned: boolean
  score?: number
  comment?: string
  exec?: string
  icon?: string
  source?: "vicinae" | "desktop"
}

export interface LauncherQueryView {
  results: LauncherAppView[]
}

export interface LauncherRecentView {
  items: LauncherAppView[]
}

export function parseLauncherApp(raw: unknown): LauncherAppView | null {
  if (!isRecord(raw)) return null
  const id = raw.id
  const name = raw.name
  if (typeof id !== "string" || typeof name !== "string") return null
  return {
    id,
    name,
    pinned: raw.pinned === true,
    score: optionalFiniteInt(raw.score),
    comment: optionalString(raw.comment),
    exec: optionalString(raw.exec),
    icon: optionalString(raw.icon),
    source:
      raw.source === "vicinae" || raw.source === "desktop"
        ? raw.source
        : undefined,
  }
}

export function adaptLauncherQuery(data: unknown): LauncherQueryView {
  if (!isRecord(data)) return { results: [] }
  const results = Array.isArray(data.results) ? data.results : []
  return {
    results: results.map(parseLauncherApp).filter((x): x is LauncherAppView => x != null),
  }
}

export function adaptLauncherRecent(data: unknown): LauncherRecentView {
  if (!isRecord(data)) return { items: [] }
  const items = Array.isArray(data.items) ? data.items : []
  return {
    items: items.map(parseLauncherApp).filter((x): x is LauncherAppView => x != null),
  }
}

// ── Todos (matches todos.rs) ──────────────────────────────────────────────────

export interface TodoItemView {
  id: string
  title: string
  description: string
  project_id: string | null
  due_at: number | null
  completed: boolean
  reminder_minutes: number | null
  created_at: number
  updated_at: number
}

export interface TodoProjectView {
  id: string
  name: string
  color: string
}

export function parseTodoItem(raw: unknown): TodoItemView | null {
  if (!isRecord(raw)) return null
  const id = raw.id
  const title = raw.title
  if (typeof id !== "string" || typeof title !== "string") return null
  const created = finiteNumber(raw.created_at)
  const updated = finiteNumber(raw.updated_at)
  if (created === null || updated === null) return null
  return {
    id,
    title,
    description: typeof raw.description === "string" ? raw.description : "",
    project_id: typeof raw.project_id === "string" ? raw.project_id : null,
    due_at: raw.due_at === null || raw.due_at === undefined ? null : finiteNumber(raw.due_at),
    completed: raw.completed === true,
    reminder_minutes:
      raw.reminder_minutes === null || raw.reminder_minutes === undefined
        ? null
        : optionalFiniteInt(raw.reminder_minutes) ?? null,
    created_at: created,
    updated_at: updated,
  }
}

export function adaptTodoList(data: unknown): TodoItemView[] {
  if (!Array.isArray(data)) return []
  return data.map(parseTodoItem).filter((x): x is TodoItemView => x != null)
}

export function parseTodoProject(raw: unknown): TodoProjectView | null {
  if (!isRecord(raw)) return null
  const id = raw.id
  const name = raw.name
  const color = raw.color
  if (typeof id !== "string" || typeof name !== "string" || typeof color !== "string") return null
  return { id, name, color }
}

export function adaptTodoProjects(data: unknown): TodoProjectView[] {
  if (!Array.isArray(data)) return []
  return data.map(parseTodoProject).filter((x): x is TodoProjectView => x != null)
}

export interface TodoParseDueDateView {
  parsed: boolean
  due_at: number | null
}

export function adaptTodoParseDueDate(data: unknown): TodoParseDueDateView {
  if (!isRecord(data)) return { parsed: false, due_at: null }
  const due =
    data.due_at === null || data.due_at === undefined ? null : finiteNumber(data.due_at)
  return { parsed: data.parsed === true, due_at: due }
}

// ── Vault (matches vault.rs) ──────────────────────────────────────────────────

export interface VaultRemoteView {
  name: string
  remote_type?: string
}

export interface VaultListView {
  remotes: VaultRemoteView[]
  rclone_available: boolean
}

export interface VaultBackupStatusView {
  state: string
  engine: string | null
  last_success_at: number | null
  last_error: string | null
  in_progress: boolean
}

export function adaptVaultList(data: unknown): VaultListView {
  if (!isRecord(data)) return { remotes: [], rclone_available: false }
  const remotes = Array.isArray(data.remotes) ? data.remotes : []
  const parsed: VaultRemoteView[] = []
  for (const r of remotes) {
    if (!isRecord(r) || typeof r.name !== "string") continue
    const remote: VaultRemoteView = { name: r.name }
    const rt = optionalString(r.remote_type)
    if (rt) remote.remote_type = rt
    parsed.push(remote)
  }
  return {
    remotes: parsed,
    rclone_available: data.rclone_available === true,
  }
}

export function adaptVaultBackupStatus(data: unknown): VaultBackupStatusView {
  if (!isRecord(data)) {
    return {
      state: "unknown",
      engine: null,
      last_success_at: null,
      last_error: null,
      in_progress: false,
    }
  }
  const last =
    data.last_success_at === null || data.last_success_at === undefined
      ? null
      : finiteNumber(data.last_success_at)
  return {
    state: typeof data.state === "string" ? data.state : "unknown",
    engine: typeof data.engine === "string" ? data.engine : null,
    last_success_at: last,
    last_error: typeof data.last_error === "string" ? data.last_error : null,
    in_progress: data.in_progress === true,
  }
}

// ── Dashboard / Sidebar (matches dashboard.rs) ───────────────────────────────

export interface DashboardQuickStatusView {
  battery: { percent: number; charging: boolean; time_remaining: string }
  network: {
    wifi_enabled: boolean
    connection_type?: string
    ethernet_connected?: boolean
    active_connection?: string
    local_ip?: string
    public_ip?: string
  }
  bluetooth: { powered: boolean; connected_count: number }
  dnd: DndPrefsView
  next_event: CalendarEvent | null
  power_profile: string
  dropdown_modules: string[]
}

export function adaptDashboardQuickStatus(data: unknown): DashboardQuickStatusView | null {
  if (!isRecord(data)) return null
  const battery = data.battery
  const network = data.network
  const bluetooth = data.bluetooth
  const dnd = data.dnd
  if (!isRecord(battery) || !isRecord(network) || !isRecord(bluetooth) || !isRecord(dnd)) {
    return null
  }
  const pct = finiteNumber(battery.percent)
  if (pct === null || typeof battery.time_remaining !== "string") return null
  const nextRaw = data.next_event
  const next_event =
    nextRaw === null || nextRaw === undefined ? null : parseCalendarEvent(nextRaw)
  const profile = data.power_profile
  const modules = data.dropdown_modules
  if (typeof profile !== "string" || !Array.isArray(modules)) return null
  if (!modules.every((x) => typeof x === "string")) return null
  return {
    battery: {
      percent: pct,
      charging: battery.charging === true,
      time_remaining: battery.time_remaining,
    },
    network: network as DashboardQuickStatusView["network"],
    bluetooth: {
      powered: bluetooth.powered === true,
      connected_count: optionalFiniteInt(bluetooth.connected_count) ?? 0,
    },
    dnd: adaptDndPrefs(dnd),
    next_event,
    power_profile: profile,
    dropdown_modules: modules as string[],
  }
}

export interface SidebarTileDataView {
  tile: string
  data: Record<string, unknown>
}

export function adaptSidebarTileData(data: unknown): SidebarTileDataView | null {
  if (!isRecord(data)) return null
  const tile = data.tile
  const tileData = data.data
  if (typeof tile !== "string" || !isRecord(tileData)) return null
  return { tile, data: tileData }
}

// ── Capture (matches capture.rs) ─────────────────────────────────────────────

export interface CaptureDevicesView {
  audio: unknown[]
  video: unknown[]
}

export function adaptCaptureDevices(data: unknown): CaptureDevicesView {
  if (!isRecord(data)) return { audio: [], video: [] }
  return {
    audio: Array.isArray(data.audio) ? data.audio : [],
    video: Array.isArray(data.video) ? data.video : [],
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
