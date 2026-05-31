pub mod power;
pub mod network;
pub mod system;
pub mod brightness;
pub mod vpn;
pub mod weather;
pub mod gamemode;
pub mod storage;
pub mod audio;
pub mod bluetooth;
pub mod performance;
pub mod security;
#[cfg(feature = "offensive-security")]
pub mod security_offensive;
pub mod devops;
pub mod productivity;
pub mod calendar;
pub mod ics;
pub mod todos;
pub mod logs;
pub mod packages;
pub mod automation;
pub mod communication;
pub mod fitness;
pub mod hyprland;
pub mod shell;
pub mod lock;
pub mod mpris;
pub mod processes;
pub mod notifications;
pub mod keybinds;
pub mod settings;
pub mod dashboard;
pub mod capture;
pub mod launcher;
pub mod vault;

use crate::types::JsonRpcRequest;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Unknown JSON-RPC method (maps to `error_codes::METHOD_NOT_FOUND`).
#[derive(Debug, Clone)]
pub struct MethodNotFound(pub String);

impl fmt::Display for MethodNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Method not found: {}", self.0)
    }
}

impl std::error::Error for MethodNotFound {}

pub type AsyncHandler = Arc<
    dyn Fn(Option<serde_json::Value>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send>> + Send + Sync,
>;

pub struct ServiceRegistry {
    handlers: HashMap<String, AsyncHandler>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<F, Fut>(&mut self, method: &str, handler: F)
    where
        F: Fn(Option<serde_json::Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let handler: AsyncHandler = Arc::new(move |params| {
            Box::pin(handler(params)) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send>>
        });
        self.handlers.insert(method.to_string(), handler);
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<serde_json::Value> {
        let method = request.method.clone();
        let params = request.params;

        if let Some(handler) = self.handlers.get(&method) {
            handler(params).await
        } else {
            Err(anyhow::Error::new(MethodNotFound(method)))
        }
    }
}
