mod rpc;
mod services;
mod types;
mod utils;

use crate::rpc::{create_error_response, create_success_response, RpcServer};
use crate::services::ServiceRegistry;
use crate::types::{error_codes, JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};
use anyhow::Result;
use serde_json;
use std::env;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();

    // CLI mode for testing
    if args.len() > 1 && args[1] == "client" {
        return run_cli_client(&args[2..]).await;
    }

    // Test mode
    if args.len() > 1 && args[1] == "test" {
        return run_tests().await;
    }

    // Normal JSON-RPC server mode
    run_server().await
}

async fn run_server() -> Result<()> {
    // Create channels for request/response and notifications
    let (request_tx, mut request_rx) = mpsc::unbounded_channel::<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>();
    let (notification_tx, _notification_rx) = mpsc::unbounded_channel::<JsonRpcNotification>();

    // Initialize services
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

    // Start request handler task
    let registry = Arc::new(Mutex::new(registry));
    let registry_clone = registry.clone();
    tokio::spawn(async move {
        while let Some((request, response_tx)) = request_rx.recv().await {
            let request_id = request.id.clone();
            let registry = registry_clone.lock().await;
            let result = registry.handle_request(request).await;
            let response = match result {
                Ok(value) => create_success_response(request_id, value),
                Err(e) => {
                    let error_msg = e.to_string();
                    create_error_response(
                        request_id,
                        error_codes::INTERNAL_ERROR,
                        error_msg,
                    )
                }
            };
            let _ = response_tx.send(response);
        }
    });

    // Start RPC server
    let rpc_server = RpcServer::new(request_tx, notification_tx);
    rpc_server.run().await?;

    Ok(())
}

async fn run_cli_client(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Usage: ags-sidecar client <method> [params_json]");
        eprintln!("\nExamples:");
        eprintln!("  ags-sidecar client Power.GetBatteryState");
        eprintln!("  ags-sidecar client Power.SetProfile '{{\"profile\":\"performance\"}}'");
        eprintln!("  ags-sidecar client Network.ScanNetworks");
        eprintln!("  ags-sidecar client System.GetStats");
        eprintln!("\nNote: This requires a running ags-sidecar server.");
        eprintln!("Start the server with: ags-sidecar");
        eprintln!("Then send requests via stdin or use this client mode.");
        return Ok(());
    }

    let method = &args[0];
    let params = if args.len() > 1 {
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

    // Print the JSON-RPC request that can be sent to the server
    let request_json = serde_json::to_string(&request)?;
    println!("{}", request_json);
    eprintln!("\nTo test, start the server and pipe this request:");
    eprintln!("  echo '{}' | ags-sidecar", request_json);

    Ok(())
}

async fn run_tests() -> Result<()> {
    println!("Running tests...");
    println!("Use 'cargo test' to run unit and integration tests");
    println!("\nAvailable test commands:");
    println!("  cargo test                    # Run all tests");
    println!("  cargo test --test integration_test  # Run integration tests");
    Ok(())
}
