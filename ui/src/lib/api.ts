// Typed fetch wrappers for the sidecar REST API at localhost:9080
const BASE = "/api"

async function call<T>(method: string, params?: Record<string, unknown>): Promise<T> {
  const isGet = !params || Object.keys(params).length === 0
  const url = `${BASE}/${encodeURIComponent(method)}`

  const res = await fetch(url, {
    method: isGet ? "GET" : "POST",
    headers: { "Content-Type": "application/json" },
    body: isGet ? undefined : JSON.stringify(params),
  })

  const json = await res.json()
  if (!json.ok) throw new Error(json.error ?? "Sidecar error")
  return json.data as T
}

// ─────────────────────────────────────────────────────────────────────────────
// Power
// ─────────────────────────────────────────────────────────────────────────────
export type PowerProfile = "performance" | "balanced" | "saver"

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
  connectNetwork: (ssid: string, password?: string) =>
    call("Network.Connect", { ssid, password }),
  disconnectNetwork: () => call("Network.Disconnect"),

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

  // Audio
  getAudioDevices: () =>
    call<{ sinks: Array<{ id: number; name: string; volume: number; is_default: boolean }>; sources: Array<{ id: number; name: string; volume: number; is_default: boolean }> }>(
      "Audio.GetDevices"
    ),
  getAudioStreams: () =>
    call<Array<{ id: number; name: string; app: string; volume: number; sink_id: number }>>(
      "Audio.GetStreams"
    ),
  setStreamVolume: (stream_id: number, volume: number) =>
    call("Audio.SetStreamVolume", { stream_id, volume }),
  setStreamMute: (stream_id: number, muted: boolean) =>
    call("Audio.SetStreamMute", { stream_id, muted }),

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
  connectVpn: (profileId: string) => call("Vpn.Connect", { profileId }),
  disconnectVpn: () => call("Vpn.Disconnect"),

  // Calendar
  getCalendarEvents: () => call<unknown[]>("Calendar.GetEvents"),

  // Packages
  getPackageUpdates: () => call<unknown[]>("Packages.GetUpdates"),

  // Logs
  getLogs: () => call<unknown[]>("Logs.Get"),

  // Security
  getSecurityStatus: () => call<Record<string, unknown>>("Security.GetStatus"),

  // Performance / Devops / Productivity / Automation / Communication / Fitness
  getPerformanceMetrics: () => call<Record<string, unknown>>("Performance.GetMetrics"),
  getDevopsStatus:       () => call<Record<string, unknown>>("Devops.GetStatus"),
  getProductivityStats:  () => call<Record<string, unknown>>("Productivity.GetStats"),
  getAutomationRules:    () => call<unknown[]>("Automation.ListRules"),
  getUnreadMessages:     () => call<Record<string, number>>("Communication.GetUnread"),
  getFitnessStats:       () => call<Record<string, unknown>>("Fitness.GetGoals"),

  // Brightness
  getBrightness: (monitor: string) => call<{ brightness: number }>("Brightness.Get", { monitor }),
  setBrightness: (monitor: string, percent: number) =>
    call("Brightness.Set", { monitor, percent }),

  // Hyprland (React bar — replaces GJS hyprland.ts)
  hyprlandGetWorkspaces: () => call<unknown>("Hyprland.GetWorkspaces"),
  hyprlandGetActiveWorkspace: () => call<unknown>("Hyprland.GetActiveWorkspace"),
  hyprlandGetClients: () => call<unknown>("Hyprland.GetClients"),
  hyprlandGetActiveWindow: () => call<unknown>("Hyprland.GetActiveWindow"),
  hyprlandDispatch: (command: string) => call<{ ok: boolean }>("Hyprland.Dispatch", { command }),

  // Session / Aura / Apps (allowlisted shell)
  sessionLock: () => call<{ ok: boolean }>("Session.Lock"),
  sessionLogout: () => call<{ ok: boolean }>("Session.Logout"),
  sessionSuspend: () => call<{ ok: boolean }>("Session.Suspend"),
  sessionReboot: () => call<{ ok: boolean }>("Session.Reboot"),
  sessionPowerOff: () => call<{ ok: boolean }>("Session.PowerOff"),
  auraToggleWindow: (name: "control-center" | "calendar" | "dropdown") =>
    call<{ ok: boolean }>("Aura.ToggleWindow", { name }),
  appsLaunch: (id: string) => call<{ ok: boolean }>("Apps.Launch", { id }),

  // Media (playerctl)
  getMediaNowPlaying: () =>
    call<{ playing: boolean; title: string; artist: string }>("Media.GetNowPlaying"),

  // Processes (task manager)
  processListTop: (limit?: number) =>
    call<Array<{ pid: number; cpu: number; name: string }>>("Process.ListTop", {
      ...(limit != null ? { limit } : {}),
    }),
  processKill: (pid: number) => call<{ ok: boolean }>("Process.Kill", { pid }),
}

export default api
