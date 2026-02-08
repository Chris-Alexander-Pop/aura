export interface VpnProfile {
    id: string
    name: string
    icon: string
    display_name: string
    interface: string
    requires_credentials: boolean
}

export interface VpnAuth {
    user?: string
    pass?: string
    mfa?: string
}

export type VpnState = 'disconnected' | 'connecting' | 'connected' | 'error'

export interface VpnStatus {
    state: VpnState
    message: string
    profile_id?: string
}

export interface BatteryState {
    percent: number
    charging: boolean
    time_remaining: string
}

export type PowerProfile = 'performance' | 'balanced' | 'saver'

export interface AccessPoint {
    ssid: string
    bssid: string
    strength: number
    frequency: number
    active: boolean
    security: string
}

export interface NetworkStatus {
    wifi_enabled: boolean
    active_connection?: string
    local_ip?: string
    public_ip?: string
}

export interface SystemStats {
    cpu: number
    ram: number
    temp: number
    gpu?: number
    storage?: number
}

export interface MonitorBrightness {
    monitor: string
    brightness: number
}

export interface WeatherData {
    temp: string
    feels_like: string
    description: string
    humidity: number
    icon: string
}


export interface BluetoothDevice {
    address: string
    path: string
    name: string
    alias: string
    connected: boolean
    paired: boolean
    trusted: boolean
    rssi?: number
    battery_percentage?: number
    device_type: string
    services: string[]
}

export interface BluetoothAdapter {
    path: string
    name: string
    alias: string
    powered: boolean
    discoverable: boolean
    pairable: boolean
    discovering: boolean
}

export interface AudioDevice {
    id: number
    name: string
    info: string
    volume: number
    is_default: boolean
}

export interface AudioStream {
    id: number
    name: string
    app: string
    volume: number
    sink_id: number
}

export interface JsonRpcRequest {
    jsonrpc: string
    method: string
    params?: any
    id?: number | string
}

export interface JsonRpcResponse {
    jsonrpc: string
    result?: any
    error?: {
        code: number
        message: string
        data?: any
    }
    id?: number | string
}

export interface JsonRpcNotification {
    jsonrpc: string
    method: string
    params: any
}
