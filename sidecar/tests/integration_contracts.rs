//! Golden CLI output contracts for foundation services (network, bluetooth, audio, packages, power).
//! Uses the same parsers as production code via [`ags_sidecar::contract_parsers`]; no host mutation.

mod common;

use ags_sidecar::contract_parsers::{
    bluetoothctl_device_not_found, build_network_status_from_nmcli, compute_time_remaining_from_sysfs,
    parse_battery_charging,
    cpu_usage_from_samples, format_minutes, format_time_from_energy, git_status_dirty,
    parse_brightnessctl_list, parse_brightnessctl_machine_line, parse_controller_list_line,
    parse_cron_line, parse_ddc_vcp_brightness, parse_device_info,
    parse_device_line, parse_devices, parse_devices_list_address, parse_df_storage_usage,
    parse_docker_image_line, package_dependency_graph_from_qi, parse_networks, parse_pacman_q,
    parse_pacman_qu, parse_pacman_search,
    parse_powerprofilesctl_output, parse_proc_stat_cpu, parse_saved_connections,
    parse_sensors_cpu_temp, parse_show_block_json, parse_streams, parse_systemd_timer_line,
};
use ags_sidecar::types::{AccessPoint, PowerProfile};
use common::load_fixture;
use serde_json::Value;

fn assert_json_object_keys(value: &Value, keys: &[&str]) {
    let obj = value.as_object().expect("JSON object");
    for key in keys {
        assert!(obj.contains_key(*key), "missing key {key}");
    }
}

fn assert_access_point_contract(ap: &AccessPoint) {
    // Contract: `Network.ScanNetworks` → `types::AccessPoint` (UI Wi‑Fi list).
    let v = serde_json::to_value(ap).expect("serialize AccessPoint");
    assert_json_object_keys(
        &v,
        &["ssid", "bssid", "strength", "frequency", "active", "security"],
    );
}

fn assert_package_row_contract(v: &Value) {
    // Contract: `Packages.GetUpgradable` / `Packages.GetInstalled` array elements.
    assert_json_object_keys(v, &["name", "version", "description", "installed"]);
}

fn assert_audio_device_contract(v: &Value) {
    // Contract: `Audio.GetDevices` sink/source entries.
    assert_json_object_keys(
        v,
        &["id", "name", "info", "volume", "is_default", "muted"],
    );
}

fn assert_audio_stream_contract(v: &Value) {
    // Contract: `Audio.GetStreams` stream mixer rows.
    assert_json_object_keys(v, &["id", "name", "app", "volume", "sink_id"]);
}

fn assert_network_status_contract(v: &Value) {
    // Contract: `Network.GetStatus` → `types::NetworkStatus`.
    // `active_ssid` / `local_ip` / `public_ip` use `skip_serializing_if = Option::is_none`.
    for key in [
        "wifi_enabled",
        "connection_type",
        "ethernet_connected",
    ] {
        assert!(v.get(key).is_some(), "missing NetworkStatus field {key}");
    }
    assert!(v.get("active_connection").is_some());
}

fn assert_bluetooth_device_contract(v: &Value) {
    // Contract: `Bluetooth.GetDevices` device objects.
    assert_json_object_keys(
        v,
        &[
            "path",
            "address",
            "name",
            "alias",
            "connected",
            "paired",
            "trusted",
            "device_type",
            "services",
        ],
    );
}

#[test]
fn contract_network_wifi_scan_json_shape() {
    let text = load_fixture("network/nmcli_wifi_list.txt");
    let networks = parse_networks(&text);
    assert!(!networks.is_empty());
    assert!(networks.iter().any(|n| n.ssid == "TestNet"));
    assert!(!networks.iter().any(|n| n.ssid == "--"));
    for ap in &networks {
        assert_access_point_contract(ap);
    }
}

#[test]
fn contract_network_wifi_prefers_active_ap() {
    let text = load_fixture("network/nmcli_wifi_duplicates.txt");
    let networks = parse_networks(&text);
    let test_net = networks.iter().find(|n| n.ssid == "TestNet").unwrap();
    assert!(test_net.active);
    assert_eq!(test_net.strength, 90);
    let weak = networks.iter().find(|n| n.ssid == "WeakNet").unwrap();
    assert!(!weak.active);
    assert_eq!(weak.strength, 80);
}

#[test]
fn contract_network_wifi_all_inactive() {
    let text = load_fixture("network/nmcli_wifi_all_inactive.txt");
    let networks = parse_networks(&text);
    assert_eq!(networks.len(), 2);
    assert!(networks.iter().all(|n| !n.active));
}

#[test]
fn contract_network_wifi_empty_scan() {
    let text = load_fixture("network/nmcli_wifi_empty.txt");
    assert!(parse_networks(&text).is_empty());
}

#[test]
fn contract_network_saved_connections_json_shape() {
    let text = load_fixture("network/nmcli_saved_connections.txt");
    let saved = parse_saved_connections(&text);
    // Contract: `Network.ListSaved` — name, uuid, autoconnect (802-11 only).
    assert_eq!(saved.len(), 3);
    for row in &saved {
        let v = serde_json::to_value(row).unwrap();
        assert_json_object_keys(&v, &["name", "uuid", "autoconnect"]);
    }
    assert!(!saved.iter().any(|s| s.name.contains("Ethernet")));
}

#[test]
fn contract_audio_wpctl_devices_json_shape() {
    let text = load_fixture("audio/wpctl_status.txt");
    let (sinks, sources) = parse_devices(&text);
    assert_eq!(sinks.len(), 1);
    assert_eq!(sources.len(), 1);
    assert!(sinks[0].muted);
    assert!(sinks[0].is_default);
    for dev in sinks.iter().chain(sources.iter()) {
        assert_audio_device_contract(&serde_json::to_value(dev).unwrap());
    }
}

#[test]
fn contract_audio_wpctl_empty_devices() {
    let text = load_fixture("audio/wpctl_status_empty.txt");
    let (sinks, sources) = parse_devices(&text);
    assert!(sinks.is_empty());
    assert!(sources.is_empty());
}

#[test]
fn contract_audio_wpctl_multiple_defaults() {
    let text = load_fixture("audio/wpctl_status_multi.txt");
    let (sinks, sources) = parse_devices(&text);
    assert_eq!(sinks.len(), 2);
    assert!(!sinks[0].is_default);
    assert!(sinks[1].is_default);
    assert!((sinks[1].volume - 0.75).abs() < f64::EPSILON);
    assert_eq!(sources.len(), 1);
    assert!(sources[0].is_default);
}

#[test]
fn contract_audio_pactl_streams_json_shape() {
    let text = load_fixture("audio/pactl_sink_inputs.txt");
    let streams = parse_streams(&text);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].app, "Firefox");
    assert_eq!(streams[0].volume, 100);
    assert_eq!(streams[0].sink_id, 47);
    assert_audio_stream_contract(&serde_json::to_value(&streams[0]).unwrap());
}

#[test]
fn contract_bluetooth_device_info_json_shape() {
    let text = load_fixture("bluetooth/bluetoothctl_info.txt");
    let dev = parse_device_info("AA:BB:CC:DD:EE:FF", &text);
    assert_eq!(dev.name, "Test Headphones");
    assert!(dev.paired);
    assert!(dev.connected);
    assert_eq!(dev.battery_percentage, Some(85));
    assert_bluetooth_device_contract(&serde_json::to_value(&dev).unwrap());
}

#[test]
fn contract_bluetooth_adapter_show_fields() {
    let text = load_fixture("bluetooth/bluetoothctl_show.txt");
    let details = parse_show_block_json(&text);
    assert_eq!(details.get("name").and_then(|v| v.as_str()), Some("My-PC"));
    assert_eq!(details.get("powered").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(details.get("discoverable").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(details.get("pairable").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn contract_bluetooth_controller_list() {
    let text = load_fixture("bluetooth/bluetoothctl_list.txt");
    let mut count = 0;
    for line in text.lines() {
        if parse_controller_list_line(line).is_some() {
            count += 1;
        }
    }
    assert_eq!(count, 1);
}

#[test]
fn contract_bluetooth_devices_list_addresses() {
    let text = load_fixture("bluetooth/bluetoothctl_devices.txt");
    let addrs: Vec<_> = text
        .lines()
        .filter_map(parse_devices_list_address)
        .collect();
    assert_eq!(addrs.len(), 2);
}

#[test]
fn contract_packages_upgradable_json_shape() {
    let text = load_fixture("packages/pacman_qu.txt");
    let pkgs = parse_pacman_qu(&text);
    assert_eq!(pkgs.len(), 2);
    for pkg in &pkgs {
        assert_package_row_contract(&serde_json::to_value(pkg).unwrap());
    }
}

#[test]
fn contract_packages_upgradable_empty() {
    let text = load_fixture("packages/pacman_qu_empty.txt");
    assert!(parse_pacman_qu(&text).is_empty());
}

#[test]
fn contract_packages_installed_json_shape() {
    let text = load_fixture("packages/pacman_q.txt");
    let pkgs = parse_pacman_q(&text);
    assert_eq!(pkgs.len(), 3);
    assert_eq!(pkgs[2].name, "vim");
    for pkg in &pkgs {
        let v = serde_json::to_value(pkg).unwrap();
        assert_package_row_contract(&v);
        assert_eq!(v.get("installed").and_then(|x| x.as_bool()), Some(true));
    }
}

#[test]
fn contract_packages_search_json_shape() {
    let text = load_fixture("packages/pacman_ss.txt");
    let pkgs = parse_pacman_search(&text);
    assert_eq!(pkgs.len(), 2);
    assert!(!pkgs[0].installed);
}

#[test]
fn contract_packages_qi_dependency_graph() {
    let qi = load_fixture("packages/pacman_qi_pacman.txt");
    let graph = package_dependency_graph_from_qi("pacman", &qi);
    assert_eq!(graph.depends.len(), 4);
    assert_eq!(graph.required_by, vec!["aura", "yay"]);
}

#[test]
fn contract_power_powerprofilesctl_profile() {
    let text = load_fixture("power/powerprofilesctl_get.txt");
    let first = text.lines().next().unwrap_or("");
    assert_eq!(
        parse_powerprofilesctl_output(first),
        Some(PowerProfile::Performance)
    );
    // Contract: `Power.GetProfile` → `{ "profile": "balanced"|"performance"|"saver" }`.
}

#[test]
fn contract_network_status_wifi_disabled_fixture() {
    let radio = load_fixture("network/nmcli_radio_wifi.txt");
    let status = build_network_status_from_nmcli(
        &radio,
        "",
        "",
        None,
        "",
        None,
    );
    assert!(!status.wifi_enabled);
    assert_eq!(status.connection_type, "none");
    assert_network_status_contract(&serde_json::to_value(&status).unwrap());
}

#[test]
fn contract_network_status_wifi_active_with_ssid() {
    let status = build_network_status_from_nmcli(
        "enabled",
        &load_fixture("network/nmcli_active_connections_wifi.txt"),
        &load_fixture("network/nmcli_dev_wifi_active.txt"),
        Some(&load_fixture("network/nmcli_device_ip4.txt")),
        "",
        None,
    );
    assert!(status.wifi_enabled);
    assert_eq!(status.connection_type, "wifi");
    assert_eq!(status.active_ssid.as_deref(), Some("CafeWiFi"));
    assert_eq!(status.local_ip.as_deref(), Some("192.168.1.42"));
    assert_network_status_contract(&serde_json::to_value(&status).unwrap());
}

#[test]
fn contract_network_status_ethernet_only_device_list() {
    let status = build_network_status_from_nmcli(
        "enabled",
        "",
        "",
        None,
        &load_fixture("network/nmcli_device_list_ethernet.txt"),
        None,
    );
    assert!(status.ethernet_connected);
    assert_eq!(status.connection_type, "ethernet");
}

#[test]
fn contract_audio_malformed_device_line_returns_none() {
    assert!(parse_device_line("│  broken row").is_none());
}

#[test]
fn contract_audio_pactl_empty_streams() {
    let streams = parse_streams(&load_fixture("audio/pactl_sink_inputs_empty.txt"));
    assert!(streams.is_empty());
}

#[test]
fn contract_audio_pactl_error_output_yields_no_streams() {
    assert!(parse_streams(&load_fixture("audio/pactl_error.txt")).is_empty());
}

#[test]
fn contract_system_proc_stat_and_cpu_delta() {
    let text = load_fixture("system/proc_stat.txt");
    let (total, idle) = parse_proc_stat_cpu(&text).expect("proc stat");
    let usage = cpu_usage_from_samples(total - 100, idle - 40, total, idle).expect("usage");
    assert!(usage > 0.0 && usage <= 1.0);
    assert!(parse_proc_stat_cpu(&load_fixture("system/proc_stat_short.txt")).is_none());
}

#[test]
fn contract_system_df_and_sensors_fixtures() {
    let usage = parse_df_storage_usage(&load_fixture("system/df_output.txt"));
    assert!(usage > 0.0 && usage < 1.0);
    let temp = parse_sensors_cpu_temp(&load_fixture("system/sensors_output.txt")).expect("temp");
    assert!(temp > 0.0);
}

#[test]
fn contract_devops_timers_images_git_cron_fixtures() {
    let timers: Vec<_> = load_fixture("devops/systemctl_timers.txt")
        .lines()
        .filter_map(parse_systemd_timer_line)
        .collect();
    assert_eq!(timers.len(), 2);
    assert!(timers[0].active);

    let images: Vec<_> = load_fixture("devops/docker_images.txt")
        .lines()
        .filter_map(parse_docker_image_line)
        .collect();
    assert_eq!(images.len(), 2);

    assert!(!git_status_dirty(&load_fixture("devops/git_porcelain_clean.txt")));
    assert!(git_status_dirty(&load_fixture("devops/git_porcelain_dirty.txt")));

    let job = parse_cron_line(
        load_fixture("devops/crontab_sample.txt")
            .lines()
            .find(|l| !l.starts_with('#'))
            .unwrap(),
        "root",
    )
    .expect("cron");
    assert!(job.command.contains("logrotate"));
}

#[test]
fn contract_brightness_ddc_vcp_fixture() {
    let b = parse_ddc_vcp_brightness(&load_fixture("brightness/ddcutil_getvcp.txt")).expect("vcp");
    assert!((b - 0.5).abs() < f64::EPSILON);
}

#[test]
fn contract_brightness_brightnessctl_list_fixture() {
    let devices = parse_brightnessctl_list(&load_fixture("brightness/brightnessctl_list.txt"));
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].0, "intel_backlight");
    assert!((devices[0].1 - 0.5).abs() < f64::EPSILON);
    assert_eq!(devices[1].0, "eDP-1");
}

#[test]
fn contract_brightness_brightnessctl_machine_fixture() {
    let text = load_fixture("brightness/brightnessctl_machine.txt");
    let parsed: Vec<_> = text
        .lines()
        .filter_map(parse_brightnessctl_machine_line)
        .collect();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, "intel_backlight");
}

#[test]
fn contract_bluetooth_device_not_found_message() {
    let text = load_fixture("bluetooth/bluetoothctl_info_not_found.txt");
    assert!(bluetoothctl_device_not_found(&text));
}

#[test]
fn contract_bluetooth_device_info_minimal_shape() {
    let dev = parse_device_info(
        "11:22:33:44:55:66",
        &load_fixture("bluetooth/bluetoothctl_info_minimal.txt"),
    );
    assert_bluetooth_device_contract(&serde_json::to_value(&dev).unwrap());
    assert!(!dev.paired);
}

#[test]
fn contract_packages_pacman_malformed_qu_lines() {
    let pkgs = parse_pacman_qu(&load_fixture("packages/pacman_qu_malformed.txt"));
    assert_eq!(pkgs.len(), 1);
}

#[test]
fn contract_power_zero_time_to_empty_sysfs_branch() {
    assert_eq!(
        compute_time_remaining_from_sysfs(false, None, Some(0), None, None),
        "Unknown"
    );
}

#[test]
fn contract_power_supply_sysfs_fixture_discharging() {
    let status = load_fixture("power_supply/status_discharging.txt");
    assert!(!parse_battery_charging(Some(status.trim())));
    let energy: u64 = load_fixture("power_supply/energy_now.txt")
        .trim()
        .parse()
        .expect("energy_now");
    let power: u64 = load_fixture("power_supply/power_now.txt")
        .trim()
        .parse()
        .expect("power_now");
    assert_eq!(
        compute_time_remaining_from_sysfs(false, None, None, Some(energy), Some(power)),
        "3h"
    );
    let capacity: u8 = load_fixture("power_supply/capacity.txt")
        .trim()
        .parse()
        .expect("capacity");
    assert_eq!(capacity, 72);
}

#[test]
fn contract_power_supply_sysfs_fixture_charging_time_to_full() {
    let status = load_fixture("power_supply/status_charging.txt");
    assert!(parse_battery_charging(Some(status.trim())));
    assert_eq!(
        compute_time_remaining_from_sysfs(true, Some(3600), None, None, None),
        "1h"
    );
}

#[test]
fn contract_power_battery_time_remaining_strings() {
    // Contract: `Power.GetBatteryState.time_remaining` from sysfs energy/power math.
    assert_eq!(format_minutes(0), "Unknown");
    assert_eq!(format_minutes(45), "45m");
    assert_eq!(format_time_from_energy(30_000_000, 10_000_000), "3h");
    assert_eq!(format_time_from_energy(30_000_000, 0), "Unknown");
}

/// Read-only live `nmcli` Wi‑Fi list (no rescan/connect). Run:
/// `AURA_CONTRACT_LIVE=1 cargo test contract_nmcli_wifi_list_live -- --ignored`
#[test]
#[ignore = "requires AURA_CONTRACT_LIVE=1 and NetworkManager"]
fn contract_nmcli_wifi_list_live() {
    if std::env::var("AURA_CONTRACT_LIVE").ok().as_deref() != Some("1") {
        panic!("Set AURA_CONTRACT_LIVE=1 to run live nmcli contract test");
    }

    let output = std::process::Command::new("nmcli")
        .args([
            "-g",
            "ACTIVE,SIGNAL,FREQ,SSID,BSSID,SECURITY",
            "d",
            "w",
        ])
        .output()
        .expect("nmcli");
    assert!(
        output.status.success(),
        "nmcli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    for ap in parse_networks(&text) {
        assert_access_point_contract(&ap);
    }
}
