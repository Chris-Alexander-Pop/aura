use ags_sidecar::build_registry;
use ags_sidecar::cli;
use ags_sidecar::notify;
use ags_sidecar::rpc::{create_success_response, error_response_for_registry_err, RpcServer};
use ags_sidecar::types::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    ags_sidecar::utils::crash::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return cli::run(&args[1..]);
    }

    run_server().await
}

async fn run_server() -> Result<()> {
    let (request_tx, mut request_rx) =
        mpsc::unbounded_channel::<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>();
    let (notify_tx, notify_rx) = broadcast::channel::<String>(256);
    notify::init(notify_tx.clone());
    notify::spawn_stdout_forwarder(notify_rx);
    ags_sidecar::services::notifications::spawn_dbus_listener();
    ags_sidecar::services::calendar::spawn_reminder_tick();
    ags_sidecar::services::todos::spawn_reminder_tick();
    ags_sidecar::services::hyprland::spawn_event_listener();

    let mut registry = build_registry();
    ags_sidecar::extensions::attach(&mut registry).await;
    let registry = Arc::new(Mutex::new(registry));

    let registry_rpc = registry.clone();
    tokio::spawn(async move {
        while let Some((request, response_tx)) = request_rx.recv().await {
            let request_id = request.id.clone();
            let method = request.method.clone();
            let params = request.params.clone();
            let handler = {
                let registry = registry_rpc.lock().await;
                registry.handler_for(&method)
            };
            let result = match handler {
                Some(handler) => {
                    let started = std::time::Instant::now();
                    let out = handler(params).await;
                    ags_sidecar::utils::rpc_log::log_rpc_completed(
                        &method,
                        request.params.as_ref(),
                        started.elapsed(),
                    );
                    out
                }
                None => Err(anyhow::Error::new(
                    ags_sidecar::services::MethodNotFound(method),
                )),
            };
            let response = match result {
                Ok(value) => create_success_response(request_id, value),
                Err(e) => error_response_for_registry_err(request_id, &e),
            };
            let _ = response_tx.send(response);
        }
    });

    let registry_http = registry.clone();
    let http_notify = notify_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = ags_sidecar::server::run(registry_http, http_notify).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    let rpc_server = RpcServer::new(request_tx);
    rpc_server.run().await?;

    Ok(())
}
