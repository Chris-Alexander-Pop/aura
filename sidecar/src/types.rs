use serde::{Deserialize, Serialize};

// VPN Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnProfile {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub display_name: String,
    pub interface: String,
    pub requires_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnAuth {
    pub user: Option<String>,
    pub pass: Option<String>,
    pub mfa: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VpnState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatus {
    pub state: VpnState,
    pub message: String,
    pub profile_id: Option<String>,
}

// Power Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percent: u8,
    pub charging: bool,
    pub time_remaining: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
}

// Network Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    pub ssid: String,
    pub bssid: String,
    pub strength: i32,
    pub frequency: i32,
    pub active: bool,
    pub security: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkStatus {
    pub wifi_enabled: bool,
    pub active_connection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_ssid: Option<String>,
    pub connection_type: String,
    pub ethernet_connected: bool,
    pub local_ip: Option<String>,
    pub public_ip: Option<String>,
}

// System Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu: f64,
    pub ram: f64,
    pub temp: f64,
    pub gpu: Option<f64>,
    pub storage: Option<f64>,
}

// Brightness Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorBrightness {
    pub monitor: String,
    pub brightness: f64,
}

// Weather Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temp: String,
    pub feels_like: String,
    pub description: String,
    pub humidity: i32,
    pub icon: String,
}

// JSON-RPC Request/Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

// Error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_ERROR: i32 = -32000;
}
