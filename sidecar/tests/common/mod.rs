use ags_sidecar::build_registry;
use ags_sidecar::notify;
use ags_sidecar::services::ServiceRegistry;
use ags_sidecar::types::JsonRpcRequest;
use anyhow::Result;
use serde_json::Value;
use std::sync::Once;

static INIT_NOTIFY: Once = Once::new();

pub fn test_registry() -> ServiceRegistry {
    INIT_NOTIFY.call_once(|| {
        notify::init_for_tests();
    });
    build_registry()
}

pub async fn call_method(
    registry: &ServiceRegistry,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: method.to_string(),
        params,
        id: Some(Value::Number(1.into())),
    };
    registry.handle_request(request).await
}
