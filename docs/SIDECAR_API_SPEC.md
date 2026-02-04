# Rust Sidecar API Specification

The AGS frontend communicates with the Rust backend via **JSON-RPC 2.0** over Standard I/O (Stdin/Stdout) or a Unix Domain Socket.

**Format**:
Request: `{"jsonrpc": "2.0", "method": "Service.Method", "params": [...], "id": 1}`
Response: `{"jsonrpc": "2.0", "result": ..., "id": 1}`
Notification (Push): `{"jsonrpc": "2.0", "method": "Service.Signal", "params": ...}`

## Service: VPN (`Vpn`)

### Methods
- `Vpn.Connect(profile_id: string, auth: { user?: string, pass?: string, mfa?: string })`
  - Initiates connection.
  - Returns: `void` (Async)
- `Vpn.Disconnect()`
  - Terminates connection.
- `Vpn.GetProfiles()`
  - Returns: `VpnProfile[]`

### Signals (Push Updates)
- `Vpn.StateChanged`
  - Payload: `{ state: "disconnected" | "connecting" | "connected" | "error", message: string }`
- `Vpn.Log`
  - Payload: `{ line: string }` (Real-time logs from openconnect)

## Service: Audio (`Audio`)
*Note: Primary volume events might come from Wireplumber directly if using AGS Native, but custom logic resides here.*

### Methods
- `Audio.SetVolume(stream: string, percent: float)`
- `Audio.ToggleMute(stream: string)`

## Service: Power (`Power`)

### Methods
- `Power.SetProfile(profile: "performance" | "balanced" | "power-saver")`
- `Power.GetBatteryState()`
  - Returns: `{ percent: int, charging: bool, time_remaining: string }`

### Signals
- `Power.BatteryChanged`
  - Payload: `{ percent: int, charging: bool }`

## Service: System (`System`)

### Methods
- `System.GetStats()`
  - Returns: `{ cpu: float, ram: float, temp: float }`

### Signals
- `System.StatsUpdate` (Polled every 2s)
  - Payload: `{ cpu: float, ram: float }`

## Service: Network (`Network`)

### Methods
- `Network.ToggleWifi(enabled: bool)`
- `Network.ScanNetworks()`

### Signals
- `Network.WifiStateChanged`
