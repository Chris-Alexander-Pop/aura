//! CLI dispatch for the `ags-sidecar` binary (testable without starting the server).

use crate::types::JsonRpcRequest;
use anyhow::{bail, Result};
use serde_json::Value;

/// Handle subcommands from argv (excluding program name).
pub fn run(args: &[String]) -> Result<()> {
    if args.first().map(String::as_str) == Some("client") {
        return run_cli_client(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("test") {
        return run_tests_hint();
    }
    bail!("server mode requires async startup; invoke from main")
}

fn run_cli_client(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Usage: ags-sidecar client <method> [params_json]");
        eprintln!("\nExamples:");
        eprintln!("  ags-sidecar client Power.GetBatteryState");
        eprintln!("  ags-sidecar client Power.SetProfile '{{\"profile\":\"performance\"}}'");
        eprintln!("  ags-sidecar client Network.ScanNetworks");
        return Ok(());
    }

    let method = &args[0];
    let params: Option<Value> = if args.len() > 1 {
        Some(serde_json::from_str(&args[1])?)
    } else {
        None
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.clone(),
        params,
        id: Some(serde_json::Value::Number(1.into())),
    };

    let request_json = serde_json::to_string(&request)?;
    println!("{request_json}");
    eprintln!("\nTo test: echo '{request_json}' | ags-sidecar");
    Ok(())
}

fn run_tests_hint() -> Result<()> {
    println!("Use 'cargo test' to run tests.");
    println!("  cargo test");
    println!("  cargo test --test integration_test");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_prints_json_rpc_request() {
        let args = vec![
            "client".into(),
            "Sidecar.GetVersion".into(),
        ];
        run(&args).expect("client run");
    }

    #[test]
    fn client_with_params() {
        let args = vec![
            "client".into(),
            "Power.SetProfile".into(),
            r#"{"profile":"balanced"}"#.into(),
        ];
        run(&args).expect("client params");
    }

    #[test]
    fn client_usage_when_empty() {
        run(&["client".into()]).expect("usage");
    }

    #[test]
    fn test_subcommand_hint() {
        run(&["test".into()]).expect("test hint");
    }

    #[test]
    fn server_mode_returns_error() {
        assert!(run(&[]).is_err());
        assert!(run(&["serve".into()]).is_err());
    }
}
