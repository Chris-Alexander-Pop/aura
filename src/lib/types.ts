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
