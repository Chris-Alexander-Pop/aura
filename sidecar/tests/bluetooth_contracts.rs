//! Golden `bluetoothctl` fixture contracts (parsers under `services/bluetooth.rs`).

mod common;

use ags_sidecar::contract_parsers::{
    bluetoothctl_device_not_found, parse_controller_list_line, parse_device_info,
    parse_devices_list_address, parse_show_block_json,
};
use common::load_fixture;

#[test]
fn contract_bluetoothctl_list_single_controller() {
    let text = load_fixture("bluetooth/bluetoothctl_list.txt");
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some((addr, name)) = parse_controller_list_line(line) {
            rows.push((addr, name));
        }
    }
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "AA:BB:CC:DD:EE:00");
    assert_eq!(rows[0].1, "My-PC");
}

#[test]
fn contract_bluetoothctl_list_multi_controllers() {
    let text = load_fixture("bluetooth/bluetoothctl_list_multi.txt");
    let mut addrs = Vec::new();
    for line in text.lines() {
        if let Some((addr, _)) = parse_controller_list_line(line) {
            addrs.push(addr);
        }
    }
    assert_eq!(addrs.len(), 2);
    assert_eq!(addrs[1], "11:22:33:44:55:66");
}

#[test]
fn contract_bluetoothctl_show_adapter_fields() {
    let text = load_fixture("bluetooth/bluetoothctl_show.txt");
    let v = parse_show_block_json(&text);
    assert_eq!(v["name"], "My-PC");
    assert_eq!(v["powered"], true);
    assert_eq!(v["discoverable"], false);
    assert_eq!(v["pairable"], true);
    assert_eq!(v["discovering"], false);
}

#[test]
fn contract_bluetoothctl_device_info_full_fixture() {
    let text = load_fixture("bluetooth/bluetoothctl_info.txt");
    let dev = parse_device_info("AA:BB:CC:DD:EE:FF", &text);
    assert_eq!(dev.name, "Test Headphones");
    assert!(dev.paired);
    assert!(dev.connected);
    assert_eq!(dev.battery_percentage, Some(85));
    assert_eq!(dev.device_type, "audio-headphones");
    assert_eq!(dev.rssi, Some(-45));
}

#[test]
fn contract_bluetoothctl_device_info_minimal_fixture() {
    let text = load_fixture("bluetooth/bluetoothctl_info_minimal.txt");
    let dev = parse_device_info("11:22:33:44:55:66", &text);
    assert_eq!(dev.name, "Unknown Device");
    assert!(!dev.paired);
    assert!(!dev.connected);
    assert_eq!(dev.battery_percentage, None);
}

#[test]
fn contract_bluetoothctl_device_not_found_fixture() {
    let text = load_fixture("bluetooth/bluetoothctl_info_not_found.txt");
    assert!(bluetoothctl_device_not_found(&text));
}

#[test]
fn contract_bluetoothctl_devices_list_skips_noise_lines() {
    let text = load_fixture("bluetooth/bluetoothctl_devices.txt");
    let mut addresses = Vec::new();
    for line in text.lines() {
        if let Some(addr) = parse_devices_list_address(line) {
            addresses.push(addr);
        }
    }
    assert_eq!(addresses.len(), 2);
    assert_eq!(addresses[0], "AA:BB:CC:DD:EE:FF");
    assert_eq!(addresses[1], "11:22:33:44:55:01");
}
