mod rpc;
mod server;
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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "client" {
        return run_cli_client(&args[2..]).await;
    }

    if args.len() > 1 && args[1] == "test" {
        return run_tests().await;
    }

    run_server().await
}

async fn run_server() -> Result<()> {
    let (request_tx, mut request_rx) =
        mpsc::unbounded_channel::<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>();
    let (notification_tx, _notification_rx) =
        mpsc::unbounded_channel::<JsonRpcNotification>();

    // Build service registry
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

    let registry = Arc::new(Mutex::new(registry));

    // Spawn stdin JSON-RPC handler (used by GTK bar / OSD via sidecar.ts)
    let registry_rpc = registry.clone();
    tokio::spawn(async move {
        while let Some((request, response_tx)) = request_rx.recv().await {
            let request_id = request.id.clone();
            let registry = registry_rpc.lock().await;
            let result = registry.handle_request(request).await;
            let response = match result {
                Ok(value) => create_success_response(request_id, value),
                Err(e) => create_error_response(
                    request_id,
                    error_codes::INTERNAL_ERROR,
                    e.to_string(),
                ),
            };
            let _ = response_tx.send(response);
        }
    });

    // Spawn HTTP + WebSocket server (used by ui/ React app)
    let registry_http = registry.clone();
    tokio::spawn(async move {
        if let Err(e) = server::run(registry_http).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    // Run stdin RPC server (blocks until EOF)
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

    let request_json = serde_json::to_string(&request)?;
    println!("{}", request_json);
    eprintln!("\nTo test: echo '{}' | ags-sidecar", request_json);
    Ok(())
}

async fn run_tests() -> Result<()> {
    println!("Use 'cargo test' to run tests.");
    println!("  cargo test");
    println!("  cargo test --test integration_test");
    Ok(())
}
