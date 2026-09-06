// Typed fetch wrappers for the sidecar REST API at localhost:9080
import { NetworkConnectError } from "@/lib/network-connect"
import {
  adaptDndPrefs,
  adaptLogEntries,
  adaptNotificationList,
  adaptPackageUpdates,
  adaptPerformanceMetrics,
  adaptSecurityStatus,
  adaptCaptureDevices,
  adaptDashboardQuickStatus,
  adaptLauncherQuery,
  adaptLauncherRecent,
  adaptTodoList,
  adaptTodoParseDueDate,
  adaptTodoProjects,
  adaptVaultBackupStatus,
  adaptVaultList,
  adaptSidebarTileData,
  parseAuraSettings,
  parseCalendarEvents,
  parseCalendars,
  parseGoogleCalendarAuthStatus,
  type AuraSettingsView,
  type DashboardQuickStatusView,
  type LauncherAppView,
  type TodoItemView,
  type TodoProjectView,
  type VaultBackupStatusView,
  type VaultListView,
  type DndPrefsView,
  type LogEntryView,
  type NotificationItemView,
  type PackageUpdateView,
  type PerformanceMetricsView,
  type SecurityStatusView,
} from "./api-types"
import { requestControlCenterPane } from "./control-center-pane"
import type { PaneId } from "@/pages/control-center/navigation"

import { sidecarAuthHeaders } from "./sidecar-http"

export type { AuraSettingsView }

const BASE = "/api"
const SIDECAR_TIMEOUT_MS = 12_000

async function call<T>(method: string, params?: Record<string, unknown>): Promise<T> {
  const data = await callData(method, params)
  return data as T
}

async function callData(
  method: string,
  params?: Record<string, unknown>,
  timeoutMs = SIDECAR_TIMEOUT_MS,
): Promise<unknown> {
  const url = `${BASE}/${encodeURIComponent(method)}`

  let res: Response
  try {
    const headers = await sidecarAuthHeaders()
    const hasParams = params && Object.keys(params).length > 0
    res = await fetch(url, {
      method: "POST",
      headers,
      body: hasParams ? JSON.stringify(params) : undefined,
      signal: AbortSignal.timeout(timeoutMs),
    })
  } catch (err) {
    if (err instanceof DOMException && err.name === "TimeoutError") {
      throw new Error(`Sidecar timed out after ${timeoutMs / 1000}s (${method})`)
    }
    throw err
  }

  const text = await res.text()
  const trimmed = text.trim()
  if (!trimmed) {
    throw new Error(
      res.ok
        ? "Sidecar returned an empty body — start ags-sidecar on :9080 or check the Vite /api proxy."
        : `Sidecar HTTP ${res.status}: empty body`
    )
  }

  let json: { ok?: boolean; error?: string; data?: unknown }
  try {
    json = JSON.parse(trimmed) as { ok?: boolean; error?: string; data?: unknown }
  } catch {
    throw new Error(
      "Could not parse sidecar JSON — another process may be answering /api, or the response was truncated."
    )
  }
  if (!json.ok) throw new Error(json.error ?? "Sidecar error")
  return json.data
}

// ─────────────────────────────────────────────────────────────────────────────
// Power
// ─────────────────────────────────────────────────────────────────────────────
export type PowerProfile = "performance" | "balanced" | "saver"

export type WorkflowView = {
  id: string
  name: string
  enabled: boolean
  triggers?: unknown
  actions?: unknown
  created_at?: number
  run_count?: number
  last_run_at?: number
}

export type WorkflowRunView = {
  ts: string
  workflow_id: string
  success: boolean
  error?: string
}

export type {
  CalendarEvent,
  DashboardQuickStatusView,
  DndPrefsView,
  LauncherAppView,
  TodoItemView,
  TodoProjectView,
  VaultBackupStatusView,
  VaultListView,
  LogEntryView,
  NotificationItemView,
  PackageUpdateView,
  PerformanceMetricsView,
  SecurityStatusView,
} from "./api-types"

export interface BatteryState {
  percent: number
  charging: boolean
  time_remaining: string
}

export const api = {
  // Power
  getBatteryState: () => call<BatteryState>("Power.GetBatteryState"),
  getPowerProfile:  () => call<{ profile: PowerProfile }>("Power.GetProfile"),
  setPowerProfile:  (profile: PowerProfile) => call("Power.SetProfile", { profile }),

  // Network
  getNetworkStatus: () =>
    call<{ wifi_enabled: boolean; active_connection?: string; local_ip?: string; public_ip?: string }>(
      "Network.GetStatus"
    ),
  scanNetworks: () =>
    call<Array<{ ssid: string; strength: number; active: boolean; security: string }>>(
      "Network.ScanNetworks"
    ),
  toggleWifi: (enabled: boolean) => call("Network.ToggleWifi", { enabled }),
  connectNetwork: async (ssid: string, password?: string) => {
    const data = (await callData("Network.Connect", {
      ssid,
      ...(password != null && password !== "" ? { password } : {}),
    })) as { success?: boolean; error?: string; needs_password?: boolean }
    if (data?.success === false) {
      throw new NetworkConnectError(
        data.error ?? "Connection failed",
        data.needs_password === true,
      )
    }
    return data
  },
  disconnectNetwork: () => call("Network.Disconnect"),
  listSavedNetworks: () =>
    call<Array<{ name: string; uuid: string; autoconnect: boolean }>>("Network.ListSaved"),
  forgetNetwork: (opts: { uuid?: string; name?: string }) => call("Network.Forget", opts),

  getKeyringStatus: () =>
    call<{ available: boolean; unlocked: boolean; message?: string }>("Security.GetKeyringStatus"),

  // Bluetooth
  getBluetoothAdapters: () =>
    call<Array<{ path: string; name: string; powered: boolean; discovering: boolean }>>(
      "Bluetooth.GetAdapters"
    ),
  getBluetoothDevices: () =>
    call<Array<{ address: string; name: string; connected: boolean; paired: boolean; battery_percentage?: number }>>(
      "Bluetooth.GetDevices"
    ),
  scanBluetooth: () => call("Bluetooth.Scan"),
  connectDevice: (device_address: string) => call("Bluetooth.Connect", { device_address }),
  disconnectDevice: (device_address: string) => call("Bluetooth.Disconnect", { device_address }),
  pairDevice: (device_address: string) => call("Bluetooth.Pair", { device_address }),
  removeDevice: (device_address: string) => call("Bluetooth.Remove", { device_address }),
  setBluetoothAdapterPower: (adapter_path: string, powered: boolean) =>
    call("Bluetooth.SetAdapterPower", { adapter_path, powered }),

  // Audio
  getAudioDevices: () =>
    call<{
      sinks: Array<{ id: number; name: string; volume: number; muted?: boolean; is_default: boolean }>
      sources: Array<{ id: number; name: string; volume: number; muted?: boolean; is_default: boolean }>
    }>("Audio.GetDevices"),
  getAudioStreams: () =>
    call<Array<{ id: number; name: string; app: string; volume: number; sink_id: number; muted?: boolean }>>(
      "Audio.GetStreams"
    ),
  setStreamVolume: (stream_id: number, volume: number) =>
    call("Audio.SetStreamVolume", { stream_id, volume }),
  setStreamMute: (stream_id: number, muted: boolean) =>
    call("Audio.SetStreamMute", { stream_id, muted }),
  setSinkMute: (device_id: number, muted: boolean) =>
    call("Audio.SetSinkMute", { device_id, muted }),
  setSourceMute: (device_id: number, muted: boolean) =>
    call("Audio.SetSourceMute", { device_id, muted }),
  setSinkVolume: (device_id: number, volume: number) =>
    call("Audio.SetSinkVolume", { device_id, volume }),
  setSourceVolume: (device_id: number, volume: number) =>
    call("Audio.SetSourceVolume", { device_id, volume }),
  setDefaultAudioDevice: (device_id: number, type: "output" | "input" = "output") =>
    call("Audio.SetDefaultDevice", { device_id, type }),
  refreshAudio: () => call("Audio.Refresh"),

  // System
  getSystemStats: () =>
    call<{ cpu: number; ram: number; temp: number; gpu?: number; storage?: number }>(
      "System.GetStats"
    ),

  // Weather
  getWeather: () =>
    call<{ temp: string; feels_like: string; description: string; humidity: number; icon: string }>(
      "Weather.Get"
    ),

  // VPN
  getVpnStatus: () =>
    call<{ state: string; message: string; profile_id?: string }>("Vpn.GetStatus"),
  getVpnProfiles: () =>
    call<
      Array<{
        id: string
        name: string
        icon?: string
        display_name?: string
        interface?: string
        requires_credentials?: boolean
      }>
    >("Vpn.GetProfiles"),
  connectVpn: (profileId: string) => call("Vpn.Connect", { profileId }),
  disconnectVpn: () => call("Vpn.Disconnect"),

  // Calendar
  getCalendarEvents: (opts?: { start_date?: number; end_date?: number }) =>
    callData("Calendar.GetEvents", opts ?? {}).then(parseCalendarEvents),
  /** For react-query: `queryFn: () => api.fetchCalendarEvents()` */
  fetchCalendarEvents: () => callData("Calendar.GetEvents").then(parseCalendarEvents),
  getCalendarUpcoming: (limit = 10) =>
    callData("Calendar.GetUpcomingEvents", { limit, days: 14 }).then(parseCalendarEvents),
  getCalendars: () => callData("Calendar.GetCalendars").then(parseCalendars),
  syncCalendars: () =>
    callData("Calendar.SyncCalendars", undefined, 60_000).then(
      (data) =>
        data as { success?: boolean; synced?: number; message?: string; provider?: string },
    ),
  createCalendarEvent: (opts: {
    title: string
    start: number
    end: number
    description?: string
    reminder_minutes?: number
  }) => call("Calendar.CreateEvent", opts),
  updateCalendarEvent: (eventId: string, updates: Record<string, unknown>) =>
    call("Calendar.UpdateEvent", { event_id: eventId, updates }),
  deleteCalendarEvent: (eventId: string) =>
    call("Calendar.DeleteEvent", { event_id: eventId }),
  getGoogleCalendarAuthStatus: () =>
    callData("Calendar.GoogleAuthStatus").then(parseGoogleCalendarAuthStatus),
  /** Starts browser OAuth in the background; poll `getGoogleCalendarAuthStatus` until connected. */
  googleCalendarStartAuth: () =>
    callData("Calendar.GoogleStartAuth").then(
      (data) => data as { started?: boolean; message?: string },
    ),
  googleCalendarDisconnect: () =>
    call<{ success?: boolean }>("Calendar.GoogleDisconnect"),

  // Packages
  getPackageUpdates: () => callData("Packages.GetUpgradable").then(adaptPackageUpdates),
  searchPackages: (query: string) =>
    call<Array<{ name: string; version: string; description: string; installed?: boolean }>>(
      "Packages.Search",
      { query }
    ),
  installPackage: (name: string) =>
    call<{ success?: boolean; error?: string }>("Packages.Install", { name }),
  removePackage: (name: string) =>
    call<{ success?: boolean; error?: string }>("Packages.Remove", { name }),
  upgradePackages: () => call<{ success?: boolean; error?: string }>("Packages.Upgrade"),
  getPackageDependencies: (name: string) =>
    call<{ dependencies: string[] }>("Packages.GetPackageDependencies", { name }),
  getPackageTransactionHistory: (limit = 50) =>
    call<Array<{ ts: string; action: string; packages: string[] }>>(
      "Packages.GetTransactionHistory",
      { limit }
    ),

  // Notifications
  listNotifications: (limit = 50) =>
    callData("Notifications.List", { limit }).then(adaptNotificationList),
  dismissNotification: (id: number) => call("Notifications.Dismiss", { id }),
  clearAllNotifications: () => call("Notifications.ClearAll"),
  getNotificationDnd: () => callData("Notifications.GetDnd").then(adaptDndPrefs),
  setNotificationDnd: (dnd: DndPrefsView) => call("Notifications.SetDnd", { dnd }),
  getNotificationRules: () =>
    call<{ muted_apps: string[] }>("Notifications.GetRules"),
  setNotificationRules: (rules: { muted_apps: string[]}) =>
    call("Notifications.SetRules", { rules }),
  invokeNotificationAction: (id: number, action_key: string) =>
    call("Notifications.InvokeAction", { id, action_key }),

  listBrightnessMonitors: () =>
    call<{ monitors: Array<{ monitor: string; brightness: number }> }>("Brightness.Get", {
      monitor: "all",
    }),

  // Keybinds
  listKeybinds: (category?: string) =>
    call<Array<{ combo: string; action: string; file: string; category: string; bind_type?: string }>>(
      "Keybinds.List",
      category ? { category } : undefined
    ),
  validateKeybinds: () => call<{ duplicates: string[]; unknown_dispatches: string[] }>("Keybinds.Validate"),

  // Logs
  getLogs: (opts?: { lines?: number; priority?: string; unit?: string }) =>
    callData("Logs.Get", opts ?? {}).then(adaptLogEntries),

  // Security
  getSecurityStatus: () => callData("Security.GetStatus").then(adaptSecurityStatus),
  listFingerprints: () =>
    call<Array<{ name: string; finger?: string }>>("Security.ListFingerprints"),
  runClamScan: (path?: string) =>
    call<{ success?: boolean; output?: string; error?: string }>("Security.RunClamScan", {
      ...(path ? { path } : {}),
    }),
  enableFirewall: () => call("Security.EnableFirewall"),
  disableFirewall: () => call("Security.DisableFirewall"),

  // Performance / Devops / Productivity / Automation / Communication / Fitness
  getPerformanceMetrics: () => callData("Performance.GetMetrics").then(adaptPerformanceMetrics),
  getDevopsStatus: () => call<Record<string, unknown>>("DevOps.GetStatus"),
  getDevopsContainers: () =>
    call<
      Array<{ id: string; name: string; image: string; status: string; ports?: string }>
    >("DevOps.GetDockerContainers"),
  startDevopsContainer: (name: string) =>
    call<{ success: boolean }>("DevOps.StartContainer", { name }),
  stopDevopsContainer: (name: string) =>
    call<{ success: boolean }>("DevOps.StopContainer", { name }),
  restartDevopsContainer: (name: string) =>
    call<{ success: boolean }>("DevOps.RestartContainer", { name }),
  getDevopsContainerLogs: (name: string, lines?: number) =>
    call<{ logs: string }>("DevOps.GetContainerLogs", { name, ...(lines != null ? { lines } : {}) }),
  getDevopsGitRepos: (path?: string) =>
    call<Array<{ path: string; name: string; branch: string; status: string }>>(
      "DevOps.GetGitRepos",
      path ? { path } : {}
    ),
  getDevopsSystemdTimers: () =>
    call<Array<{ name: string; next_run: string; last_run: string; active: boolean }>>(
      "DevOps.GetSystemdTimers"
    ),
  getDevopsKubernetesPods: () =>
    call<{ pods: string }>("DevOps.GetKubernetesPods"),
  getProductivityStats: () => call<Record<string, unknown>>("Productivity.GetStats"),
  getProductivityTasks: () =>
    call<
      Array<{
        id: string
        title: string
        description: string
        due_date?: number | null
        completed: boolean
      }>
    >("Productivity.GetTasks"),
  createProductivityTask: (title: string, description?: string) =>
    call("Productivity.CreateTask", { title, description: description ?? "" }),
  deleteProductivityTask: (taskId: string) =>
    call("Productivity.DeleteTask", { task_id: taskId }),
  updateProductivityTask: (
    taskId: string,
    updates: {
      title?: string
      description?: string
      completed?: boolean
      due_date?: number | null
    }
  ) => call<{ success: boolean }>("Productivity.UpdateTask", { task_id: taskId, updates }),
  setProductivityFocusMode: (enabled: boolean) =>
    call("Productivity.SetFocusMode", { enabled }),
  getAutomationRules: () =>
    call<WorkflowView[]>("Automation.GetWorkflows"),
  createAutomationWorkflow: (opts: { name: string; actions: unknown; triggers?: unknown }) =>
    call<WorkflowView>("Automation.CreateWorkflow", {
      name: opts.name,
      actions: opts.actions,
      ...(opts.triggers != null ? { triggers: opts.triggers } : {}),
    }),
  triggerAutomation: (workflowId: string) =>
    call<{ success: boolean }>("Automation.Trigger", { workflow_id: workflowId }),
  enableAutomationWorkflow: (workflowId: string) =>
    call<{ success: boolean }>("Automation.EnableWorkflow", { workflow_id: workflowId }),
  disableAutomationWorkflow: (workflowId: string) =>
    call<{ success: boolean }>("Automation.DisableWorkflow", { workflow_id: workflowId }),
  deleteAutomationWorkflow: (workflowId: string) =>
    call<{ success: boolean; deleted?: boolean }>("Automation.DeleteWorkflow", {
      workflow_id: workflowId,
    }),
  getAutomationHistory: (workflowId?: string, limit = 20) =>
    call<WorkflowRunView[]>("Automation.GetWorkflowHistory", {
      workflow_id: workflowId,
      limit,
    }),
  getUnreadMessages:     () => call<Record<string, number>>("Communication.GetUnread"),
  getCommunicationApps: () => call<string[]>("Communication.GetCommunicationApps"),
  launchCommunicationApp: (app: string) =>
    call<{ success: boolean }>("Communication.LaunchApp", { app_name: app }),
  importCommunicationUnread: (counts: Record<string, number>) =>
    call<{ ok: boolean }>("Communication.ImportUnread", { counts }),
  getFitnessStats:       () => call<Record<string, unknown>>("Fitness.GetGoals"),
  fitnessSetGoal: (type: string, target: number) =>
    call<{ id: string; goal_type: string; target: number; current: number }>("Fitness.SetGoal", {
      type,
      target,
    }),
  fitnessStartWorkout: (type: string) =>
    call<{ id: string; workout_type: string; start_time: number }>("Fitness.StartWorkout", { type }),
  fitnessStopWorkout: (id?: string) =>
    call<{ id?: string; success?: boolean; message?: string }>(
      "Fitness.StopWorkout",
      id ? { id } : {}
    ),
  getFitnessWorkoutHistory: () =>
    call<
      Array<{
        id: string
        workout_type: string
        start_time: number
        end_time?: number | null
        duration_seconds?: number | null
      }>
    >("Fitness.GetWorkoutHistory"),
  getFitnessActivity: () =>
    call<{ date: string; workout_minutes?: number; steps: number }>("Fitness.GetActivity"),

  // Brightness
  getBrightness: (monitor: string) => call<{ brightness: number }>("Brightness.Get", { monitor }),
  setBrightness: (monitor: string, percent: number) =>
    call("Brightness.Set", { monitor, percent }),

  // Hyprland (React bar — replaces GJS hyprland.ts)
  hyprlandGetWorkspaces: () =>
    call<import("./api-types").HyprWorkspace[]>("Hyprland.GetWorkspaces"),
  hyprlandGetActiveWorkspace: () =>
    call<import("./api-types").HyprActiveWorkspace | null>("Hyprland.GetActiveWorkspace"),
  hyprlandGetBarSnapshot: () =>
    call<import("./api-types").HyprBarSnapshot>("Hyprland.GetBarSnapshot"),
  hyprlandGetClients: () => call<import("./api-types").HyprClient[]>("Hyprland.GetClients"),
  hyprlandGetActiveWindow: () =>
    call<import("./api-types").HyprActiveWindow | null>("Hyprland.GetActiveWindow"),
  hyprlandGetMonitors: () => call<import("./api-types").HyprMonitor[]>("Hyprland.GetMonitors"),
  hyprlandDispatch: (command: string) => call<{ ok: boolean }>("Hyprland.Dispatch", { command }),

  // Session / Aura / Apps (allowlisted shell)
  sessionLock: () => call<{ ok: boolean }>("Session.Lock"),
  sessionLogout: () => call<{ ok: boolean }>("Session.Logout"),
  sessionSuspend: () => call<{ ok: boolean }>("Session.Suspend"),
  sessionReboot: () => call<{ ok: boolean }>("Session.Reboot"),
  sessionPowerOff: () => call<{ ok: boolean }>("Session.PowerOff"),
  auraToggleWindow: (name: "control-center" | "calendar" | "dropdown" | "module-hub" | "launcher") =>
    call<{ ok: boolean }>("Aura.ToggleWindow", { name }),
  openControlCenterPane: (pane: PaneId) => {
    requestControlCenterPane(pane)
    return call<{ ok: boolean }>("Aura.ToggleWindow", { name: "control-center" })
  },
  appsLaunch: (id: string) => call<{ ok: boolean }>("Apps.Launch", { id }),

  // Media (playerctl / MPRIS)
  getMediaNowPlaying: () => call<import("./api-types").MediaNowPlaying>("Media.GetNowPlaying"),
  mediaGetPlayers: () => call<string[]>("Audio.Media.GetPlayers"),
  mediaPlayPause: (playerName?: string) =>
    call<{ success: boolean }>("Audio.Media.PlayPause", {
      ...(playerName ? { player_name: playerName } : {}),
    }),
  mediaNext: (playerName?: string) =>
    call<{ success: boolean }>("Audio.Media.Next", {
      ...(playerName ? { player_name: playerName } : {}),
    }),
  mediaPrevious: (playerName?: string) =>
    call<{ success: boolean }>("Audio.Media.Previous", {
      ...(playerName ? { player_name: playerName } : {}),
    }),

  // Aura settings (SQLite-backed shell prefs)
  getAuraSettings: () =>
    call<{ settings: AuraSettingsView; schema_version: number }>("Settings.Get").then((data) => {
      const settings = parseAuraSettings(data.settings)
      if (!settings) throw new Error("Invalid Settings.Get payload")
      return { settings, schema_version: data.schema_version }
    }),
  setAuraSettings: (partial: Partial<AuraSettingsView>) =>
    call<{ settings: AuraSettingsView }>("Settings.Set", { partial }).then((data) => {
      const settings = parseAuraSettings(data.settings)
      if (!settings) throw new Error("Invalid Settings.Set payload")
      return { settings }
    }),
  getAuraSettingsSchema: () => call<Record<string, unknown>>("Settings.GetSchema"),
  resetAuraSettings: () => call<{ ok: boolean }>("Settings.Reset"),

  // Appearance (theme + night light)
  getNightLight: () =>
    call<{ enabled: boolean; temperature: number }>("Appearance.GetNightLight"),
  setNightLight: (partial: { enabled?: boolean; temperature?: number }) =>
    call<{ ok: boolean; state: { enabled: boolean; temperature: number } }>(
      "Appearance.SetNightLight",
      partial
    ),
  getAppearanceTheme: () => call<{ theme: string }>("Appearance.GetTheme"),
  setAppearanceTheme: (theme: string) =>
    call<{ ok: boolean; theme: string }>("Appearance.SetTheme", { theme }),

  // System tray
  trayList: () =>
    call<{ items: Array<{ id: string; title: string; icon_name?: string; status: string; category: string }> }>(
      "Tray.List"
    ),
  trayActivate: (id: string) => call<{ ok: boolean }>("Tray.Activate", { id }),
  traySecondaryActivate: (id: string) =>
    call<{ ok: boolean }>("Tray.SecondaryActivate", { id }),

  // Processes (task manager)
  processListTop: (limit?: number) =>
    call<Array<{ pid: number; cpu: number; name: string; confirmation_token: string }>>(
      "Process.ListTop",
      {
      ...(limit != null ? { limit } : {}),
    }),
  processKill: (pid: number, confirmationToken: string) =>
    call<{ ok: boolean }>("Process.Kill", { pid, confirmation_token: confirmationToken }),

  // Launcher
  launcherQuery: (query: string) =>
    callData("Launcher.Query", { query }).then(adaptLauncherQuery),
  launcherRun: (id: string, source?: "vicinae" | "desktop") =>
    call<{ ok: boolean }>("Launcher.Run", { id, ...(source ? { source } : {}) }),
  vicinaeExec: (id: string, source: "vicinae" | "desktop" = "vicinae") =>
    call<{ ok: boolean }>("Vicinae.Exec", { id, source }),
  launcherRecent: () => callData("Launcher.Recent").then(adaptLauncherRecent),
  launcherPin: (id: string, pinned: boolean) =>
    call<{ ok: boolean; id: string; pinned: boolean }>("Launcher.Pin", { id, pinned }),
  launcherVicinaeQuery: (query: string) =>
    callData("Launcher.VicinaeQuery", { query }).then(adaptLauncherQuery),

  // Todos
  todosList: (opts?: { project_id?: string; include_completed?: boolean }) =>
    callData("Todos.List", opts ?? {}).then(adaptTodoList),
  todosCreate: (opts: {
    title: string
    description?: string
    project_id?: string
    due_at?: number
    due_text?: string
    reminder_minutes?: number
  }) => call<TodoItemView>("Todos.Create", opts),
  todosUpdate: (opts: {
    id: string
    title?: string
    description?: string
    project_id?: string | null
    due_at?: number | null
    completed?: boolean
    reminder_minutes?: number | null
  }) => call<TodoItemView>("Todos.Update", opts),
  todosDelete: (id: string) => call<{ deleted: boolean }>("Todos.Delete", { id }),
  todosListProjects: () => callData("Todos.ListProjects").then(adaptTodoProjects),
  todosParseDueDate: (text: string) =>
    callData("Todos.ParseDueDate", { text }).then(adaptTodoParseDueDate),
  todosCreateProject: (name: string) => call<TodoProjectView>("Todos.CreateProject", { name }),

  // Vault
  vaultList: () => callData("Vault.List").then(adaptVaultList),
  vaultBackupStatus: () => callData("Vault.Backup.Status").then(adaptVaultBackupStatus),
  vaultGetEntry: (key: string) => call<{ key: string; value?: string }>("Vault.GetEntry", { key }),
  vaultSetEntry: (key: string, value: string) =>
    call<{ ok: boolean }>("Vault.SetEntry", { key, value }),

  // Dashboard / Sidebar
  dashboardGetQuickStatus: () =>
    callData("Dashboard.GetQuickStatus").then((data) => {
      const status = adaptDashboardQuickStatus(data)
      if (!status) throw new Error("Invalid Dashboard.GetQuickStatus payload")
      return status
    }),
  sidebarGetTileData: (tile: string) =>
    callData("Sidebar.GetTileData", { tile }).then((data) => {
      const tileData = adaptSidebarTileData(data)
      if (!tileData) throw new Error("Invalid Sidebar.GetTileData payload")
      return tileData
    }),

  // Capture
  captureScreenshot: (opts: {
    mode: "region" | "window" | "full"
    output: "clipboard" | "file"
    path?: string
  }) => call<{ ok: boolean; tool_missing?: boolean; tool?: string; error?: string }>(
    "Capture.Screenshot",
    opts
  ),
  captureRecordStart: () =>
    call<{ ok: boolean; tool_missing?: boolean; tool?: string; path?: string }>(
      "Capture.RecordStart"
    ),
  captureRecordStop: () => call<{ ok: boolean; stopped?: boolean }>("Capture.RecordStop"),
  captureListDevices: () => callData("Capture.ListDevices").then(adaptCaptureDevices),
}

export default api
