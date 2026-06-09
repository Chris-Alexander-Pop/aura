//! Smoke tests for the `ags-sidecar` binary CLI subcommands.

mod common;

use std::process::Command;

#[test]
fn binary_client_emits_json_rpc() {
    let bin = env!("CARGO_BIN_EXE_ags-sidecar");
    let output = Command::new(bin)
        .args(["client", "Sidecar.GetVersion"])
        .output()
        .expect("spawn ags-sidecar client");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"jsonrpc\""));
    assert!(stdout.contains("Sidecar.GetVersion"));
}

#[test]
fn binary_test_subcommand_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_ags-sidecar");
    let output = Command::new(bin)
        .arg("test")
        .output()
        .expect("spawn ags-sidecar test");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo test"));
}

#[test]
fn cli_lib_client_matches_binary_shape() {
    ags_sidecar::cli::run(&[
        "client".into(),
        "Power.GetBatteryState".into(),
    ])
    .expect("cli run");
}
