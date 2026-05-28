//! Library surface for integration tests and shared registry setup.
pub mod notify;
pub mod rpc;
pub mod server;
pub mod services;
pub mod types;
pub mod utils;

use crate::services::ServiceRegistry;

/// Register all sidecar RPC handlers (same set as the `ags-sidecar` binary).
pub fn build_registry() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();
    services::power::register(&mut registry);
    services::network::register(&mut registry);
    services::system::register(&mut registry);
    services::brightness::register(&mut registry);
    services::vpn::register(&mut registry);
    services::weather::register(&mut registry);
    services::gamemode::register(&mut registry);
    services::storage::register(&mut registry);
    services::audio::register(&mut registry);
    services::bluetooth::register(&mut registry);
    services::performance::register(&mut registry);
    services::security::register(&mut registry);
    services::devops::register(&mut registry);
    services::productivity::register(&mut registry);
    services::calendar::register(&mut registry);
    services::logs::register(&mut registry);
    services::packages::register(&mut registry);
    services::automation::register(&mut registry);
    services::communication::register(&mut registry);
    services::fitness::register(&mut registry);
    services::hyprland::register(&mut registry);
    services::shell::register(&mut registry);
    services::mpris::register(&mut registry);
    services::processes::register(&mut registry);
    services::notifications::register(&mut registry);
    services::keybinds::register(&mut registry);
    registry
}

/// Golden CLI parser entry points for integration contract tests (`tests/integration_contracts.rs`).
#[doc(hidden)]
pub mod contract_parsers {
    use crate::services::audio::{AudioDevice, AudioStream};
    use crate::services::bluetooth::BluetoothDevice;
    use crate::services::network::SavedNetwork;
    use crate::services::packages::Package;
    use crate::types::{AccessPoint, PowerProfile};

    pub fn parse_networks(output: &str) -> Vec<AccessPoint> {
        crate::services::network::parse_networks(output)
    }

    pub fn parse_saved_connections(output: &str) -> Vec<SavedNetwork> {
        crate::services::network::parse_saved_connections(output)
    }

    pub fn parse_wifi_radio_enabled(output: &str) -> bool {
        crate::services::network::parse_wifi_radio_enabled(output)
    }

    pub fn build_network_status_from_nmcli(
        wifi_radio: &str,
        active_connections: &str,
        wifi_devices: &str,
        device_ip: Option<&str>,
        device_list: &str,
        public_ip: Option<String>,
    ) -> crate::types::NetworkStatus {
        crate::services::network::build_network_status_from_nmcli(
            wifi_radio,
            active_connections,
            wifi_devices,
            device_ip,
            device_list,
            public_ip,
        )
    }

    pub fn parse_device_line(line: &str) -> Option<AudioDevice> {
        crate::services::audio::parse_device_line(line)
    }

    pub fn bluetoothctl_device_not_found(output: &str) -> bool {
        crate::services::bluetooth::bluetoothctl_device_not_found(output)
    }

    pub fn compute_time_remaining_from_sysfs(
        charging: bool,
        time_to_full_now: Option<u64>,
        time_to_empty_now: Option<u64>,
        energy_now: Option<u64>,
        power_now: Option<u64>,
    ) -> String {
        crate::services::power::compute_time_remaining_from_sysfs(
            charging,
            time_to_full_now,
            time_to_empty_now,
            energy_now,
            power_now,
        )
    }

    pub fn parse_devices(output: &str) -> (Vec<AudioDevice>, Vec<AudioDevice>) {
        crate::services::audio::parse_devices(output)
    }

    pub fn parse_streams(output: &str) -> Vec<AudioStream> {
        crate::services::audio::parse_streams(output)
    }

    pub fn parse_device_info(address: &str, output: &str) -> BluetoothDevice {
        crate::services::bluetooth::parse_device_info(address, output)
    }

    /// Adapter `bluetoothctl show` fields exposed as JSON for contract tests.
    pub fn parse_show_block_json(output: &str) -> serde_json::Value {
        let d = crate::services::bluetooth::parse_show_block(output);
        serde_json::json!({
            "name": d.name,
            "alias": d.alias,
            "powered": d.powered,
            "discoverable": d.discoverable,
            "pairable": d.pairable,
            "discovering": d.discovering,
        })
    }

    pub fn parse_controller_list_line(line: &str) -> Option<(String, String)> {
        crate::services::bluetooth::parse_controller_list_line(line)
    }

    pub fn parse_devices_list_address(line: &str) -> Option<String> {
        crate::services::bluetooth::parse_devices_list_address(line)
    }

    pub fn parse_pacman_qu(output: &str) -> Vec<Package> {
        crate::services::packages::parse_pacman_qu(output)
    }

    pub fn parse_pacman_q(output: &str) -> Vec<Package> {
        crate::services::packages::parse_pacman_q(output)
    }

    pub fn parse_pacman_search(output: &str) -> Vec<Package> {
        crate::services::packages::parse_pacman_search(output)
    }

    pub fn format_minutes(minutes: u64) -> String {
        crate::services::power::format_minutes(minutes)
    }

    pub fn format_time_from_energy(energy_uwh: u64, power_uw: u64) -> String {
        crate::services::power::format_time_from_energy(energy_uwh, power_uw)
    }

    pub fn parse_powerprofilesctl_output(out: &str) -> Option<PowerProfile> {
        crate::services::power::parse_powerprofilesctl_output(out)
    }

    pub fn firewall_status_from_ufw(output: &str) -> serde_json::Value {
        crate::services::security::firewall_status_from_ufw(output)
    }

    pub fn parse_ufw_numbered_rules(output: &str) -> Vec<crate::services::security::FirewallRule> {
        crate::services::security::parse_ufw_numbered_rules(output)
    }

    pub fn ssh_status_json(active_output: &str, enabled: bool) -> serde_json::Value {
        crate::services::security::ssh_status_json(active_output, enabled)
    }

    pub fn parse_ssh_connections(output: &str) -> Vec<crate::services::security::SshConnection> {
        crate::services::security::parse_ssh_connections(output)
    }

    pub fn parse_failed_login_lines(
        output: &str,
        source: &str,
    ) -> Vec<crate::services::security::SecurityLog> {
        crate::services::security::parse_failed_login_lines(output, source)
    }

    pub fn parse_encryption_devices(output: &str) -> (bool, Vec<String>) {
        crate::services::security::parse_encryption_devices(output)
    }

    pub fn parse_nmap_xml_ports(output: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        crate::services::security::nmap_ports_as_json(output)
    }
}
