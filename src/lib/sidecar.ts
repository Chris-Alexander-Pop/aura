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

class SidecarService extends GObject.Object {
    static {
        GObject.registerClass({
            GTypeName: 'SidecarService',
            Signals: {
                'battery-state': { param_types: [GObject.TYPE_JSOBJECT] },
                'power-profile': { param_types: [GObject.TYPE_STRING] },
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

    private _spawn() {
        try {
            // Locate sidecar binary
            const home = GLib.get_home_dir()
            // Prefer debug build for development
            let path = `${home}/.config/ags/sidecar/target/debug/ags-sidecar`
            if (!GLib.file_test(path, GLib.FileTest.EXISTS)) {
                // Fallback to release or system path if needed
                path = `${home}/.config/ags/sidecar/target/release/ags-sidecar`
            }
            
            if (!GLib.file_test(path, GLib.FileTest.EXISTS)) {
                console.error('Sidecar binary not found at ' + path)
                return
            }

            console.log('Spawning sidecar from:', path)

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
        }
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
            console.error('Failed to parse sidecar message:', text, e)
        }
    }

    private _handleNotification(msg: JsonRpcNotification) {
        console.log('Sidecar Notification:', msg.method, msg.params)
        
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
    public async getGamemodeStatus() { return this.send('Gamemode.GetStatus') }
    
    // Storage
    public async getStorageConfig() { return this.send('Storage.GetConfig') }

    // Performance
    public async getPerformanceMetrics() { return this.send('Performance.GetMetrics') }

    // Security
    public async getSecurityStatus() { return this.send('Security.GetStatus') }

    // Devops
    public async getDevopsStatus() { return this.send('Devops.GetStatus') }

    // Productivity
    public async getProductivityStats() { return this.send('Productivity.GetStats') }

    // Calendar
    public async getCalendarEvents() { return this.send('Calendar.GetEvents') }

    // Logs
    public async getLogs() { return this.send('Logs.Get') }

    // Packages
    public async getPackageUpdates() { return this.send('Packages.GetUpdates') }

    // Automation
    public async triggerAutomation(id: string) { return this.send('Automation.Trigger', { id }) }

    // Communication
    public async getUnreadMessages() { return this.send('Communication.GetUnread') }

    // Fitness
    public async getFitnessGoals() { return this.send('Fitness.GetGoals') }
}

// Singleton instance
const sidecar = new SidecarService()
export default sidecar
