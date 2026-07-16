// @ts-ignore
import GObject from 'gi://GObject'
import Gio from 'gi://Gio'
import GLib from 'gi://GLib'
import { 
    JsonRpcRequest, 
    JsonRpcResponse, 
    JsonRpcNotification,
    BluetoothDevice,
    BluetoothAdapter,
    AudioDevice,
    AudioStream,
    NetworkStatus,
    AccessPoint
} from './types'
import { recordSidecarExit, recordUncaught } from './crash-log'

const DEBUG_NOTIFICATIONS = GLib.getenv('AURA_DEBUG') === '1'

/** High-frequency internal refresh signals — omit unless AURA_DEBUG=1. */
const QUIET_NOTIFICATIONS = new Set([
    'Hyprland.StateChanged',
    'Performance.MetricsChanged',
    'Tray.Changed',
])

function sanitizeLogValue(value: unknown, maxLen = 64): string {
    if (value === null || value === undefined) return ''
    const s = typeof value === 'string' ? value : JSON.stringify(value)
    if (s.includes('<!DOCTYPE') || s.includes('<html')) return '<rejected>'
    return s.length <= maxLen ? s : `${s.slice(0, maxLen - 1)}…`
}

function formatNotificationLine(method: string, params: Record<string, unknown> | undefined): string {
    const p = params ?? {}

    switch (method) {
        case 'Network.StateChanged': {
            const parts: string[] = []
            if (p.connection_type) parts.push(`type=${p.connection_type}`)
            if (p.active_ssid) parts.push(`ssid=${p.active_ssid}`)
            if (p.local_ip) parts.push(`local=${p.local_ip}`)
            if (p.public_ip) parts.push(`public=${sanitizeLogValue(p.public_ip)}`)
            if (p.wifi_enabled !== undefined) parts.push(`wifi=${p.wifi_enabled ? 'on' : 'off'}`)
            return parts.length ? parts.join(' ') : 'changed'
        }
        case 'Hyprland.StateChanged': {
            const areas = Array.isArray(p.areas) ? p.areas.join(',') : ''
            return areas ? `areas=${areas}` : 'changed'
        }
        case 'Notifications.Changed':
            return `reason=${p.reason ?? 'changed'}`
        case 'Power.BatteryState':
            return [
                p.percent != null && `${p.percent}%`,
                p.status && String(p.status),
                p.time_remaining != null && `${p.time_remaining}m`,
            ].filter(Boolean).join(' ') || 'changed'
        case 'Power.Profile':
            return String(p.profile ?? 'changed')
        case 'Audio.StateChanged':
            return 'changed'
        case 'Settings.Changed':
            return p.field ? `field=${p.field}` : 'changed'
        case 'Shell.Osd':
            return [p.kind, p.label, p.percent != null && `${p.percent}%`].filter(Boolean).join(' ')
        default:
            return sanitizeLogValue(p)
    }
}

class SidecarService extends GObject.Object {
    static {
        GObject.registerClass({
            GTypeName: 'SidecarService',
            Signals: {
                'battery-state': { param_types: [GObject.TYPE_JSOBJECT] },
                'power-profile': { param_types: [GObject.TYPE_STRING] },
                'audio-state': { param_types: [GObject.TYPE_JSOBJECT] },
                'notification': { param_types: [GObject.TYPE_STRING, GObject.TYPE_JSOBJECT] },
            }
        }, this)
    }


    private _proc: Gio.Subprocess | null = null
    private _stdin: Gio.DataOutputStream | null = null
    private _nextId = 1
    private _pendingRequests = new Map<number | string, [(resolve: any) => void, (reject: any) => void]>()
    private _buffer = ''

    constructor() {
        super()
        this._spawn()
    }

    private _resolveSidecarPath(): string | null {
        const fromEnv = GLib.getenv('AURA_SIDECAR')
        if (fromEnv && GLib.file_test(fromEnv, GLib.FileTest.EXISTS)) {
            return fromEnv
        }

        const home = GLib.get_home_dir()
        const xdgConfig = GLib.getenv('XDG_CONFIG_HOME') || `${home}/.config`
        const candidates = [
            `${xdgConfig}/ags/sidecar/target/release/ags-sidecar`,
            `${xdgConfig}/ags/sidecar/target/debug/ags-sidecar`,
            `${home}/Engineering/Productivity/ags/sidecar/target/release/ags-sidecar`,
            `${home}/Engineering/Productivity/ags/sidecar/target/debug/ags-sidecar`,
        ]

        for (const path of candidates) {
            if (GLib.file_test(path, GLib.FileTest.EXISTS)) {
                return path
            }
        }
        return null
    }

    private _spawn() {
        try {
            const path = this._resolveSidecarPath()
            if (!path) {
                console.error(
                    'Sidecar binary not found. Set AURA_SIDECAR or build to ' +
                    '~/.config/ags/sidecar/target/{debug,release}/ags-sidecar'
                )
                return
            }

            console.log(`sidecar spawn ${path}`)

            this._proc = new Gio.Subprocess({
                argv: [path],
                flags: Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDOUT_PIPE,
            })

            this._proc.init(null)
            this._stdin = new Gio.DataOutputStream({
                base_stream: this._proc.get_stdin_pipe()!,
            })

            this._readLoop()
        } catch (e) {
            console.error('Failed to spawn sidecar:', e)
            recordUncaught(e, 'sidecar spawn')
        }
    }

    private _recordSidecarExitIfNeeded() {
        const proc = this._proc
        if (!proc) return

        try {
            // Reap so exit status / signal are available
            proc.wait(null)
        } catch {
            // Process may already be reaped
        }

        let exit_status: number | undefined
        let signal: number | undefined
        let message = 'sidecar process exited unexpectedly'

        try {
            if (proc.get_if_signaled()) {
                signal = proc.get_term_sig()
                message = `sidecar terminated by signal ${signal}`
            } else if (proc.get_if_exited()) {
                exit_status = proc.get_exit_status()
                if (exit_status === 0) {
                    // Clean exit (e.g. intentional shutdown) — do not dump
                    return
                }
                message = `sidecar exited with status ${exit_status}`
            }
        } catch {
            // Status may still be unavailable; dump with message only
        }

        recordSidecarExit({ message, exit_status, signal })
    }

    private _readLine(stream: Gio.DataInputStream): Promise<[Uint8Array | null, any]> {
        return new Promise((resolve, reject) => {
            stream.read_line_async(0, null, (obj, res) => {
                try {
                    const result = obj!.read_line_finish(res)
                    resolve(result as any)
                } catch (e) {
                    reject(e)
                }
            })
        })
    }

    private async _readLoop() {
        if (!this._proc) return
        const pipe = this._proc.get_stdout_pipe()
        if (!pipe) return

        const stdout = new Gio.DataInputStream({
            base_stream: pipe,
        })

        while (true) {
            try {
                const [line] = await this._readLine(stdout)
                if (!line) break

                const text = new TextDecoder().decode(line)
                this._handleMessage(text)
            } catch (e) {
                console.error('Error reading from sidecar:', e)
                break
            }
        }

        this._recordSidecarExitIfNeeded()
    }

    private _handleMessage(text: string) {
        if (!text.trim()) return

        try {
            const msg = JSON.parse(text)
            
            // Response
            if (msg.id !== undefined) {
                const handler = this._pendingRequests.get(msg.id)
                if (handler) {
                    this._pendingRequests.delete(msg.id)
                    const [resolve, reject] = handler
                    if (msg.error) {
                        reject(msg.error)
                    } else {
                        resolve(msg.result)
                    }
                }
            } 
            // Notification
            else if (msg.method) {
                this._handleNotification(msg as JsonRpcNotification)
            }
        } catch (e) {
            const trimmed = text.trim()
            // hyprctl dispatch (and similar) used to inherit stdout and print plain "ok"
            if (trimmed === 'ok' || trimmed === 'error') {
                if (DEBUG_NOTIFICATIONS) {
                    console.log(`sidecar: ignored non-json stdout line: ${trimmed}`)
                }
                return
            }
            console.error('Failed to parse sidecar message:', text, e)
        }
    }

    private _handleNotification(msg: JsonRpcNotification) {
        if (DEBUG_NOTIFICATIONS || !QUIET_NOTIFICATIONS.has(msg.method)) {
            const detail = formatNotificationLine(msg.method, msg.params as Record<string, unknown> | undefined)
            console.log(`sidecar notify ${msg.method}${detail ? ` ${detail}` : ''}`)
        }

        // General signal
        this.emit('notification', msg.method, msg.params)

        // Specific signals mapping
        switch (msg.method) {
            case 'Power.BatteryState':
                this.emit('battery-state', msg.params)
                break
            case 'Power.Profile':
                this.emit('power-profile', msg.params.profile)
                break
            case 'Audio.StateChanged':
                this.emit('audio-state', msg.params)
                break
        }
    }

    public async send(method: string, params?: any): Promise<any> {
        if (!this._stdin) throw new Error('Sidecar not running')

        return new Promise((resolve, reject) => {
            const id = this._nextId++
            this._pendingRequests.set(id, [resolve, reject])

            const req: JsonRpcRequest = {
                jsonrpc: '2.0',
                method,
                params,
                id,
            }

            const str = JSON.stringify(req) + '\n'
            this._stdin!.put_string(str, null)
        })
    }

    // --- Typed Wrappers ---

    // Power
    public async getBatteryState() { return this.send('Power.GetBatteryState') }
    public async getPowerProfile() { return this.send('Power.GetProfile') }
    public async setPowerProfile(profile: 'performance' | 'balanced' | 'saver') { return this.send('Power.SetProfile', { profile }) }

    // Network
    public async getNetworkStatus(): Promise<NetworkStatus> { return this.send('Network.GetStatus') }
    public async scanNetworks(): Promise<AccessPoint[]> { return this.send('Network.ScanNetworks') }
    public async toggleWifi(enabled: boolean) { return this.send('Network.ToggleWifi', { enabled }) }
    public async connectNetwork(ssid: string, password?: string) { return this.send('Network.Connect', { ssid, password }) }
    public async disconnectNetwork() { return this.send('Network.Disconnect') }

    // System
    public async getSystemStats() { return this.send('System.GetStats') }

    // Brightness
    public async getBrightness(monitor: string) { return this.send('Brightness.Get', { monitor }) }
    public async setBrightness(monitor: string, percent: number) { return this.send('Brightness.Set', { monitor, percent }) }

    // Audio
    public async getAudioState(): Promise<{ sinks: AudioDevice[], sources: AudioDevice[] }> { return this.send('Audio.GetDevices') }
    public async getAudioStreams(): Promise<AudioStream[]> { return this.send('Audio.GetStreams') }
    public async setStreamVolume(stream_id: number, volume: number) { return this.send('Audio.SetStreamVolume', { stream_id, volume }) }
    public async setStreamMute(stream_id: number, muted: boolean) { return this.send('Audio.SetStreamMute', { stream_id, muted }) }
    public async setDefaultDevice(device_id: number, type: 'output' | 'input' = 'output') { return this.send('Audio.SetDefaultDevice', { device_id, type }) }
    public async setSinkMute(device_id: number, muted: boolean) { return this.send('Audio.SetSinkMute', { device_id, muted }) }
    public async setSourceMute(device_id: number, muted: boolean) { return this.send('Audio.SetSourceMute', { device_id, muted }) }
    public async setSinkVolume(device_id: number, volume: number) { return this.send('Audio.SetSinkVolume', { device_id, volume }) }
    // Weather
    public async getWeather() { return this.send('Weather.Get') }
    // VPN
    public async getVpnStatus() { return this.send('Vpn.GetStatus') }
    public async connectVpn(profileId: string) { return this.send('Vpn.Connect', { profileId }) }
    public async disconnectVpn() { return this.send('Vpn.Disconnect') }

    // Bluetooth
    public async getBluetoothAdapters(): Promise<BluetoothAdapter[]> { return this.send('Bluetooth.GetAdapters') }
    public async getBluetoothDevices(): Promise<BluetoothDevice[]> { return this.send('Bluetooth.GetDevices') }
    public async scanBluetooth() { return this.send('Bluetooth.Scan') }
    public async stopScanBluetooth() { return this.send('Bluetooth.StopScan') }
    public async pairDevice(device_address: string) { return this.send('Bluetooth.Pair', { device_address }) }
    public async connectDevice(device_address: string) { return this.send('Bluetooth.Connect', { device_address }) }
    public async disconnectDevice(device_address: string) { return this.send('Bluetooth.Disconnect', { device_address }) }
    public async removeDevice(device_address: string) { return this.send('Bluetooth.Remove', { device_address }) }
    public async setAdapterPower(adapter_path: string, powered: boolean) { return this.send('Bluetooth.SetAdapterPower', { adapter_path, powered }) }
    public async setAdapterDiscoverable(adapter_path: string, discoverable: boolean) { return this.send('Bluetooth.SetAdapterDiscoverable', { adapter_path, discoverable }) }

    // Gamemode
    public async getGamemodeStatus() { return this.send('GameMode.IsEnabled') }
    
    // Storage
    public async getStorageConfig() {
        return this.send('Storage.Get', { namespace: 'aura', key: 'config' })
    }

    // Performance
    public async getPerformanceMetrics() { return this.send('Performance.GetMetrics') }

    // Security
    public async getSecurityStatus() { return this.send('Security.GetStatus') }

    // Devops
    public async getDevopsStatus() { return this.send('DevOps.GetStatus') }

    // Productivity
    public async getProductivityStats() { return this.send('Productivity.GetStats') }

    // Calendar
    public async getCalendarEvents() { return this.send('Calendar.GetEvents') }

    // Logs
    public async getLogs() { return this.send('Logs.Get') }

    // Packages
    public async getPackageUpdates() { return this.send('Packages.GetUpgradable') }

    // Automation
    public async triggerAutomation(workflow_id: string) {
        return this.send('Automation.RunWorkflow', { workflow_id })
    }

    // Communication
    public async getUnreadMessages() { return this.send('Communication.GetUnread') }

    // Fitness
    public async getFitnessGoals() { return this.send('Fitness.GetGoals') }
}

// Singleton instance
const sidecar = new SidecarService()
export default sidecar
