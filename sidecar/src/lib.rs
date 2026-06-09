//! Library surface for integration tests and shared registry setup.
pub mod cli;
pub mod notify;
pub mod openapi;
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
    services::todos::register(&mut registry);
    services::logs::register(&mut registry);
    services::packages::register(&mut registry);
    services::automation::register(&mut registry);
    services::communication::register(&mut registry);
    services::fitness::register(&mut registry);
    services::hyprland::register(&mut registry);
    services::shell::register(&mut registry);
    services::lock::register(&mut registry);
    services::mpris::register(&mut registry);
    services::processes::register(&mut registry);
    services::notifications::register(&mut registry);
    services::keybinds::register(&mut registry);
    services::settings::register(&mut registry);
    services::dashboard::register(&mut registry);
    services::capture::register(&mut registry);
    services::launcher::register(&mut registry);
    services::vault::register(&mut registry);
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

    pub fn map_nmcli_connect_error(raw: &str) -> String {
        crate::services::network::map_nmcli_connect_error(raw)
    }

    pub fn nmcli_connect_output_success(output: &str) -> bool {
        crate::services::network::nmcli_connect_output_success(output)
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

    pub fn parse_battery_charging(status: Option<&str>) -> bool {
        crate::services::power::parse_battery_charging(status)
    }

    pub fn parse_devices(output: &str) -> (Vec<AudioDevice>, Vec<AudioDevice>) {
        crate::services::audio::parse_devices(output)
    }

    pub fn parse_pactl_list_sinks(output: &str) -> Vec<crate::services::audio::PactlSinkSummary> {
        crate::services::audio::parse_pactl_list_sinks(output)
    }

    pub fn parse_streams(output: &str) -> Vec<AudioStream> {
        crate::services::audio::parse_streams(output)
    }

    pub use crate::services::mpris::PlayerctlPlayback;

    pub fn parse_playerctl_status(s: &str) -> PlayerctlPlayback {
        crate::services::mpris::parse_playerctl_status(s)
    }

    pub fn parse_now_playing_line(s: &str) -> (String, String) {
        crate::services::mpris::parse_now_playing_line(s)
    }

    pub fn playing_from_status(status: PlayerctlPlayback) -> bool {
        crate::services::mpris::playing_from_status(status)
    }

    pub fn parse_playerctl_list(output: &str) -> Vec<String> {
        crate::services::mpris::parse_playerctl_list(output)
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

    pub fn package_dependency_graph_from_qi(
        name: &str,
        qi_output: &str,
    ) -> crate::services::packages::PackageDependencyGraph {
        crate::services::packages::package_dependency_graph_from_qi(name, qi_output)
    }

    pub fn parse_pactree_reverse(output: &str) -> Vec<String> {
        crate::services::packages::parse_pactree_reverse(output)
    }

    pub fn parse_fprintd_list(
        output: &str,
    ) -> Vec<crate::services::security::FingerprintEntry> {
        crate::services::security::parse_fprintd_list(output)
    }

    pub fn parse_chage_l(output: &str) -> crate::services::security::PasswordPolicyStatus {
        crate::services::security::parse_chage_l(output)
    }

    pub fn parse_meminfo_cached_buffers_kb(output: &str) -> (u64, u64) {
        crate::services::performance::parse_meminfo_cached_buffers_kb(output)
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

    pub fn firewall_status_from_firewalld(output: &str) -> serde_json::Value {
        crate::services::security::firewall_status_from_firewalld(output)
    }

    pub fn firewall_status_none() -> serde_json::Value {
        crate::services::security::firewall_status_none()
    }

    pub fn keyring_status_json(available: bool) -> serde_json::Value {
        crate::services::security::keyring_status_json(available)
    }

    pub fn filter_certificate_filenames(
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Vec<String> {
        crate::services::security::filter_certificate_filenames(names)
    }

    pub fn vpn_connections_from_pgrep(
        openconnect_running: bool,
        openvpn_running: bool,
    ) -> Vec<String> {
        crate::services::security::vpn_connections_from_pgrep(openconnect_running, openvpn_running)
    }

    pub fn parse_sudo_log_lines(
        output: &str,
    ) -> Vec<crate::services::security::SecurityLog> {
        crate::services::security::parse_sudo_log_lines(output)
    }

    pub fn parse_ss_listening_ports(output: &str) -> Vec<String> {
        crate::services::security::parse_ss_listening_ports(output)
    }

    pub fn vpn_interface_connected(
        ip_output: &str,
        iface: &str,
        vpn_process_running: bool,
    ) -> bool {
        crate::services::vpn::vpn_interface_connected(ip_output, iface, vpn_process_running)
    }

    pub fn ip_link_interface_up(output: &str, iface: &str) -> bool {
        crate::services::vpn::ip_link_interface_up(output, iface)
    }

    pub fn parse_wireguard_conf(conf: &str) -> crate::services::vpn::WireGuardConfigSummary {
        crate::services::vpn::parse_wireguard_conf(conf)
    }

    pub fn parse_rclone_listremotes(output: &str) -> Vec<crate::services::vault::VaultRemote> {
        crate::services::vault::parse_rclone_listremotes(output)
    }

    pub fn communication_unread_counts_schema_valid(value: &serde_json::Value) -> bool {
        crate::services::communication::unread_counts_schema_valid(value)
    }

    pub fn parse_iface_ipv4(ip_output: &str, iface: &str) -> Option<String> {
        crate::services::vpn::parse_iface_ipv4(ip_output, iface)
    }

    pub fn load_vpn_profile_defs_from_dir(
        config_dir: &std::path::Path,
    ) -> Vec<crate::types::VpnProfile> {
        crate::services::vpn::load_public_profiles_from_dir(config_dir)
    }

    pub fn parse_proc_stat_cpu(content: &str) -> Option<(u64, u64)> {
        crate::services::system::parse_proc_stat_cpu(content)
    }

    pub fn cpu_usage_from_samples(
        last_total: u64,
        last_idle: u64,
        total: u64,
        idle: u64,
    ) -> Option<f64> {
        crate::services::system::cpu_usage_from_samples(last_total, last_idle, total, idle)
    }

    pub fn parse_sensors_cpu_temp(output: &str) -> Option<f64> {
        crate::services::system::parse_sensors_cpu_temp(output)
    }

    pub fn parse_df_storage_usage(output: &str) -> f64 {
        crate::services::system::parse_df_storage_usage(output)
    }

    pub fn parse_nvidia_gpu_utilization(output: &str) -> Option<f64> {
        crate::services::system::parse_nvidia_gpu_utilization(output)
    }

    pub fn parse_docker_image_line(line: &str) -> Option<crate::services::devops::DockerImage> {
        crate::services::devops::parse_docker_image_line(line)
    }

    pub fn parse_systemd_timer_line(line: &str) -> Option<crate::services::devops::SystemdTimer> {
        crate::services::devops::parse_systemd_timer_line(line)
    }

    pub fn parse_cron_line(line: &str, user: &str) -> Option<crate::services::devops::CronJob> {
        crate::services::devops::parse_cron_line(line, user)
    }

    pub fn git_status_dirty(porcelain: &str) -> bool {
        crate::services::devops::git_status_dirty(porcelain)
    }

    pub fn parse_ddc_vcp_brightness(output: &str) -> Option<f64> {
        crate::services::brightness::parse_ddc_vcp_brightness(output)
    }

    pub fn parse_brightnessctl_list(output: &str) -> Vec<(String, f64)> {
        crate::services::brightness::parse_brightnessctl_list(output)
    }

    pub fn parse_brightnessctl_machine_line(line: &str) -> Option<(String, f64)> {
        crate::services::brightness::parse_brightnessctl_machine_line(line)
    }

    pub fn build_list_top_json(
        rows: &mut [(sysinfo::Pid, f32, String)],
        limit: usize,
    ) -> Vec<serde_json::Value> {
        crate::services::processes::build_list_top_json(rows, limit)
    }
}
